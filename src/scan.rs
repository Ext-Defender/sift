use chrono::prelude::*;
use jwalk::WalkDir;
use regex::Regex;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
// use crossbeam::{Sender, Receiver};

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
        for dir in walk.into_iter() {
            if !full_scan {
                if self.verbose {
                    match dir?.metadata() {
                        Ok(c) => {
                            let time: DateTime<Utc> = DateTime::from(c.modified()?);
                            if time.ge(&last_time_stamp.unwrap()) {
                                println!("{:?}", time);
                            }
                        }
                        Err(e) => {
                            println!("ERROR: {e}");
                            continue;
                        }
                    }
                }
            } else {
                if self.verbose {
                    println!("{:?}", dir?.metadata());
                }
            }
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
