use crate::config::{CopyastConfig, PathMode};
use crate::domain::{IncrementalStats, TextFile};
use crate::pipeline::file_ops::{append_suffix, create_parent_directory, replace_file};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

const CACHE_VERSION: &str = "copyast-cache-v3";

// Nếu format mà Writer tạo ra thay đổi,
// ta tăng version này để cache cũ mất hiệu lực
const OUTPUT_FORMAT_VERSION: &str = "1";

// Đại diện ngắn gọn cho trạng thái nội dung của một file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FileFingerprint {
    size: u64,
    content_hash: u64,
}

// Trạng thái được đọc từ cache của lần chạy trước
struct CacheState {
    configuration_hash: u64,
    files: BTreeMap<u64, FileFingerprint>,
}

// Gom các số liệu so sánh vào một kiểu có tên để tránh nhầm thứ tự của tuple.
struct FileChangeCounts {
    changed_files: usize,
    unchanged_files: usize,
    removed_files: usize,
}

pub(crate) struct IncrementalTracker {
    cache_path: PathBuf,

    // Output có tồn tại trước khi chạy Copyast hay không
    is_output_available: bool,

    // Cache cũ có tồn tại và đúng format hay không
    is_cache_valid: bool,

    // Fingerprint của những config ảnh hưởng tới output
    configuration_hash: u64,

    // Trạng thái của các file trong lần quét hiện tại
    current_fingerprints: BTreeMap<u64, FileFingerprint>,

    stats: IncrementalStats,
}

impl IncrementalTracker {
    // So sánh danh sách file hiện tại với cache của lần chạy trước.
    pub(crate) fn analyze(config: &CopyastConfig, files: &[TextFile]) -> io::Result<Self> {
        let output_path = &config.output_path;
        let cache_path = Self::build_cache_path(output_path);

        // Dùng BTreeMap thay cho HashMap
        // - giúp nội dung cache luôn có thứ tự ổn định
        // -> Cache dễ kiểm tra bằng mắt
        // -> Cache không thay đổi thứ tự vô ích giữa các lần chạy
        // -> Kết quả ổn định mà không cần gọi sort() riêng
        // Lưu ý: BTreeMap không ngăn được hash collision
        // Nếu hai đường dẫn tạo ra cùng một path_id, chúng vẫn được xem là cùng khóa
        // Việc chọn BTreeMap chỉ liên quan đến cách lưu và sắp xếp khóa
        let previous_state = load_cache(&cache_path)?;
        let is_cache_valid = previous_state.is_some();

        let configuration_hash = hash_configuration(config);

        let is_configuration_changed = previous_state
            .as_ref()
            .map(|state| state.configuration_hash != configuration_hash)
            .unwrap_or(false);

        let current_fingerprints = fingerprint_files(files);

        let change_counts = compare_file_states(
            previous_state.as_ref(),
            &current_fingerprints,
            is_configuration_changed,
        );

        Ok(Self {
            cache_path,
            is_output_available: output_path.is_file(),
            is_cache_valid,
            configuration_hash,
            current_fingerprints,
            stats: IncrementalStats {
                changed_files: change_counts.changed_files,
                unchanged_files: change_counts.unchanged_files,
                removed_files: change_counts.removed_files,
                is_configuration_changed,

                // Chỉ chuyển thành true sau khi Writer thực sự ghi file.
                is_output_written: false,
            },
        })
    }

    // Cho biết output có cần được tạo hoặc cập nhật hay không
    pub(crate) fn should_update_output(&self) -> bool {
        !self.is_output_available
            || !self.is_cache_valid
            || self.stats.is_configuration_changed
            || self.stats.changed_files > 0
            || self.stats.removed_files > 0
    }

    // Trả về kết quả thống kê
    pub(crate) fn stats(&self) -> IncrementalStats {
        self.stats.clone()
    }

    // Lưu trạng thái hiện tại để lần chạy sau có thể so sánh
    // Chỉ nên gọi hàm này sau khi Writer ghi output thành công
    pub(crate) fn save_cache(&self) -> io::Result<()> {
        create_parent_directory(&self.cache_path)?;

        // Không ghi trực tiếp vào cache chính.
        // Nếu chương trình dừng giữa chừng, cache cũ
        // sẽ không bị biến thành một file dở dang
        let temporary_cache_path = append_suffix(&self.cache_path, ".tmp");

        if let Err(error) = write_cache(
            &temporary_cache_path,
            self.configuration_hash,
            &self.current_fingerprints,
        ) {
            let _ = fs::remove_file(&temporary_cache_path);
            return Err(error);
        }

        if let Err(error) = replace_file(&temporary_cache_path, &self.cache_path) {
            let _ = fs::remove_file(&temporary_cache_path);
            return Err(error);
        }

        Ok(())
    }

    // Tạo đường dẫn cache dựa trên đường dẫn output
    // Ví dụ:
    // context.txt → context.txt.copyast-cache
    pub(crate) fn build_cache_path(output_path: impl AsRef<Path>) -> PathBuf {
        append_suffix(output_path.as_ref(), ".copyast-cache")
    }

    // Scanner sử dụng đường dẫn này để không đọc
    // nhầm cache đang được ghi dở
    pub(crate) fn build_temporary_cache_path(output_path: impl AsRef<Path>) -> PathBuf {
        let cache_path = Self::build_cache_path(output_path);

        append_suffix(&cache_path, ".tmp")
    }
}

fn write_cache(
    path: &Path,
    configuration_hash: u64,
    files: &BTreeMap<u64, FileFingerprint>,
) -> io::Result<()> {
    let cache_file = File::create(path)?;
    let mut writer = BufWriter::new(cache_file);

    writeln!(writer, "{CACHE_VERSION}")?;
    writeln!(writer, "config\t{configuration_hash}")?;

    // Lưu số lượng file để phát hiện cache bị ghi thiếu giữa chừng.
    writeln!(writer, "count\t{}", files.len())?;

    for (path_id, fingerprint) in files {
        writeln!(
            writer,
            "file\t{path_id}\t{}\t{}",
            fingerprint.size, fingerprint.content_hash,
        )?;
    }

    writer.flush()?;
    writer.get_ref().sync_all()
}

// So sánh trạng thái file hiện tại với cache cũ
fn compare_file_states(
    previous_state: Option<&CacheState>,
    current_fingerprints: &BTreeMap<u64, FileFingerprint>,
    is_configuration_changed: bool,
) -> FileChangeCounts {
    let Some(previous_state) = previous_state else {
        // Không có cache:
        // tất cả file hiện tại được xem là changed
        return FileChangeCounts {
            changed_files: current_fingerprints.len(),
            unchanged_files: 0,
            removed_files: 0,
        };
    };

    let mut removed_files = 0_usize;

    // File tồn tại trong cache nhưng không còn
    // trong lần quét hiện tại được xem là removed
    for path_id in previous_state.files.keys() {
        if !current_fingerprints.contains_key(path_id) {
            removed_files += 1;
        }
    }

    // Khi config render thay đổi, tất cả file
    // cần được ghi lại dù source không thay đổi
    if is_configuration_changed {
        return FileChangeCounts {
            changed_files: current_fingerprints.len(),
            unchanged_files: 0,
            removed_files,
        };
    }

    let mut changed_files = 0_usize;
    let mut unchanged_files = 0_usize;

    for (path_id, current_fingerprint) in current_fingerprints {
        match previous_state.files.get(path_id) {
            Some(previous_fingerprint) if previous_fingerprint == current_fingerprint => {
                unchanged_files += 1;
            }

            _ => {
                changed_files += 1;
            }
        }
    }

    FileChangeCounts {
        changed_files,
        unchanged_files,
        removed_files,
    }
}

// Chuyển danh sách TextFile thành:
// path_id → FileFingerprint
fn fingerprint_files(files: &[TextFile]) -> BTreeMap<u64, FileFingerprint> {
    let mut fingerprints = BTreeMap::new();

    for file in files {
        let normalized_path = normalize_path(&file.path);

        let path_id = hash_stably(normalized_path.as_bytes());

        let fingerprint = FileFingerprint {
            size: file.count_bytes(),
            content_hash: hash_stably(file.content.as_bytes()),
        };

        fingerprints.insert(path_id, fingerprint);
    }

    fingerprints
}

// Đọc cache của lần chạy trước
// Nếu cache chưa tồn tại hoặc khác version
// ta coi như chưa có dữ liệu incremental
fn load_cache(cache_path: &Path) -> io::Result<Option<CacheState>> {
    let bytes = match fs::read(cache_path) {
        Ok(bytes) => bytes,

        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(None);
        }

        Err(error) => {
            return Err(error);
        }
    };

    // Cache không phải UTF-8 được xem là cache hỏng
    let Ok(content) = String::from_utf8(bytes) else {
        return Ok(None);
    };

    let mut lines = content.lines();

    if lines.next() != Some(CACHE_VERSION) {
        return Ok(None);
    }

    let Some(configuration_line) = lines.next() else {
        return Ok(None);
    };

    let Some(configuration_hash_text) = configuration_line.strip_prefix("config\t") else {
        return Ok(None);
    };

    let Ok(configuration_hash) = configuration_hash_text.parse::<u64>() else {
        return Ok(None);
    };

    let Some(count_line) = lines.next() else {
        return Ok(None);
    };

    let Some(expected_file_count_text) = count_line.strip_prefix("count\t") else {
        return Ok(None);
    };

    let Ok(expected_file_count) = expected_file_count_text.parse::<usize>() else {
        return Ok(None);
    };

    let mut fingerprints = BTreeMap::new();

    for line in lines {
        if line.is_empty() {
            continue;
        }

        let mut parts = line.split('\t');

        if parts.next() != Some("file") {
            return Ok(None);
        }

        let Some(path_id_text) = parts.next() else {
            return Ok(None);
        };

        let Some(size_text) = parts.next() else {
            return Ok(None);
        };

        let Some(content_hash_text) = parts.next() else {
            return Ok(None);
        };

        // Một dòng hợp lệ chỉ có đúng bốn cột:
        // file, path_id, size và content_hash
        if parts.next().is_some() {
            return Ok(None);
        }

        let Ok(path_id) = path_id_text.parse::<u64>() else {
            return Ok(None);
        };

        let Ok(size) = size_text.parse::<u64>() else {
            return Ok(None);
        };

        let Ok(content_hash) = content_hash_text.parse::<u64>() else {
            return Ok(None);
        };

        let replaced_fingerprint =
            fingerprints.insert(path_id, FileFingerprint { size, content_hash });

        // Hai dòng có cùng path_id được xem
        // là dấu hiệu cache không hợp lệ
        if replaced_fingerprint.is_some() {
            return Ok(None);
        }
    }

    // Nếu số file không đúng với header,
    // cache có thể đã bị ghi thiếu
    if fingerprints.len() != expected_file_count {
        return Ok(None);
    }

    Ok(Some(CacheState {
        configuration_hash,
        files: fingerprints,
    }))
}

// Tạo fingerprint cho những config thực sự
// ảnh hưởng tới nội dung file output
fn hash_configuration(config: &CopyastConfig) -> u64 {
    let path_mode = match config.path_mode {
        PathMode::Auto => "auto",
        PathMode::Relative => "relative",
        PathMode::Absolute => "absolute",
    };

    let normalized_inputs = config
        .input_paths
        .iter()
        .map(|input_path| normalize_path(input_path))
        .collect::<Vec<_>>()
        .join("\ninput=");

    let configuration = format!(
        concat!(
            "output-format={}\n",
            "input={}\n",
            "path-mode={}\n",
            "deduplicate={}"
        ),
        OUTPUT_FORMAT_VERSION, normalized_inputs, path_mode, config.should_deduplicate,
    );

    hash_stably(configuration.as_bytes())
}

// Chuẩn hóa đường dẫn trước khi hash
// Windows chấp nhận cả "\" và "/" và thường không phân biệt
// chữ hoa/chữ thường, nên ta chuẩn hóa để kết quả ổn định hơn
fn normalize_path(path: &Path) -> String {
    let mut normalized = path.to_string_lossy().replace('\\', "/");

    if cfg!(windows) {
        normalized.make_ascii_lowercase();
    }

    normalized
}

// Thuật toán FNV-1a 64-bit
// Hash này cần ổn định giữa nhiều lần chạy chương trình,
// nhưng không được dùng cho mật khẩu hoặc mục đích bảo mật
fn hash_stably(bytes: &[u8]) -> u64 {
    // 14695981039346656037, 1099511628211 là các hằng số tiêu chuẩn của FNV-1a 64-bit
    // Ta không nên tự ý thay hai hằng số này nếu vẫn muốn gọi hàm là FNV-1a 64-bit.

    // Đây là giá trị khởi đầu của hash.
    // Nếu bắt đầu bằng 0: let mut hash = 0;
    // thì các byte 0 ở đầu dữ liệu có thể khiến trạng thái hash tiếp tục bằng 0:
    // - 0 XOR 0 = 0
    // - 0 × PRIME = 0
    // Giá trị khởi đầu khác 0 giúp thuật toán xử lý tốt hơn với:
    // - Dữ liệu rỗng
    // - Dữ liệu ngắn
    // - Dữ liệu có nhiều byte 0
    // - Những chuỗi có tiền tố tương tự nhau
    const OFFSET_BASIS: u64 = 0xcbf29ce484222325; // tương đương 14695981039346656037

    // Sau khi trộn một byte vào hash, ta nhân với số nguyên tố:
    // hash = hash.wrapping_mul(PRIME);
    // Phép nhân này làm cho thay đổi từ một byte lan sang nhiều bit trong kết quả
    // Ví dụ, hai chuỗi chỉ khác một ký tự:
    // src/main.rs
    // src/Main.rs
    // sẽ có kết quả hash rất khác nhau. PRIME là một số lẻ.
    // Vì vậy nó không có ước chung với 2^64, giúp phép nhân modulo 2^64 không dễ làm mất thông tin như khi nhân với một số chẵn.
    const PRIME: u64 = 0x100000001b3; // tương đương 1099511628211

    let mut hash = OFFSET_BASIS;

    // bytes có kiểu: &[u8]
    //  - mỗi byte được lấy ra có kiểu: &u8
    //  - chuyển u8 thành u64: u64::from(*byte)
    //  - *byte lấy giá trị u8 ra khỏi reference: &u8 -> u8
    //  - sau đó u64::from(...) chuyển nó thành u64:
    //  - u8 -> u64
    // XOR byte vào hash
    //  - hash ^= u64::from(*byte); -> là dạng viết ngắn của: hash = hash ^ u64::from(*byte);
    //  - phép XOR trộn byte hiện tại vào trạng thái hash
    // Nhân với FNV prime
    //  - hash = hash.wrapping_mul(PRIME);
    //  - u64 chỉ chứa tối đa: 18_446_744_073_709_551_615
    //  - Phép nhân có thể vượt giới hạn này. Với thuật toán hash, việc tràn số là có chủ ý. wrapping_mul() thực hiện phép nhân theo modulo 2^64:
    //  - Nó làm giá trị quay vòng thay vì panic.
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }

    // FNV-1a thực hiện:
    // Tương ứng:
    //  - hash ^= byte;
    //  - hash = hash.wrapping_mul(PRIME);
    // Biến thể FNV-1 cũ làm ngược lại:
    //  - Nhân trước
    //  - XOR sau

    // Cache cần hash giống nhau qua nhiều lần chạy chương trình.
    // Ta tự triển khai FNV với:
    // - Hằng số cố định.
    // - Phép tính cố định.
    // - Kết quả ổn định.
    // - Không phụ thuộc seed ngẫu nhiên.
    // Trong khi đó, chi tiết thuật toán của DefaultHasher
    // không được Rust cam kết sẽ giữ nguyên mãi mãi giữa các phiên bản.
    // Tuy nhiên, FNV-1a chỉ phù hợp với cache và phát hiện thay đổi thông thường. Nó không phù hợp cho:
    // - Mật khẩu.
    // - Chữ ký số.
    // - Kiểm tra dữ liệu chống giả mạo.
    // - Input do kẻ tấn công chủ động tạo.
    hash
}
