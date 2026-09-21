pub const TEMPLATE: &str = r#"

# =========================
# IDE
# =========================

.vscode/
.idea/
*.iml
*.suo
*.user


# =========================
# OS
# =========================

.DS_Store
Thumbs.db
Desktop.ini


# =========================
# Logs
# =========================

*.log
logs/


# =========================
# Temporary
# =========================

tmp/
temp/
*.tmp
*.bak


# =========================
# Environment
# =========================

.env
.env.*
!.env.example


# =========================
# Generated
# =========================

generated/
out/
dist/

"#;
