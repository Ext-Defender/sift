use std::path::PathBuf;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::SystemTime;

use crate::csv_writer::writer;
use crate::scan_settings::ScanSettings;
use crate::scanner::scan;
use crate::sift::ScanMessage;

use crossbeam::channel::unbounded;
use jwalk::WalkDir;
use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;

use regex::{Regex, RegexBuilder};

const MAX_SCANS: usize = 2;

pub fn scan_manager(scan_settings: ScanSettings) {
    build_logger(scan_settings.output_dir.clone());
    let last_time_stamp = match scan_settings.last_scan_time_stamp {
        Some(t) => {
            if !scan_settings.full_scan {
                SystemTime::from(t)
            } else {
                SystemTime::UNIX_EPOCH
            }
        }
        None => SystemTime::UNIX_EPOCH,
    };

    let patterns = Arc::new(load_regex(
        scan_settings.keywords,
        scan_settings.case_sensitive,
    ));

    let instant = std::time::Instant::now();

    let mut handles: Vec<JoinHandle<()>> = Vec::new();
    for root in scan_settings.roots {
        let output_dir = scan_settings.output_dir.clone();
        let patterns = patterns.clone();
        // let root_clone = root.clone();
        println!("Starting scan: {}", root);

        let handle = thread::spawn(move || {
            let (tx, rx) = unbounded::<ScanMessage>();
            let root_path = PathBuf::from(&root);
            let dir_walk = WalkDir::new(root_path);
            writer(output_dir, &root, rx);
            match scan(
                dir_walk,
                tx.clone(),
                patterns,
                last_time_stamp,
                scan_settings.verbose,
            ) {
                Ok(_) => (),
                Err(e) => eprintln!("{:?} panic at {}", e, root),
            }

            println!("Scan complete: {root}");
        });

        handles.push(handle);

        while handles.len() == MAX_SCANS {
            handles.retain(|h| !h.is_finished());
        }
    }
    for handle in handles {
        match handle.join() {
            Ok(_) => (),
            Err(e) => eprintln!("{:?}", e),
        };
    }

    println!("Scan(s) completed");
}

fn load_regex(keywords: Vec<String>, case_sensitive: bool) -> Vec<Regex> {
    keywords
        .iter()
        .map(|kw| {
            RegexBuilder::new(&kw)
                .case_insensitive(!case_sensitive)
                .build()
                .unwrap()
        })
        .collect()
}

fn build_logger(output_path: PathBuf) {
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S %Z)(utc)} : {l} : {M} : {T} : {m}\n",
        )))
        .build(format!("{}/scan.log", output_path.to_str().unwrap()))
        .unwrap();
    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Warn))
        .unwrap();
    log4rs::init_config(config).unwrap();
}
