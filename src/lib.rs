pub mod config;
pub mod csv_writer;
pub mod encryption;
pub mod file_handler;
pub mod run;
pub mod scan;
pub mod scan_manager;
pub mod scan_settings;
pub mod scanner;
pub mod settings;

pub mod sift {
    use serde::Serialize;

    pub enum ScanMessage {
        Msg(Row),
        END,
    }

    #[derive(Serialize)]
    pub struct Row {
        pub findings: String,
        pub filename: String,
        pub path: String,
    }
}
