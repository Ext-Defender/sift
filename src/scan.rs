use chrono::prelude::*;
use jwalk::WalkDir;
use regex::Regex;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
// use crossbeam::{Sender, Receiver};
use crate::file_handler;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
#[derive(Debug)]
pub struct Scan {
    pub full_scan: bool,
    pub time_stamp: DateTime<Utc>,
    pub verbose: bool,
    pub keywords: Vec<String>,
    pub roots: Vec<String>,
    pub last_scan_time_stamp: Option<DateTime<Utc>>,
}

impl Scan {
    pub fn new(
        full_scan: bool,
        verbose: bool,
        keywords: Vec<String>,
        roots: Vec<String>,
        last_scan_time_stamp: Option<DateTime<Utc>>,
    ) -> Self {
        let time_stamp = Utc::now();
        let scan = Self {
            full_scan,
            time_stamp,
            verbose,
            keywords,
            roots,
            last_scan_time_stamp,
        };

        for root in &scan.roots {
            match scan.scan(&root, last_scan_time_stamp, full_scan) {
                Ok(_) => println!("'{root}' scan complete"),
                Err(e) => println!("!'{root}' scan failed with error {e}"),
            }
        }

        scan
    }

    fn scan(
        &self,
        root: &String,
        last_time_stamp: Option<DateTime<Utc>>,
        full_scan: bool,
    ) -> Result<(), Box<dyn Error>> {
        if self.verbose {
            println!("Full scan: {full_scan}");
            println!("Scanning: {root}");
            println!("Keywords: {:?}", self.keywords);
            println!("Start time: {}\n\n", self.time_stamp);
        }
        let root = root.to_string();
        let patterns = Arc::new(load_regex(&self.keywords));
        let root = PathBuf::from(root);
        let walk = WalkDir::new(root);

        let (tx, rx): (
            Sender<(Vec<String>, String)>,
            Receiver<(Vec<String>, String)>,
        ) = mpsc::channel();
        let mut threads = Vec::new();

        for dir in walk.into_iter() {
            let file_meta = dir?;
            let path = file_meta.path();
            let thread_tx = tx.clone();
            let patterns = Arc::new(patterns.clone());
            let thr = thread::spawn(move || {
                let ret = match path.extension() {
                    Some(ext) => match ext.to_str() {
                        Some("pdf") => file_handler::scan_pdf(&path, &patterns),
                        Some("txt") => file_handler::scan_txt(&path, &patterns),
                        Some("rtf") => Ok(None),
                        Some("xlsx") => Ok(None),
                        Some("pptx") => Ok(None),
                        Some("docx") => Ok(None),
                        Some("wpd") => Ok(None),
                        Some("doc") | Some("ppt") | Some("xls") => Ok(None),
                        _ => Ok(None),
                    },
                    None => Ok(None),
                };
                match ret {
                    Ok(Some(res)) => {
                        thread_tx.send(res).unwrap();
                    }
                    _ => (),
                }
            });
            threads.push(thr);

            // if self.verbose {
            //     println!("Scanning: {:?}", path);
            // }
        }

        let mut thread_rets = Vec::with_capacity(5);
        for ret in rx.try_iter() {
            thread_rets.push(ret);
        }

        for t in threads {
            match t.join() {
                Ok(_) => (),
                Err(_) => (),
            }
        }

        for ret in thread_rets {
            println!("{:?}", ret);
        }

        if self.verbose {
            println!("Finished: {}", Utc::now());
        }

        Ok(())
    }
}

fn load_regex(keywords: &Vec<String>) -> Vec<Regex> {
    let mut res = Vec::new();
    for keyword in keywords {
        res.push(Regex::new(keyword).unwrap())
    }
    res
}
