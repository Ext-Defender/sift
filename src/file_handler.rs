use msg_parser::Outlook;
use regex::Regex;
use std::error::Error;
use std::fs;
use std::io::BufReader;
use std::io::Read;
use std::path::PathBuf;
use zip;

use xml::reader::EventReader;

//testing
// use std::time::Instant;

pub fn scan_file(path: &PathBuf, patterns: &Vec<Regex>) -> Option<(Vec<String>, String)> {
    let ret = match path.extension() {
        Some(ext) => match ext.to_str() {
            Some("pdf") => scan_pdf(&path, &patterns),
            Some("xlsx") | Some("pptx") | Some("docx") => {
                scan_ooxml(&path, &patterns).unwrap_or(None)
            }
            Some("txt") | Some("xml") | Some("html") | Some("htm") | Some("csv") => {
                scan_txt(&path, &patterns).unwrap_or(None)
            }
            Some("rtf") | Some("wpd") => scan_rtf(&path, &patterns).unwrap_or(None),
            Some("doc") | Some("ppt") | Some("xls") => {
                scan_legacy_office(&path, &patterns).unwrap_or(None)
            }
            Some("msg") => scan_msg(&path, &patterns),
            _ => None,
        },
        _ => None,
    };

    ret
}

fn scan_msg(path: &PathBuf, patterns: &Vec<Regex>) -> Option<(Vec<String>, String)> {
    let mut findings: Vec<String> = Vec::new();
    let content = match Outlook::from_path(path) {
        Ok(c) => c,
        Err(_) => return None,
    };
    let content = content.to_json();
    for pattern in patterns {
        match pattern.captures(content.as_ref().unwrap()) {
            Some(cap) => {
                for finding in cap.iter() {
                    let finding = finding.unwrap().as_str().to_string();
                    if !findings.contains(&finding) {
                        findings.push(finding);
                    }
                }
            }
            _ => (),
        }
    }
    Some((findings, path.to_str().unwrap().to_string()))
}

fn scan_pdf(path: &PathBuf, patterns: &Vec<Regex>) -> Option<(Vec<String>, String)> {
    use lopdf::Document;

    let mut findings = Vec::new();

    let content = match Document::load(path) {
        Ok(doc) => {
            let pages = doc.get_pages();
            let mut texts = String::new();

            for (i, _) in pages.iter().enumerate() {
                let page_number = (i + 1) as u32;
                let text = doc.extract_text(&[page_number]);
                texts.push_str(&text.unwrap_or_default());
            }

            // println!("Text: {}", texts);
            texts
        }
        Err(_) => return None,
    };

    for pattern in patterns.iter() {
        match pattern.captures(&content) {
            Some(cap) => {
                for finding in cap.iter() {
                    let finding = finding.unwrap().as_str().to_string();
                    if !findings.contains(&finding) {
                        findings.push(finding);
                    }
                }
            }
            _ => (),
        }
    }

    Some((findings, path.to_str().unwrap().to_string()))
}

fn scan_ooxml(
    path: &PathBuf,
    patterns: &Vec<Regex>,
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
                    for pattern in patterns.iter() {
                        match pattern.captures(&c) {
                            Some(cap) => {
                                for finding in cap.iter() {
                                    let finding = finding.unwrap().as_str().to_string();
                                    if !findings.contains(&finding) {
                                        findings.push(finding);
                                    }
                                }
                            }
                            _ => (),
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

fn scan_legacy_office(
    path: &PathBuf,
    patterns: &Vec<Regex>,
) -> Result<Option<(Vec<String>, String)>, Box<dyn Error>> {
    let mut findings = vec![];
    let mut file = fs::File::open(path)?;
    let mut buffer = vec![];
    file.read_to_end(&mut buffer)?;
    let mut content = String::new();
    for c in &buffer {
        if c.is_ascii() {
            let ch = *c as char;
            if ch.is_ascii_alphanumeric() {
                content.push(ch);
            }
            if ch.is_ascii_whitespace() {
                content.push(' ');
            }
        }
    }

    for pattern in patterns.iter() {
        match pattern.captures(&content) {
            Some(cap) => {
                for finding in cap.iter() {
                    let finding = finding.unwrap().as_str().to_string();
                    if !findings.contains(&finding) {
                        findings.push(finding);
                    }
                }
            }
            _ => (),
        }
    }

    if findings.len() > 0 {
        return Ok(Some((findings, path.to_str().unwrap().to_string())));
    }

    Ok(None)
}

fn scan_txt(
    path: &PathBuf,
    patterns: &Vec<Regex>,
) -> Result<Option<(Vec<String>, String)>, Box<dyn Error>> {
    let mut findings: Vec<String> = vec![];
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Ok(None), //TODO log
    };
    for pattern in patterns.iter() {
        match pattern.captures(&content) {
            Some(cap) => {
                for finding in cap.iter() {
                    let finding = finding.unwrap().as_str().to_string();
                    if !findings.contains(&finding) {
                        findings.push(finding);
                    }
                }
            }
            _ => (),
        }
    }
    if findings.len() > 0 {
        return Ok(Some((findings, path.to_str().unwrap().to_string())));
    }
    Ok(None)
}

fn scan_rtf(
    path: &PathBuf,
    patterns: &Vec<Regex>,
) -> Result<Option<(Vec<String>, String)>, Box<dyn Error>> {
    let mut findings = vec![];
    let file = fs::File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;

    for pattern in patterns.iter() {
        match pattern.captures(&buf) {
            Some(cap) => {
                for finding in cap.iter() {
                    let finding = finding.unwrap().as_str().to_string();
                    if !findings.contains(&finding) {
                        findings.push(finding);
                    }
                }
            }
            _ => (),
        }
    }

    if findings.len() > 0 {
        return Ok(Some((findings, path.to_str().unwrap().to_string())));
    }

    Ok(None)
}
