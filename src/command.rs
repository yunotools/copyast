use crate::{
    CopyastConfig, CopyastError, CopyastResult, CopyastService, DEFAULT_MAX_FILE_SIZE,
    IgnoreGenerator, IgnoreTemplate, PathMode, TokenModel, logger,
};
use clap::error::ErrorKind;
use clap::{ArgGroup, Parser, ValueEnum};
use std::path::PathBuf;

const DEFAULT_INPUT_PATH: &str = ".";
const DEFAULT_OUTPUT_PATH: &str = "copyast-output.txt";
const DEFAULT_IGNORE_OUTPUT_PATH: &str = ".copyastignore";

#[derive(Default)]
pub struct CopyastCommand {
    service: CopyastService,
}

impl CopyastCommand {
    pub fn new() -> Self {
        Self::default()
    }

    fn execute_action(&self, action: CopyastAction) -> Result<(), CopyastError> {
        match action {
            CopyastAction::Run(config) => {
                let result = self
                    .service
                    .run(&config)
                    .map_err(|error| build_command_error(error.to_string()))?;

                log_run_summary(&result, config.is_dry_run);
                Ok(())
            }
            CopyastAction::GenerateIgnore {
                template_selection,
                output_path,
            } => {
                IgnoreGenerator::generate_to(&template_selection, &output_path)
                    .map_err(|error| build_command_error(error.to_string()))?;

                logger::success(&format!(
                    "Generated {} from template `{template_selection}`",
                    output_path.display(),
                ));
                Ok(())
            }
            CopyastAction::ListTemplates => {
                print_available_templates();
                Ok(())
            }
        }
    }
}

impl CopyastCommand {
    pub fn execute(&self, raw_args: &[String]) -> Result<(), CopyastError> {
        let Some(cli_args) = parse_cli_args(raw_args)? else {
            return Ok(());
        };

        self.execute_action(cli_args.into_action()?)
    }
}

enum CopyastAction {
    Run(CopyastConfig),
    GenerateIgnore {
        template_selection: String,
        output_path: PathBuf,
    },
    ListTemplates,
}

#[derive(Debug, Parser)]
#[command(
    name = "copyast",
    about = "Copy text source files into one context file for AI",
    version,
    override_usage = "copyast [-i <PATH> ...] [-o <FILE>] [OPTIONS]\n       copyast --gen-ignore <TEMPLATE> [-o <FILE>]\n       copyast --list-templates",
    group(
        ArgGroup::new("template_action")
            .args(["ignore_template_selection", "should_list_templates"])
            .multiple(false)
    )
)]
struct CopyastCliArgs {
    /// Tạo các quy tắc ignore. Kết hợp nhiều template bằng dấu phẩy: rust,tauri.
    #[arg(long = "gen-ignore", value_name = "TEMPLATE")]
    ignore_template_selection: Option<String>,

    /// In toàn bộ template ngôn ngữ và framework hiện có.
    #[arg(long = "list-templates")]
    should_list_templates: bool,

    /// File hoặc thư mục cần quét. Có thể sử dụng nhiều lần. Mặc định là thư mục hiện tại.
    #[arg(short = 'i', long = "input", value_name = "PATH")]
    input_paths: Vec<PathBuf>,

    /// Chọn file output. Có thể dùng khi quét hoặc với --gen-ignore.
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    output_path: Option<PathBuf>,

    /// Kiểu đường dẫn được ghi trong header của mỗi file. Mặc định là auto.
    #[arg(long, value_enum)]
    path_mode: Option<CliPathMode>,

    /// Họ tokenizer AI dùng để ước lượng token. Mặc định là o200k.
    #[arg(long, value_enum)]
    token_model: Option<CliTokenModel>,

    /// Chỉ ghi lại output khi source hoặc cấu hình render thay đổi.
    #[arg(long = "incremental")]
    is_incremental: bool,

    /// Chỉ giữ file đầu tiên trong mỗi nhóm có nội dung trùng hoàn toàn.
    #[arg(long = "deduplicate")]
    should_deduplicate: bool,

    /// Chỉ quét và báo cáo, không ghi output hoặc cache incremental.
    #[arg(long = "dry-run")]
    is_dry_run: bool,

    /// Không sử dụng .gitignore, .dockerignore, .helmignore và các file tương tự.
    #[arg(long = "no-ignore")]
    is_ignore_disabled: bool,

    /// Bao gồm cả file và thư mục ẩn.
    #[arg(long = "hidden")]
    should_include_hidden: bool,

    /// Thêm tên file có cú pháp tương thích với gitignore. Có thể sử dụng nhiều lần.
    #[arg(long = "ignore-file", value_name = "NAME")]
    additional_ignore_file_names: Vec<String>,

    /// Kích thước tối đa của một source file, tính bằng byte. Mặc định là 10 MiB.
    #[arg(long = "max-file-size", value_name = "BYTES")]
    max_file_size_bytes: Option<u64>,

    /// Tắt log của ứng dụng, ngoại trừ các lỗi.
    #[arg(short = 'q', long = "quiet")]
    is_quiet: bool,
}

impl CopyastCliArgs {
    fn into_action(mut self) -> Result<CopyastAction, CopyastError> {
        if let Some(template_selection) = self.ignore_template_selection.take() {
            self.ensure_no_run_options("--gen-ignore")?;

            return Ok(CopyastAction::GenerateIgnore {
                template_selection,
                output_path: build_ignore_output_path(self.output_path),
            });
        }

        if self.should_list_templates {
            self.ensure_no_run_options("--list-templates")?;

            if self.output_path.is_some() {
                return Err(build_command_error(
                    "-o/--output cannot be used together with --list-templates",
                ));
            }

            return Ok(CopyastAction::ListTemplates);
        }

        Ok(CopyastAction::Run(self.into_config()))
    }

    fn ensure_no_run_options(&self, action_name: &str) -> Result<(), CopyastError> {
        let Some(option_name) = self.find_run_only_option() else {
            return Ok(());
        };

        Err(build_command_error(format!(
            "{option_name} cannot be used together with {action_name}"
        )))
    }

    fn find_run_only_option(&self) -> Option<&'static str> {
        [
            (!self.input_paths.is_empty(), "-i/--input"),
            (self.path_mode.is_some(), "--path-mode"),
            (self.token_model.is_some(), "--token-model"),
            (self.is_incremental, "--incremental"),
            (self.should_deduplicate, "--deduplicate"),
            (self.is_dry_run, "--dry-run"),
            (self.is_ignore_disabled, "--no-ignore"),
            (self.should_include_hidden, "--hidden"),
            (
                !self.additional_ignore_file_names.is_empty(),
                "--ignore-file",
            ),
            (self.max_file_size_bytes.is_some(), "--max-file-size"),
        ]
        .into_iter()
        .find_map(|(is_present, option_name)| is_present.then_some(option_name))
    }

    fn into_config(self) -> CopyastConfig {
        let input_paths = if self.input_paths.is_empty() {
            vec![PathBuf::from(DEFAULT_INPUT_PATH)]
        } else {
            self.input_paths
        };

        let output_path = build_copy_output_path(
            self.output_path
                .unwrap_or_else(|| PathBuf::from(DEFAULT_OUTPUT_PATH)),
        );

        CopyastConfig::new(input_paths, output_path)
            .with_path_mode(self.path_mode.unwrap_or(CliPathMode::Auto).into())
            .with_token_model(self.token_model.unwrap_or(CliTokenModel::O200k).into())
            .with_incremental(self.is_incremental)
            .with_deduplication(self.should_deduplicate)
            .with_dry_run(self.is_dry_run)
            .with_ignore_files(!self.is_ignore_disabled)
            .with_hidden_files(self.should_include_hidden)
            .with_additional_ignore_file_names(self.additional_ignore_file_names)
            .with_max_file_size_bytes(self.max_file_size_bytes.unwrap_or(DEFAULT_MAX_FILE_SIZE))
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliPathMode {
    Auto,
    Relative,
    Absolute,
}

impl From<CliPathMode> for PathMode {
    fn from(value: CliPathMode) -> Self {
        match value {
            CliPathMode::Auto => Self::Auto,
            CliPathMode::Relative => Self::Relative,
            CliPathMode::Absolute => Self::Absolute,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliTokenModel {
    Cl100k,
    O200k,
    Claude,
    Gemini,
}

impl From<CliTokenModel> for TokenModel {
    fn from(value: CliTokenModel) -> Self {
        match value {
            CliTokenModel::Cl100k => Self::Cl100k,
            CliTokenModel::O200k => Self::O200k,
            CliTokenModel::Claude => Self::Claude,
            CliTokenModel::Gemini => Self::Gemini,
        }
    }
}

fn parse_cli_args(raw_args: &[String]) -> Result<Option<CopyastCliArgs>, CopyastError> {
    match CopyastCliArgs::try_parse_from(raw_args) {
        Ok(cli_args) => Ok(Some(cli_args)),
        Err(error)
            if matches!(
                error.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) =>
        {
            error
                .print()
                .map_err(|print_error| build_command_error(print_error.to_string()))?;
            Ok(None)
        }
        Err(error) => Err(build_command_error(error.to_string())),
    }
}

fn build_copy_output_path(output_path: PathBuf) -> PathBuf {
    if output_path.is_dir() {
        output_path.join(DEFAULT_OUTPUT_PATH)
    } else {
        output_path
    }
}

fn build_ignore_output_path(output_path: Option<PathBuf>) -> PathBuf {
    let output_path = output_path.unwrap_or_else(|| PathBuf::from(DEFAULT_IGNORE_OUTPUT_PATH));

    if output_path.is_dir() {
        output_path.join(DEFAULT_IGNORE_OUTPUT_PATH)
    } else {
        output_path
    }
}

fn print_available_templates() {
    println!("Available ignore templates:");

    for template_name in IgnoreTemplate::list_available_names() {
        println!("  {template_name}");
    }
}

fn log_run_summary(result: &CopyastResult, dry_run: bool) {
    if dry_run {
        logger::success("Dry run completed; no output was written");
    } else {
        logger::success(&format!("Output: {}", result.output_path.display()));
    }

    logger::info(&format!(
        "Files: {} copied, {} skipped; context: {} bytes, approximately {} tokens",
        result.scan_stats.copied_files,
        result.scan_stats.count_skipped_files(),
        result.total_bytes,
        result.estimated_tokens,
    ));

    if result.scan_stats.count_skipped_files() > 0 {
        logger::info(&format!(
            "Skipped detail: {} ignored, {} binary/non-UTF-8, {} unreadable, {} too large",
            result.scan_stats.ignored_files,
            result.scan_stats.binary_files,
            result.scan_stats.unreadable_files,
            result.scan_stats.oversized_files,
        ));
    }

    if !result.detected_languages.is_empty() {
        logger::info(&format!(
            "Languages: {}",
            result.detected_languages.join(", ")
        ));
    }

    if !result.detected_frameworks.is_empty() {
        logger::info(&format!(
            "Frameworks: {}",
            result.detected_frameworks.join(", ")
        ));
    }

    if !result.duplicate_groups.is_empty() {
        logger::warn(&format!(
            "Found {} exact duplicate group(s); use --deduplicate to remove copies",
            result.duplicate_groups.len(),
        ));
    }

    if let Some(incremental_stats) = &result.incremental_stats {
        logger::info(&format!(
            "Incremental: {} changed, {} unchanged, {} removed, output written: {}",
            incremental_stats.changed_files,
            incremental_stats.unchanged_files,
            incremental_stats.removed_files,
            incremental_stats.is_output_written,
        ));
    }
}

fn build_command_error(message: impl Into<String>) -> CopyastError {
    CopyastError::Command(message.into())
}
