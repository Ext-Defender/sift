use std::thread::JoinHandle;
use std::{error::Error, sync::Arc, thread, time::SystemTime};

use crate::sift::ScanMessage;
use crate::sift::ScanMessage::{Msg, END};
use crate::{file_handler, sift::Row};
use crossbeam::channel::Sender;
use jwalk::WalkDirGeneric;
use regex::Regex;

pub fn scan(
    dir_walk: WalkDirGeneric<((), ())>,
    tx: Sender<ScanMessage>,
    patterns: Arc<Vec<Regex>>,
    last_timestamp: SystemTime,
    verbose: bool,
    max_file_threads: usize,
) -> Result<(), Box<dyn Error>> {
    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    let mut now = std::time::Instant::now();

    for dir_result in dir_walk.into_iter() {
        let dir_entry = dir_result?;
        let path = dir_entry.path();
        let scan_file: bool = match dir_entry.metadata() {
            Ok(meta) => match meta.modified() {
                Ok(modified) => last_timestamp.le(&modified),
                Err(_) => true,
            },
            Err(_) => true,
        };
        let patterns = patterns.clone();
        let current_tx = tx.clone();

        if now.elapsed().as_secs() >= 30 {
            println!("Scanning {}", dir_entry.parent_path().to_str().unwrap());
            now = std::time::Instant::now();
        }

        if scan_file {
            let file_name = dir_entry.file_name().to_string_lossy().to_string();

            if verbose {
                println!("Attempting to scan: {}", file_name);
            }

            let handle = thread::Builder::new()
                .name(format!("{}", dir_entry.path().to_string_lossy()))
                .spawn(move || {
                    let findings = file_handler::scan_file(&path, &patterns);

                    if findings.is_some() {
                        if verbose {
                            println!("Findings in {}", file_name);
                        }
                        let findings = findings_to_string(findings.unwrap().0);
                        match current_tx.send(Msg(Row {
                            findings: findings.clone(),
                            filename: file_name,
                            path: path.to_string_lossy().to_string(),
                        })) {
                            Ok(_) => (),
                            Err(e) => {
                                println!("{}: didn't get sent.", findings);
                                eprintln!("{}", e);
                            }
                        }
                    }
                })
                .unwrap();
            handles.push(handle);

            while handles.len() == max_file_threads {
                handles.retain(|h| !h.is_finished());
            }
        }
    }
    println!("Sending writer termination.");
    for handle in handles {
        match handle.join() {
            Ok(_) => (),
            Err(_) => (),
        }
    }
    match tx.send(END) {
        Ok(_) => println!("Writer close message sent successfully"),
        Err(e) => eprintln!("{}", e),
    };

    Ok(())
}

fn findings_to_string(findings: Vec<String>) -> String {
    let mut words = String::new();
    for word in findings {
        words.push_str(&word);
        words.push_str(", ");
    }
    words.trim().to_string();
    words.trim_end_matches(",").to_string();
    words
}
