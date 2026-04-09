//! CUDA ML 套件建構器的型別定義
//!
//! 包含 CudaPackageId、BuildContext 等核心型別

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

/// CUDA ML 套件識別碼
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CudaPackageId {
    // ── PyTorch 核心 ──
    Torch,
    TorchVision,
    TorchAudio,
    // ── Attention / Memory 加速 ──
    FlashAttention,
    XFormers,
    FlashInfer,
    // ── 量化推論 ──
    BitsAndBytes,
    ExLlamaV2,
    AutoGptq,
    AutoAwq,
    // ── C++ 推論引擎 ──
    LlamaCppPython,
    CTranslate2,
    TensorRt,
    // ── 訓練加速 ──
    TransformerEngine,
    DeepSpeed,
    // ── GPU 計算 ──
    Vllm,
    CuPy,
    // ── 純 Python / 包裝層 ──
    Unsloth,
}

impl CudaPackageId {
    /// pip 套件名稱
    pub fn pip_name(self) -> &'static str {
        match self {
            Self::Torch => "torch",
            Self::TorchVision => "torchvision",
            Self::TorchAudio => "torchaudio",
            Self::FlashAttention => "flash-attn",
            Self::XFormers => "xformers",
            Self::FlashInfer => "flashinfer-python",
            Self::BitsAndBytes => "bitsandbytes",
            Self::ExLlamaV2 => "exllamav2",
            Self::AutoGptq => "auto-gptq",
            Self::AutoAwq => "autoawq",
            Self::LlamaCppPython => "llama-cpp-python",
            Self::CTranslate2 => "ctranslate2",
            Self::TensorRt => "tensorrt",
            Self::TransformerEngine => "transformer-engine",
            Self::DeepSpeed => "deepspeed",
            Self::Vllm => "vllm",
            Self::CuPy => "cupy",
            Self::Unsloth => "unsloth",
        }
    }

    /// 顯示名稱
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Torch => "PyTorch",
            Self::TorchVision => "TorchVision",
            Self::TorchAudio => "TorchAudio",
            Self::FlashAttention => "Flash Attention 2",
            Self::XFormers => "xFormers",
            Self::FlashInfer => "FlashInfer (LLM serving kernels)",
            Self::BitsAndBytes => "bitsandbytes (4/8-bit quantization)",
            Self::ExLlamaV2 => "ExLlamaV2 (GPTQ/EXL2 inference)",
            Self::AutoGptq => "AutoGPTQ (GPTQ quantization)",
            Self::AutoAwq => "AutoAWQ (AWQ quantization)",
            Self::LlamaCppPython => "llama-cpp-python (C++ LLM inference)",
            Self::CTranslate2 => "CTranslate2 (C++ Transformer engine)",
            Self::TensorRt => "TensorRT (NVIDIA graph optimizer)",
            Self::TransformerEngine => "Transformer Engine (FP8)",
            Self::DeepSpeed => "DeepSpeed (distributed training)",
            Self::Vllm => "vLLM (inference server)",
            Self::CuPy => "CuPy (GPU NumPy)",
            Self::Unsloth => "Unsloth (LoRA fine-tuning)",
        }
    }

    /// 是否需要先準備 torch 作為建構依賴
    pub fn requires_torch_build_dependency(self) -> bool {
        matches!(
            self,
            Self::TorchVision
                | Self::TorchAudio
                | Self::FlashAttention
                | Self::XFormers
                | Self::FlashInfer
                | Self::BitsAndBytes
                | Self::ExLlamaV2
                | Self::AutoGptq
                | Self::AutoAwq
                | Self::TransformerEngine
                | Self::DeepSpeed
                | Self::Vllm
        )
    }

    pub fn requires_torch_runtime(self) -> bool {
        matches!(
            self,
            Self::Torch
                | Self::TorchVision
                | Self::TorchAudio
                | Self::FlashAttention
                | Self::XFormers
                | Self::FlashInfer
                | Self::BitsAndBytes
                | Self::ExLlamaV2
                | Self::AutoGptq
                | Self::AutoAwq
                | Self::TransformerEngine
                | Self::DeepSpeed
                | Self::Vllm
                | Self::Unsloth
        )
    }

    #[cfg(test)]
    pub fn is_torch_package(self) -> bool {
        matches!(self, Self::Torch | Self::TorchVision | Self::TorchAudio)
    }

    /// wheel 檔案名稱前綴（用於快取掃描）
    pub fn wheel_prefix(self) -> &'static str {
        match self {
            Self::Torch => "torch-",
            Self::TorchVision => "torchvision-",
            Self::TorchAudio => "torchaudio-",
            Self::FlashAttention => "flash_attn-",
            Self::XFormers => "xformers-",
            Self::FlashInfer => "flashinfer",
            Self::BitsAndBytes => "bitsandbytes-",
            Self::ExLlamaV2 => "exllamav2-",
            Self::AutoGptq => "auto_gptq-",
            Self::AutoAwq => "autoawq-",
            Self::LlamaCppPython => "llama_cpp_python-",
            Self::CTranslate2 => "ctranslate2-",
            Self::TensorRt => "tensorrt-",
            Self::TransformerEngine => "transformer_engine-",
            Self::DeepSpeed => "deepspeed-",
            Self::Vllm => "vllm-",
            Self::CuPy => "cupy-",
            Self::Unsloth => "unsloth-",
        }
    }

    /// 建構時需要的額外環境變數
    pub fn build_env_vars(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Self::TransformerEngine => &[("NVTE_FRAMEWORK", "pytorch")],
            Self::DeepSpeed => &[("DS_BUILD_OPS", "1"), ("DS_BUILD_UTILS", "1")],
            Self::AutoGptq => &[("BUILD_CUDA_EXT", "1")],
            Self::LlamaCppPython => &[("CMAKE_ARGS", "-DGGML_CUDA=on")],
            _ => &[],
        }
    }

    pub fn max_parallel_jobs(self) -> usize {
        match self {
            Self::Torch => 5,
            Self::TorchVision
            | Self::TorchAudio
            | Self::LlamaCppPython
            | Self::CTranslate2
            | Self::CuPy => 4,
            Self::FlashAttention
            | Self::XFormers
            | Self::FlashInfer
            | Self::BitsAndBytes
            | Self::ExLlamaV2
            | Self::AutoGptq
            | Self::TransformerEngine
            | Self::DeepSpeed
            | Self::Vllm => 3,
            Self::AutoAwq | Self::TensorRt | Self::Unsloth => 2,
        }
    }

    pub fn estimated_memory_per_job_gb(self) -> f64 {
        match self {
            Self::Torch => 10.0,
            Self::TorchVision
            | Self::TorchAudio
            | Self::LlamaCppPython
            | Self::CTranslate2
            | Self::CuPy => 8.0,
            Self::FlashAttention
            | Self::XFormers
            | Self::FlashInfer
            | Self::BitsAndBytes
            | Self::ExLlamaV2
            | Self::AutoGptq
            | Self::TransformerEngine
            | Self::DeepSpeed
            | Self::Vllm => 12.0,
            Self::AutoAwq | Self::TensorRt | Self::Unsloth => 2.0,
        }
    }

    fn dynamic_headroom_ratio(self) -> f64 {
        match self {
            Self::Torch => 0.20,
            Self::TorchVision
            | Self::TorchAudio
            | Self::LlamaCppPython
            | Self::CTranslate2
            | Self::CuPy => 0.18,
            Self::FlashAttention
            | Self::XFormers
            | Self::FlashInfer
            | Self::BitsAndBytes
            | Self::ExLlamaV2
            | Self::AutoGptq
            | Self::TransformerEngine
            | Self::DeepSpeed
            | Self::Vllm => 0.22,
            Self::AutoAwq | Self::TensorRt | Self::Unsloth => 0.10,
        }
    }
}

/// 所有可用套件
pub const ALL_PACKAGES: [CudaPackageId; 18] = [
    // PyTorch core
    CudaPackageId::Torch,
    CudaPackageId::TorchVision,
    CudaPackageId::TorchAudio,
    // Attention / Memory
    CudaPackageId::FlashAttention,
    CudaPackageId::XFormers,
    CudaPackageId::FlashInfer,
    // Quantization
    CudaPackageId::BitsAndBytes,
    CudaPackageId::ExLlamaV2,
    CudaPackageId::AutoGptq,
    CudaPackageId::AutoAwq,
    // C++ inference
    CudaPackageId::LlamaCppPython,
    CudaPackageId::CTranslate2,
    CudaPackageId::TensorRt,
    // Training
    CudaPackageId::TransformerEngine,
    CudaPackageId::DeepSpeed,
    // GPU compute
    CudaPackageId::Vllm,
    CudaPackageId::CuPy,
    // Pure Python
    CudaPackageId::Unsloth,
];

/// 每個 nvcc 編譯執行緒的預估記憶體用量 (GB)
const MEMORY_PER_CUDA_JOB_GB: f64 = 6.0;
const MAX_JOBS_ENV: &str = "OPS_TOOLS_CUDA_BUILDER_MAX_JOBS";

/// CUDA 建構環境
pub struct BuildContext {
    pub cuda_home: PathBuf,
    pub cuda_version: String,
    pub cache_dir: PathBuf,
    pub wheels_dir: PathBuf,
    pub ccache_dir: PathBuf,
    pub sources_dir: PathBuf,
    pub venv_dir: PathBuf,
    pub venv_python: PathBuf,
    pub python_path: PathBuf,
    pub python_version: String,
    pub uv_available: bool,
    pub ccache_path: Option<PathBuf>,
    pub ninja_path: Option<PathBuf>,
    pub clang_path: Option<PathBuf>,
    pub clangxx_path: Option<PathBuf>,
    pub mold_path: Option<PathBuf>,
    pub lld_path: Option<PathBuf>,
    pub gcc_path: Option<PathBuf>,
    pub gxx_path: Option<PathBuf>,
    /// GPU 計算能力（例: "12.0"），用於 TORCH_CUDA_ARCH_LIST
    pub gpu_arch: String,
    /// CPU 核心數
    pub cpu_count: usize,
    /// 啟動時可用記憶體 (GB)，僅用於顯示系統資訊
    pub available_memory_gb: f64,
    /// 啟動時依 CPU / 記憶體得到的全域平行編譯上限
    pub max_build_jobs: usize,
}

impl BuildContext {
    pub fn detect() -> Option<Self> {
        let home_dir = env::var_os("HOME").map(PathBuf::from)?;
        let cache_dir = home_dir.join(".ml-packages");
        let wheels_dir = cache_dir.join("wheels");
        let ccache_dir = cache_dir.join("ccache");
        let sources_dir = cache_dir.join("sources");
        let venv_dir = cache_dir.join("venv");
        let venv_python = venv_dir.join("bin/python");

        let cuda_home = detect_cuda_home()?;
        let cuda_version = detect_cuda_version(&cuda_home)?;

        let python_path = detect_preferred_python()?;
        let python_version = detect_python_version(&python_path)?;
        let uv_available = find_executable("uv").is_some();
        let ccache_path = find_executable("ccache");
        let ninja_path = find_executable("ninja");
        let clang_path = find_executable("clang");
        let clangxx_path = find_executable("clang++");
        let mold_path = find_executable("mold");
        let lld_path = find_executable("ld.lld");
        let gcc_path = find_executable("gcc");
        let gxx_path = find_executable("g++");
        let gpu_arch = detect_gpu_arch().unwrap_or_else(|| "8.0".to_string());

        let cpu_count = detect_cpu_count();
        let available_memory_gb = detect_available_memory_gb();
        let max_build_jobs = calculate_max_build_jobs(cpu_count, available_memory_gb);

        Some(Self {
            cuda_home,
            cuda_version,
            cache_dir,
            wheels_dir,
            ccache_dir,
            sources_dir,
            venv_dir,
            venv_python,
            python_path,
            python_version,
            uv_available,
            ccache_path,
            ninja_path,
            clang_path,
            clangxx_path,
            mold_path,
            lld_path,
            gcc_path,
            gxx_path,
            gpu_arch,
            cpu_count,
            available_memory_gb,
            max_build_jobs,
        })
    }

    pub fn venv_ready(&self) -> bool {
        self.venv_python.is_file()
    }

    pub fn torch_backend_tag(&self) -> String {
        format!("cu{}", cuda_index_suffix(&self.cuda_version))
    }

    pub fn torch_index_url(&self) -> String {
        format!(
            "https://download.pytorch.org/whl/{}",
            self.torch_backend_tag()
        )
    }

    pub fn optimization_labels(&self) -> Vec<&'static str> {
        let mut labels = Vec::new();

        if self.ccache_path.is_some() {
            labels.push("ccache");
        }
        if self.ninja_path.is_some() {
            labels.push("ninja");
        }
        if self.use_clang_toolchain() {
            labels.push("clang");
        }
        if self.mold_path.is_some() {
            labels.push("mold");
        } else if self.lld_path.is_some() {
            labels.push("lld");
        }

        labels
    }

    pub fn use_clang_toolchain(&self) -> bool {
        self.clang_path.is_some() && self.clangxx_path.is_some()
    }

    pub fn preferred_cc(&self) -> Option<&Path> {
        if self.use_clang_toolchain() {
            self.clang_path.as_deref()
        } else {
            self.gcc_path.as_deref()
        }
    }

    pub fn preferred_cxx(&self) -> Option<&Path> {
        if self.use_clang_toolchain() {
            self.clangxx_path.as_deref()
        } else {
            self.gxx_path.as_deref()
        }
    }

    pub fn preferred_linker_flag(&self) -> Option<&'static str> {
        if self.mold_path.is_some() {
            Some("mold")
        } else if self.lld_path.is_some() {
            Some("lld")
        } else {
            None
        }
    }

    fn current_available_memory_gb(&self) -> f64 {
        detect_available_memory_gb()
    }

    pub fn effective_build_jobs(&self, package: CudaPackageId) -> usize {
        let override_jobs = env::var(MAX_JOBS_ENV)
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|value| *value > 0);

        override_jobs
            .unwrap_or_else(|| {
                let available_memory_gb = self.current_available_memory_gb();
                let max_build_jobs = calculate_max_build_jobs(self.cpu_count, available_memory_gb);
                calculate_package_build_jobs(available_memory_gb, max_build_jobs, package)
            })
            .min(package.max_parallel_jobs())
            .max(1)
    }
}

// ============================================================================
// 內部偵測函式
// ============================================================================

fn detect_cuda_home() -> Option<PathBuf> {
    if let Some(home) = env::var_os("CUDA_HOME") {
        let path = PathBuf::from(home);
        if path.join("bin/nvcc").is_file() {
            return Some(path);
        }
    }

    let candidates = [
        "/usr/local/cuda",
        "/usr/local/cuda-13.0",
        "/usr/local/cuda-13",
        "/usr/local/cuda-12.8",
        "/usr/local/cuda-12.6",
        "/usr/local/cuda-12.4",
        "/usr/local/cuda-12.1",
    ];

    candidates
        .iter()
        .map(PathBuf::from)
        .find(|p| p.join("bin/nvcc").is_file())
}

fn detect_cuda_version(cuda_home: &Path) -> Option<String> {
    let nvcc = cuda_home.join("bin/nvcc");
    let output = Command::new(nvcc).arg("--version").output().ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        if let Some(pos) = line.find("release ") {
            let version_part = &line[pos + 8..];
            if let Some(comma_pos) = version_part.find(',') {
                return Some(version_part[..comma_pos].to_string());
            }
            return Some(version_part.trim().to_string());
        }
    }
    None
}

fn cuda_index_suffix(version: &str) -> String {
    let parts: Vec<&str> = version.split('.').collect();
    let major = parts.first().unwrap_or(&"13");
    let minor = parts.get(1).unwrap_or(&"0");
    format!("{}{}", major, minor)
}

fn find_executable(name: &str) -> Option<PathBuf> {
    let path_var = env::var_os("PATH")?;
    env::split_paths(&path_var)
        .map(|dir| dir.join(name))
        .find(|candidate| candidate.is_file())
}

fn detect_preferred_python() -> Option<PathBuf> {
    ["python3.11", "python3.10", "python3"]
        .iter()
        .find_map(|name| {
            let path = find_executable(name)?;
            let version = detect_python_version(&path)?;

            if version.starts_with("3.11.") || version.starts_with("3.10.") || *name == "python3" {
                Some(path)
            } else {
                None
            }
        })
}

fn detect_python_version(python: &Path) -> Option<String> {
    let output = Command::new(python).arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let version_line = if stdout.trim().is_empty() {
        stderr.trim()
    } else {
        stdout.trim()
    };

    version_line.strip_prefix("Python ").map(|v| v.to_string())
}

// ============================================================================
// GPU 架構偵測
// ============================================================================

/// 偵測 GPU 計算能力（例: "12.0"），用於 TORCH_CUDA_ARCH_LIST
fn detect_gpu_arch() -> Option<String> {
    let output = Command::new("nvidia-smi")
        .args(["--query-gpu=compute_cap", "--format=csv,noheader"])
        .output()
        .ok()?;

    let raw = String::from_utf8_lossy(&output.stdout);
    let cap = raw.trim().lines().next()?.trim().to_string();

    // "12.1" → "12.0"（取 major.0 以匹配 SM 架構）
    let major = cap.split('.').next()?;
    Some(format!("{}.0", major))
}

// ============================================================================
// CPU / Memory 偵測
// ============================================================================

fn detect_cpu_count() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

/// 讀取 /proc/meminfo 取得可用記憶體 (GB)
fn detect_available_memory_gb() -> f64 {
    let content = match std::fs::read_to_string("/proc/meminfo") {
        Ok(c) => c,
        Err(_) => return 8.0, // 無法讀取時的保守預設值
    };

    // 優先使用 MemAvailable（含 reclaimable cache）
    for line in content.lines() {
        if line.starts_with("MemAvailable:") {
            if let Some(gb) = parse_meminfo_kb(line) {
                return gb;
            }
        }
    }

    // Fallback: MemFree + Buffers + Cached
    let mut total_kb: u64 = 0;
    for line in content.lines() {
        if line.starts_with("MemFree:")
            || line.starts_with("Buffers:")
            || (line.starts_with("Cached:") && !line.starts_with("CachedSwap"))
        {
            if let Some(kb) = line
                .split_whitespace()
                .nth(1)
                .and_then(|s| s.parse::<u64>().ok())
            {
                total_kb += kb;
            }
        }
    }

    if total_kb > 0 {
        total_kb as f64 / 1024.0 / 1024.0
    } else {
        8.0
    }
}

fn parse_meminfo_kb(line: &str) -> Option<f64> {
    let kb: u64 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some(kb as f64 / 1024.0 / 1024.0)
}

/// 計算最佳平行編譯數 = min(cpu, memory / 6GB)，至少 1
fn calculate_max_build_jobs(cpu_count: usize, memory_gb: f64) -> usize {
    let mem_jobs = (memory_gb / MEMORY_PER_CUDA_JOB_GB).floor() as usize;
    cpu_count.min(mem_jobs).max(1)
}

fn calculate_package_build_jobs(
    available_memory_gb: f64,
    max_build_jobs: usize,
    package: CudaPackageId,
) -> usize {
    let dynamic_headroom_gb = calculate_dynamic_headroom_gb(available_memory_gb, package);
    let memory_budget_gb = (available_memory_gb - dynamic_headroom_gb).max(0.0);
    let memory_limited_jobs = if memory_budget_gb <= 0.0 {
        1
    } else {
        (memory_budget_gb / package.estimated_memory_per_job_gb()).floor() as usize
    };

    max_build_jobs
        .min(package.max_parallel_jobs())
        .min(memory_limited_jobs.max(1))
        .max(1)
}

fn calculate_dynamic_headroom_gb(available_memory_gb: f64, package: CudaPackageId) -> f64 {
    let ratio_headroom_gb = available_memory_gb * package.dynamic_headroom_ratio();
    let minimum_headroom_gb = package.estimated_memory_per_job_gb() * 0.75;
    ratio_headroom_gb.max(minimum_headroom_gb)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_pip_names_are_valid() {
        for pkg in &ALL_PACKAGES {
            assert!(!pkg.pip_name().is_empty());
            assert!(!pkg.display_name().is_empty());
            assert!(!pkg.wheel_prefix().is_empty());
        }
    }

    #[test]
    fn torch_dependency_helper_matches_expected_packages() {
        let dependent = [
            CudaPackageId::TorchVision,
            CudaPackageId::TorchAudio,
            CudaPackageId::FlashAttention,
            CudaPackageId::XFormers,
            CudaPackageId::FlashInfer,
            CudaPackageId::BitsAndBytes,
            CudaPackageId::ExLlamaV2,
            CudaPackageId::AutoGptq,
            CudaPackageId::AutoAwq,
            CudaPackageId::TransformerEngine,
            CudaPackageId::DeepSpeed,
            CudaPackageId::Vllm,
        ];

        for pkg in dependent {
            assert!(pkg.requires_torch_build_dependency());
        }
    }

    #[test]
    fn torch_package_helper_matches_expected_packages() {
        assert!(CudaPackageId::Torch.is_torch_package());
        assert!(CudaPackageId::TorchVision.is_torch_package());
        assert!(CudaPackageId::TorchAudio.is_torch_package());
        assert!(!CudaPackageId::FlashAttention.is_torch_package());
    }

    #[test]
    fn torch_itself_is_not_a_torch_build_dependency() {
        assert!(!CudaPackageId::Torch.requires_torch_build_dependency());
    }

    #[test]
    fn torch_runtime_helper_matches_expected_packages() {
        assert!(CudaPackageId::Unsloth.requires_torch_runtime());
        assert!(CudaPackageId::Torch.requires_torch_runtime());
        assert!(CudaPackageId::XFormers.requires_torch_runtime());
        assert!(!CudaPackageId::LlamaCppPython.requires_torch_runtime());
        assert!(!CudaPackageId::CTranslate2.requires_torch_runtime());
    }

    #[test]
    fn max_build_jobs_respects_memory() {
        // 4 cores, 6 GB → min(4, 6/6) = 1
        assert_eq!(calculate_max_build_jobs(4, 6.0), 1);
        // 12 cores, 64 GB → min(12, floor(64/6)) = 10
        assert_eq!(calculate_max_build_jobs(12, 64.0), 10);
        // 8 cores, 3 GB → min(8, 1) = 1
        assert_eq!(calculate_max_build_jobs(8, 3.0), 1);
        // 1 core, 0.5 GB → max(min(1, 0), 1) = 1
        assert_eq!(calculate_max_build_jobs(1, 0.5), 1);
    }

    #[test]
    fn package_job_caps_stay_conservative() {
        assert_eq!(CudaPackageId::Torch.max_parallel_jobs(), 5);
        assert_eq!(CudaPackageId::TorchVision.max_parallel_jobs(), 4);
        assert_eq!(CudaPackageId::FlashAttention.max_parallel_jobs(), 3);
        assert_eq!(CudaPackageId::Vllm.max_parallel_jobs(), 3);
        assert_eq!(CudaPackageId::Unsloth.max_parallel_jobs(), 2);
    }

    #[test]
    fn dynamic_headroom_scales_with_current_available_memory() {
        let flash_attn_high = calculate_dynamic_headroom_gb(48.0, CudaPackageId::FlashAttention);
        let flash_attn_low = calculate_dynamic_headroom_gb(24.0, CudaPackageId::FlashAttention);
        let torchvision_headroom = calculate_dynamic_headroom_gb(48.0, CudaPackageId::TorchVision);

        assert!((flash_attn_high - 10.56).abs() < 0.01);
        assert!((flash_attn_low - 9.0).abs() < 0.01);
        assert!((torchvision_headroom - 8.64).abs() < 0.01);
    }

    #[test]
    fn package_jobs_follow_current_available_memory() {
        assert_eq!(
            calculate_package_build_jobs(48.0, 8, CudaPackageId::FlashAttention),
            3
        );
        assert_eq!(
            calculate_package_build_jobs(64.0, 10, CudaPackageId::FlashAttention),
            3
        );
        assert_eq!(
            calculate_package_build_jobs(48.0, 8, CudaPackageId::TorchVision),
            4
        );
        assert_eq!(
            calculate_package_build_jobs(24.0, 4, CudaPackageId::FlashAttention),
            1
        );
        assert_eq!(
            calculate_package_build_jobs(36.0, 6, CudaPackageId::FlashAttention),
            2
        );
        assert_eq!(
            calculate_package_build_jobs(60.0, 8, CudaPackageId::Torch),
            4
        );
    }

    #[test]
    fn all_packages_have_build_strategy() {
        for pkg in &ALL_PACKAGES {
            assert!(!pkg.pip_name().is_empty(), "{:?} missing pip name", pkg);
        }
    }

    #[test]
    fn cuda_index_suffix_parses_versions() {
        assert_eq!(cuda_index_suffix("13.0"), "130");
        assert_eq!(cuda_index_suffix("12.1"), "121");
        assert_eq!(cuda_index_suffix("12.4"), "124");
    }
}
