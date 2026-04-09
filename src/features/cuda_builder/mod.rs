//! CUDA ML 套件建構器
//!
//! 從原始碼重建 CUDA 加速的 ML 套件（Flash Attention 2、xFormers、PyTorch）
//! 自動管理 uv venv 隔離建構環境，避免 PEP 668 限制。
//!
//! 目錄結構：
//!   ~/.ml-packages/wheels/  — 快取的 .whl 檔案
//!   ~/.ml-packages/venv/    — 建構用隔離環境

mod builder;
mod types;

use crate::i18n::{self, keys};
use crate::ui::{Console, Prompts};
use types::{ALL_PACKAGES, BuildContext, CudaPackageId};

pub fn run() {
    let console = Console::new();
    let prompts = Prompts::new();

    console.header(i18n::t(keys::CUDA_BUILDER_HEADER));

    // 偵測 CUDA 環境
    console.info(i18n::t(keys::CUDA_BUILDER_DETECTING));
    let Some(ctx) = BuildContext::detect() else {
        console.error(i18n::t(keys::CUDA_BUILDER_CUDA_NOT_FOUND));
        return;
    };

    console.success(&crate::tr!(
        keys::CUDA_BUILDER_CUDA_FOUND,
        version = ctx.cuda_version,
        path = ctx.cuda_home.display()
    ));
    console.info(&crate::tr!(
        keys::CUDA_BUILDER_CACHE_DIR,
        path = ctx.cache_dir.display()
    ));
    console.info(&crate::tr!(
        keys::CUDA_BUILDER_GPU_ARCH,
        arch = ctx.gpu_arch
    ));
    console.info(&crate::tr!(
        keys::CUDA_BUILDER_SYSTEM_INFO,
        cpu = ctx.cpu_count,
        memory = format!("{:.1}", ctx.available_memory_gb),
        jobs = ctx.max_build_jobs
    ));
    let optimizations = ctx.optimization_labels();
    if !optimizations.is_empty() {
        console.info(&crate::tr!(
            keys::CUDA_BUILDER_OPTIMIZATIONS,
            optimizations = optimizations.join(", ")
        ));
    }
    console.blank_line();

    let options = vec![
        i18n::t(keys::CUDA_BUILDER_MODE_BUILD),
        i18n::t(keys::CUDA_BUILDER_MODE_INSTALL),
        i18n::t(keys::CUDA_BUILDER_MODE_STATUS),
        i18n::t(keys::CUDA_BUILDER_MODE_CLEAN),
    ];

    let Some(selection) = prompts.select(i18n::t(keys::CUDA_BUILDER_SELECT_MODE), &options) else {
        console.warning(i18n::t(keys::CUDA_BUILDER_CANCELLED));
        return;
    };

    match selection {
        0 => run_build(&console, &prompts, &ctx),
        1 => run_install(&console, &prompts, &ctx),
        2 => run_status(&console, &ctx),
        3 => run_clean(&console, &prompts, &ctx),
        _ => unreachable!(),
    }
}

/// 建構模式：自動建立 venv，並將選取套件從原始碼重建為 wheels
fn run_build(console: &Console, prompts: &Prompts, ctx: &BuildContext) {
    let items: Vec<String> = ALL_PACKAGES
        .iter()
        .map(|pkg| {
            let cached = !builder::scan_cached_wheels(ctx, *pkg).is_empty();
            let status = if cached {
                i18n::t(keys::CUDA_BUILDER_STATUS_CACHED)
            } else {
                i18n::t(keys::CUDA_BUILDER_STATUS_NOT_CACHED)
            };
            format!("{} {}", pkg.display_name(), status)
        })
        .collect();

    let defaults: Vec<bool> = ALL_PACKAGES
        .iter()
        .map(|pkg| builder::scan_cached_wheels(ctx, *pkg).is_empty())
        .collect();

    let selected = prompts.multi_select(
        i18n::t(keys::CUDA_BUILDER_SELECT_PACKAGES),
        &items,
        &defaults,
    );

    if selected.is_empty() {
        console.info(i18n::t(keys::CUDA_BUILDER_NO_SELECTION));
        return;
    }

    let selected_packages: Vec<CudaPackageId> =
        selected.iter().map(|&idx| ALL_PACKAGES[idx]).collect();

    // 確保快取目錄存在
    if let Err(err) = builder::ensure_cache_dir(ctx) {
        console.error(&err.to_string());
        return;
    }

    // 自動建立 / 確認建構用 venv
    console.info(i18n::t(keys::CUDA_BUILDER_CREATING_VENV));
    if let Err(err) = builder::ensure_venv(ctx) {
        console.error(&err.to_string());
        return;
    }
    console.success(i18n::t(keys::CUDA_BUILDER_VENV_READY));

    // 安裝建構工具到 venv
    console.info(i18n::t(keys::CUDA_BUILDER_ENSURING_BUILD_TOOLS));
    if let Err(err) = builder::ensure_build_tools(ctx) {
        console.warning(&err.to_string());
    }

    let mut success_count = 0;
    let mut failed_count = 0;

    let torch_selected = selected_packages.contains(&CudaPackageId::Torch);
    let dependent_packages: Vec<CudaPackageId> = selected_packages
        .iter()
        .copied()
        .filter(|pkg| *pkg != CudaPackageId::Torch && pkg.requires_torch_build_dependency())
        .collect();
    let independent_packages: Vec<CudaPackageId> = selected_packages
        .iter()
        .copied()
        .filter(|pkg| *pkg != CudaPackageId::Torch && !pkg.requires_torch_build_dependency())
        .collect();

    let mut torch_ready_for_dependents = dependent_packages.is_empty();
    let mut dependent_failures_counted = false;

    if torch_selected {
        if build_package(console, ctx, CudaPackageId::Torch) {
            success_count += 1;

            if !dependent_packages.is_empty() {
                console.blank_line();
                console.info(i18n::t(keys::CUDA_BUILDER_INSTALLING_TORCH_DEP));
                match builder::install_cached_packages_to_venv(ctx, &[CudaPackageId::Torch]) {
                    Ok(()) => torch_ready_for_dependents = true,
                    Err(err) => {
                        console.error(&crate::tr!(
                            keys::CUDA_BUILDER_BUILD_FAILED,
                            package = "torch (build dependency)"
                        ));
                        console.error(&err.to_string());
                        failed_count += dependent_packages.len();
                        dependent_failures_counted = true;
                    }
                }
            }
        } else {
            failed_count += 1;
        }
    }

    if !torch_selected && !dependent_packages.is_empty() {
        console.blank_line();
        console.info(i18n::t(keys::CUDA_BUILDER_INSTALLING_TORCH_DEP));
        let cached_torch_available =
            !builder::scan_cached_wheels(ctx, CudaPackageId::Torch).is_empty();

        if cached_torch_available {
            match builder::install_cached_packages_to_venv(ctx, &[CudaPackageId::Torch]) {
                Ok(()) => torch_ready_for_dependents = true,
                Err(_) => {
                    if !build_package(console, ctx, CudaPackageId::Torch) {
                        console.error(&crate::tr!(
                            keys::CUDA_BUILDER_BUILD_FAILED,
                            package = "torch (build dependency)"
                        ));
                        failed_count += dependent_packages.len();
                        dependent_failures_counted = true;
                    } else {
                        match builder::install_cached_packages_to_venv(ctx, &[CudaPackageId::Torch])
                        {
                            Ok(()) => torch_ready_for_dependents = true,
                            Err(err) => {
                                console.error(&crate::tr!(
                                    keys::CUDA_BUILDER_BUILD_FAILED,
                                    package = "torch (build dependency)"
                                ));
                                console.error(&err.to_string());
                                failed_count += dependent_packages.len();
                                dependent_failures_counted = true;
                            }
                        }
                    }
                }
            }
        } else if !build_package(console, ctx, CudaPackageId::Torch) {
            console.error(&crate::tr!(
                keys::CUDA_BUILDER_BUILD_FAILED,
                package = "torch (build dependency)"
            ));
            failed_count += dependent_packages.len();
            dependent_failures_counted = true;
        } else {
            match builder::install_cached_packages_to_venv(ctx, &[CudaPackageId::Torch]) {
                Ok(()) => torch_ready_for_dependents = true,
                Err(err) => {
                    console.error(&crate::tr!(
                        keys::CUDA_BUILDER_BUILD_FAILED,
                        package = "torch (build dependency)"
                    ));
                    console.error(&err.to_string());
                    failed_count += dependent_packages.len();
                    dependent_failures_counted = true;
                }
            }
        }
    }

    if !torch_ready_for_dependents && !dependent_packages.is_empty() && !dependent_failures_counted
    {
        failed_count += dependent_packages.len();
    }

    if torch_ready_for_dependents {
        for pkg in dependent_packages {
            if build_package(console, ctx, pkg) {
                success_count += 1;
            } else {
                failed_count += 1;
            }
        }
    }

    for pkg in independent_packages {
        if build_package(console, ctx, pkg) {
            success_count += 1;
        } else {
            failed_count += 1;
        }
    }

    console.show_summary(
        i18n::t(keys::CUDA_BUILDER_SUMMARY),
        success_count,
        failed_count,
    );
}

fn build_package(console: &Console, ctx: &BuildContext, package: CudaPackageId) -> bool {
    console.blank_line();
    console.info(&crate::tr!(
        keys::CUDA_BUILDER_BUILDING_PACKAGE,
        package = package.display_name()
    ));

    match builder::build_source_package(ctx, package) {
        Ok(()) => {
            console.success(&crate::tr!(
                keys::CUDA_BUILDER_BUILD_SUCCESS,
                package = package.display_name()
            ));
            true
        }
        Err(err) => {
            console.error(&crate::tr!(
                keys::CUDA_BUILDER_BUILD_FAILED,
                package = package.display_name()
            ));
            console.error(&err.to_string());
            false
        }
    }
}

/// 安裝模式：從快取強制安裝已建構的套件到使用者環境
fn run_install(console: &Console, prompts: &Prompts, ctx: &BuildContext) {
    let cached: Vec<(CudaPackageId, Vec<String>)> = ALL_PACKAGES
        .iter()
        .map(|pkg| (*pkg, builder::scan_cached_wheels(ctx, *pkg)))
        .filter(|(_, wheels)| !wheels.is_empty())
        .collect();

    if cached.is_empty() {
        console.warning(i18n::t(keys::CUDA_BUILDER_NO_CACHED));
        return;
    }

    let items: Vec<String> = cached
        .iter()
        .map(|(pkg, wheels)| format!("{} ({})", pkg.display_name(), wheels[0]))
        .collect();
    let defaults = vec![true; items.len()];

    let selected = prompts.multi_select(
        i18n::t(keys::CUDA_BUILDER_SELECT_INSTALL),
        &items,
        &defaults,
    );

    if selected.is_empty() {
        console.info(i18n::t(keys::CUDA_BUILDER_NO_SELECTION));
        return;
    }

    let packages: Vec<CudaPackageId> = selected.iter().map(|&idx| cached[idx].0).collect();

    console.blank_line();
    console.info(i18n::t(keys::CUDA_BUILDER_INSTALLING));
    match builder::force_install_packages(ctx, &packages) {
        Ok(()) => {
            console.success(i18n::t(keys::CUDA_BUILDER_INSTALL_SUCCESS));
        }
        Err(err) => {
            console.error(i18n::t(keys::CUDA_BUILDER_INSTALL_FAILED));
            console.error(&err.to_string());
        }
    }
}

/// 狀態模式：顯示快取內容
fn run_status(console: &Console, ctx: &BuildContext) {
    console.blank_line();
    console.info(i18n::t(keys::CUDA_BUILDER_CACHE_STATUS));
    console.blank_line();

    let mut has_cached = false;
    for pkg in &ALL_PACKAGES {
        let wheels = builder::scan_cached_wheels(ctx, *pkg);
        if wheels.is_empty() {
            console.list_item(
                "○",
                &format!(
                    "{}: {}",
                    pkg.display_name(),
                    i18n::t(keys::CUDA_BUILDER_STATUS_NOT_CACHED)
                ),
            );
        } else {
            has_cached = true;
            for wheel in &wheels {
                console.success_item(&format!("{}: {}", pkg.display_name(), wheel));
            }
        }
    }

    if !has_cached {
        console.blank_line();
        console.warning(i18n::t(keys::CUDA_BUILDER_CACHE_EMPTY));
    }
}

/// 清理模式：刪除快取目錄
fn run_clean(console: &Console, prompts: &Prompts, ctx: &BuildContext) {
    if !ctx.wheels_dir.exists() {
        console.info(i18n::t(keys::CUDA_BUILDER_CACHE_EMPTY));
        return;
    }

    run_status(console, ctx);
    console.blank_line();

    if !prompts.confirm(i18n::t(keys::CUDA_BUILDER_CONFIRM_CLEAN)) {
        console.info(i18n::t(keys::CUDA_BUILDER_CANCELLED));
        return;
    }

    match builder::clean_cache(ctx) {
        Ok(()) => console.success(i18n::t(keys::CUDA_BUILDER_CLEAN_SUCCESS)),
        Err(err) => console.error(&err.to_string()),
    }
}
