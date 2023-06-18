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
use regex::Regex;

pub fn scan_manager(scan_settings: ScanSettings) {
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

    let mut handles: Vec<JoinHandle<()>> = Vec::new();
    for root in scan_settings.roots {
        let output_dir = scan_settings.output_dir.clone();
        let patterns = patterns.clone();

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
    }
    for handle in handles {
        match handle.join() {
            Ok(_) => (),
            Err(e) => eprintln!("{:?}", e),
        };
    }
}

fn load_regex(keywords: Vec<String>, case_sensitive: bool) -> Vec<Regex> {
    keywords
        .iter()
        .map(|kw| {
            let mut kw = kw.clone();
            if !case_sensitive {
                kw = "(?i)".to_owned() + &kw;
            }
            Regex::new(&kw).unwrap()
        })
        .collect()
}
