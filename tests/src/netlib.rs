use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    sync::LazyLock,
};

use test_case::test_case;

// URLs for Netlib data
const NETLIB_DATA_URL: &str = "https://www.netlib.org/lp/data/";
const NETLIB_INFEAS_URL: &str = "https://www.netlib.org/lp/infeas/";
const EMPS_URL: &str = "https://www.netlib.org/lp/data/emps.c";

// Local filenames
const DECOMP_FILE: &str = "emps.c";
const DECOMP_EXECUTABLE: &str = "emps";

// Tolerance for objective comparison
const TOL: f64 = 1e-4;

static NETLIB_SUITE: &str = "netlib_suite";

// Ensure the test suite directory exists
fn setup_suite_dir() -> PathBuf {
    let dir = Path::new(NETLIB_SUITE);
    fs::create_dir_all(dir).expect("Failed to create netlib_suite directory");
    dir.to_path_buf()
}

// Ensure emps.c is downloaded
fn ensure_emps_source(suite_dir: &Path) -> PathBuf {
    let path = suite_dir.join(DECOMP_FILE);
    if !path.exists() {
        println!("Downloading emps.c...");
        let bytes = reqwest::blocking::get(EMPS_URL)
            .expect("Failed to download emps.c")
            .bytes()
            .expect("Failed to read emps.c content");
        fs::write(&path, &bytes).expect("Failed to write emps.c");
    }
    path
}

// Ensure emps is compiled
fn ensure_emps_binary(suite_dir: &Path, src: &Path) -> PathBuf {
    let bin = suite_dir.join(DECOMP_EXECUTABLE);
    if bin.exists() {
        return bin;
    }

    let compiler = std::env::var("CC").unwrap_or_else(|_| "cc".into());

    let output = Command::new(&compiler)
        .arg("-O2")
        .arg(src)
        .arg("-o")
        .arg(&bin)
        .output()
        .expect("Failed to invoke compiler for emps.c");

    if !output.status.success() {
        panic!(
            "Failed to compile emps:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    bin
}

// Lazy static to ensure emps is ready before any test
static EMPS: LazyLock<PathBuf> = LazyLock::new(|| {
    let suite_dir = setup_suite_dir();
    let src = ensure_emps_source(&suite_dir);
    ensure_emps_binary(&suite_dir, &src)
});

// Helper to download LP files
fn ensure_lp_file(name: &str, expected: Option<f64>) -> PathBuf {
    let suite_dir = setup_suite_dir();
    let path = suite_dir.join(name);

    if !path.exists() {
        let url = match expected {
            Some(_) => format!("{NETLIB_DATA_URL}{name}"),
            None => format!("{NETLIB_INFEAS_URL}{name}"),
        };
        let bytes = reqwest::blocking::get(&url)
            .unwrap_or_else(|_| panic!("Failed to download {}", name))
            .bytes()
            .unwrap_or_else(|_| panic!("Failed to read {}", name));
        fs::write(&path, &bytes).unwrap_or_else(|_| panic!("Failed to write {}", name));
    }

    path
}

// Run emps on the LP file

fn run_emps(lp: &Path) {
    let emps_abs = EMPS.canonicalize().expect("Failed to canonicalize emps binary");
    let workdir = lp.parent().unwrap().canonicalize().unwrap();

    // Run emps and capture stdout/stderr
    let mut child = Command::new(&emps_abs)
        .arg("-s")
        .current_dir(&workdir)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn emps");

    // Write LP to stdin and close
    {
        let mut stdin = child.stdin.take().expect("Failed to open stdin");
        let input = fs::read(lp).expect("Failed to read LP file");
        stdin.write_all(&input).expect("Failed to write to emps stdin");
    }

    // Wait for completion and collect output
    let output = child.wait_with_output().expect("Failed to wait for emps");

    if !output.status.success() {
        panic!(
            "emps failed on {}\nstdout:\n{}\nstderr:\n{}",
            lp.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    println!(
        "emps finished successfully. stdout:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
}

// Run cnvx solver on the produced MPS
fn run_cnvx(mps: &Path) -> String {
    let cnvx = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("target/release/cnvx");
    if !cnvx.exists() {
        panic!("cnvx binary not found at {}. Build the workspace first.", cnvx.display());
    }

    let output = Command::new(cnvx)
        .arg("solve")
        .arg(mps)
        .output()
        .expect("Failed to run cnvx solve");

    if !output.status.success() {
        panic!(
            "cnvx solve failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

// Validate the output
fn assert_output(output: &str, expected: Option<f64>) {
    let lower = output.to_lowercase();

    match expected {
        Some(val) => {
            if !lower.contains("optimal") {
                panic!("Expected optimal solution, got:\n{}", output);
            }
            let found =
                extract_objective(output).expect("Failed to parse objective value");
            if (found - val).abs() > TOL {
                panic!("Objective mismatch: expected {}, got {}\n{}", val, found, output);
            }
        }
        None => {
            if !lower.contains("infeasible") && !lower.contains("unbounded") {
                panic!("Expected infeasible or unbounded, got:\n{}", output);
            }
        }
    }
}

fn extract_objective(output: &str) -> Option<f64> {
    output.split_whitespace().find_map(|tok| tok.parse().ok())
}

#[test_case("afiro", Some(-4.6475314286E+02))]
#[test_case("adlittle", Some(2.2549496316E+05))]
#[test_case("sc50b", Some(-7.0000000000E+01))]
fn netlib_test(name: &str, expected: Option<f64>) {
    let lp = ensure_lp_file(name, expected);
    run_emps(&lp);

    let mps = lp.with_extension("mps");
    if !mps.exists() {
        panic!("emps did not produce expected file {}", mps.display());
    }

    let output = run_cnvx(&mps);
    assert_output(&output, expected);
}
