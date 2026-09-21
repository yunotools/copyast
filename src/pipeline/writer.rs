use crate::config::{CopyastConfig, PathMode};
use crate::domain::TextFile;
use crate::pipeline::file_ops::{
    append_suffix, create_parent_directory, replace_file, resolve_absolute_path,
};
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

const COPYAST_BANNER: &str = "=== Yunotools-Copyast ===";

pub(crate) fn write_output(files: &[TextFile], config: &CopyastConfig) -> io::Result<()> {
    create_parent_directory(&config.output_path)?;

    // Ghi ra file tạm trước để output cũ không bị dở dang nếu có lỗi.
    let temporary_output = build_temporary_path(&config.output_path);

    if let Err(error) = write_files_to_path(files, config, &temporary_output) {
        let _ = fs::remove_file(&temporary_output);
        return Err(error);
    }

    if let Err(error) = replace_file(&temporary_output, &config.output_path) {
        let _ = fs::remove_file(&temporary_output);
        return Err(error);
    }

    Ok(())
}

pub(crate) fn build_temporary_path(output_path: &Path) -> PathBuf {
    append_suffix(output_path, ".copyast-tmp")
}

// Dùng chung cho Writer và TokenEstimator để hai module tính cùng một header.
pub(crate) fn render_header(file: &TextFile, config: &CopyastConfig) -> String {
    let displayed_path = resolve_header_path(&file.path, config);

    format!(
        "{COPYAST_BANNER}\n=== FILE: {} ===\n",
        displayed_path.display(),
    )
}

fn write_files_to_path(
    files: &[TextFile],
    config: &CopyastConfig,
    output_path: &Path,
) -> io::Result<()> {
    let output_file = File::create(output_path)?;
    let mut writer = BufWriter::new(output_file);

    for (file_index, file) in files.iter().enumerate() {
        // Chèn một dòng trống giữa hai file.
        if file_index > 0 {
            writeln!(writer)?;
        }

        writer.write_all(render_header(file, config).as_bytes())?;
        writer.write_all(file.content.as_bytes())?;

        // Bảo đảm file tiếp theo luôn bắt đầu trên một dòng mới.
        if !file.content.ends_with('\n') {
            writeln!(writer)?;
        }
    }

    writer.flush()?;
    writer.get_ref().sync_all()
}

fn resolve_header_path(file_path: &Path, config: &CopyastConfig) -> PathBuf {
    let should_use_absolute_path = match config.path_mode {
        PathMode::Absolute => true,
        PathMode::Relative => false,
        // Với nhiều input, chỉ cần một input tuyệt đối là toàn bộ header dùng
        // đường dẫn tuyệt đối để định dạng kết quả luôn nhất quán.
        PathMode::Auto => config
            .input_paths
            .iter()
            .any(|input_path| input_path.is_absolute()),
    };

    if should_use_absolute_path {
        return resolve_absolute_path(file_path);
    }

    if let [input_path] = config.input_paths.as_slice() {
        return resolve_single_input_header_path(file_path, input_path);
    }

    // Với nhiều input, dùng gốc chung để giữ tên thư mục nguồn trong header.
    // Ví dụ `frontend/src/app.ts` và `backend/src/main.rs` không cùng bị rút gọn
    // thành `src/...`, tránh gây nhầm lẫn hoặc trùng đường dẫn cho AI.
    let absolute_file_path = resolve_absolute_path(file_path);

    common_input_root(config)
        .and_then(|root| {
            absolute_file_path
                .strip_prefix(root)
                .map(Path::to_path_buf)
                .ok()
        })
        .unwrap_or(absolute_file_path)
}

fn resolve_single_input_header_path(file_path: &Path, input_path: &Path) -> PathBuf {
    let input_root = if input_path.is_file() {
        input_path.parent().unwrap_or(Path::new("."))
    } else {
        input_path
    };

    if let Ok(relative_path) = file_path.strip_prefix(input_root) {
        return relative_path.to_path_buf();
    }

    let absolute_file_path = resolve_absolute_path(file_path);
    let absolute_input_root = resolve_absolute_path(input_root);

    absolute_file_path
        .strip_prefix(absolute_input_root)
        .map(Path::to_path_buf)
        .unwrap_or_else(|_| file_path.to_path_buf())
}

fn common_input_root(config: &CopyastConfig) -> Option<PathBuf> {
    let roots = config
        .input_paths
        .iter()
        .map(|input_path| {
            let root = if input_path.is_file() {
                input_path.parent().unwrap_or(Path::new("."))
            } else {
                input_path.as_path()
            };

            resolve_absolute_path(root)
        })
        .collect::<Vec<_>>();

    let first_root = roots.first()?;
    let first_components = first_root.components().collect::<Vec<_>>();
    let mut common_length = first_components.len();

    for root in roots.iter().skip(1) {
        let components = root.components().collect::<Vec<_>>();
        common_length = common_length.min(components.len());

        for component_index in 0..common_length {
            if !components_equal(
                first_components[component_index].as_os_str(),
                components[component_index].as_os_str(),
            ) {
                common_length = component_index;
                break;
            }
        }
    }

    if common_length == 0 {
        return None;
    }

    let mut common_root = PathBuf::new();
    for component in first_components.into_iter().take(common_length) {
        common_root.push(component.as_os_str());
    }

    Some(common_root)
}

fn components_equal(first: &std::ffi::OsStr, second: &std::ffi::OsStr) -> bool {
    if cfg!(windows) {
        first
            .to_string_lossy()
            .eq_ignore_ascii_case(&second.to_string_lossy())
    } else {
        first == second
    }
}
