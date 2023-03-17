use pdf_extract;
use regex::Regex;
use std::error::Error;
use std::fs;
use std::io::BufReader;
use std::io::Read;
use std::path::PathBuf;
use zip;

use xml::reader::EventReader;

pub fn scan_pdf(
    path: &PathBuf,
    keyword_patterns: &Vec<Regex>,
) -> Result<Option<(Vec<String>, String)>, Box<dyn Error>> {
    let mut findings: Vec<String> = vec![];
    let content = match pdf_extract::extract_text(&path) {
        Ok(c) => c,
        Err(_) => return Ok(None),
    };
    for pattern in keyword_patterns.iter() {
        if pattern.is_match(&content) {
            findings.push(pattern.to_string());
        }
    }
    if findings.len() > 0 {
        return Ok(Some((findings, path.to_str().unwrap().to_string())));
    }
    Ok(None)
}

pub fn scan_ooxml(
    path: &PathBuf,
    keyword_patterns: &Vec<Regex>,
) -> Result<Option<(Vec<String>, String)>, Box<dyn Error>> {
    let mut findings = vec![];
    let file = fs::File::open(path).unwrap();
    let mut archive = match zip::ZipArchive::new(file) {
        Ok(z) => z,
        Err(_) => return Ok(None),
    };
    let mut items = vec![];

    for item in archive.file_names() {
        items.push(item.to_owned());
    }
    for item in items {
        let reader = BufReader::new(archive.by_name(&item)?);
        let parser = EventReader::new(reader);
        for event in parser {
            match event {
                Ok(xml::reader::XmlEvent::Characters(c)) => {
                    for pattern in keyword_patterns.iter() {
                        if pattern.is_match(&c) && !findings.contains(&pattern.to_string()) {
                            findings.push(pattern.to_string());
                        }
                    }
                }
                _ => continue,
            }
        }
    }

    if findings.len() > 0 {
        return Ok(Some((findings, path.to_str().unwrap().to_string())));
    }

    Ok(None)
}

pub fn scan_legacy_office(
    path: &PathBuf,
    keyword_patterns: &Vec<Regex>,
) -> Result<Option<(Vec<String>, String)>, Box<dyn Error>> {
    let mut findings = vec![];
    let mut file = fs::File::open(path)?;
    let mut buffer = vec![];
    file.read_to_end(&mut buffer)?;
    let mut contents = String::new();
    for c in &buffer {
        if c.is_ascii() {
            let ch = *c as char;
            if ch.is_ascii_alphanumeric() {
                contents.push(ch);
            }
            if ch.is_ascii_whitespace() {
                contents.push(' ');
            }
        }
    }

    for keyword in keyword_patterns.iter() {
        if keyword.is_match(&contents) {
            findings.push(keyword.to_string())
        }
    }

    if findings.len() > 0 {
        return Ok(Some((findings, path.to_str().unwrap().to_string())));
    }

    Ok(None)
}

pub fn scan_txt(
    path: &PathBuf,
    keyword_patterns: &Vec<Regex>,
) -> Result<Option<(Vec<String>, String)>, Box<dyn Error>> {
    let mut findings: Vec<String> = vec![];
    let contents = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Ok(None), //TODO log
    };
    for keyword in keyword_patterns.iter() {
        if keyword.is_match(&contents) {
            findings.push(keyword.to_string())
        }
    }
    if findings.len() > 0 {
        return Ok(Some((findings, path.to_str().unwrap().to_string())));
    }
    Ok(None)
}

pub fn scan_rtf(
    path: &PathBuf,
    keyword_patterns: &Vec<Regex>,
) -> Result<Option<(Vec<String>, String)>, Box<dyn Error>> {
    let mut findings = vec![];
    let file = fs::File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;

    for keyword in keyword_patterns.iter() {
        if keyword.is_match(&buf) {
            findings.push(keyword.to_string())
        }
    }

    if findings.len() > 0 {
        return Ok(Some((findings, path.to_str().unwrap().to_string())));
    }

    Ok(None)
}

// pub fn scan_wpd(path: PathBuf, keyword_patterns: &Vec<Regex>) -> Option<(Vec<String>, String)> {
//     todo!();
// }
