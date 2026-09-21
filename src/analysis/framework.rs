//! Nhận diện framework từ marker file và dependency manifest.

use crate::domain::TextFile;
use std::fmt;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Framework {
    React,
    NextJs,
    Vue,
    Nuxt,
    Angular,
    Svelte,
    SvelteKit,
    Express,
    NestJs,
    Electron,
    Tauri,
    Django,
    Flask,
    FastApi,
    ActixWeb,
    Axum,
    Rocket,
    Gin,
    Fiber,
    SpringBoot,
    Ktor,
    Laravel,
    Symfony,
    Rails,
    Flutter,
}

impl fmt::Display for Framework {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display_name = match self {
            Self::React => "React",
            Self::NextJs => "Next.js",
            Self::Vue => "Vue",
            Self::Nuxt => "Nuxt",
            Self::Angular => "Angular",
            Self::Svelte => "Svelte",
            Self::SvelteKit => "SvelteKit",
            Self::Express => "Express",
            Self::NestJs => "NestJS",
            Self::Electron => "Electron",
            Self::Tauri => "Tauri",
            Self::Django => "Django",
            Self::Flask => "Flask",
            Self::FastApi => "FastAPI",
            Self::ActixWeb => "Actix Web",
            Self::Axum => "Axum",
            Self::Rocket => "Rocket",
            Self::Gin => "Gin",
            Self::Fiber => "Fiber",
            Self::SpringBoot => "Spring Boot",
            Self::Ktor => "Ktor",
            Self::Laravel => "Laravel",
            Self::Symfony => "Symfony",
            Self::Rails => "Ruby on Rails",
            Self::Flutter => "Flutter",
        };

        formatter.write_str(display_name)
    }
}

// Kết quả chi tiết của một framework được phát hiện
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedFramework {
    pub framework: Framework,

    // Có tìm thấy file đặc trưng của framework không
    pub has_marker: bool,

    // Có tìm thấy framework trong dependency manifest không
    pub has_dependency: bool,
}

// Quy tắc nhận diện của một framework
struct FrameworkRule {
    framework: Framework,

    // Các file đặc trưng, ví dụ next.config.js
    marker_files: &'static [&'static str],

    // Các file khai báo dependency cần kiểm tra
    manifest_files: &'static [&'static str],

    // Các chuỗi dependency cần tìm
    dependency_patterns: &'static [&'static str],
}

const FRAMEWORK_RULES: &[FrameworkRule] = &[
    FrameworkRule {
        framework: Framework::React,
        marker_files: &[],
        manifest_files: &["package.json"],
        dependency_patterns: &["\"react\""],
    },
    FrameworkRule {
        framework: Framework::NextJs,
        marker_files: &["next.config.js", "next.config.mjs", "next.config.ts"],
        manifest_files: &["package.json"],
        dependency_patterns: &["\"next\""],
    },
    FrameworkRule {
        framework: Framework::Vue,
        marker_files: &[],
        manifest_files: &["package.json"],
        dependency_patterns: &["\"vue\""],
    },
    FrameworkRule {
        framework: Framework::Nuxt,
        marker_files: &["nuxt.config.js", "nuxt.config.ts"],
        manifest_files: &["package.json"],
        dependency_patterns: &["\"nuxt\""],
    },
    FrameworkRule {
        framework: Framework::Angular,
        marker_files: &["angular.json"],
        manifest_files: &["package.json"],
        dependency_patterns: &["\"@angular/core\""],
    },
    FrameworkRule {
        framework: Framework::Svelte,
        marker_files: &[],
        manifest_files: &["package.json"],
        dependency_patterns: &["\"svelte\""],
    },
    FrameworkRule {
        framework: Framework::SvelteKit,
        marker_files: &["svelte.config.js"],
        manifest_files: &["package.json"],
        dependency_patterns: &["\"@sveltejs/kit\""],
    },
    FrameworkRule {
        framework: Framework::Express,
        marker_files: &[],
        manifest_files: &["package.json"],
        dependency_patterns: &["\"express\""],
    },
    FrameworkRule {
        framework: Framework::NestJs,
        marker_files: &["nest-cli.json"],
        manifest_files: &["package.json"],
        dependency_patterns: &["\"@nestjs/core\""],
    },
    FrameworkRule {
        framework: Framework::Electron,
        marker_files: &[],
        manifest_files: &["package.json"],
        dependency_patterns: &["\"electron\""],
    },
    FrameworkRule {
        framework: Framework::Tauri,
        marker_files: &["tauri.conf.json", "src-tauri/tauri.conf.json"],
        manifest_files: &["Cargo.toml", "package.json"],
        dependency_patterns: &["tauri", "\"@tauri-apps/api\""],
    },
    FrameworkRule {
        framework: Framework::Django,
        marker_files: &["manage.py"],
        manifest_files: &["requirements.txt", "pyproject.toml", "Pipfile"],
        dependency_patterns: &["django"],
    },
    FrameworkRule {
        framework: Framework::Flask,
        marker_files: &[],
        manifest_files: &["requirements.txt", "pyproject.toml", "Pipfile"],
        dependency_patterns: &["flask"],
    },
    FrameworkRule {
        framework: Framework::FastApi,
        marker_files: &[],
        manifest_files: &["requirements.txt", "pyproject.toml", "Pipfile"],
        dependency_patterns: &["fastapi"],
    },
    FrameworkRule {
        framework: Framework::ActixWeb,
        marker_files: &[],
        manifest_files: &["Cargo.toml"],
        dependency_patterns: &["actix-web"],
    },
    FrameworkRule {
        framework: Framework::Axum,
        marker_files: &[],
        manifest_files: &["Cargo.toml"],
        dependency_patterns: &["axum"],
    },
    FrameworkRule {
        framework: Framework::Rocket,
        marker_files: &[],
        manifest_files: &["Cargo.toml"],
        dependency_patterns: &["rocket"],
    },
    FrameworkRule {
        framework: Framework::Gin,
        marker_files: &[],
        manifest_files: &["go.mod"],
        dependency_patterns: &["github.com/gin-gonic/gin"],
    },
    FrameworkRule {
        framework: Framework::Fiber,
        marker_files: &[],
        manifest_files: &["go.mod"],
        dependency_patterns: &["github.com/gofiber/fiber"],
    },
    FrameworkRule {
        framework: Framework::SpringBoot,
        marker_files: &[],
        manifest_files: &["pom.xml", "build.gradle", "build.gradle.kts"],
        dependency_patterns: &["spring-boot"],
    },
    FrameworkRule {
        framework: Framework::Ktor,
        marker_files: &[],
        manifest_files: &["build.gradle", "build.gradle.kts"],
        dependency_patterns: &["io.ktor", "ktor-server"],
    },
    FrameworkRule {
        framework: Framework::Laravel,
        marker_files: &["artisan"],
        manifest_files: &["composer.json"],
        dependency_patterns: &["laravel/framework"],
    },
    FrameworkRule {
        framework: Framework::Symfony,
        marker_files: &["symfony.lock", "bin/console"],
        manifest_files: &["composer.json"],
        dependency_patterns: &["symfony/framework-bundle"],
    },
    FrameworkRule {
        framework: Framework::Rails,
        marker_files: &["config/application.rb"],
        manifest_files: &["Gemfile"],
        dependency_patterns: &["rails"],
    },
    FrameworkRule {
        framework: Framework::Flutter,
        marker_files: &[".metadata"],
        manifest_files: &["pubspec.yaml"],
        dependency_patterns: &["sdk: flutter"],
    },
];

pub struct FrameworkDetector;

impl FrameworkDetector {
    // Nhận diện framework từ tất cả gốc input rồi trả về một danh sách chung.
    pub fn analyze_many(roots: &[&Path], files: &[TextFile]) -> Vec<DetectedFramework> {
        let mut detected_frameworks = Vec::new();

        for rule in FRAMEWORK_RULES {
            let has_marker = roots
                .iter()
                .any(|root| has_marker(root, files, rule.marker_files));

            let has_dependency =
                has_dependency(files, rule.manifest_files, rule.dependency_patterns);

            if !has_marker && !has_dependency {
                continue;
            }

            detected_frameworks.push(DetectedFramework {
                framework: rule.framework,
                has_marker,
                has_dependency,
            });
        }

        detected_frameworks
    }
}

// Kiểm tra file marker
// Ta kiểm tra cả filesystem và danh sách TextFile vì marker
// có thể nằm trong project nhưng không được scanner đọc
fn has_marker(root: &Path, files: &[TextFile], marker_files: &[&str]) -> bool {
    marker_files.iter().any(|marker| {
        root.join(marker).is_file()
            || files
                .iter()
                .any(|file| path_matches_marker(&file.path, marker))
    })
}

// Kiểm tra dependency trong các file manifest
fn has_dependency(
    files: &[TextFile],
    manifest_files: &[&str],
    dependency_patterns: &[&str],
) -> bool {
    files.iter().any(|file| {
        if !has_file_name(&file.path, manifest_files) {
            return false;
        }

        let normalized_content = file.content.to_ascii_lowercase();

        dependency_patterns
            .iter()
            .any(|pattern| normalized_content.contains(pattern))
    })
}

// Kiểm tra tên cuối của path
// Ví dụ:
// project/frontend/package.json
// có tên cuối là package.json
fn has_file_name(path: &Path, expected_names: &[&str]) -> bool {
    let Some(file_name) = path.file_name() else {
        return false;
    };

    let file_name = file_name.to_string_lossy();

    expected_names
        .iter()
        .any(|expected| file_name.eq_ignore_ascii_case(expected))
}

// Kiểm tra đường dẫn có kết thúc bằng marker hay không
// Hàm này hỗ trợ marker có nhiều phần như:
// src-tauri/tauri.conf.json
// config/application.rb
fn path_matches_marker(path: &Path, marker: &str) -> bool {
    let normalized_path = path
        .to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase();

    let normalized_marker = marker.replace('\\', "/").to_ascii_lowercase();

    normalized_path == normalized_marker
        || normalized_path.ends_with(&format!("/{normalized_marker}"))
}
