use clap::Parser;
use serde_json::Value;
use std::collections::HashSet;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const DEFAULT_METRICS: &[&str] = &[
    "lines",
    "regions",
    "functions",
    "branches",
    "instantiations",
    "mcdc",
];

#[derive(Parser, Debug)]
#[command(name = "cargo-cmin")]
#[command(about = "Coverage-based testcase minimizer for invalid corpus")]
struct Args {
    #[arg(long, default_value = "tests/invalid")]
    input: PathBuf,
    #[arg(long)]
    output: Option<PathBuf>,
    #[arg(long, default_value = "tython")]
    target_bin: String,
    #[arg(long)]
    target_path: Option<PathBuf>,
    #[arg(long, default_value = "cargo")]
    cargo: String,
    #[arg(long)]
    llvm_profdata: Option<PathBuf>,
    #[arg(long)]
    llvm_cov: Option<PathBuf>,
    #[arg(long, default_value_t = 30)]
    timeout: u64,
    #[arg(
        long,
        default_value = "lines,regions,functions,branches,instantiations,mcdc"
    )]
    metrics: String,
    #[arg(long)]
    scope: Vec<PathBuf>,
    #[arg(long, default_value_t = false)]
    keep_success: bool,
    #[arg(long, default_value_t = false)]
    force: bool,
    #[arg(long, default_value_t = false)]
    dry_run: bool,
    #[arg(long, default_value_t = false)]
    clean: bool,
    #[arg(long, default_value_t = false)]
    rebuild: bool,
}

#[derive(Debug, Clone)]
struct CaseResult {
    name: String,
    path: PathBuf,
    size_bytes: u64,
    features: HashSet<String>,
}

fn is_windows() -> bool {
    cfg!(windows)
}

fn find_tool(explicit: &Option<PathBuf>, fallback: &str) -> Result<PathBuf, String> {
    if let Some(p) = explicit {
        if p.exists() {
            return Ok(p.clone());
        }
        return Err(format!("Tool not found: {}", p.display()));
    }

    let path_var = env::var_os("PATH").ok_or_else(|| "PATH is not set".to_string())?;
    for dir in env::split_paths(&path_var) {
        let cand = dir.join(fallback);
        if cand.exists() {
            return Ok(cand);
        }
        if is_windows() {
            let cand_exe = dir.join(format!("{fallback}.exe"));
            if cand_exe.exists() {
                return Ok(cand_exe);
            }
        }
    }

    Err(format!("Required tool not found in PATH: {fallback}"))
}

fn resolve_rooted(root: &Path, p: &Path) -> PathBuf {
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        root.join(p)
    }
}

fn run_checked(cmd: &mut Command, step: &str) -> Result<(), String> {
    let out = cmd
        .output()
        .map_err(|e| format!("Failed to run {step}: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "{step} failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(())
}

fn clean_repo(cargo: &str, repo_root: &Path) -> Result<(), String> {
    let mut cmd = Command::new(cargo);
    cmd.arg("clean").current_dir(repo_root);
    run_checked(&mut cmd, "cargo clean")
}

fn build_instrumented_binary(
    cargo: &str,
    repo_root: &Path,
    target_bin: &str,
) -> Result<PathBuf, String> {
    let mut cmd = Command::new(cargo);
    cmd.arg("build")
        .arg("--bin")
        .arg(target_bin)
        .current_dir(repo_root)
        .env("CARGO_INCREMENTAL", "0");

    let existing = env::var("RUSTFLAGS").unwrap_or_default();
    let cov = "-C instrument-coverage";
    let merged = if existing.trim().is_empty() {
        cov.to_string()
    } else {
        format!("{} {}", existing.trim(), cov)
    };
    cmd.env("RUSTFLAGS", merged);

    run_checked(&mut cmd, "cargo build (instrumented)")?;

    let exe_name = if is_windows() {
        format!("{target_bin}.exe")
    } else {
        target_bin.to_string()
    };
    let exe = repo_root.join("target").join("debug").join(exe_name);
    if !exe.exists() {
        return Err(format!("Instrumented binary not found: {}", exe.display()));
    }
    Ok(exe)
}

fn wait_with_timeout(
    child: &mut std::process::Child,
    timeout_secs: u64,
) -> Result<std::process::ExitStatus, String> {
    let deadline = Instant::now() + Duration::from_secs(timeout_secs);
    loop {
        match child
            .try_wait()
            .map_err(|e| format!("try_wait failed: {e}"))?
        {
            Some(status) => return Ok(status),
            None => {
                if Instant::now() >= deadline {
                    child.kill().ok();
                    child.wait().ok();
                    return Err("process timed out".to_string());
                }
                thread::sleep(Duration::from_millis(20));
            }
        }
    }
}

fn run_case(
    binary: &Path,
    case_dir: &Path,
    profraw_pattern: &Path,
    timeout_secs: u64,
) -> Result<i32, String> {
    let mut cmd = Command::new(binary);
    cmd.arg("main.py")
        .current_dir(case_dir)
        .env("LLVM_PROFILE_FILE", profraw_pattern)
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn test process: {e}"))?;
    let status = wait_with_timeout(&mut child, timeout_secs)?;
    Ok(status.code().unwrap_or(1))
}

fn discover_cases(input_dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut cases = Vec::new();
    for entry in fs::read_dir(input_dir)
        .map_err(|e| format!("Failed to read {}: {e}", input_dir.display()))?
    {
        let entry = entry.map_err(|e| format!("Failed to read dir entry: {e}"))?;
        let p = entry.path();
        if p.is_dir() && p.join("main.py").is_file() {
            cases.push(p);
        }
    }
    cases.sort();
    Ok(cases)
}

fn case_size_bytes(case_dir: &Path) -> Result<u64, String> {
    fn walk(acc: &mut u64, p: &Path) -> Result<(), String> {
        for entry in fs::read_dir(p).map_err(|e| format!("Failed to read {}: {e}", p.display()))? {
            let entry =
                entry.map_err(|e| format!("Failed to read entry in {}: {e}", p.display()))?;
            let path = entry.path();
            let meta = entry
                .metadata()
                .map_err(|e| format!("Failed to stat {}: {e}", path.display()))?;
            if meta.is_dir() {
                walk(acc, &path)?;
            } else if meta.is_file() {
                *acc += meta.len();
            }
        }
        Ok(())
    }

    let mut total = 0;
    walk(&mut total, case_dir)?;
    Ok(total)
}

fn parse_metrics(input: &str) -> Result<HashSet<String>, String> {
    let metrics: HashSet<String> = input
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .collect();

    let allowed: HashSet<String> = DEFAULT_METRICS.iter().map(|s| (*s).to_string()).collect();
    let invalid: Vec<_> = metrics.difference(&allowed).cloned().collect();
    if !invalid.is_empty() {
        return Err(format!("Unknown metrics: {}", invalid.join(", ")));
    }
    Ok(metrics)
}

fn in_scope(filename: &str, scope_prefixes: &[PathBuf]) -> bool {
    if scope_prefixes.is_empty() {
        return true;
    }
    let abs = fs::canonicalize(filename).unwrap_or_else(|_| PathBuf::from(filename));
    scope_prefixes.iter().any(|p| abs.starts_with(p))
}

fn has_positive_number(v: &Value) -> bool {
    match v {
        Value::Number(n) => n.as_f64().unwrap_or(0.0) > 0.0,
        Value::Array(a) => a.iter().any(has_positive_number),
        Value::Object(m) => m.values().any(has_positive_number),
        _ => false,
    }
}

fn extract_features(
    export_json: &Value,
    metrics: &HashSet<String>,
    scope_prefixes: &[PathBuf],
) -> HashSet<String> {
    let mut out = HashSet::new();

    let payload = export_json
        .get("data")
        .and_then(Value::as_array)
        .and_then(|a| a.first())
        .unwrap_or(&Value::Null);

    if payload.is_null() {
        return out;
    }

    if let Some(files) = payload.get("files").and_then(Value::as_array) {
        for file_entry in files {
            let filename = match file_entry.get("filename").and_then(Value::as_str) {
                Some(s) if in_scope(s, scope_prefixes) => s,
                _ => continue,
            };

            if metrics.contains("lines") {
                if let Some(segments) = file_entry.get("segments").and_then(Value::as_array) {
                    for seg in segments {
                        let Some(seg_arr) = seg.as_array() else {
                            continue;
                        };
                        if seg_arr.len() < 4 {
                            continue;
                        }
                        let line = seg_arr[0].as_i64().unwrap_or(0);
                        let count = seg_arr[2].as_i64().unwrap_or(0);
                        let has_count = seg_arr[3].as_bool().unwrap_or(false);
                        if has_count && count > 0 {
                            out.insert(format!("L|{filename}|{line}"));
                        }
                    }
                }
            }

            if metrics.contains("branches") {
                if let Some(branches) = file_entry.get("branches").and_then(Value::as_array) {
                    for br in branches {
                        let Some(arr) = br.as_array() else { continue };
                        if arr.len() < 5 {
                            continue;
                        }
                        let key = arr[0..4]
                            .iter()
                            .map(|v| v.to_string())
                            .collect::<Vec<_>>()
                            .join(",");
                        let mut outcome_idx = 0;
                        for v in &arr[4..] {
                            if let Some(n) = v.as_f64() {
                                if n > 0.0 {
                                    out.insert(format!("B|{filename}|{key}|{outcome_idx}"));
                                }
                                outcome_idx += 1;
                            }
                        }
                    }
                }
            }

            if metrics.contains("mcdc") {
                if let Some(recs) = file_entry.get("mcdc_records").and_then(Value::as_array) {
                    for rec in recs {
                        if has_positive_number(rec) {
                            out.insert(format!("M|file|{filename}|{rec}"));
                        }
                    }
                }
            }
        }
    }

    if let Some(functions) = payload.get("functions").and_then(Value::as_array) {
        for fn_entry in functions {
            let name = fn_entry
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let count = fn_entry.get("count").and_then(Value::as_i64).unwrap_or(0);

            let fn_files: Vec<String> = fn_entry
                .get("filenames")
                .and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .filter_map(Value::as_str)
                        .map(ToOwned::to_owned)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            let scoped_files: Vec<String> = fn_files
                .iter()
                .filter(|f| in_scope(f, scope_prefixes))
                .cloned()
                .collect();
            if scoped_files.is_empty() {
                continue;
            }

            if metrics.contains("functions") && count > 0 {
                out.insert(format!("F|{name}"));
            }

            if metrics.contains("instantiations") && count > 0 {
                let mut unique = scoped_files.clone();
                unique.sort();
                unique.dedup();
                out.insert(format!("I|{name}|{}", unique.join(",")));
            }

            if metrics.contains("regions") {
                if let Some(regions) = fn_entry.get("regions").and_then(Value::as_array) {
                    for reg in regions {
                        let Some(arr) = reg.as_array() else { continue };
                        if arr.len() < 5 {
                            continue;
                        }
                        let exec_count = arr[4].as_i64().unwrap_or(0);
                        if exec_count <= 0 {
                            continue;
                        }

                        let file_id = arr.get(5).and_then(Value::as_u64).unwrap_or(0) as usize;
                        let reg_file = fn_files
                            .get(file_id)
                            .cloned()
                            .or_else(|| scoped_files.first().cloned())
                            .unwrap_or_default();
                        if !in_scope(&reg_file, scope_prefixes) {
                            continue;
                        }

                        let kind = arr.get(7).and_then(Value::as_i64).unwrap_or(0);
                        let l1 = arr[0].as_i64().unwrap_or(0);
                        let c1 = arr[1].as_i64().unwrap_or(0);
                        let l2 = arr[2].as_i64().unwrap_or(0);
                        let c2 = arr[3].as_i64().unwrap_or(0);
                        out.insert(format!("R|{reg_file}|{l1}|{c1}|{l2}|{c2}|{kind}"));
                    }
                }
            }

            if metrics.contains("branches") {
                if let Some(branches) = fn_entry.get("branches").and_then(Value::as_array) {
                    for br in branches {
                        let Some(arr) = br.as_array() else { continue };
                        if arr.len() < 5 {
                            continue;
                        }
                        let key = arr[0..4]
                            .iter()
                            .map(|v| v.to_string())
                            .collect::<Vec<_>>()
                            .join(",");
                        let mut outcome_idx = 0;
                        for v in &arr[4..] {
                            if let Some(n) = v.as_f64() {
                                if n > 0.0 {
                                    out.insert(format!("FB|{name}|{key}|{outcome_idx}"));
                                }
                                outcome_idx += 1;
                            }
                        }
                    }
                }
            }

            if metrics.contains("mcdc") {
                if let Some(recs) = fn_entry.get("mcdc_records").and_then(Value::as_array) {
                    for rec in recs {
                        if has_positive_number(rec) {
                            out.insert(format!("M|fn|{name}|{rec}"));
                        }
                    }
                }
            }
        }
    }

    out
}

fn merge_profraw(
    llvm_profdata: &Path,
    profraw_files: &[PathBuf],
    profdata_out: &Path,
) -> Result<(), String> {
    let mut cmd = Command::new(llvm_profdata);
    cmd.arg("merge").arg("-sparse");
    for f in profraw_files {
        cmd.arg(f);
    }
    cmd.arg("-o").arg(profdata_out);
    run_checked(&mut cmd, "llvm-profdata merge")
}

fn export_cov_json(llvm_cov: &Path, profdata: &Path, binary: &Path) -> Result<Value, String> {
    let out = Command::new(llvm_cov)
        .arg("export")
        .arg("--instr-profile")
        .arg(profdata)
        .arg(binary)
        .output()
        .map_err(|e| format!("Failed to run llvm-cov export: {e}"))?;

    if !out.status.success() {
        return Err(format!(
            "llvm-cov export failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ));
    }

    serde_json::from_slice::<Value>(&out.stdout)
        .map_err(|e| format!("Invalid llvm-cov JSON output: {e}"))
}

fn greedy_minimize(cases: &[CaseResult]) -> Vec<CaseResult> {
    let mut all = HashSet::new();
    for c in cases {
        all.extend(c.features.iter().cloned());
    }

    let mut uncovered = all.clone();
    let mut chosen: Vec<CaseResult> = Vec::new();

    while !uncovered.is_empty() {
        let mut best_idx: Option<usize> = None;
        let mut best_gain = 0usize;

        for (i, c) in cases.iter().enumerate() {
            let gain = c.features.intersection(&uncovered).count();
            if gain == 0 {
                continue;
            }
            if gain > best_gain {
                best_gain = gain;
                best_idx = Some(i);
                continue;
            }
            if gain == best_gain {
                if let Some(bi) = best_idx {
                    let b = &cases[bi];
                    if c.size_bytes < b.size_bytes
                        || (c.size_bytes == b.size_bytes && c.name < b.name)
                    {
                        best_idx = Some(i);
                    }
                }
            }
        }

        let Some(bi) = best_idx else { break };
        let best = cases[bi].clone();
        for f in &best.features {
            uncovered.remove(f);
        }
        chosen.push(best);
    }

    let mut i = 0usize;
    while i < chosen.len() {
        let mut union = HashSet::new();
        for (j, c) in chosen.iter().enumerate() {
            if j != i {
                union.extend(c.features.iter().cloned());
            }
        }
        if union == all {
            chosen.remove(i);
        } else {
            i += 1;
        }
    }

    chosen.sort_by(|a, b| a.name.cmp(&b.name));
    chosen
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| format!("Failed to create {}: {e}", dst.display()))?;
    for entry in fs::read_dir(src).map_err(|e| format!("Failed to read {}: {e}", src.display()))? {
        let entry = entry.map_err(|e| format!("Failed to read entry in {}: {e}", src.display()))?;
        let path = entry.path();
        let target = dst.join(entry.file_name());
        let meta = entry
            .metadata()
            .map_err(|e| format!("Failed to stat {}: {e}", path.display()))?;
        if meta.is_dir() {
            copy_dir_recursive(&path, &target)?;
        } else if meta.is_file() {
            fs::copy(&path, &target).map_err(|e| {
                format!(
                    "Failed to copy {} -> {}: {e}",
                    path.display(),
                    target.display()
                )
            })?;
        }
    }
    Ok(())
}

fn main() -> Result<(), String> {
    let args = Args::parse();
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let input_dir = resolve_rooted(&repo_root, &args.input);
    let output_dir = args.output.as_ref().map(|p| resolve_rooted(&repo_root, p));
    if !input_dir.is_dir() {
        return Err(format!(
            "Input directory not found: {}",
            input_dir.display()
        ));
    }

    let metrics = parse_metrics(&args.metrics)?;

    let scope_prefixes: Vec<PathBuf> = if args.scope.is_empty() {
        vec![repo_root.clone()]
    } else {
        args.scope
            .iter()
            .map(|p| resolve_rooted(&repo_root, p))
            .collect()
    };

    let llvm_profdata = find_tool(&args.llvm_profdata, "llvm-profdata")?;
    let llvm_cov = find_tool(&args.llvm_cov, "llvm-cov")?;

    let binary = if let Some(target_path) = &args.target_path {
        let p = resolve_rooted(&repo_root, target_path);
        if !p.exists() {
            return Err(format!("Binary not found: {}", p.display()));
        }
        p
    } else {
        if args.clean {
            println!("[1/5] Cleaning build artifacts...");
            clean_repo(&args.cargo, &repo_root)?;
        }

        let default_bin = repo_root
            .join("target")
            .join("debug")
            .join(if is_windows() {
                format!("{}.exe", args.target_bin)
            } else {
                args.target_bin.clone()
            });

        if args.rebuild || !default_bin.exists() {
            println!("[2/5] Building instrumented binary...");
            build_instrumented_binary(&args.cargo, &repo_root, &args.target_bin)?
        } else {
            default_bin
        }
    };

    let cases = discover_cases(&input_dir)?;
    if cases.is_empty() {
        return Err(format!(
            "No testcase directories with main.py found under {}",
            input_dir.display()
        ));
    }

    println!("[3/5] Running {} testcases with coverage...", cases.len());

    let mut kept_results: Vec<CaseResult> = Vec::new();
    let mut skipped_success = 0usize;
    let mut skipped_no_profile = 0usize;

    let temp_root = env::temp_dir().join(format!("tython-cmin-{}", std::process::id()));
    if temp_root.exists() {
        fs::remove_dir_all(&temp_root).ok();
    }
    fs::create_dir_all(&temp_root)
        .map_err(|e| format!("Failed to create temp dir {}: {e}", temp_root.display()))?;
    let profraw_dir = temp_root.join("profraw");
    fs::create_dir_all(&profraw_dir)
        .map_err(|e| format!("Failed to create temp dir {}: {e}", profraw_dir.display()))?;

    for (idx, case) in cases.iter().enumerate() {
        let name = case
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or_else(|| format!("Invalid testcase path: {}", case.display()))?
            .to_string();

        let pattern = profraw_dir.join(format!("{:04}-{}-%p.profraw", idx + 1, name));
        let code = match run_case(&binary, case, &pattern, args.timeout) {
            Ok(c) => c,
            Err(_) => continue,
        };

        if code == 0 && !args.keep_success {
            skipped_success += 1;
            continue;
        }

        let prefix = format!("{:04}-{}-", idx + 1, name);
        let mut profraw_files = Vec::new();
        for e in fs::read_dir(&profraw_dir)
            .map_err(|er| format!("Failed to read {}: {er}", profraw_dir.display()))?
        {
            let e = e.map_err(|er| format!("Failed to read profraw entry: {er}"))?;
            let p = e.path();
            let fname = p.file_name().and_then(OsStr::to_str).unwrap_or_default();
            if fname.starts_with(&prefix) && fname.ends_with(".profraw") {
                profraw_files.push(p);
            }
        }

        if profraw_files.is_empty() {
            skipped_no_profile += 1;
            continue;
        }

        let profdata = temp_root.join(format!("{:04}-{}.profdata", idx + 1, name));
        if merge_profraw(&llvm_profdata, &profraw_files, &profdata).is_err() {
            continue;
        }

        let export = match export_cov_json(&llvm_cov, &profdata, &binary) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let features = extract_features(&export, &metrics, &scope_prefixes);
        let size_bytes = case_size_bytes(case)?;
        kept_results.push(CaseResult {
            name,
            path: case.clone(),
            size_bytes,
            features,
        });

        if (idx + 1) % 25 == 0 || idx + 1 == cases.len() {
            println!("  processed {}/{}", idx + 1, cases.len());
        }
    }

    fs::remove_dir_all(&temp_root).ok();

    if kept_results.is_empty() {
        return Err("No usable testcase coverage data collected".to_string());
    }

    println!("[4/5] Minimizing corpus...");
    let selected = greedy_minimize(&kept_results);

    let mut all_features = HashSet::new();
    for c in &kept_results {
        all_features.extend(c.features.iter().cloned());
    }
    let mut selected_features = HashSet::new();
    for c in &selected {
        selected_features.extend(c.features.iter().cloned());
    }

    println!(
        "  usable: {} | selected: {} | features kept: {}/{}",
        kept_results.len(),
        selected.len(),
        selected_features.len(),
        all_features.len()
    );
    if skipped_success > 0 {
        println!("  skipped-success: {skipped_success}");
    }
    if skipped_no_profile > 0 {
        println!("  skipped-no-profile: {skipped_no_profile}");
    }

    println!("[5/5] Writing output...");
    if args.dry_run {
        for c in &selected {
            println!("{}", c.name);
        }
        return Ok(());
    }

    if let Some(output_dir) = output_dir {
        if output_dir.exists() {
            if !args.force {
                return Err(format!(
                    "Output directory exists: {} (use --force)",
                    output_dir.display()
                ));
            }
            fs::remove_dir_all(&output_dir)
                .map_err(|e| format!("Failed to remove {}: {e}", output_dir.display()))?;
        }
        fs::create_dir_all(&output_dir)
            .map_err(|e| format!("Failed to create {}: {e}", output_dir.display()))?;

        for c in &selected {
            copy_dir_recursive(&c.path, &output_dir.join(&c.name))?;
        }

        let manifest = output_dir.join("cmin_manifest.txt");
        let mut body = String::new();
        for c in &selected {
            body.push_str(&c.name);
            body.push('\n');
        }
        fs::write(&manifest, body)
            .map_err(|e| format!("Failed to write {}: {e}", manifest.display()))?;

        println!("  wrote minimized corpus to {}", output_dir.display());
    } else {
        let selected_names: HashSet<&str> = selected.iter().map(|c| c.name.as_str()).collect();
        for case in &cases {
            let name = case
                .file_name()
                .and_then(OsStr::to_str)
                .ok_or_else(|| format!("Invalid testcase path: {}", case.display()))?;
            if !selected_names.contains(name) {
                fs::remove_dir_all(case)
                    .map_err(|e| format!("Failed to delete testcase {}: {e}", case.display()))?;
            }
        }

        let manifest = input_dir.join("cmin_manifest.txt");
        let mut body = String::new();
        for c in &selected {
            body.push_str(&c.name);
            body.push('\n');
        }
        fs::write(&manifest, body)
            .map_err(|e| format!("Failed to write {}: {e}", manifest.display()))?;

        println!("  pruned corpus in place at {}", input_dir.display());
    }
    Ok(())
}
