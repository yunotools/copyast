//! Nhận diện ngôn ngữ từ extension, shebang và marker file.

use crate::domain::TextFile;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::Path;

// Cần Ord vì BTreeMap và BTreeSet cần biết cách sắp xếp key:
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    Java,
    Kotlin,
    C,
    Cpp,
    CSharp,
    Php,
    Ruby,
    Swift,
    Dart,
    Shell,
    PowerShell,
    Lua,
    Elixir,
    Haskell,
    Scala,
    Terraform,
    Html,
    Css,
    Sql,
    Dockerfile,
}

// Cho phép chuyển Language thành text bằng `to_string()`.
impl fmt::Display for Language {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display_name = match self {
            Self::Rust => "Rust",
            Self::Python => "Python",
            Self::JavaScript => "JavaScript",
            Self::TypeScript => "TypeScript",
            Self::Go => "Go",
            Self::Java => "Java",
            Self::Kotlin => "Kotlin",
            Self::C => "C",
            Self::Cpp => "C++",
            Self::CSharp => "C#",
            Self::Php => "PHP",
            Self::Ruby => "Ruby",
            Self::Swift => "Swift",
            Self::Dart => "Dart",
            Self::Shell => "Shell",
            Self::PowerShell => "PowerShell",
            Self::Lua => "Lua",
            Self::Elixir => "Elixir",
            Self::Haskell => "Haskell",
            Self::Scala => "Scala",
            Self::Terraform => "Terraform",
            Self::Html => "HTML",
            Self::Css => "CSS",
            Self::Sql => "SQL",
            Self::Dockerfile => "Dockerfile",
        };

        formatter.write_str(display_name)
    }
}

// Thông tin chi tiết về một ngôn ngữ được phát hiện.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedLanguage {
    pub language: Language,

    // Số file thuộc ngôn ngữ này.
    pub file_count: usize,

    // Độ tin cậy từ 0 đến 100.
    pub confidence: u8,

    // Có tìm thấy file đặc trưng của ngôn ngữ hay không.
    pub has_marker: bool,
}

// Dữ liệu tạm dùng trong quá trình tính toán.
#[derive(Default)]
struct LanguageEvidence {
    file_count: usize,
    has_marker: bool,
}

pub struct LanguageDetector;

impl LanguageDetector {
    // Nhận diện trên nhiều gốc dự án nhưng vẫn tổng hợp thành một báo cáo chung.
    pub fn analyze_many(roots: &[&Path], files: &[TextFile]) -> Vec<DetectedLanguage> {
        // Dùng BTreeMap để lưu dữ liệu theo dạng key-value:
        // Rust       → 20 file, có Cargo.toml
        // TypeScript → 10 file, có tsconfig.json
        // Python     → 2 file, không có marker
        let mut evidence_by_language = BTreeMap::<Language, LanguageEvidence>::new();

        // Nhận diện bằng phần mở rộng và shebang
        for file in files {
            let Some(language) = detect_file_language(file) else {
                continue;
            };

            evidence_by_language.entry(language).or_default().file_count += 1;
        }

        // Nhận diện bằng các file đặc trưng
        // BTreeSet cũng tự loại các giá trị trùng.
        // Nếu cả pyproject.toml và requirements.txt tồn tại, Python vẫn chỉ xuất hiện một lần.
        for root in roots {
            for language in find_marker_languages(root) {
                evidence_by_language.entry(language).or_default().has_marker = true;
            }
        }

        let total_recognized_files = evidence_by_language
            .values()
            .map(|evidence| evidence.file_count)
            .sum::<usize>()
            .max(1);

        let mut detected_languages = evidence_by_language
            .into_iter()
            .map(|(language, language_evidence)| DetectedLanguage {
                language,
                file_count: language_evidence.file_count,
                confidence: calculate_confidence(
                    language_evidence.file_count,
                    total_recognized_files,
                    language_evidence.has_marker,
                ),
                has_marker: language_evidence.has_marker,
            })
            .collect::<Vec<_>>();

        // Ngôn ngữ có confidence cao nhất -> đứng trước
        detected_languages.sort_by(|left, right| {
            right
                .confidence
                .cmp(&left.confidence)
                .then_with(|| right.file_count.cmp(&left.file_count))
        });

        detected_languages
    }
}

// Nhận diện ngôn ngữ của một TextFile.
fn detect_file_language(file: &TextFile) -> Option<Language> {
    detect_path_language(&file.path).or_else(|| detect_shebang_language(&file.content))
}

// Nhận diện bằng tên file hoặc phần mở rộng.
fn detect_path_language(path: &Path) -> Option<Language> {
    let file_name = path.file_name()?.to_string_lossy().to_ascii_lowercase();

    if file_name == "dockerfile" || file_name.starts_with("dockerfile.") {
        return Some(Language::Dockerfile);
    }

    let extension = path.extension()?.to_string_lossy().to_ascii_lowercase();

    match extension.as_str() {
        "rs" => Some(Language::Rust),

        "py" | "pyw" | "pyi" => Some(Language::Python),

        "js" | "jsx" | "mjs" | "cjs" => Some(Language::JavaScript),

        "ts" | "tsx" | "mts" | "cts" => Some(Language::TypeScript),

        "go" => Some(Language::Go),

        "java" => Some(Language::Java),

        "kt" | "kts" => Some(Language::Kotlin),

        "c" | "h" => Some(Language::C),

        "cc" | "cpp" | "cxx" | "hpp" | "hxx" => Some(Language::Cpp),

        "cs" => Some(Language::CSharp),

        "php" => Some(Language::Php),

        "rb" => Some(Language::Ruby),

        "swift" => Some(Language::Swift),

        "dart" => Some(Language::Dart),

        "sh" | "bash" | "zsh" | "fish" => Some(Language::Shell),

        "ps1" | "psm1" | "psd1" => Some(Language::PowerShell),

        "lua" => Some(Language::Lua),

        "ex" | "exs" => Some(Language::Elixir),

        "hs" | "lhs" => Some(Language::Haskell),

        "scala" | "sc" => Some(Language::Scala),

        "tf" | "tfvars" => Some(Language::Terraform),

        "html" | "htm" => Some(Language::Html),

        "css" | "scss" | "sass" | "less" => Some(Language::Css),

        "sql" => Some(Language::Sql),

        _ => None,
    }
}

// Nhận diện script không có extension bằng shebang.
// Ví dụ:
// #!/usr/bin/env python3
// #!/usr/bin/env bash
fn detect_shebang_language(content: &str) -> Option<Language> {
    let first_line = content.lines().next()?;

    if !first_line.starts_with("#!") {
        return None;
    }

    let shebang = first_line.to_ascii_lowercase();

    if shebang.contains("python") {
        Some(Language::Python)
    } else if shebang.contains("node") {
        Some(Language::JavaScript)
    } else if shebang.contains("bash")
        || shebang.contains("zsh")
        || shebang.contains("fish")
        || shebang.ends_with("/sh")
    {
        Some(Language::Shell)
    } else if shebang.contains("ruby") {
        Some(Language::Ruby)
    } else {
        None
    }
}

// Tìm ngôn ngữ dựa trên các file đặc trưng
// BTreeSet cũng tự loại các giá trị trùng.
// Nếu cả pyproject.toml và requirements.txt tồn tại, Python vẫn chỉ xuất hiện một lần.
fn find_marker_languages(root: &Path) -> BTreeSet<Language> {
    let mut languages = BTreeSet::new();

    let markers = [
        ("Cargo.toml", Language::Rust),
        ("pyproject.toml", Language::Python),
        ("requirements.txt", Language::Python),
        ("package.json", Language::JavaScript),
        ("tsconfig.json", Language::TypeScript),
        ("go.mod", Language::Go),
        ("pom.xml", Language::Java),
        ("build.gradle", Language::Java),
        ("build.gradle.kts", Language::Kotlin),
        ("Gemfile", Language::Ruby),
        ("composer.json", Language::Php),
        ("pubspec.yaml", Language::Dart),
        ("Package.swift", Language::Swift),
        ("mix.exs", Language::Elixir),
        ("stack.yaml", Language::Haskell),
        ("main.tf", Language::Terraform),
        ("Dockerfile", Language::Dockerfile),
    ];

    for (file_name, language) in markers {
        if root.join(file_name).exists() {
            languages.insert(language);
        }
    }

    languages
}

// Tính confidence dựa trên:
// - tỉ lệ file của ngôn ngữ;
// - sự xuất hin của marker
fn calculate_confidence(file_count: usize, total_files: usize, has_marker: bool) -> u8 {
    const FILE_EVIDENCE_WEIGHT: usize = 70;
    const MARKER_EVIDENCE_WEIGHT: usize = 30;

    // Bảo vệ dữ liệu đầu vào
    // Về logic, file_count không nên lớn hơn total_files
    let normalized_file_count = file_count.min(total_files);

    // saturating_mul() ngăn tràn số, checked_div() xử lý total_files = 0.
    let file_score = normalized_file_count
        .saturating_mul(FILE_EVIDENCE_WEIGHT)
        .checked_div(total_files)
        .unwrap_or(0);

    let marker_score = if has_marker {
        MARKER_EVIDENCE_WEIGHT
    } else {
        0
    };

    let final_score = file_score.saturating_add(marker_score).min(100);

    // u8::try_from() là phép chuyển kiểu có kiểm tra
    u8::try_from(final_score).unwrap_or(100)
}
