pub mod error;
pub mod config;
pub mod path_utils;
pub mod result;
pub mod traits;

pub use error::{OperationError, Result};
pub use config::{AppConfig, load_config, save_config};
pub use result::{OperationResult, OperationStats, OperationType};
pub use traits::{FileCleaner, FileScanner};
