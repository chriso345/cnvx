use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    sync::LazyLock,
};

use cnvx::prelude::*;
use cnvx::solvers::LpAutoSolver;
use cnvx_parse::parse;
use test_case::test_case;

// URLs for Netlib data
const NETLIB_DATA_URL: &str = "https://www.netlib.org/lp/data/";
const NETLIB_INFEAS_URL: &str = "https://www.netlib.org/lp/infeas/";

// Local filenames
const VENDOR_SOURCE: &str = "vendor/emps.c";
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
    let vendor_src = Path::new(VENDOR_SOURCE);
    ensure_emps_binary(&suite_dir, vendor_src)
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
fn run_cnvx(mps: &Path) -> Result<Solution, String> {
    let contents = fs::read_to_string(mps).expect("Failed to read MPS file");

    let ext = "mps"; // or infer from file extension
    let model: Model = match parse(&contents, ext) {
        Ok(m) => m,
        Err(e) => return Err(format!("Failed to parse MPS file: {}", e)),
    };

    let mut solver = LpAutoSolver::new(&model);
    match solver.solve() {
        Ok(sol) => Ok(sol),
        Err(e) => Err(format!("Solver failed: {}", e)),
    }
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

    let output: Solution = match run_cnvx(&mps) {
        Ok(sol) => sol,
        Err(e) => panic!("cnvx failed on {}: {}", mps.display(), e),
    };

    if output.status != SolveStatus::Optimal {
        panic!("cnvx did not find optimal solution for {}", mps.display());
    }

    let obj = match output.objective_value {
        Some(obj) => obj,
        None => panic!("cnvx did not return objective value for {}", mps.display()),
    };

    if let Some(expected) = expected {
        if (obj - expected).abs() > TOL {
            panic!(
                "Objective value mismatch for {}: expected {}, got {}",
                mps.display(),
                expected,
                obj
            );
        }
    } else {
        println!("No expected objective provided for {}, got {}", mps.display(), obj);
    }
}
