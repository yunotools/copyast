//! Ignore template cho Python.

pub const TEMPLATE: &str = r#"

# Python
__pycache__/
*.py[cod]
*$py.class
.Python
build/
dist/
downloads/
eggs/
.eggs/
*.egg-info/
.installed.cfg
*.egg
.venv/
venv/
env/
ENV/
.pytest_cache/
.mypy_cache/
.ruff_cache/
.pyre/
.tox/
.nox/
.coverage
.coverage.*
htmlcov/
"#;
