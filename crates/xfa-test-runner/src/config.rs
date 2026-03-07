use std::path::PathBuf;
use std::time::Duration;

pub struct Config {
    pub corpus_dir: PathBuf,
    #[allow(dead_code)]
    pub db_path: PathBuf,
    pub workers: usize,
    pub timeout: Duration,
    pub test_filter: Option<Vec<String>>,
    pub resume: bool,
    pub run_id: String,
}

impl Config {
    pub fn new(
        corpus_dir: PathBuf,
        db_path: PathBuf,
        workers: usize,
        timeout_secs: u64,
        tests: Option<String>,
        resume: bool,
        run_id: Option<String>,
    ) -> Self {
        let run_id = run_id.unwrap_or_else(|| {
            let now = chrono::Utc::now();
            format!("run-{}", now.format("%Y%m%d-%H%M%S"))
        });
        let test_filter = tests.map(|t| t.split(',').map(|s| s.trim().to_string()).collect());
        Self {
            corpus_dir,
            db_path,
            workers,
            timeout: Duration::from_secs(timeout_secs),
            test_filter,
            resume,
            run_id,
        }
    }
}
