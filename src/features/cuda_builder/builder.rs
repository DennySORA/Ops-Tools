//! CUDA ML 套件建構邏輯
//!
//! 所有建構操作都在 ~/.ml-packages/venv/ 隔離環境中執行，
//! 避免 PEP 668 (externally-managed-environment) 限制。

use crate::core::{OperationError, Result};
use crate::i18n::{self, keys};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

use super::types::BuildContext;
use super::types::CudaPackageId;

// ============================================================================
// 快取管理
// ============================================================================

/// 確保快取目錄存在
pub fn ensure_cache_dir(ctx: &BuildContext) -> Result<()> {
    fs::create_dir_all(&ctx.wheels_dir).map_err(|err| OperationError::Io {
        path: ctx.wheels_dir.display().to_string(),
        source: err,
    })?;
    fs::create_dir_all(&ctx.sources_dir).map_err(|err| OperationError::Io {
        path: ctx.sources_dir.display().to_string(),
        source: err,
    })?;

    if ctx.ccache_path.is_some() {
        fs::create_dir_all(&ctx.ccache_dir).map_err(|err| OperationError::Io {
            path: ctx.ccache_dir.display().to_string(),
            source: err,
        })?;
    }

    Ok(())
}

/// 掃描快取中的 wheel 檔案
pub fn scan_cached_wheels(ctx: &BuildContext, package: CudaPackageId) -> Vec<String> {
    cached_wheel_paths(ctx, package)
        .into_iter()
        .filter_map(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().to_string())
        })
        .collect()
}

/// 清除指定套件的舊快取，確保下次一定重編
pub fn clear_cached_artifacts(ctx: &BuildContext, package: CudaPackageId) -> Result<()> {
    let prefix = package.wheel_prefix();

    let Ok(entries) = fs::read_dir(&ctx.wheels_dir) else {
        return Ok(());
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with(prefix) {
            continue;
        }

        fs::remove_file(&path).map_err(|err| OperationError::Io {
            path: path.display().to_string(),
            source: err,
        })?;
    }

    Ok(())
}

/// 清理快取目錄
pub fn clean_cache(ctx: &BuildContext) -> Result<()> {
    if ctx.wheels_dir.exists() {
        fs::remove_dir_all(&ctx.wheels_dir).map_err(|err| OperationError::Io {
            path: ctx.wheels_dir.display().to_string(),
            source: err,
        })?;
    }
    Ok(())
}

// ============================================================================
// Venv 生命週期管理
// ============================================================================

/// 建立建構用的隔離 venv（若已存在則跳過）
pub fn ensure_venv(ctx: &BuildContext) -> Result<()> {
    if ctx.venv_ready() && venv_uses_python(ctx) {
        return Ok(());
    }

    if ctx.venv_dir.exists() {
        fs::remove_dir_all(&ctx.venv_dir).map_err(|err| OperationError::Io {
            path: ctx.venv_dir.display().to_string(),
            source: err,
        })?;
    }

    let venv_str = ctx.venv_dir.display().to_string();
    let python_str = ctx.python_path.display().to_string();

    // 優先用 uv venv（更快）
    if ctx.uv_available {
        let status = Command::new("uv")
            .args(["venv", "--python", &python_str, &venv_str])
            .status()
            .map_err(|err| command_error("uv venv", &err.to_string()))?;

        if status.success() {
            return Ok(());
        }
    }

    // Fallback: python3 -m venv
    let status = Command::new(&ctx.python_path)
        .args(["-m", "venv", &venv_str])
        .status()
        .map_err(|err| command_error("python3 -m venv", &err.to_string()))?;

    if !status.success() {
        return Err(OperationError::Command {
            command: "venv".to_string(),
            message: i18n::t(keys::CUDA_BUILDER_VENV_FAILED).to_string(),
        });
    }

    Ok(())
}

/// 安裝建構所需的依賴到 venv（pip、ninja、setuptools 等）
pub fn ensure_build_tools(ctx: &BuildContext) -> Result<()> {
    let packages = [
        "pip",
        "ninja",
        "cmake",
        "packaging",
        "setuptools<81",
        "wheel",
        "numpy",
        "pillow",
        "cython",
        "scikit-build-core",
        "pybind11",
        "setuptools-scm",
        "typing_extensions",
        "pyyaml",
        "requests",
        "sympy",
        "networkx",
        "jinja2",
        "meson",
        "meson-python",
    ];
    if ctx.uv_available {
        uv_install_to_venv(ctx, &packages)
    } else {
        pip_install_to_venv(ctx, &packages)
    }
}

// ============================================================================
// 下載與建構（全部透過 venv 執行）
// ============================================================================

/// 從原始碼建構套件 wheel
///
/// 自動套用：
/// - `TORCH_CUDA_ARCH_LIST` → 只編譯目標 GPU 架構（大幅減少編譯時間）
/// - `MAX_JOBS` → 依 CPU/RAM 計算的最佳平行數
/// - 套件專屬環境變數（例: NVTE_FRAMEWORK、DS_BUILD_OPS）
pub fn build_source_package(ctx: &BuildContext, package: CudaPackageId) -> Result<()> {
    clear_cached_artifacts(ctx, package)?;

    match package {
        CudaPackageId::Torch => build_pytorch_from_repo(ctx),
        CudaPackageId::TorchVision => build_torchvision_from_repo(ctx),
        CudaPackageId::TorchAudio => build_torchaudio_from_repo(ctx),
        CudaPackageId::CTranslate2 => build_ctranslate2_from_repo(ctx),
        CudaPackageId::TransformerEngine => build_transformer_engine_from_repo(ctx),
        _ => {
            if run_source_build(ctx, package, false).is_ok() {
                return Ok(());
            }

            clear_cached_artifacts(ctx, package)?;
            run_source_build(ctx, package, true)
        }
    }
}

fn run_source_build(
    ctx: &BuildContext,
    package: CudaPackageId,
    use_build_isolation: bool,
) -> Result<()> {
    let build_jobs = ctx.effective_build_jobs(package);
    let wheels_dir_str = ctx.wheels_dir.display().to_string();
    let mut cmd = Command::new(&ctx.venv_python);
    cmd.args([
        "-m",
        "pip",
        "wheel",
        "--no-binary=:all:",
        "--no-cache-dir",
        package.pip_name(),
        "--no-deps",
        "-w",
        &wheels_dir_str,
    ]);

    if !use_build_isolation {
        cmd.arg("--no-build-isolation");
    }

    let mut cmake_args = configure_base_build_command(&mut cmd, ctx, build_jobs);

    if package == CudaPackageId::CuPy {
        cmd.env("CUPY_NVCC_GENERATE_CODE", cupy_generate_code(ctx));
    }

    if package == CudaPackageId::FlashAttention {
        cmd.env("FLASH_ATTN_CUDA_ARCHS", ctx.gpu_arch.replace('.', ""));
        cmd.env("NVCC_THREADS", "1");
    }

    // 套件專屬環境變數
    for (key, value) in package.build_env_vars() {
        if *key == "CMAKE_ARGS" {
            cmake_args.push((*value).to_string());
        } else {
            cmd.env(key, value);
        }
    }

    if !cmake_args.is_empty() {
        cmd.env("CMAKE_ARGS", cmake_args.join(" "));
    }

    let status = cmd.status().map_err(|err| {
        command_error(
            &format!("pip wheel {}", package.pip_name()),
            &err.to_string(),
        )
    })?;

    if !status.success() {
        return Err(OperationError::Command {
            command: format!("pip wheel {}", package.pip_name()),
            message: crate::tr!(
                keys::CUDA_BUILDER_BUILD_FAILED,
                package = package.display_name()
            ),
        });
    }

    Ok(())
}

fn build_pytorch_from_repo(ctx: &BuildContext) -> Result<()> {
    let build_jobs = ctx.effective_build_jobs(CudaPackageId::Torch);
    let source_dir = ensure_repo_checkout(
        ctx,
        "pytorch",
        "https://github.com/pytorch/pytorch.git",
        "v2.10.0",
        true,
    )?;
    let wheels_dir_str = ctx.wheels_dir.display().to_string();
    let requirements = source_dir.join("requirements.txt");
    let requirements_str = requirements.display().to_string();
    let venv_ninja = ctx.venv_dir.join("bin/ninja");

    run_venv_python_in_dir_with_jobs(
        ctx,
        &source_dir,
        &["-m", "pip", "install", "-r", &requirements_str],
        1,
    )?;

    let mut cmd = Command::new(&ctx.venv_python);
    cmd.current_dir(&source_dir).args([
        "-m",
        "pip",
        "wheel",
        "--no-build-isolation",
        "--no-deps",
        "-v",
        "-w",
        &wheels_dir_str,
        ".",
    ]);

    configure_base_build_command(&mut cmd, ctx, build_jobs);
    cmd.env("CMAKE_PREFIX_PATH", &ctx.venv_dir)
        // PyTorch's setup helper consumes CMAKE_* env vars as cache definitions.
        // Keep Ninja on PATH, but avoid passing generator state through env because
        // it can produce an invalid CMAKE_MAKE_PROGRAM resolution.
        .env_remove("CMAKE_GENERATOR")
        .env_remove("NINJA")
        .env("USE_CUDA", "1")
        .env("USE_NINJA", "1")
        .env("USE_CUDNN", "0")
        .env("USE_CUSPARSELT", "0")
        .env("USE_CUDSS", "0")
        .env("USE_CUFILE", "0")
        .env("USE_DISTRIBUTED", "0")
        .env("USE_MPI", "0")
        .env("USE_GLOO", "0")
        .env("USE_TENSORPIPE", "0")
        .env("USE_KINETO", "0")
        .env("USE_FBGEMM", "0")
        .env("USE_FBGEMM_GENAI", "0")
        .env("USE_MKLDNN", "0")
        .env("USE_NNPACK", "0")
        .env("USE_PYTORCH_QNNPACK", "0")
        .env("USE_KLEIDIAI", "0")
        .env("USE_XNNPACK", "0")
        .env("USE_FLASH_ATTENTION", "0")
        .env("USE_MEM_EFF_ATTENTION", "0")
        .env("BUILD_TEST", "0")
        .env("USE_NUMPY", "1")
        .env("PYTORCH_BUILD_VERSION", "2.10.0")
        .env("PYTORCH_BUILD_NUMBER", "1");

    if venv_ninja.is_file() {
        cmd.env("CMAKE_MAKE_PROGRAM", &venv_ninja);
    }

    run_checked_command(cmd, "pip wheel torch repo")
}

fn build_torchvision_from_repo(ctx: &BuildContext) -> Result<()> {
    let source_dir = ensure_repo_checkout(
        ctx,
        "vision",
        "https://github.com/pytorch/vision.git",
        "v0.25.0",
        false,
    )?;
    run_setup_py_bdist_wheel(
        ctx,
        &source_dir,
        CudaPackageId::TorchVision,
        &[
            ("BUILD_VERSION", "0.25.0"),
            ("PYTORCH_VERSION", "2.10.0"),
            ("FORCE_CUDA", "1"),
        ],
    )
}

fn build_torchaudio_from_repo(ctx: &BuildContext) -> Result<()> {
    let source_dir = ensure_repo_checkout(
        ctx,
        "audio",
        "https://github.com/pytorch/audio.git",
        "v2.10.0",
        false,
    )?;
    run_setup_py_bdist_wheel(
        ctx,
        &source_dir,
        CudaPackageId::TorchAudio,
        &[
            ("BUILD_VERSION", "2.10.0"),
            ("PYTORCH_VERSION", "2.10.0"),
            ("USE_CUDA", "1"),
        ],
    )
}

fn build_ctranslate2_from_repo(ctx: &BuildContext) -> Result<()> {
    let build_jobs = ctx.effective_build_jobs(CudaPackageId::CTranslate2);
    let source_dir = ensure_repo_checkout(
        ctx,
        "CTranslate2",
        "https://github.com/OpenNMT/CTranslate2.git",
        "v4.7.1",
        true,
    )?;
    let build_dir = source_dir.join("build");
    let install_dir = source_dir.join("install");

    if build_dir.exists() {
        fs::remove_dir_all(&build_dir).map_err(|err| OperationError::Io {
            path: build_dir.display().to_string(),
            source: err,
        })?;
    }
    fs::create_dir_all(&build_dir).map_err(|err| OperationError::Io {
        path: build_dir.display().to_string(),
        source: err,
    })?;

    let mut configure = Command::new("cmake");
    configure.current_dir(&build_dir).args([
        "..",
        "-DCMAKE_BUILD_TYPE=Release",
        "-DWITH_CUDA=ON",
        "-DOPENMP_RUNTIME=COMP",
        "-DWITH_MKL=OFF",
        "-DWITH_DNNL=OFF",
        "-DWITH_OPENBLAS=OFF",
    ]);
    configure.arg(format!("-DCUDA_ARCH_LIST={}", ctx.gpu_arch));
    configure.arg(format!("-DCMAKE_INSTALL_PREFIX={}", install_dir.display()));
    configure_base_build_command(&mut configure, ctx, build_jobs);
    run_checked_command(configure, "cmake configure CTranslate2")?;

    let mut build = Command::new("cmake");
    build
        .current_dir(&build_dir)
        .args(["--build", ".", "-j", &build_jobs.to_string()]);
    run_checked_command(build, "cmake build CTranslate2")?;

    let mut install = Command::new("cmake");
    install.current_dir(&build_dir).args(["--install", "."]);
    run_checked_command(install, "cmake install CTranslate2")?;

    let python_dir = source_dir.join("python");
    let ld_library_path = install_dir.join("lib");
    let mut cmd = Command::new(&ctx.venv_python);
    cmd.current_dir(&python_dir)
        .args([
            "setup.py",
            "bdist_wheel",
            "--dist-dir",
            &ctx.wheels_dir.display().to_string(),
        ])
        .env("CTRANSLATE2_ROOT", &install_dir)
        .env(
            "LD_LIBRARY_PATH",
            prepend_path_env(&ld_library_path, "LD_LIBRARY_PATH"),
        );
    configure_base_build_command(&mut cmd, ctx, build_jobs);
    run_checked_command(cmd, "python setup.py bdist_wheel CTranslate2")
}

fn build_transformer_engine_from_repo(ctx: &BuildContext) -> Result<()> {
    let build_jobs = ctx.effective_build_jobs(CudaPackageId::TransformerEngine);
    let source_dir = ensure_repo_checkout(
        ctx,
        "TransformerEngine",
        "https://github.com/NVIDIA/TransformerEngine.git",
        "v2.10",
        true,
    )?;
    let mut extra_env = vec![
        ("NVTE_FRAMEWORK", "pytorch".to_string()),
        ("NVTE_CUDA_ARCHS", ctx.gpu_arch.replace('.', "")),
    ];

    let cudnn_root = venv_site_packages_dir(ctx).join("nvidia").join("cudnn");
    if cudnn_root.exists() {
        extra_env.push(("CUDNN_PATH", cudnn_root.display().to_string()));
    }

    let mut cmd = Command::new(&ctx.venv_python);
    cmd.current_dir(&source_dir).args([
        "setup.py",
        "bdist_wheel",
        "--dist-dir",
        &ctx.wheels_dir.display().to_string(),
    ]);
    configure_base_build_command(&mut cmd, ctx, build_jobs);

    if let Some((_, cudnn_path)) = extra_env.iter().find(|(key, _)| *key == "CUDNN_PATH") {
        let cudnn_lib = PathBuf::from(cudnn_path).join("lib");
        cmd.env(
            "LD_LIBRARY_PATH",
            prepend_path_env(&cudnn_lib, "LD_LIBRARY_PATH"),
        );
    }

    for (key, value) in extra_env {
        cmd.env(key, value);
    }

    run_checked_command(cmd, "python setup.py bdist_wheel TransformerEngine")
}

fn run_setup_py_bdist_wheel(
    ctx: &BuildContext,
    source_dir: &Path,
    package: CudaPackageId,
    extra_env: &[(&str, &str)],
) -> Result<()> {
    let build_jobs = ctx.effective_build_jobs(package);
    let mut cmd = Command::new(&ctx.venv_python);
    cmd.current_dir(source_dir).args([
        "setup.py",
        "bdist_wheel",
        "--dist-dir",
        &ctx.wheels_dir.display().to_string(),
    ]);
    configure_base_build_command(&mut cmd, ctx, build_jobs);

    for (key, value) in extra_env {
        cmd.env(key, value);
    }

    for (key, value) in package.build_env_vars() {
        cmd.env(key, value);
    }

    run_checked_command(
        cmd,
        &format!("python setup.py bdist_wheel {}", package.pip_name()),
    )
}

fn ensure_repo_checkout(
    ctx: &BuildContext,
    directory_name: &str,
    repo_url: &str,
    reference: &str,
    recursive: bool,
) -> Result<PathBuf> {
    let repo_dir = ctx.sources_dir.join(directory_name);

    if repo_dir.join(".git").exists() {
        let mut fetch = Command::new("git");
        fetch
            .current_dir(&repo_dir)
            .args(["fetch", "--depth", "1", "origin", "tag", reference]);
        run_checked_command(fetch, &format!("git fetch {}", repo_url))?;

        let mut checkout = Command::new("git");
        checkout
            .current_dir(&repo_dir)
            .args(["checkout", "-f", reference]);
        run_checked_command(checkout, &format!("git checkout {}", reference))?;
    } else {
        let mut clone = Command::new("git");
        clone.args([
            "clone",
            "--branch",
            reference,
            "--depth",
            "1",
            repo_url,
            &repo_dir.display().to_string(),
        ]);
        run_checked_command(clone, &format!("git clone {}", repo_url))?;
    }

    if recursive {
        let mut submodules = Command::new("git");
        submodules.current_dir(&repo_dir).args([
            "submodule",
            "update",
            "--init",
            "--recursive",
            "--depth",
            "1",
            "--jobs",
            "8",
        ]);
        run_checked_command(
            submodules,
            &format!("git submodule update {}", directory_name),
        )?;
    }

    Ok(repo_dir)
}

fn configure_base_build_command(
    cmd: &mut Command,
    ctx: &BuildContext,
    build_jobs: usize,
) -> Vec<String> {
    let cuda_home_str = ctx.cuda_home.display().to_string();
    let cuda_bin = ctx.cuda_home.join("bin");
    let venv_bin = ctx.venv_dir.join("bin");
    let venv_ninja = venv_bin.join("ninja");
    let nvcc_path = cuda_bin.join("nvcc");
    let current_path = env::var_os("PATH").unwrap_or_default();
    let joined_path = env::join_paths(
        std::iter::once(venv_bin.clone())
            .chain(std::iter::once(cuda_bin.clone()))
            .chain(env::split_paths(&current_path)),
    )
    .unwrap_or(current_path);
    let makeflags = format!("-j{}", build_jobs);
    let mut cmake_args = vec![
        format!("-DCMAKE_CUDA_COMPILER={}", nvcc_path.display()),
        format!("-DCUDAToolkit_ROOT={}", ctx.cuda_home.display()),
    ];

    cmd.env("CUDA_HOME", &cuda_home_str)
        .env("CUDAToolkit_ROOT", &cuda_home_str)
        .env("CUDACXX", &nvcc_path)
        .env("CMAKE_CUDA_COMPILER", &nvcc_path)
        .env("PATH", &joined_path)
        .env("MAX_JOBS", build_jobs.to_string())
        .env("NUM_JOBS", build_jobs.to_string())
        .env("CMAKE_BUILD_PARALLEL_LEVEL", build_jobs.to_string())
        .env("MAKEFLAGS", &makeflags)
        .env("CARGO_BUILD_JOBS", build_jobs.to_string())
        .env("NINJAFLAGS", &makeflags)
        .env("NVCC_THREADS", if build_jobs <= 2 { "1" } else { "2" })
        .env("TORCH_CUDA_ARCH_LIST", &ctx.gpu_arch);

    if ctx.ninja_path.is_some() || venv_ninja.is_file() {
        cmd.env("CMAKE_GENERATOR", "Ninja");
        cmd.env("NINJA", &venv_ninja);
    }

    if let Some(cc) = ctx.preferred_cc() {
        let cc_value = if ctx.ccache_path.is_some() {
            format!("ccache {}", cc.display())
        } else {
            cc.display().to_string()
        };
        cmd.env("CC", cc_value);
    }

    if let Some(cxx) = ctx.preferred_cxx() {
        let cxx_value = if ctx.ccache_path.is_some() {
            format!("ccache {}", cxx.display())
        } else {
            cxx.display().to_string()
        };
        cmd.env("CXX", cxx_value);
    }

    if let Some(host_cxx) = ctx.gxx_path.as_deref() {
        cmd.env("CUDAHOSTCXX", host_cxx);
    }

    if ctx.ccache_path.is_some() {
        cmd.env("CCACHE_DIR", &ctx.ccache_dir)
            .env("CCACHE_COMPRESS", "1")
            .env("CCACHE_COMPILERCHECK", "content");
        cmake_args.push("-DCMAKE_C_COMPILER_LAUNCHER=ccache".to_string());
        cmake_args.push("-DCMAKE_CXX_COMPILER_LAUNCHER=ccache".to_string());
        cmake_args.push("-DCMAKE_CUDA_COMPILER_LAUNCHER=ccache".to_string());
    }

    if ctx.use_clang_toolchain() {
        if let Some(cc) = ctx.clang_path.as_deref() {
            cmake_args.push(format!("-DCMAKE_C_COMPILER={}", cc.display()));
        }
        if let Some(cxx) = ctx.clangxx_path.as_deref() {
            cmake_args.push(format!("-DCMAKE_CXX_COMPILER={}", cxx.display()));
        }
    }

    if let Some(linker) = ctx.preferred_linker_flag() {
        let linker_flag = format!("-fuse-ld={linker}");
        cmd.env("LDFLAGS", &linker_flag);
        cmake_args.push(format!("-DCMAKE_EXE_LINKER_FLAGS={linker_flag}"));
        cmake_args.push(format!("-DCMAKE_SHARED_LINKER_FLAGS={linker_flag}"));
    }

    cmake_args
}

fn run_checked_command(mut cmd: Command, description: &str) -> Result<()> {
    let status = cmd
        .status()
        .map_err(|err| command_error(description, &err.to_string()))?;

    if !status.success() {
        return Err(command_error(
            description,
            &format!("exit code: {}", status.code().unwrap_or(-1)),
        ));
    }

    Ok(())
}

fn prepend_path_env(prefix: &Path, env_name: &str) -> String {
    let existing = env::var_os(env_name)
        .map(|value| env::split_paths(&value).collect::<Vec<_>>())
        .unwrap_or_default();
    env::join_paths(std::iter::once(prefix.to_path_buf()).chain(existing))
        .unwrap_or_else(|_| prefix.into())
        .to_string_lossy()
        .to_string()
}

fn run_venv_python_in_dir_with_jobs(
    ctx: &BuildContext,
    dir: &Path,
    args: &[&str],
    build_jobs: usize,
) -> Result<()> {
    let mut cmd = Command::new(&ctx.venv_python);
    cmd.current_dir(dir).args(args);
    configure_base_build_command(&mut cmd, ctx, build_jobs);
    run_checked_command(cmd, &format!("python {}", args.join(" ")))
}

fn cupy_generate_code(ctx: &BuildContext) -> String {
    let arch = ctx.gpu_arch.replace('.', "");
    format!("arch=compute_{arch},code=sm_{arch};arch=compute_{arch},code=compute_{arch}")
}

fn venv_site_packages_dir(ctx: &BuildContext) -> PathBuf {
    let major_minor = ctx
        .python_version
        .split('.')
        .take(2)
        .collect::<Vec<_>>()
        .join(".");
    ctx.venv_dir
        .join("lib")
        .join(format!("python{major_minor}"))
        .join("site-packages")
}

/// 將已建好的 wheel 裝回 build venv，供後續套件編譯時使用
pub fn install_cached_packages_to_venv(
    ctx: &BuildContext,
    packages: &[CudaPackageId],
) -> Result<()> {
    let wheel_paths = collect_cached_wheel_paths(ctx, packages)?;

    if ctx.uv_available {
        let venv_str = ctx.venv_dir.display().to_string();
        let mut args = vec![
            "pip".to_string(),
            "install".to_string(),
            "--reinstall".to_string(),
            "--no-deps".to_string(),
        ];
        args.extend(wheel_paths);
        let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();

        let status = Command::new("uv")
            .args(&arg_refs)
            .env("VIRTUAL_ENV", &venv_str)
            .status()
            .map_err(|err| command_error("uv pip install", &err.to_string()))?;

        if !status.success() {
            return Err(command_error(
                &format!("uv pip install {}", arg_refs.join(" ")),
                &format!("exit code: {}", status.code().unwrap_or(-1)),
            ));
        }

        Ok(())
    } else {
        let mut args = vec![
            "-m".to_string(),
            "pip".to_string(),
            "install".to_string(),
            "--force-reinstall".to_string(),
            "--no-deps".to_string(),
        ];
        args.extend(wheel_paths);
        let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
        run_venv_python(ctx, &arg_refs)
    }
}

// ============================================================================
// 安裝到使用者環境（非 build venv）
// ============================================================================

/// 從快取強制安裝套件到使用者環境
///
/// 自動偵測目標：
/// - 有 VIRTUAL_ENV → 裝進該 venv
/// - 無 VIRTUAL_ENV → 用 `--user --break-system-packages` 裝到使用者 site-packages
///
/// 套件本體一律直接指定本地 wheel 路徑，避免 pip/uv 重新從索引站挑版本。
pub fn force_install_packages(ctx: &BuildContext, packages: &[CudaPackageId]) -> Result<()> {
    let wheel_paths = collect_cached_wheel_paths(ctx, packages)?;

    let user_has_venv = env::var("VIRTUAL_ENV")
        .ok()
        .filter(|v| !v.is_empty() && *v != ctx.venv_dir.display().to_string())
        .is_some();
    let needs_torch_runtime = packages.iter().any(|pkg| pkg.requires_torch_runtime());

    if ctx.uv_available && user_has_venv {
        let mut args = vec![
            "pip".to_string(),
            "install".to_string(),
            "--reinstall".to_string(),
        ];
        let torch_backend = ctx.torch_backend_tag().to_string();
        if needs_torch_runtime {
            args.push("--torch-backend".to_string());
            args.push(torch_backend);
        }
        args.extend(wheel_paths);
        let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
        run_streaming("uv", &arg_refs)
    } else {
        let mut args = vec![
            "-m".to_string(),
            "pip".to_string(),
            "install".to_string(),
            "--force-reinstall".to_string(),
            "--prefer-binary".to_string(),
        ];

        if user_has_venv {
            // Active user venv: install into that environment directly.
        } else {
            args.push("--user".to_string());
            args.push("--break-system-packages".to_string());
        }

        if needs_torch_runtime {
            args.push("--index-url".to_string());
            args.push(ctx.torch_index_url());
            args.push("--extra-index-url".to_string());
            args.push("https://pypi.org/simple".to_string());
        }

        args.extend(wheel_paths);
        let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();

        if user_has_venv {
            run_venv_python(ctx, &arg_refs)
        } else {
            run_python(ctx, &arg_refs)
        }
    }
}

// ============================================================================
// 內部輔助函式
// ============================================================================

/// 使用 venv 的 python 執行指令（stdout/stderr 直接串流到終端）
fn run_venv_python(ctx: &BuildContext, args: &[&str]) -> Result<()> {
    let cuda_home_str = ctx.cuda_home.display().to_string();
    let status = Command::new(&ctx.venv_python)
        .args(args)
        .env("CUDA_HOME", &cuda_home_str)
        .status()
        .map_err(|err| command_error(&format!("python {}", args.join(" ")), &err.to_string()))?;

    if !status.success() {
        return Err(command_error(
            &format!("python {}", args.join(" ")),
            &format!("exit code: {}", status.code().unwrap_or(-1)),
        ));
    }

    Ok(())
}

fn run_python(ctx: &BuildContext, args: &[&str]) -> Result<()> {
    let cuda_home_str = ctx.cuda_home.display().to_string();
    let status = Command::new(&ctx.python_path)
        .args(args)
        .env("CUDA_HOME", &cuda_home_str)
        .status()
        .map_err(|err| command_error(&format!("python {}", args.join(" ")), &err.to_string()))?;

    if !status.success() {
        return Err(command_error(
            &format!("python {}", args.join(" ")),
            &format!("exit code: {}", status.code().unwrap_or(-1)),
        ));
    }

    Ok(())
}

fn pip_install_to_venv(ctx: &BuildContext, packages: &[&str]) -> Result<()> {
    let mut args = vec!["-m", "pip", "install"];
    args.extend_from_slice(packages);
    run_venv_python(ctx, &args)
}

/// 透過 uv 安裝套件到建構 venv
fn uv_install_to_venv(ctx: &BuildContext, packages: &[&str]) -> Result<()> {
    let venv_str = ctx.venv_dir.display().to_string();
    let mut args = vec!["pip", "install"];
    args.extend_from_slice(packages);

    let status = Command::new("uv")
        .args(&args)
        .env("VIRTUAL_ENV", &venv_str)
        .status()
        .map_err(|err| command_error("uv pip install", &err.to_string()))?;

    if !status.success() {
        return Err(command_error(
            &format!("uv pip install {}", packages.join(" ")),
            &format!("exit code: {}", status.code().unwrap_or(-1)),
        ));
    }

    Ok(())
}

/// 執行外部指令（stdout/stderr 直接串流到終端）
fn run_streaming(program: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(program).args(args).status().map_err(|err| {
        command_error(&format!("{} {}", program, args.join(" ")), &err.to_string())
    })?;

    if !status.success() {
        return Err(command_error(
            &format!("{} {}", program, args.join(" ")),
            &format!("exit code: {}", status.code().unwrap_or(-1)),
        ));
    }

    Ok(())
}

fn command_error(command: &str, message: &str) -> OperationError {
    OperationError::Command {
        command: command.to_string(),
        message: message.to_string(),
    }
}

fn collect_cached_wheel_paths(
    ctx: &BuildContext,
    packages: &[CudaPackageId],
) -> Result<Vec<String>> {
    let wheel_paths: Vec<String> = packages
        .iter()
        .flat_map(|package| cached_wheel_paths(ctx, *package))
        .map(|path| path.display().to_string())
        .collect();

    if wheel_paths.is_empty() {
        let package_names = packages
            .iter()
            .map(|package| package.pip_name())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(command_error(
            &format!("cache lookup {}", package_names),
            "no cached wheel files found",
        ));
    }

    Ok(wheel_paths)
}

fn cached_wheel_paths(ctx: &BuildContext, package: CudaPackageId) -> Vec<PathBuf> {
    let prefix = package.wheel_prefix();

    let Ok(entries) = fs::read_dir(&ctx.wheels_dir) else {
        return Vec::new();
    };

    let mut paths: Vec<PathBuf> = entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(prefix) && name.ends_with(".whl") {
                Some(entry.path())
            } else {
                None
            }
        })
        .collect();

    paths.sort();
    paths
}

/// 檢查路徑是否為 venv 的一部分（用於排除 build venv）
#[allow(dead_code)]
fn is_build_venv(path: &Path, ctx: &BuildContext) -> bool {
    path.starts_with(&ctx.venv_dir)
}

fn venv_uses_python(ctx: &BuildContext) -> bool {
    let Ok(output) = Command::new(&ctx.venv_python)
        .args([
            "-c",
            "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')",
        ])
        .output()
    else {
        return false;
    };

    if !output.status.success() {
        return false;
    }

    let venv_version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let expected = ctx
        .python_version
        .split('.')
        .take(2)
        .collect::<Vec<_>>()
        .join(".");

    venv_version == expected
}
