use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigFile {
    pub initial_scan: bool,
    pub output_directory: Option<String>,
    pub keywords: Vec<String>,
    pub roots: Vec<String>,
    pub secret: Option<String>,
    pub time_last_scan: String,
    pub max_scan_threads: usize,
    pub max_file_threads: usize,
    pub max_write_lines: u16,
}

impl ::std::default::Default for ConfigFile {
    fn default() -> Self {
        Self {
            initial_scan: true,
            output_directory: None,
            keywords: Vec::new(),
            roots: Vec::new(),
            secret: None,
            time_last_scan: String::new(),
            max_scan_threads: 2,
            max_file_threads: 5,
            max_write_lines: 10000,
        }
    }
}
