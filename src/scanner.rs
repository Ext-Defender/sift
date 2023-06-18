/*TODO: scanner function built to work as an independent thread that helps to consume a
central iterator available to multiple scanners */

use std::thread::JoinHandle;
use std::{error::Error, sync::Arc, thread, time::SystemTime};

use crate::sift::ScanMessage;
use crate::sift::ScanMessage::{Msg, END};
use crate::{file_handler, sift::Row};
use crossbeam::channel::Sender;
use jwalk::WalkDirGeneric;
use regex::Regex;
/// Handles file routing for filetype scanning
///
/// # Arguments
///
/// * 'dirWalk' - jwalk instance of WalkDir
/// * 'tx' - crossbeam sender for transfering findings to csv writer thread
/// * 'patterns' - Vec<Regex>
/// * 'last_timestamp' - SystemTime
pub fn scan(
    dir_walk: WalkDirGeneric<((), ())>,
    tx: Sender<ScanMessage>,
    patterns: Arc<Vec<Regex>>,
    last_timestamp: SystemTime,
    verbose: bool,
) -> Result<(), Box<dyn Error>> {
    let mut handles: Vec<JoinHandle<()>> = Vec::new();
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

        if scan_file {
            let file_name = dir_entry.file_name().to_string_lossy().to_string();

            if verbose {
                println!("Attempting to scan: {}", file_name);
            }

            let handle = thread::spawn(move || {
                let findings = match path.extension() {
                    Some(ext) => match ext.to_str() {
                        Some("pdf") => file_handler::scan_pdf(&path, &patterns).unwrap_or(None),
                        Some("xlsx") | Some("pptx") | Some("docx") => {
                            file_handler::scan_ooxml(&path, &patterns).unwrap_or(None)
                        }
                        Some("txt") | Some("xml") | Some("html") | Some("htm") => {
                            file_handler::scan_txt(&path, &patterns).unwrap_or(None)
                        }
                        Some("rtf") => file_handler::scan_rtf(&path, &patterns).unwrap_or(None),
                        Some("wpd") => file_handler::scan_rtf(&path, &patterns).unwrap_or(None),
                        Some("doc") | Some("ppt") | Some("xls") => {
                            file_handler::scan_legacy_office(&path, &patterns).unwrap_or(None)
                        }
                        _ => None,
                    },
                    _ => None,
                };

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
            });
            handles.push(handle);
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
