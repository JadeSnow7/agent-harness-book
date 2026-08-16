//! M2 CLI：在临时 workspace 上运行单步工具场景。

use m2_tool_runtime::{copy_fixtures, default_fixtures_dir, run_scenario};
use std::env;
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() -> ExitCode {
    let enable_bash = env::args().any(|arg| arg == "--enable-bash");
    let fixtures = default_fixtures_dir();
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let root = env::temp_dir().join(format!("m2-tool-runtime-{stamp}"));
    if let Err(error) = copy_fixtures(&fixtures, &root) {
        eprintln!("m2-tool-runtime: {error}");
        return ExitCode::from(1);
    }
    let result = run_scenario(&root, enable_bash);
    let _ = std::fs::remove_dir_all(&root);
    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(error) => {
            eprintln!("m2-tool-runtime: {error}");
            ExitCode::from(1)
        }
    }
}
