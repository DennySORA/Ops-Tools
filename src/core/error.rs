use std::fmt;
use std::io;
use crate::i18n::{self, keys};

/// 統一的操作錯誤類型
#[derive(Debug)]
#[allow(dead_code)]
pub enum OperationError {
    /// IO 相關錯誤（檔案讀寫、目錄操作）
    Io { path: String, source: io::Error },

    /// 外部命令執行錯誤
    Command { command: String, message: String },

    /// 配置錯誤（環境變數缺失等）
    Config { key: String, message: String },

    /// 驗證錯誤（輸入不合法）
    Validation(String),

    /// 使用者取消操作
    Cancelled,

    /// 缺少 Cargo.toml
    MissingCargoToml,
}

impl fmt::Display for OperationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(
                    f,
                    "{}",
                    crate::tr!(keys::ERROR_IO, path = path, source = source)
                )
            }
            Self::Command { command, message } => {
                write!(
                    f,
                    "{}",
                    crate::tr!(keys::ERROR_COMMAND, command = command, message = message)
                )
            }
            Self::Config { key, message } => {
                write!(
                    f,
                    "{}",
                    crate::tr!(keys::ERROR_CONFIG, key = key, message = message)
                )
            }
            Self::Validation(msg) => {
                write!(f, "{}", crate::tr!(keys::ERROR_VALIDATION, message = msg))
            }
            Self::Cancelled => write!(f, "{}", i18n::t(keys::ERROR_CANCELLED)),
            Self::MissingCargoToml => write!(
                f,
                "{}",
                i18n::t(keys::RUST_UPGRADER_VALIDATION_MISSING_CARGO)
            ),
        }
    }
}

impl std::error::Error for OperationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<io::Error> for OperationError {
    fn from(err: io::Error) -> Self {
        Self::Io {
            path: String::new(),
            source: err,
        }
    }
}

/// 方便的 Result 別名
pub type Result<T> = std::result::Result<T, OperationError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_io_error() {
        let err = OperationError::Io {
            path: "/test/path".to_string(),
            source: io::Error::new(io::ErrorKind::NotFound, "not found"),
        };
        assert!(err.to_string().contains("/test/path"));
    }

    #[test]
    fn test_display_command_error() {
        let err = OperationError::Command {
            command: "pnpm".to_string(),
            message: "not installed".to_string(),
        };
        assert!(err.to_string().contains("pnpm"));
    }

    #[test]
    fn test_display_config_error() {
        let err = OperationError::Config {
            key: "API_KEY".to_string(),
            message: "missing".to_string(),
        };
        assert!(err.to_string().contains("API_KEY"));
    }
}
