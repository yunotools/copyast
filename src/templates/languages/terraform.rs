//! Ignore template cho Terraform và OpenTofu.

pub const TEMPLATE: &str = r#"

# Terraform và OpenTofu
.terraform/
*.tfstate
*.tfstate.*
crash.log
crash.*.log
override.tf
override.tf.json
*_override.tf
*_override.tf.json
"#;
