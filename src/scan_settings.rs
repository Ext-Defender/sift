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
    pub max_scan_threads: usize,
    pub max_file_threads: usize,
    pub max_write_lines: u16,
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
        max_scan_threads: usize,
        max_file_threads: usize,
        max_write_lines: u16,
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
            max_scan_threads,
            max_file_threads,
            max_write_lines,
        }
    }
}
