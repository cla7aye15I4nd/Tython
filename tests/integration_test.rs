use assert_cmd::cargo::cargo_bin_cmd;

/// Integration test that runs main.py with both tython and python,
/// then compares outputs to ensure compatibility.
#[test]
fn test_tython_python_compatibility() {
    let test_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");

    // Run with tython
    let tython_output = cargo_bin_cmd!("tython")
        .arg("main.py")
        .current_dir(&test_dir)
        .output()
        .expect("Failed to run tython");

    // Run with python
    let python_output = std::process::Command::new("python3")
        .arg("main.py")
        .current_dir(&test_dir)
        .output()
        .expect("Failed to run python");

    // Both should succeed
    assert!(
        tython_output.status.success(),
        "Tython failed: {}",
        String::from_utf8_lossy(&tython_output.stderr)
    );
    assert!(
        python_output.status.success(),
        "Python failed: {}",
        String::from_utf8_lossy(&python_output.stderr)
    );

    // Outputs should be identical
    let tython_out = String::from_utf8_lossy(&tython_output.stdout);
    let python_out = String::from_utf8_lossy(&python_output.stdout);
    assert_eq!(
        tython_out.trim(),
        python_out.trim(),
        "Output mismatch!\nTython:\n{}\nPython:\n{}",
        tython_out,
        python_out
    );
}
