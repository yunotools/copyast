use std::path::PathBuf;

// File text đã được Copyast đọc thành công
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextFile {
    pub path: PathBuf,
    // nội dung UTF-8
    pub content: String,
}

impl TextFile {
    pub fn new(path: impl Into<PathBuf>, content: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            content: content.into(),
        }
    }

    pub fn count_bytes(&self) -> u64 {
        u64::try_from(self.content.len()).unwrap_or(u64::MAX)
    }
}

// Thống kê các stat khi scan file
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScanStats {
    pub discovered_files: usize,
    pub copied_files: usize,
    pub ignored_files: usize,
    pub binary_files: usize,
    pub unreadable_files: usize,
    pub oversized_files: usize,
}

impl ScanStats {
    // tổng số file bỏ qua
    pub fn count_skipped_files(&self) -> usize {
        self.ignored_files
            .saturating_add(self.binary_files)
            .saturating_add(self.unreadable_files)
            .saturating_add(self.oversized_files)
    }
}

// Nhóm file có nội dung giống hệt nhau
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateGroup {
    pub file_paths: Vec<PathBuf>,

    // số byte tiết kệm được nếu loại các bản sao
    pub redundant_bytes: u64,
}

// Kết quả của Incremental mode
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IncrementalStats {
    pub changed_files: usize,
    pub unchanged_files: usize,
    pub removed_files: usize,

    // Những cấu hình ảnh hưởng tới nội dung output
    // có thay đổi so với lần chạy trước hay không
    pub is_configuration_changed: bool,

    pub is_output_written: bool,
}

// Kết quả đầy đủ của 1 lần chạy Copyast
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopyastResult {
    pub output_path: PathBuf,
    pub scan_stats: ScanStats,
    pub total_bytes: u64,
    pub estimated_tokens: u64,
    pub detected_languages: Vec<String>,
    pub detected_frameworks: Vec<String>,
    pub duplicate_groups: Vec<DuplicateGroup>,
    pub incremental_stats: Option<IncrementalStats>,
}
