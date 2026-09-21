//! Ignore template cho Laravel.

pub const TEMPLATE: &str = r#"

# Laravel
vendor/
node_modules/
.env
.env.*
!.env.example
storage/framework/cache/
storage/framework/sessions/
storage/framework/views/
storage/logs/
bootstrap/cache/*.php
public/build/
"#;
