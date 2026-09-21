mod generator;
mod template;

pub use generator::IgnoreGenerator;
pub use template::IgnoreTemplate;

use ::ignore::WalkBuilder;

// Những file ignore sử dụng cú pháp tương thích với gitignore
// `.gitignore` và `.ignore` được crate `ignore` hỗ trợ sẵn,
// nên không cần đặt trong danh sách này
pub(crate) const CUSTOM_IGNORE_FILES: &[&str] = &[
    ".copyastignore",
    ".dockerignore",
    ".helmignore",
    ".npmignore",
    ".eslintignore",
    ".prettierignore",
    ".stylelintignore",
    ".vercelignore",
    ".nowignore",
    ".netlifyignore",
    ".gcloudignore",
    ".cfignore",
    ".cloudflareignore",
    ".wranglerignore",
    ".slugignore",
    ".terraformignore",
    ".terragruntignore",
    ".azureignore",
    ".funcignore",
    ".eleventyignore",
    ".parcelignore",
    ".nodemonignore",
    ".nxignore",
    ".bazelignore",
    ".bazeliskignore",
    ".serverlessignore",
    ".sstignore",
    ".cursorignore",
    ".aiderignore",
    ".codeiumignore",
    ".copilotignore",
    ".continueignore",
    ".windsurfignore",
    ".clineignore",
    ".rooignore",
    ".geminiignore",
    ".fdignore",
    ".rgignore",
];

// Cấu hình WalkBuilder để hỗ trợ ignore file trong toàn bộ cây thư mục
pub(crate) fn configure_walker(
    walker_builder: &mut WalkBuilder,
    should_respect_ignore_files: bool,
    should_include_hidden_files: bool,
    additional_ignore_file_names: &[String],
) {
    walker_builder
        // hidden(true) nghĩa là bỏ qua file và folder ẩn
        .hidden(!should_include_hidden_files)
        .parents(should_respect_ignore_files)
        .ignore(should_respect_ignore_files)
        .git_ignore(should_respect_ignore_files)
        .git_global(should_respect_ignore_files)
        .git_exclude(should_respect_ignore_files)
        // Cho phép .gitignore hoạt động ngoài Git repository
        .require_git(false);

    if should_respect_ignore_files {
        for file_name in CUSTOM_IGNORE_FILES {
            walker_builder.add_custom_ignore_filename(file_name);
        }

        for file_name in additional_ignore_file_names {
            if !file_name.trim().is_empty() {
                walker_builder.add_custom_ignore_filename(file_name);
            }
        }
    }
}
