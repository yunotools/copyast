use std::io;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CopyastError {
    #[error("command error: {0}")]
    Command(String),

    // #[from] giúp toán tử ? tự chuyển:
    // io::Error -> CopyastError::Io
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error(
        "input path does not exist: {}",
        .path.display()
    )]
    InputNotFound { path: PathBuf },

    #[error(
        "input path must be a regular file or directory: {}",
        .path.display()
    )]
    UnsupportedInputType { path: PathBuf },

    #[error("output path cannot be empty")]
    EmptyOutputPath,

    #[error("at least one input path is required")]
    NoInputPaths,

    #[error(
        "input and output cannot be the same file: {}",
        .path.display()
    )]
    InputOutputConflict { path: PathBuf },

    #[error(
        "output path points to a directory: {}",
        .path.display()
    )]
    OutputIsDirectory { path: PathBuf },

    #[error("unknown ignore template: {name}")]
    UnknownIgnoreTemplate { name: String },
}
