use crate::file_handler;
use chrono::prelude::*;
use csv::WriterBuilder;
use jwalk::WalkDir;
use regex::Regex;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;
use std::{fs, thread};
#[derive(Debug)]
pub struct Scan {
    pub full_scan: bool,
    pub time_stamp: DateTime<Utc>,
    pub verbose: bool,
    pub keywords: Vec<String>,
    pub roots: Vec<String>,
    pub last_scan_time_stamp: Option<DateTime<Utc>>,
}

#[derive(serde::Serialize)]
struct Row {
    keywords: String,
    path: String,
}

impl Scan {
    pub fn new(
        full_scan: bool,
        verbose: bool,
        keywords: Vec<String>,
        roots: Vec<String>,
        last_scan_time_stamp: Option<DateTime<Utc>>,
        output_dir: PathBuf,
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
                Ok(result) => {
                    println!("'{root}' scan complete");

                    match result {
                        Some(findings) => {
                            if !Path::new(&output_dir).exists() {
                                fs::create_dir_all(&output_dir).unwrap();
                            }
                            let mut output_filename = output_dir.clone();

                            let mut root = root.replace("\\", "");
                            root = root.replace(":", "");
                            root = root.replace("/", "");

                            let mut time_stamp = time_stamp.to_string();

                            time_stamp = time_stamp.replace(":", "-");
                            time_stamp = time_stamp.replace(".", "-");
                            time_stamp = time_stamp.replace(" ", "_");

                            output_filename.push(format!("{root}_{time_stamp}.csv"));

                            match fs::File::create(&output_filename) {
                                Ok(_) => {
                                    println!("Output file created: {}", output_filename.display())
                                }
                                Err(e) => eprintln!("{e}"),
                            }

                            let mut writer = WriterBuilder::new()
                                .has_headers(true)
                                .from_path(output_filename)
                                .unwrap();

                            println!("\n\nWriting to csv in output directory.\n\n");

                            for rec in findings {
                                let mut words = String::new();
                                for word in rec.0 {
                                    words.push_str(&word);
                                    words.push_str(", ");
                                }
                                words = words.trim().to_string();
                                words = words.trim_end_matches(",").to_string();
                                // words = words.replace(" ", " | ");
                                writer
                                    .serialize(Row {
                                        keywords: words,
                                        path: rec.1,
                                    })
                                    .unwrap();
                            }

                            println!("\n\nFinished writing: {}\n\n", Utc::now());
                        }
                        None => println!("INFO: No findings"),
                    }
                }
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
    ) -> Result<Option<Vec<(Vec<String>, String)>>, Box<dyn Error>> {
        if self.verbose {
            println!("Full scan: {full_scan}");
            println!("Scanning: {root}");
            println!("Keywords: {:?}", self.keywords);
            println!("Start time: {}\n\n", self.time_stamp);
        }

        let root = root.to_string();
        let patterns = Arc::new(load_regex(&self.keywords));
        let root = PathBuf::from(&root);
        let walk = WalkDir::new(&root);

        let mut threads = Vec::new();

        let mut findings = Vec::new();

        let mut last_time_stamp = match last_time_stamp {
            Some(t) => SystemTime::from(t),
            None => SystemTime::UNIX_EPOCH,
        };
        if full_scan {
            last_time_stamp = SystemTime::UNIX_EPOCH;
        }

        if !self.verbose {
            println!("Scanning {:?}... ", root);
        }
        for obj in walk.into_iter() {
            let file_meta = obj?;
            let path = file_meta.path();
            let scan_file: bool = match file_meta.metadata() {
                Ok(meta) => match meta.modified() {
                    Ok(modified) => last_time_stamp.le(&modified),
                    Err(_) => true,
                },
                Err(_) => true,
            };

            let patterns = Arc::new(patterns.clone());

            if scan_file {
                if self.verbose {
                    println!("Scanning: {:?}", file_meta.path());
                }
                let thr = thread::spawn(move || {
                    let ret = match path.extension() {
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
                    ret
                });

                threads.push(thr);
            }
        }

        threads.into_iter().for_each(|th| match th.join() {
            Ok(Some(r)) => findings.push(r),
            _ => (),
        });

        println!("\n\nFinished Scan: {}\n\n", Utc::now());

        if self.verbose {
            for finding in &findings {
                println!("{:?}", finding);
            }
        }

        if findings.is_empty() {
            return Ok(None);
        }
        Ok(Some(findings))
    }
}

fn load_regex(keywords: &Vec<String>) -> Vec<Regex> {
    keywords.iter().map(|kw| Regex::new(kw).unwrap()).collect()
}
