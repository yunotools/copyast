//! Ignore template cho Django.

pub const TEMPLATE: &str = r#"

# Django
__pycache__/
*.py[cod]
.venv/
venv/
.pytest_cache/
.mypy_cache/
db.sqlite3
db.sqlite3-journal
staticfiles/
media/
htmlcov/
.coverage
"#;
