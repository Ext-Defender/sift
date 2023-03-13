use regex::Regex;
#[allow(unused, dead_code)]
use std::path::PathBuf;

pub fn scan_pdf(path: PathBuf, keyword_patterns: &Vec<Regex>) -> Option<(Vec<String>, String)> {
    println!("{:?}", path);
    None
}

pub fn scan_xlsx(path: PathBuf, keyword_patterns: &Vec<Regex>) -> Option<(Vec<String>, String)> {
    todo!();
}

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

pub fn scan_txt(path: PathBuf, keyword_patterns: &Vec<Regex>) -> Option<(Vec<String>, String)> {
    todo!();
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
