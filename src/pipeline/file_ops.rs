//! Các thao tác đường dẫn và ghi file dùng chung trong pipeline.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

// Thêm suffix vào cuối đường dẫn mà không làm mất extension.
// Ví dụ: context.txt + ".copyast-tmp" -> context.txt.copyast-tmp.
pub(crate) fn append_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut suffixed_path = path.as_os_str().to_os_string();
    suffixed_path.push(suffix);
    PathBuf::from(suffixed_path)
}

// canonicalize() xử lý relative path, ".", ".." và symbolic link.
// Khi output chưa tồn tại, ta ghép relative path với current directory.
pub(crate) fn resolve_absolute_path(path: &Path) -> PathBuf {
    if let Ok(canonical_path) = path.canonicalize() {
        return normalize_windows_verbatim_path(canonical_path);
    }

    if path.is_absolute() {
        return path.to_path_buf();
    }

    std::env::current_dir()
        .map(|current_directory| current_directory.join(path))
        .unwrap_or_else(|_| path.to_path_buf())
}

// canonicalize() trên Windows thường trả về dạng `\\?\C:\...`.
// Prefix này hữu ích cho Windows API nhưng không thân thiện khi hiển thị trong header
#[cfg(windows)]
fn normalize_windows_verbatim_path(path: PathBuf) -> PathBuf {
    use std::path::{Component, Prefix};

    let mut components = path.components();
    let Some(Component::Prefix(prefix_component)) = components.next() else {
        return path;
    };

    let mut normalized_path = match prefix_component.kind() {
        Prefix::VerbatimDisk(drive_letter) => {
            PathBuf::from(format!("{}:\\", char::from(drive_letter)))
        }
        Prefix::VerbatimUNC(server, share) => {
            let mut unc_path = PathBuf::from(r"\\");
            unc_path.push(server);
            unc_path.push(share);
            unc_path
        }
        _ => return path,
    };

    for component in components {
        if let Component::Normal(path_segment) = component {
            normalized_path.push(path_segment);
        }
    }

    normalized_path
}

#[cfg(not(windows))]
fn normalize_windows_verbatim_path(path: PathBuf) -> PathBuf {
    path
}

// Trên Windows, đường dẫn thường không phân biệt chữ hoa/chữ thường.
pub(crate) fn are_paths_equal(first_path: &Path, second_path: &Path) -> bool {
    normalized_path_key(first_path) == normalized_path_key(second_path)
}

// Tạo khóa ổn định để phát hiện cùng một file được tìm thấy từ nhiều input
// khác nhau, ví dụ `./src/main.rs` và `C:\project\src\main.rs`.
pub(crate) fn normalized_path_key(path: &Path) -> String {
    let mut normalized = resolve_absolute_path(path)
        .to_string_lossy()
        .replace('\\', "/");

    if cfg!(windows) {
        normalized.make_ascii_lowercase();
    }

    normalized
}

// Ví dụ: exports/ai/context.txt sẽ tạo folder exports/ai nếu cần.
pub(crate) fn create_parent_directory(path: &Path) -> io::Result<()> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };

    if parent.as_os_str().is_empty() {
        return Ok(());
    }

    fs::create_dir_all(parent)
}

// Trên Unix, rename có thể thay thế file đích.
// Trên Windows, file đích cần được xóa trước.
pub(crate) fn replace_file(source: &Path, destination: &Path) -> io::Result<()> {
    #[cfg(windows)]
    if destination.exists() {
        fs::remove_file(destination)?;
    }

    fs::rename(source, destination)
}
