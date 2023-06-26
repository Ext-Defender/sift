use std::path::PathBuf;

use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct ScanSettings {
    pub full_scan: bool,
    pub time_stamp: DateTime<Utc>,
    pub verbose: bool,
    pub keywords: Vec<String>,
    pub roots: Vec<String>,
    pub last_scan_time_stamp: Option<DateTime<Utc>>,
    pub output_dir: PathBuf,
    pub case_sensitive: bool,
}

impl ScanSettings {
    pub fn new(
        full_scan: bool,
        verbose: bool,
        keywords: Vec<String>,
        roots: Vec<String>,
        last_scan_time_stamp: Option<DateTime<Utc>>,
        output_dir: PathBuf,
        case_sensitive: bool,
    ) -> Self {
        let time_stamp = Utc::now();
        Self {
            full_scan,
            time_stamp,
            verbose,
            keywords,
            roots,
            last_scan_time_stamp,
            output_dir,
            case_sensitive,
        }
    }
}
