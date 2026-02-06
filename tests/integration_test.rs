use std::path::PathBuf;

use assert_cmd::cargo::cargo_bin_cmd;

/// Integration test that runs main.py with both tython and python,
/// then compares outputs to ensure compatibility.
#[test]
fn test_tython_python_compatibility() {
    // Get the path to the tests directory
    let mut test_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    test_dir.push("tests");

    let main_py = test_dir.join("main.py");

    // Run with tython
    let tython_output = cargo_bin_cmd!("tython")
        .arg(main_py.to_str().unwrap())
        .current_dir(&test_dir)
        .output()
        .expect("Failed to run tython");

    // Run with python
    let python_output = std::process::Command::new("python")
        .arg("main.py")
        .current_dir(&test_dir)
        .output()
        .expect("Failed to run python");

    // Convert outputs to strings
    let tython_stdout = String::from_utf8_lossy(&tython_output.stdout);
    let python_stdout = String::from_utf8_lossy(&python_output.stdout);

    // Both should succeed (exit code 0)
    assert!(
        tython_output.status.success(),
        "Tython execution failed with stderr: {}",
        String::from_utf8_lossy(&tython_output.stderr)
    );

    assert!(
        python_output.status.success(),
        "Python execution failed with stderr: {}",
        String::from_utf8_lossy(&python_output.stderr)
    );

    // Compare outputs - they should be identical
    assert_eq!(
        tython_stdout.trim(),
        python_stdout.trim(),
        "Tython output does not match Python output!\n\nTython output:\n{}\n\nPython output:\n{}",
        tython_stdout,
        python_stdout
    );
}
