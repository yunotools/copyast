use crate::analysis::{DuplicateDetector, FrameworkDetector, LanguageDetector, TokenEstimator};
use crate::config::CopyastConfig;
use crate::domain::{CopyastResult, IncrementalStats, TextFile};
use crate::error::CopyastError;
use crate::logger;
use crate::pipeline::file_ops::are_paths_equal;
use crate::pipeline::incremental::IncrementalTracker;
use crate::pipeline::{scanner, writer};
use std::path::Path;

// Facade điều phối toàn bộ pipeline Copyast.
#[derive(Default)]
pub struct CopyastService;

impl CopyastService {
    pub fn run(&self, config: &CopyastConfig) -> Result<CopyastResult, CopyastError> {
        validate_config(config)?;
        logger::info("Copyast scanning started");

        let scan_output = scanner::scan_files(config);
        let mut files = scan_output.files;
        let scan_stats = scan_output.stats;

        logger::info(&format!("Copied files: {}", scan_stats.copied_files));

        if scan_stats.count_skipped_files() > 0 {
            logger::warn(&format!(
                "Skipped files: {}",
                scan_stats.count_skipped_files()
            ));
        }

        let detection_roots = resolve_detection_roots(config);

        let detected_languages = LanguageDetector::analyze_many(&detection_roots, &files)
            .into_iter()
            .map(|detected| detected.language.to_string())
            .collect::<Vec<_>>();

        logger::info(&format!("Detected languages: {detected_languages:?}"));

        let detected_frameworks = FrameworkDetector::analyze_many(&detection_roots, &files)
            .into_iter()
            .map(|detected| detected.framework.to_string())
            .collect::<Vec<_>>();

        logger::info(&format!("Detected frameworks: {detected_frameworks:?}"));

        // Tìm nhóm trùng trước khi xóa để vẫn trả đủ báo cáo.
        let duplicate_groups = DuplicateDetector::find_duplicates(&files);

        if config.should_deduplicate {
            let removed_files = DuplicateDetector::remove_duplicates(&mut files);
            logger::info(&format!("Removed duplicate files: {removed_files}"));
        }

        let token_estimate = TokenEstimator::estimate(&files, config);

        logger::info(&format!(
            "Estimated tokens for {}: {}",
            token_estimate.token_model.as_str(),
            token_estimate.estimated_tokens,
        ));

        let incremental_stats = if config.is_incremental {
            Some(write_incremental_output(config, &files)?)
        } else {
            write_output(config, &files)?;
            None
        };

        logger::success("Copyast finished");

        Ok(CopyastResult {
            output_path: config.output_path.clone(),
            scan_stats,
            total_bytes: token_estimate.byte_count,
            estimated_tokens: token_estimate.estimated_tokens,
            detected_languages,
            detected_frameworks,
            duplicate_groups,
            incremental_stats,
        })
    }
}

fn validate_config(config: &CopyastConfig) -> Result<(), CopyastError> {
    if config.output_path.as_os_str().is_empty() {
        return Err(CopyastError::EmptyOutputPath);
    }

    if config.output_path.is_dir() {
        return Err(CopyastError::OutputIsDirectory {
            path: config.output_path.clone(),
        });
    }

    if config.input_paths.is_empty() {
        return Err(CopyastError::NoInputPaths);
    }

    for input_path in &config.input_paths {
        if !input_path.exists() {
            return Err(CopyastError::InputNotFound {
                path: input_path.clone(),
            });
        }

        if !input_path.is_file() && !input_path.is_dir() {
            return Err(CopyastError::UnsupportedInputType {
                path: input_path.clone(),
            });
        }

        // Bất kỳ file input nào trùng output đều có nguy cơ bị Writer ghi đè.
        if input_path.is_file() && are_paths_equal(input_path, &config.output_path) {
            return Err(CopyastError::InputOutputConflict {
                path: input_path.clone(),
            });
        }
    }

    Ok(())
}

fn resolve_detection_roots(config: &CopyastConfig) -> Vec<&Path> {
    config
        .input_paths
        .iter()
        .map(|input_path| {
            if input_path.is_file() {
                input_path.parent().unwrap_or(Path::new("."))
            } else {
                input_path.as_path()
            }
        })
        .collect()
}

fn write_output(config: &CopyastConfig, files: &[TextFile]) -> Result<(), CopyastError> {
    if config.is_dry_run {
        logger::info("Dry-run enabled: output was not written");
        return Ok(());
    }

    writer::write_output(files, config)?;
    Ok(())
}

fn write_incremental_output(
    config: &CopyastConfig,
    files: &[TextFile],
) -> Result<IncrementalStats, CopyastError> {
    let tracker = IncrementalTracker::analyze(config, files)?;
    let mut stats = tracker.stats();

    if config.is_dry_run {
        logger::info("Dry-run enabled: output and cache were not written");
        return Ok(stats);
    }

    if !tracker.should_update_output() {
        logger::info("No source changes detected: output was not rewritten");
        return Ok(stats);
    }

    // Cache chỉ được xác nhận sau khi output đã ghi thành công.
    writer::write_output(files, config)?;
    tracker.save_cache()?;
    stats.is_output_written = true;

    Ok(stats)
}
