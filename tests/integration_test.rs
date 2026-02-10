use assert_cmd::cargo::cargo_bin_cmd;
use tython::ast::Type;
use tython::tir::type_rules;
use tython::tir::{BinOpKind, UnaryOpKind};

#[test]
fn test_tython_python_compatibility() {
    let test_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");

    let tython_output = cargo_bin_cmd!("tython")
        .arg("main.py")
        .current_dir(&test_dir)
        .output()
        .expect("Failed to run tython");

    let python_output = std::process::Command::new("python3")
        .arg("main.py")
        .current_dir(&test_dir)
        .output()
        .expect("Failed to run python");

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

#[test]
fn test_invalid_programs_produce_compilation_errors() {
    let invalid_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("invalid");

    assert!(
        invalid_dir.is_dir(),
        "Invalid test directory not found: {}",
        invalid_dir.display()
    );

    let mut test_dirs: Vec<_> = std::fs::read_dir(&invalid_dir)
        .expect("Failed to read invalid test directory")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.file_type().ok()?.is_dir() {
                Some(entry.path())
            } else {
                None
            }
        })
        .collect();

    assert!(
        !test_dirs.is_empty(),
        "No test subdirectories found in {}",
        invalid_dir.display()
    );

    test_dirs.sort();

    let mut failures = Vec::new();

    for test_dir in &test_dirs {
        let main_py = test_dir.join("main.py");
        let test_name = test_dir.file_name().unwrap().to_string_lossy().to_string();

        assert!(
            main_py.exists(),
            "Missing main.py in invalid test: {}",
            test_name
        );

        let output = cargo_bin_cmd!("tython")
            .arg("main.py")
            .current_dir(test_dir)
            .output()
            .unwrap_or_else(|e| panic!("Failed to run tython for '{}': {}", test_name, e));

        if output.status.success() {
            failures.push(format!(
                "  '{}': expected compilation error but tython succeeded\n    stdout: {}",
                test_name,
                String::from_utf8_lossy(&output.stdout).trim()
            ));
        }
    }

    if !failures.is_empty() {
        panic!(
            "Some invalid programs compiled successfully:\n{}",
            failures.join("\n")
        );
    }
}

const ALL_BINOPS: &[BinOpKind] = &[
    BinOpKind::Add,
    BinOpKind::Sub,
    BinOpKind::Mul,
    BinOpKind::Div,
    BinOpKind::FloorDiv,
    BinOpKind::Mod,
    BinOpKind::Pow,
    BinOpKind::BitAnd,
    BinOpKind::BitOr,
    BinOpKind::BitXor,
    BinOpKind::LShift,
    BinOpKind::RShift,
];

const ALL_UNARYOPS: &[UnaryOpKind] = &[
    UnaryOpKind::Neg,
    UnaryOpKind::Pos,
    UnaryOpKind::Not,
    UnaryOpKind::BitNot,
];

const TESTABLE_TYPES: &[Type] = &[Type::Int, Type::Float, Type::Bool];

fn type_to_python(ty: &Type) -> (&str, &str) {
    match ty {
        Type::Int => ("int", "1"),
        Type::Float => ("float", "1.0"),
        Type::Bool => ("bool", "True"),
        _ => unreachable!(),
    }
}

fn unaryop_to_python(op: UnaryOpKind) -> &'static str {
    match op {
        UnaryOpKind::Neg => "-",
        UnaryOpKind::Pos => "+",
        UnaryOpKind::Not => "not ",
        UnaryOpKind::BitNot => "~",
    }
}

#[test]
fn test_invalid_op_type_combinations() {
    let tmp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let mut failures = Vec::new();
    let mut tested = 0;

    // BinOp: test all invalid (op, left_type, right_type) combinations
    for &op in ALL_BINOPS {
        for left_ty in TESTABLE_TYPES {
            for right_ty in TESTABLE_TYPES {
                if type_rules::lookup_binop(op, left_ty, right_ty).is_some() {
                    continue;
                }

                let (left_ann, left_lit) = type_to_python(left_ty);
                let (right_ann, right_lit) = type_to_python(right_ty);
                let label = format!("{} {} {}", left_ann, op, right_ann);

                let code = format!(
                    "def main() -> None:\n    x: {} = {}\n    y: {} = {}\n    z = x {} y\n",
                    left_ann, left_lit, right_ann, right_lit, op
                );

                let file_path = tmp_dir.path().join(format!("binop_{}.py", tested));
                std::fs::write(&file_path, &code).expect("Failed to write test file");

                let output = cargo_bin_cmd!("tython")
                    .arg(&file_path)
                    .output()
                    .unwrap_or_else(|e| panic!("Failed to run tython for '{}': {}", label, e));

                if output.status.success() {
                    failures.push(format!(
                        "  '{}': expected type error but compiled successfully",
                        label
                    ));
                }
                tested += 1;
            }
        }
    }

    // UnaryOp: test all invalid (op, operand_type) combinations
    for &op in ALL_UNARYOPS {
        for operand_ty in TESTABLE_TYPES {
            if type_rules::lookup_unaryop(op, operand_ty).is_some() {
                continue;
            }

            let (ann, lit) = type_to_python(operand_ty);
            let op_str = unaryop_to_python(op);
            let label = format!("{}({})", op_str.trim(), ann);

            let code = format!(
                "def main() -> None:\n    x: {} = {}\n    y = {}x\n",
                ann, lit, op_str
            );

            let file_path = tmp_dir.path().join(format!("unaryop_{}.py", tested));
            std::fs::write(&file_path, &code).expect("Failed to write test file");

            let output = cargo_bin_cmd!("tython")
                .arg(&file_path)
                .output()
                .unwrap_or_else(|e| panic!("Failed to run tython for '{}': {}", label, e));

            if output.status.success() {
                failures.push(format!(
                    "  '{}': expected type error but compiled successfully",
                    label
                ));
            }
            tested += 1;
        }
    }

    assert!(tested > 0, "No invalid combinations were tested");

    if !failures.is_empty() {
        panic!(
            "Some invalid op/type combos compiled successfully ({} of {} failed):\n{}",
            failures.len(),
            tested,
            failures.join("\n")
        );
    }
}
