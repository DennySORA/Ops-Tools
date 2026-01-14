//! 套件管理器的型別定義
//!
//! 包含 PackageAction、PackageId、SupportedOs 等核心型別

use crate::i18n::{self, keys};
use std::env;
use std::path::PathBuf;

use super::shell::is_command_available;

// ============================================================================
// 作業系統與套件管理器
// ============================================================================

/// 支援的作業系統
#[derive(Clone, Copy, Debug)]
pub enum SupportedOs {
    Linux,
    Macos,
}

impl SupportedOs {
    /// 偵測目前作業系統
    pub fn detect() -> Option<Self> {
        match env::consts::OS {
            "linux" => Some(Self::Linux),
            "macos" => Some(Self::Macos),
            _ => None,
        }
    }

    /// 取得顯示用標籤
    pub fn label(self) -> &'static str {
        match self {
            Self::Linux => "Linux",
            Self::Macos => "macOS",
        }
    }

    /// 取得 Go 語言風格的 OS 識別碼
    pub fn go_os(self) -> &'static str {
        match self {
            Self::Linux => "linux",
            Self::Macos => "darwin",
        }
    }

    /// 取得 kubectl 風格的 OS 識別碼
    pub fn kubectl_os(self) -> &'static str {
        self.go_os()
    }
}

/// 系統套件管理器
#[derive(Clone, Copy, Debug)]
pub enum PackageManager {
    Brew,
    Apt,
    Dnf,
    Yum,
    Pacman,
    Zypper,
    Apk,
}

impl PackageManager {
    /// 偵測系統套件管理器
    pub fn detect(os: SupportedOs) -> Option<Self> {
        match os {
            SupportedOs::Macos => {
                if is_command_available("brew").is_some() {
                    Some(Self::Brew)
                } else {
                    None
                }
            }
            SupportedOs::Linux => {
                if is_command_available("apt-get").is_some() {
                    Some(Self::Apt)
                } else if is_command_available("dnf").is_some() {
                    Some(Self::Dnf)
                } else if is_command_available("yum").is_some() {
                    Some(Self::Yum)
                } else if is_command_available("pacman").is_some() {
                    Some(Self::Pacman)
                } else if is_command_available("zypper").is_some() {
                    Some(Self::Zypper)
                } else if is_command_available("apk").is_some() {
                    Some(Self::Apk)
                } else {
                    None
                }
            }
        }
    }
}

// ============================================================================
// 套件操作
// ============================================================================

/// 套件操作類型
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackageAction {
    Install,
    Update,
    Remove,
}

impl PackageAction {
    /// 取得顯示用標籤
    pub fn label(self) -> &'static str {
        match self {
            Self::Install => i18n::t(keys::PACKAGE_MANAGER_ACTION_INSTALL),
            Self::Update => i18n::t(keys::PACKAGE_MANAGER_ACTION_UPDATE),
            Self::Remove => i18n::t(keys::PACKAGE_MANAGER_ACTION_REMOVE),
        }
    }
}

/// 套件識別碼
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackageId {
    Nvm,
    Pnpm,
    Rust,
    Go,
    Terraform,
    Kubectl,
    Kubectx,
    K9s,
    Git,
    Uv,
    Tmux,
    Vim,
    Ffmpeg,
}

/// 套件定義
#[derive(Clone, Copy, Debug)]
pub struct PackageDefinition {
    pub id: PackageId,
    pub name: &'static str,
}

/// 取得所有套件定義
pub fn package_definitions() -> Vec<PackageDefinition> {
    vec![
        PackageDefinition {
            id: PackageId::Nvm,
            name: "nvm",
        },
        PackageDefinition {
            id: PackageId::Pnpm,
            name: "pnpm",
        },
        PackageDefinition {
            id: PackageId::Rust,
            name: "Rust",
        },
        PackageDefinition {
            id: PackageId::Go,
            name: "Go",
        },
        PackageDefinition {
            id: PackageId::Terraform,
            name: "Terraform",
        },
        PackageDefinition {
            id: PackageId::Kubectl,
            name: "kubectl",
        },
        PackageDefinition {
            id: PackageId::Kubectx,
            name: "kubectx",
        },
        PackageDefinition {
            id: PackageId::K9s,
            name: "k9s",
        },
        PackageDefinition {
            id: PackageId::Git,
            name: "git",
        },
        PackageDefinition {
            id: PackageId::Uv,
            name: "uv",
        },
        PackageDefinition {
            id: PackageId::Tmux,
            name: "tmux",
        },
        PackageDefinition {
            id: PackageId::Vim,
            name: "vim",
        },
        PackageDefinition {
            id: PackageId::Ffmpeg,
            name: "ffmpeg",
        },
    ]
}

// ============================================================================
// 操作上下文
// ============================================================================

/// 操作上下文，儲存執行時狀態
pub struct ActionContext {
    pub(crate) os: SupportedOs,
    pub(crate) package_manager: Option<PackageManager>,
    pub(crate) sudo_available: bool,
    pub(crate) home_dir: PathBuf,
    pub(crate) temp_dir: PathBuf,
    pub(crate) apt_updated: bool,
    pub(crate) pacman_synced: bool,
    pub(crate) hashicorp_repo_ready: bool,
}

impl ActionContext {
    /// 建立新的操作上下文
    pub fn new(os: SupportedOs) -> Self {
        let home_dir = env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        let temp_dir = env::temp_dir();
        let package_manager = PackageManager::detect(os);
        let sudo_available = is_command_available("sudo").is_some();

        Self {
            os,
            package_manager,
            sudo_available,
            home_dir,
            temp_dir,
            apt_updated: false,
            pacman_synced: false,
            hashicorp_repo_ready: false,
        }
    }

    /// 取得作業系統
    #[allow(dead_code)]
    pub fn os(&self) -> SupportedOs {
        self.os
    }

    /// 取得 home 目錄
    #[allow(dead_code)]
    pub fn home_dir(&self) -> &PathBuf {
        &self.home_dir
    }

    /// 取得暫存目錄
    #[allow(dead_code)]
    pub fn temp_dir(&self) -> &PathBuf {
        &self.temp_dir
    }

    /// 是否有 sudo 權限
    #[allow(dead_code)]
    pub fn has_sudo(&self) -> bool {
        self.sudo_available
    }
}
