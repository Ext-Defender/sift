use chrono::{Datelike, Timelike, Utc};
use crossbeam::channel::Receiver;
use csv::{Writer, WriterBuilder};
use std::collections::VecDeque;
use std::error::Error;
use std::thread;
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

use crate::sift::ScanMessage;
use crate::sift::ScanMessage::{Msg, END};

/// Writes scan findings to csv.
///
/// # Arguments
///
/// * 'rx' - crossbeam receiver that receives thread_message enum.
/// * "root" - the starting point for the scan.
/// * "output_path" - the designated output directory for the csv files.
pub fn writer(output_path: PathBuf, root: &String, rx: Receiver<ScanMessage>) {
    let max_lines: u16 = 10000;
    let mut written_lines: u16 = 0;
    let mut file_suffix: u32 = 1;

    // Create path if it doesn't exist
    if !Path::new(&output_path).exists() {
        fs::create_dir_all(&output_path).unwrap();
    }

    // Create timestamp
    let now = Utc::now();
    let hour = now.hour();
    let timestamp = format!(
        "{:02}-{:02}-{:02}utc_{}{}{}",
        hour,
        now.minute(),
        now.second(),
        now.month(),
        now.day(),
        now.year()
    );

    // Make root safe to be in filename
    let root = root.replace("\\", "").replace(":", "").replace("/", "");

    let output_file = update_filename(&root, &timestamp, &output_path, file_suffix);

    let mut writer = build_writer(&output_file).unwrap();

    let mut queue: VecDeque<ScanMessage> = VecDeque::new();
    thread::spawn(move || loop {
        if written_lines == max_lines {
            file_suffix += 1;
            writer = build_writer(&update_filename(
                &root,
                &timestamp,
                &output_path,
                file_suffix,
            ))
            .unwrap();
            written_lines = 0;
        }
        match rx.try_recv() {
            Ok(m) => queue.push_back(m),
            Err(_) => (),
        }
        if !queue.is_empty() {
            match queue.pop_front().unwrap() {
                Msg(r) => {
                    written_lines += 1;
                    writer.serialize(r).unwrap()
                }
                END => {
                    println!("Close message received: Writer closed");
                    break;
                }
            }
        }
    });
}

fn build_writer(file_path: &PathBuf) -> Result<Writer<File>, Box<dyn Error>> {
    let writer = WriterBuilder::new()
        .has_headers(true)
        .from_path(file_path)?;
    Ok(writer)
}

fn update_filename(
    root: &String,
    timestamp: &String,
    file_path: &PathBuf,
    file_suffix: u32,
) -> PathBuf {
    let mut updated_filename = file_path.clone();
    updated_filename.push(format!("{root}_{timestamp}_{file_suffix}.csv"));
    updated_filename
}
