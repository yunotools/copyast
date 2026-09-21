use super::template::IgnoreTemplate;
use crate::CopyastError;
use std::fs;
use std::path::Path;

pub struct IgnoreGenerator;

impl IgnoreGenerator {
    pub fn generate_to(
        template_selection: &str,
        output_path: impl AsRef<Path>,
    ) -> Result<(), CopyastError> {
        let ignore_content = IgnoreTemplate::generate(template_selection).ok_or_else(|| {
            CopyastError::UnknownIgnoreTemplate {
                name: template_selection.to_owned(),
            }
        })?;

        let output_path = output_path.as_ref();

        if let Some(parent) = output_path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)?;
        }

        fs::write(output_path, ignore_content)?;

        Ok(())
    }
}
