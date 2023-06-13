use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigFile {
    pub initial_scan: bool,
    pub output_directory: Option<String>,
    pub keywords: Vec<String>,
    pub roots: Vec<String>,
    pub secret: Option<String>,
    pub time_last_scan: String,
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
        }
    }
}
