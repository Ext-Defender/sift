use chrono::prelude::*;
use confy;
use rpassword;
use std::error::Error;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::from_utf8;
use std::{env, fs};

use crate::args::Args;
use crate::encryption;
use crate::settings::ConfigFile;

use crate::scan_manager::scan_manager;
use crate::scan_settings::ScanSettings;

pub fn run(config: Args) -> Result<(), Box<dyn Error>> {
    if config.reset_settings {
        println!("***Clearing configs***");
        confy::store("sift", &*config.config_file, ConfigFile::default())?;
    }

    let mut app_settings: ConfigFile = confy::load("sift", &*config.config_file)?;

    let key = "SIFTPW";
    let mut password = match env::var(key) {
        Ok(p) => {
            println!("INFO: Using password from env\n");
            p
        }
        Err(_) => String::new(),
    };

    if config.roots.is_some() {
        app_settings.initial_scan = true;
        for root in config.roots.unwrap() {
            if !app_settings.roots.contains(&root) && Path::new(&root).exists() {
                println!("adding root: {}\n", root);
                app_settings.roots.push(root.clone());
            } else {
                println!("already in list: {}\n", root);
            }
        }
    }

    if config.remove_roots.is_some() {
        for root_to_remove in config.remove_roots.unwrap() {
            let i = app_settings.roots.iter().position(|r| *r == root_to_remove);
            match i {
                Some(i) => {
                    println!("removing root: {}\n", root_to_remove);
                    app_settings.roots.remove(i);
                }
                None => println!("not found: {}\n", root_to_remove),
            }
        }
    }

    if config.display_patterns
        || config.scan
        || config.full_scan
        || config.remove_patterns.is_some()
        || config.add_patterns.is_some()
        || config.pattern_file.is_some()
    {
        if password.is_empty() {
            password = match app_settings.secret {
                Some(_) => rpassword::prompt_password("Enter password: ")?,
                None => rpassword::prompt_password("Enter new password: ")?,
            };
        }

        let valid_password: bool = match app_settings.secret {
            None => {
                let hashed_password = encryption::hash_password(&password).unwrap();
                app_settings.secret = Some(hashed_password);
                true
            }
            _ => encryption::verify_password(&password, app_settings.secret.as_ref().unwrap())
                .unwrap(),
        };

        if !valid_password {
            eprintln!("\nInvalid password");
            std::process::exit(1);
        }
    }

    if config.add_patterns.is_some() {
        let keywords = load_keywords(&app_settings.keywords, &password).unwrap();
        for word in config.add_patterns.unwrap() {
            if !keywords.contains(&word) {
                println!("adding patterns: {}", word);
                app_settings
                    .keywords
                    .push(encryption::encrypt(word.as_bytes(), &password));
                app_settings.initial_scan = true;
            }
        }
        println!();
    }

    if config.pattern_file.is_some() {
        let keywords = load_keywords(&app_settings.keywords, &password).unwrap();
        let mut file = fs::File::open(config.pattern_file.unwrap()).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let patterns: Vec<&str> = contents
            .split(|c| c == ',' || c == '\n' || c == '\r')
            .collect();
        for pattern in patterns {
            let pattern = pattern.to_string();
            if !keywords.contains(&pattern) && pattern.len() > 0 {
                println!("adding patterns: {:?}", pattern);
                app_settings
                    .keywords
                    .push(encryption::encrypt(pattern.as_bytes(), &password));
                app_settings.initial_scan = true;
            }
        }
    }

    if config.remove_patterns.is_some() {
        let mut keywords = load_keywords(&app_settings.keywords, &password).unwrap();
        for word in config.remove_patterns.unwrap() {
            let i = keywords.iter().position(|w| *w == word);
            match i {
                Some(i) => {
                    println!("removing patterns: {}", word);
                    keywords.remove(i);
                    app_settings.keywords.remove(i);
                }
                None => println!("Not found: {}", word),
            }
        }
        println!();
    }

    if config.output_directory.is_some() {
        println!(
            "changing output directory to: {:?}",
            config.output_directory.as_ref().unwrap()
        );
        app_settings.output_directory = config.output_directory;
    }

    confy::store("sift", &*config.config_file, &app_settings)?;

    if config.print_settings {
        println!("{:^60}", "_Config Settings_");
        println!("Config name:{:^60}", config.config_file);
        println!("Max scan threads:{:^50}", app_settings.max_scan_threads);
        println!("Max file threads:{:^50}", app_settings.max_file_threads);
        println!("Max write lines:{:^51}", app_settings.max_write_lines);
        println!("Initial scan:{:^58}", app_settings.initial_scan);
        println!(
            "Output directory:{:^50}",
            app_settings.output_directory.as_ref().unwrap()
        );
        println!("Roots:{:#?}", app_settings.roots);
        println!("Last scan:{:^63}", app_settings.time_last_scan);
        println!("\nConfig file path:");
        println!(
            "\t{}\n",
            confy::get_configuration_file_path("sift", &*config.config_file)
                .unwrap()
                .display()
        );
    }

    let keywords = load_keywords(&app_settings.keywords, &password).unwrap();

    if config.display_patterns {
        println!("{:^50}", "_keywords_");
        for (index, keyword) in keywords.iter().enumerate() {
            println!("{:<1}: {:>5}", index + 1, keyword);
        }
        println!();
    }

    if !prescan_checks(&app_settings) {
        println!("!!!Pre-scan checks failed.!!!");
        return Ok(());
    }

    if config.scan || config.full_scan {
        let full_scan: bool;
        if app_settings.initial_scan {
            println!("Conducting initial scan.");
            full_scan = true;
        } else {
            full_scan = config.full_scan;
        }
        app_settings.initial_scan = false;
        let last_scan_time: DateTime<Utc> = match app_settings.time_last_scan.parse() {
            Ok(t) => t,
            Err(_) => Utc::now(),
        };
        let scan_settings = ScanSettings::new(
            full_scan,
            config.verbose,
            keywords,
            app_settings.roots.clone(),
            Some(last_scan_time),
            PathBuf::from(&app_settings.output_directory.as_ref().unwrap()),
            config.case_sensitive,
            app_settings.max_scan_threads,
            app_settings.max_file_threads,
            app_settings.max_write_lines,
        );
        app_settings.time_last_scan = Utc::now().to_string();
        scan_manager(scan_settings);
    }

    confy::store("sift", &*config.config_file, &app_settings)?;

    Ok(())
}

// HELPERS //
fn load_keywords(
    encrypted_keywords: &Vec<String>,
    password: &str,
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut decrypted_keywords: Vec<String> = Vec::new();
    for word in encrypted_keywords {
        let decrypted_bytes = encryption::decrypt(word.as_str(), password)?;
        let decrypted_word = from_utf8(&decrypted_bytes)?;
        decrypted_keywords.push(String::from(decrypted_word));
    }
    Ok(decrypted_keywords)
}

fn prescan_checks(app_settings: &ConfigFile) -> bool {
    let mut scan_status = true;
    if app_settings.output_directory.is_none() {
        println!("!Pre-scan check failed:: No output directory designated.");
        scan_status = false
    }
    if app_settings.keywords.is_empty() {
        println!("!Pre-scan check failed:: No keywords designated.");
        scan_status = false;
    }
    if app_settings.roots.is_empty() {
        println!("!Pre-scan check failed:: No root directories designated.");
        scan_status = false;
    }
    if app_settings.secret.is_none() {
        println!("!Pre-scan check failed:: No application secret stored");
        scan_status = false;
    }
    if scan_status {
        println!("Pre-scan checks passed");
    }
    scan_status
}
