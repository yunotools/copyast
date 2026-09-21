mod analysis;
mod command;
mod config;
mod domain;
mod error;
mod ignore;
pub mod logger;
mod pipeline;
mod service;
mod templates;

pub use command::CopyastCommand;

pub use config::{CopyastConfig, DEFAULT_MAX_FILE_SIZE, PathMode, TokenModel};

pub use analysis::{
    DetectedFramework, DetectedLanguage, DuplicateDetector, Framework, FrameworkDetector, Language,
    LanguageDetector, TokenEstimate, TokenEstimator,
};

pub use domain::{CopyastResult, DuplicateGroup, IncrementalStats, ScanStats, TextFile};

pub use error::CopyastError;

pub use ignore::{IgnoreGenerator, IgnoreTemplate};

pub use service::CopyastService;
