use msg_parser::Outlook;
use regex::Regex;
use std::fs;
use std::io::BufReader;
use std::io::Read;
use std::path::PathBuf;
use zip;

use xml::reader::EventReader;

pub fn scan_file(path: &PathBuf, patterns: &Vec<Regex>) -> Option<(Vec<String>, String)> {
    let ret = match path.extension() {
        Some(ext) => match ext.to_str() {
            Some("pdf") => scan_pdf(&path, &patterns),
            Some("xlsx") | Some("pptx") | Some("docx") => scan_ooxml(&path, &patterns),
            Some("txt") | Some("xml") | Some("html") | Some("htm") | Some("csv") => {
                scan_txt(&path, &patterns)
            }
            Some("rtf") | Some("wpd") => scan_rtf(&path, &patterns),
            Some("doc") | Some("ppt") | Some("xls") => scan_legacy_office(&path, &patterns),
            Some("msg") => scan_msg(&path, &patterns),
            _ => None,
        },
        _ => None,
    };

    ret
}

fn scan_msg(path: &PathBuf, patterns: &Vec<Regex>) -> Option<(Vec<String>, String)> {
    let content = match Outlook::from_path(path) {
        Ok(c) => c,
        Err(e) => {
            log::error!("failed to load msg file: {}", e);
            return None;
        }
    };
    let content = match content.to_json() {
        Ok(c) => c,
        Err(e) => {
            log::error!("failed to load msg json: {}", e);
            return None;
        }
    };

    let findings = search_content(content, patterns);

    if findings.len() > 0 {
        return Some((findings, path.to_str().unwrap().to_string()));
    }

    None
}

fn scan_pdf(path: &PathBuf, patterns: &Vec<Regex>) -> Option<(Vec<String>, String)> {
    use lopdf::Document;

    let content = match Document::load(path) {
        Ok(doc) => {
            let pages = doc.get_pages();
            let mut texts = String::new();

            for (i, _) in pages.iter().enumerate() {
                let page_number = (i + 1) as u32;
                let text = doc.extract_text(&[page_number]);
                texts.push_str(&text.unwrap_or_default());
            }

            texts
        }
        Err(e) => {
            log::error!("failed to load pdf: {}", e);
            return None;
        }
    };

    let findings = search_content(content, patterns);

    if findings.len() > 0 {
        return Some((findings, path.to_str().unwrap().to_string()));
    }

    None
}

fn scan_ooxml(path: &PathBuf, patterns: &Vec<Regex>) -> Option<(Vec<String>, String)> {
    let file = fs::File::open(path).unwrap();
    let mut archive = match zip::ZipArchive::new(file) {
        Ok(z) => z,
        Err(_) => return None,
    };
    let mut items = vec![];
    let mut content = String::new();
    for item in archive.file_names() {
        items.push(item.to_owned());
    }
    for item in items {
        let reader = BufReader::new(match archive.by_name(&item) {
            Ok(i) => i,
            Err(_) => continue,
        });
        let parser = EventReader::new(reader);
        for event in parser {
            match event {
                Ok(xml::reader::XmlEvent::Characters(c)) => {
                    content.push_str(&c);
                }
                _ => continue,
            }
        }
    }

    let findings = search_content(content, patterns);

    if findings.len() > 0 {
        return Some((findings, path.to_str().unwrap().to_string()));
    }

    None
}

fn scan_legacy_office(path: &PathBuf, patterns: &Vec<Regex>) -> Option<(Vec<String>, String)> {
    // let mut findings = vec![];
    let mut file = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) => {
            log::error!("failed to open legacy office file: {}", e);
            return None;
        }
    };
    let mut buffer = vec![];
    match file.read_to_end(&mut buffer) {
        Ok(_) => (),
        Err(e) => {
            log::error!("legacy office file failed to read: {}", e);
            return None;
        }
    };
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

    // for pattern in patterns.iter() {
    //     match pattern.captures(&content) {
    //         Some(cap) => {
    //             for finding in cap.iter() {
    //                 let finding = finding.unwrap().as_str().to_string();
    //                 if !findings.contains(&finding) {
    //                     findings.push(finding);
    //                 }
    //             }
    //         }
    //         _ => (),
    //     }
    // }

    let findings = search_content(content, patterns);

    if findings.len() > 0 {
        return Some((findings, path.to_str().unwrap().to_string()));
    }

    None
}

fn scan_txt(path: &PathBuf, patterns: &Vec<Regex>) -> Option<(Vec<String>, String)> {
    // let mut findings: Vec<String> = vec![];
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            log::error!("ERROR reading txt file: {}", e);
            return None;
        }
    };
    // for pattern in patterns.iter() {
    //     match pattern.captures(&content) {
    //         Some(cap) => {
    //             for finding in cap.iter() {
    //                 let finding = finding.unwrap().as_str().to_string();
    //                 if !findings.contains(&finding) {
    //                     findings.push(finding);
    //                 }
    //             }
    //         }
    //         _ => (),
    //     }
    // }

    let findings = search_content(content, patterns);

    if findings.len() > 0 {
        return Some((findings, path.to_str().unwrap().to_string()));
    }

    None
}

fn scan_rtf(path: &PathBuf, patterns: &Vec<Regex>) -> Option<(Vec<String>, String)> {
    // let mut findings = vec![];
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) => {
            log::error!("ERROR rtf failed to load: {}", e);
            return None;
        }
    };
    let mut reader = BufReader::new(file);
    let mut content = String::new();
    match reader.read_to_string(&mut content) {
        Ok(_) => (),
        Err(e) => {
            log::error!("ERROR processing rtf: {}", e);
        }
    }

    // for pattern in patterns.iter() {
    //     match pattern.captures(&buf) {
    //         Some(cap) => {
    //             for finding in cap.iter() {
    //                 let finding = finding.unwrap().as_str().to_string();
    //                 if !findings.contains(&finding) {
    //                     findings.push(finding);
    //                 }
    //             }
    //         }
    //         _ => (),
    //     }
    // }

    let findings = search_content(content, patterns);

    if findings.len() > 0 {
        return Some((findings, path.to_str().unwrap().to_string()));
    }

    None
}

fn search_content(content: String, patterns: &Vec<Regex>) -> Vec<String> {
    let mut findings = Vec::new();

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
            None => (),
        }
    }

    findings
}
