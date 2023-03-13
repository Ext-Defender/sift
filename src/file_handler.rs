use pdf_extract;
use regex::Regex;
use std::error::Error;
use std::fs;
#[allow(unused, dead_code)]
use std::path::PathBuf;

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

pub fn scan_xlsx(path: PathBuf, keyword_patterns: &Vec<Regex>) {}

pub fn scan_docx(path: PathBuf, keyword_patterns: &Vec<Regex>) -> Option<(Vec<String>, String)> {
    todo!();
}

pub fn scan_pptx(path: PathBuf, keyword_patterns: &Vec<Regex>) -> Option<(Vec<String>, String)> {
    todo!();
}

pub fn scan_legacy_office(
    path: PathBuf,
    keyword_patterns: &Vec<Regex>,
) -> Option<(Vec<String>, String)> {
    todo!();
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

pub fn scan_rtf(path: PathBuf, keyword_patterns: &Vec<Regex>) -> Option<(Vec<String>, String)> {
    todo!();
}

pub fn scan_wpd(path: PathBuf, keyword_patterns: &Vec<Regex>) -> Option<(Vec<String>, String)> {
    todo!();
}

pub fn scan_xml(path: PathBuf, keyword_patterns: &Vec<Regex>) -> Option<(Vec<String>, String)> {
    todo!();
}
