use copyast::CopyastCommand;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

struct TestWorkspace {
    path: PathBuf,
}

impl TestWorkspace {
    fn new() -> Self {
        let unique_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be after the Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "copyast-multiple-inputs-{}-{unique_id}",
            std::process::id()
        ));

        fs::create_dir_all(&path).expect("test workspace must be created");
        Self { path }
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn repeated_input_options_merge_sources_without_copying_overlaps_twice() {
    let workspace = TestWorkspace::new();
    let frontend = workspace.path.join("frontend");
    let backend = workspace.path.join("backend");
    let frontend_file = frontend.join("src/app.ts");
    let backend_file = backend.join("src/main.rs");
    let output = workspace.path.join("context.txt");

    write_text(&frontend_file, "const frontendValue = 1;\n");
    write_text(&backend_file, "fn backend_value() {}\n");

    let arguments = vec![
        "copyast".to_owned(),
        "--input".to_owned(),
        path_argument(&frontend),
        "--input".to_owned(),
        path_argument(&backend),
        // File này đã nằm trong input `frontend`; nó không được xuất hiện lần hai.
        "--input".to_owned(),
        path_argument(&frontend_file),
        "--output".to_owned(),
        path_argument(&output),
        "--path-mode".to_owned(),
        "relative".to_owned(),
    ];

    CopyastCommand::new()
        .execute(&arguments)
        .expect("multiple inputs must be merged successfully");

    let content = fs::read_to_string(output).expect("output must be readable");
    let normalized_content = content.replace('\\', "/");

    assert!(normalized_content.contains("=== FILE: frontend/src/app.ts ==="));
    assert!(normalized_content.contains("=== FILE: backend/src/main.rs ==="));
    assert_eq!(content.matches("const frontendValue = 1;").count(), 1);
    assert_eq!(content.matches("fn backend_value() {}").count(), 1);
}

#[test]
fn single_input_uses_the_same_flag_based_interface() {
    let workspace = TestWorkspace::new();
    let input = workspace.path.join("source.txt");
    let output = workspace.path.join("context.txt");
    write_text(&input, "backward compatible\n");

    let arguments = vec![
        "copyast".to_owned(),
        "--input".to_owned(),
        path_argument(&input),
        "--output".to_owned(),
        path_argument(&output),
    ];

    CopyastCommand::new()
        .execute(&arguments)
        .expect("a single input must use the same interface successfully");

    let content = fs::read_to_string(output).expect("output must be readable");
    assert!(content.contains("backward compatible"));
}

fn write_text(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().expect("test file must have a parent"))
        .expect("test directory must be created");
    fs::write(path, content).expect("test file must be written");
}

fn path_argument(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
