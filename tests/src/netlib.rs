use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, process::Command};

use crate::{ARGS, GREEN, RED, RESET, TOL};

const NETLIB_DATA_URL: &str = "https://www.netlib.org/lp/data/";
const NETLIB_INFEAS_URL: &str = "https://www.netlib.org/lp/infeas/";
const EMPS_URL: &str = "https://www.netlib.org/lp/data/emps.c";

const DECOMP_FILE: &str = "emps.c";
const DECOMP_EXECUTABLE: &str = "emps";

// FIXME: This is not an exhaustive list of Netlib tests. Rather these are just some
//  small tests that the solver can solve in a reasonable amount of time, and that
//  the current implementation of the solver can solve correctly.
const TESTS: &[(&str, Option<f64>)] = &[
    ("afiro", Some(-4.6475314286E+02)),
    ("adlittle", Some(2.2549496316E+05)),
    ("sc50b", Some(-7.0000000000E+01)),
];

pub fn run_tests(suite_dir: &str) {
    let suite_dir = Path::new(suite_dir);
    fs::create_dir_all(suite_dir).unwrap();

    let emps_src = ensure_emps_source(suite_dir);
    let emps_bin = ensure_emps_binary(suite_dir, &emps_src);

    let mut failures = Vec::new();

    for (name, expected) in TESTS {
        if ARGS.verbose {
            println!("\nRunning test: {}", name);
        }

        match run_single_test(suite_dir, &emps_bin, name, *expected) {
            Ok(()) => {
                println!("{GREEN}(PASS){RESET} {}", name);
            }
            Err(err) => {
                eprintln!("{RED}(FAIL){RESET} {}\n{}", name, err);
                failures.push(*name);
            }
        }
    }

    if !failures.is_empty() {
        panic!("{} Netlib tests failed: {:?}", failures.len(), failures);
    }
}

fn ensure_emps_source(suite_dir: &Path) -> PathBuf {
    let path = suite_dir.join(DECOMP_FILE);

    if !path.exists() {
        if ARGS.verbose {
            println!("Downloading emps.c");
        }

        let bytes = reqwest::blocking::get(EMPS_URL)
            .expect("failed to download emps.c")
            .bytes()
            .expect("failed to read emps.c");

        fs::write(&path, &bytes).expect("failed to write emps.c");
    }

    path
}

fn ensure_emps_binary(suite_dir: &Path, src: &Path) -> PathBuf {
    let bin = suite_dir.join(DECOMP_EXECUTABLE);

    if bin.exists() {
        return bin;
    }

    let compiler = std::env::var("CC").unwrap_or_else(|_| "cc".into());

    let output = Command::new(compiler)
        .arg("-O2")
        .arg(src)
        .arg("-o")
        .arg(&bin)
        .output()
        .expect("failed to invoke compiler");

    if !output.status.success() {
        panic!(
            "Failed to compile emps:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }

    bin
}

fn run_single_test(
    suite_dir: &Path,
    emps: &Path,
    name: &str,
    expected: Option<f64>,
) -> Result<(), String> {
    let lp = ensure_lp_file(suite_dir, name, expected)?;
    run_emps(emps, &lp)?;

    let mps = suite_dir.join(format!("{}.mps", name));
    if !mps.exists() {
        return Err(format!("emps did not produce expected file {}", mps.display()));
    }

    let output = run_cnvx_solve(&mps)?;
    assert_output(&output, expected)
}

fn ensure_lp_file(
    suite_dir: &Path,
    name: &str,
    expected: Option<f64>,
) -> Result<PathBuf, String> {
    let url = match expected {
        Some(_) => format!("{NETLIB_DATA_URL}{name}"),
        None => format!("{NETLIB_INFEAS_URL}{name}"),
    };

    let path = suite_dir.join(name);

    if !path.exists() {
        let bytes = reqwest::blocking::get(&url)
            .map_err(|e| format!("download failed for {}: {}", name, e))?
            .bytes()
            .map_err(|e| format!("read failed for {}: {}", name, e))?;

        fs::write(&path, &bytes)
            .map_err(|e| format!("write failed for {}: {}", name, e))?;
    }

    Ok(path)
}

fn run_emps(emps: &Path, lp: &Path) -> Result<(), String> {
    let workdir = lp
        .parent()
        .ok_or("LP file has no parent directory")?
        .canonicalize()
        .map_err(|e| format!("failed to canonicalize suite dir: {}", e))?;

    let emps_abs = emps
        .canonicalize()
        .map_err(|e| format!("failed to canonicalize emps binary path: {}", e))?;

    if ARGS.verbose {
        println!("Running emps on {} in {}", lp.display(), workdir.display());
        println!("Using emps binary at {}", emps_abs.display());
    }

    let mut child = Command::new(&emps_abs)
        .arg("-s")
        .current_dir(&workdir)
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to spawn emps: {}", e))?;

    {
        let mut stdin = child.stdin.take().ok_or("failed to open stdin")?;
        let input = fs::read(lp).map_err(|e| format!("read failed: {}", e))?;
        stdin
            .write_all(&input)
            .map_err(|e| format!("stdin write failed: {}", e))?;
    }

    let status = child.wait().map_err(|e| e.to_string())?;
    if !status.success() {
        return Err("emps failed".into());
    }

    Ok(())
}

fn run_cnvx_solve(mps: &Path) -> Result<String, String> {
    let cnvx = Path::new("target/release/cnvx");

    if !cnvx.exists() {
        return Err(
            "cnvx binary not found (expected target/debug/cnvx). Did you build the workspace?"
                .into(),
        );
    }

    let output = Command::new(cnvx)
        .arg("solve")
        .arg(mps)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .map_err(|e| format!("failed to run cnvx solve: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "cnvx solve failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        ));
    }

    Ok(format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    ))
}

fn assert_output(output: &str, expected: Option<f64>) -> Result<(), String> {
    let lower = output.to_lowercase();

    match expected {
        Some(expected_val) => {
            if !lower.contains("optimal") {
                return Err(format!("expected optimal solution, got:\n{}", output));
            }

            let found = extract_objective(output)
                .ok_or_else(|| format!("failed to parse objective value\n{}", output))?;

            if (found - expected_val).abs() > TOL {
                return Err(format!(
                    "objective mismatch: expected {}, got {}\n{}",
                    expected_val, found, output
                ));
            }

            Ok(())
        }
        None => {
            if lower.contains("infeasible") || lower.contains("unbounded") {
                Ok(())
            } else {
                Err(format!("expected infeasible or unbounded, got:\n{}", output))
            }
        }
    }
}

fn extract_objective(output: &str) -> Option<f64> {
    output.split_whitespace().find_map(|tok| tok.parse::<f64>().ok())
}
