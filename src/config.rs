use std::path::PathBuf;

// Kích thước tối đa mặc định của 1 file: 10 MiB
pub const DEFAULT_MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

// Quyết định đường dẫn ghi trong header của mỗi file
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PathMode {
    // tự động
    //  - input tương đối -> Header tương đối
    //  - input tuyệt đối -> Header tuyệt đối
    #[default]
    Auto,

    Relative,
    Absolute,
}

// Loại tokenizer mà bộ ước lượng token mô phỏng
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TokenModel {
    Cl100k,

    #[default]
    O200k,

    Claude,
    Gemini,
}

impl TokenModel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Cl100k => "cl100k",
            Self::O200k => "o200k",
            Self::Claude => "claude",
            Self::Gemini => "gemini",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CopyastConfig {
    // Toàn bộ file và folder cần đọc rồi gộp chung vào output.
    pub input_paths: Vec<PathBuf>,

    // File hoặc folder nhận kết quả
    pub output_path: PathBuf,

    // Cách hiển thị đường dẫn trong header
    pub path_mode: PathMode,

    // Model dùng để ước lượng token
    pub token_model: TokenModel,

    // Chỉ cập nhật output khi source thay đổi
    pub is_incremental: bool,

    // Chỉ giữ file đầu tiên nếu nội dung bị trùng
    pub should_deduplicate: bool,

    // Chỉ quét và báo cáo, không ghi output
    pub is_dry_run: bool,

    // Có sử dụng các file ignore hay không
    pub should_respect_ignore: bool,

    // Có quét các file/folder ẩn hay không
    pub should_include_hidden: bool,

    // Các tên ignore file bổ sung, ví dụ `.mycompanyignore`.
    pub additional_ignore_file_names: Vec<String>,

    pub max_file_size_bytes: u64,
}

impl CopyastConfig {
    pub fn new(input_paths: Vec<PathBuf>, output_path: impl Into<PathBuf>) -> Self {
        Self {
            input_paths,
            output_path: output_path.into(),
            path_mode: PathMode::Auto,
            token_model: TokenModel::O200k,
            is_incremental: false,
            should_deduplicate: false,
            is_dry_run: false,
            should_respect_ignore: true,
            should_include_hidden: false,
            additional_ignore_file_names: Vec::new(),
            max_file_size_bytes: DEFAULT_MAX_FILE_SIZE,
        }
    }

    pub fn with_path_mode(mut self, path_mode: PathMode) -> Self {
        self.path_mode = path_mode;
        self
    }

    pub fn with_token_model(mut self, token_model: TokenModel) -> Self {
        self.token_model = token_model;
        self
    }

    pub fn with_incremental(mut self, is_enabled: bool) -> Self {
        self.is_incremental = is_enabled;
        self
    }

    pub fn with_deduplication(mut self, is_enabled: bool) -> Self {
        self.should_deduplicate = is_enabled;
        self
    }

    pub fn with_dry_run(mut self, is_enabled: bool) -> Self {
        self.is_dry_run = is_enabled;
        self
    }

    pub fn with_ignore_files(mut self, is_enabled: bool) -> Self {
        self.should_respect_ignore = is_enabled;
        self
    }

    pub fn with_hidden_files(mut self, is_enabled: bool) -> Self {
        self.should_include_hidden = is_enabled;
        self
    }

    pub fn with_additional_ignore_file_names(mut self, file_names: Vec<String>) -> Self {
        self.additional_ignore_file_names = file_names;
        self
    }

    pub fn with_max_file_size_bytes(mut self, max_file_size_bytes: u64) -> Self {
        self.max_file_size_bytes = max_file_size_bytes;
        self
    }
}
