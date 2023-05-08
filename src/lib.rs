use chrono::prelude::*;
#[allow(unused, dead_code)]
use clap::{value_parser, Arg, ArgAction, Command};
use confy;
use rpassword;
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::str::from_utf8;

mod encryption;
pub mod file_handler;
mod scan;

#[derive(Debug)]
pub struct Config {
    scan: bool,
    full_scan: bool,
    verbose: bool,
    root: Option<Vec<String>>,
    remove_roots: Option<Vec<String>>,
    display_root: bool,
    add_keywords: Option<Vec<String>>,
    remove_keywords: Option<Vec<String>>,
    display_keywords: bool,
    output_directory: Option<String>,
    print_settings: bool,
    reset_settings: bool,
    case_sensitive: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub initial_scan: bool,
    pub output_directory: Option<String>,
    pub keywords: Vec<String>,
    pub roots: Vec<String>,
    pub secret: Option<String>,
    pub time_last_scan: String,
}

impl ::std::default::Default for Settings {
    fn default() -> Self {
        Self {
            initial_scan: true,
            output_directory: None,
            keywords: Vec::new(),
            roots: Vec::new(),
            secret: None,
            time_last_scan: String::new(),
        }
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    if config.reset_settings {
        println!("Clearing configs");
        confy::store("sift", None, Settings::default())?;
    }

    let mut app_settings: Settings = confy::load("sift", None)?;

    let key = "SIFTPW";
    let mut password = match env::var(key) {
        Ok(p) => {
            println!("INFO: Using password from env");
            p
        }
        Err(_) => String::new(),
    };

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
        _ => encryption::verify_password(&password, app_settings.secret.as_ref().unwrap()).unwrap(),
    };

    if !valid_password {
        eprintln!("\nInvalid password");
        std::process::exit(1);
    }
    if config.root.is_some() {
        println!("adding roots: {:?}", config.root);
        app_settings.initial_scan = true;
        for root in config.root.unwrap() {
            if !app_settings.roots.contains(&root) {
                app_settings.roots.push(root);
            } else {
                println!("!'{}' already in root list", root);
            }
        }
    }

    if config.remove_roots.is_some() {
        println!("removing roots: {:?}", config.remove_roots);
        for root_to_remove in config.remove_roots.unwrap() {
            for (index, root) in app_settings.roots.clone().iter().enumerate() {
                if &root_to_remove == root {
                    app_settings.roots.remove(index);
                }
            }
        }
    }

    if config.add_keywords.is_some() {
        let keywords = load_keywords(&app_settings.keywords, &password).unwrap();
        println!("adding keywords: {:?}", config.add_keywords);
        app_settings.initial_scan = true;
        for word in config.add_keywords.unwrap() {
            if !keywords.contains(&word) {
                app_settings
                    .keywords
                    .push(encryption::encrypt(word.as_bytes(), &password));
                app_settings.initial_scan = true;
            }
        }
    }

    if config.remove_keywords.is_some() {
        let keywords = load_keywords(&app_settings.keywords, &password).unwrap();
        println!("removing keywords: {:?}", config.remove_keywords);
        for word in config.remove_keywords.unwrap() {
            for (index, keyword) in keywords.iter().enumerate() {
                if &word == keyword {
                    app_settings.keywords.remove(index);
                }
            }
        }
    }

    if config.output_directory.is_some() {
        println!(
            "changing output directory to: {:?}",
            config.output_directory
        );
        app_settings.initial_scan = true;
        app_settings.output_directory = config.output_directory;
    }

    confy::store("sift", None, &app_settings)?;

    let keywords = load_keywords(&app_settings.keywords, &password).unwrap();

    if config.display_keywords {
        println!("_keywords_");
        for (index, keyword) in keywords.iter().enumerate() {
            println!("{:<1}: {:>5}", index + 1, keyword);
        }
    }

    if config.display_root {
        println!("_roots_");
        for (index, root) in app_settings.roots.iter().enumerate() {
            println!("{:<1}: {:>5}", index + 1, root);
        }
    }

    if config.print_settings {
        println!("{:#?}", app_settings);
        println!("{:?}", confy::get_configuration_file_path("sift", None));
    }

    if !prescan_checks(&app_settings) {
        println!("!!!Pre-scan checks failed.!!!");
        return Ok(());
    }

    if config.scan || config.full_scan {
        if config.full_scan {
            app_settings.initial_scan = false;
            let scan = scan::Scan::new(
                config.full_scan,
                config.verbose,
                keywords.clone(),
                app_settings.roots.clone(),
                None,
                PathBuf::from(&app_settings.output_directory.as_ref().unwrap()),
                config.case_sensitive,
            );
            app_settings.time_last_scan = scan.time_stamp.to_string();
        } else if app_settings.initial_scan {
            app_settings.initial_scan = false;
            let scan = scan::Scan::new(
                true,
                config.verbose,
                keywords.clone(),
                app_settings.roots.clone(),
                None,
                PathBuf::from(&app_settings.output_directory.as_ref().unwrap()),
                config.case_sensitive,
            );
            app_settings.time_last_scan = scan.time_stamp.to_string();
        } else {
            let last_scan_time: DateTime<Utc> = app_settings.time_last_scan.parse().unwrap();
            let scan = scan::Scan::new(
                false,
                config.verbose,
                keywords.clone(),
                app_settings.roots.clone(),
                Some(last_scan_time),
                PathBuf::from(&app_settings.output_directory.as_ref().unwrap()),
                config.case_sensitive,
            );
            app_settings.time_last_scan = scan.time_stamp.to_string();
        }
    }

    confy::store("sift", None, &app_settings)?;

    Ok(())
}

pub fn get_args() -> Result<Config, Box<dyn Error>> {
    let matches = Command::new("sift")
        .author("Bryan Vinton, bryan.vinton18@gmail.com")
        .version("0.1.0")
        .about("Searches for keywords in various document types")
        .arg(
            Arg::new("scan")
                .short('s')
                .help("conducts scan of directory, after initial scan defaults to partial")
                .action(ArgAction::SetTrue)
                .conflicts_with("full_scan"),
        )
        .arg(
            Arg::new("full_scan")
                .short('S')
                .help("conducts full scan of directory")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .help("verbose output")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("root")
                .short('r')
                .help("add directory to search")
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("remove_root")
                .short('R')
                .help("remove a directory from search")
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("display_root")
                .short('m')
                .help("display directories to be searched")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("add_keyword")
                .short('a')
                .value_name("keywords_to_add")
                .action(ArgAction::Append)
                .num_args(1..)
                .help("adds keyword(s) to database"),
        )
        .arg(
            Arg::new("remove_keyword")
                .short('A')
                .value_name("keywords_to_remove")
                .help("removes keyword from database"),
        )
        .arg(
            Arg::new("display_keywords")
                .short('k')
                .value_name("display_keywords")
                .help("displays decrypted keywords list")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("set_output_directory")
                .short('o')
                .value_name("output_directory")
                .value_parser(value_parser!(String))
                .help("sets the output directory in app settings"),
        )
        .arg(
            Arg::new("print_output_directory")
                .short('l')
                .help("prints the output directory path"),
        )
        .arg(
            Arg::new("print_settings")
                .short('z')
                .value_name("print_settings")
                .action(ArgAction::SetTrue)
                .help("Print settings from config file."),
        )
        .arg(
            Arg::new("reset_config")
                .short('q')
                .action(ArgAction::SetTrue)
                .help("Resets the config file to default (only way to reset password)"),
        )
        .arg(
            Arg::new("case-sensitive")
                .short('i')
                .help("Makes scan case-sensitive: scans are not case-sensitive by default.")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    Ok(Config {
        scan: matches.get_flag("scan"),
        full_scan: matches.get_flag("full_scan"),
        verbose: matches.get_flag("verbose"),
        root: match matches.get_many::<String>("root") {
            Some(c) => Some(c.into_iter().map(|v| v.clone()).collect()),
            None => None,
        },
        remove_roots: match matches.get_many::<String>("remove_root") {
            Some(c) => Some(c.into_iter().map(|v| v.clone()).collect()),
            None => None,
        },
        display_root: matches.get_flag("display_root"),
        add_keywords: match matches.get_many::<String>("add_keyword") {
            Some(c) => Some(c.into_iter().map(|v| v.clone()).collect()),
            None => None,
        },
        remove_keywords: match matches.get_many::<String>("remove_keyword") {
            Some(c) => Some(c.into_iter().map(|v| v.clone()).collect()),
            None => None,
        },
        display_keywords: matches.get_flag("display_keywords"),
        output_directory: match matches.get_one::<String>("set_output_directory") {
            Some(c) => Some(c.clone()),
            None => None,
        },
        print_settings: matches.get_flag("print_settings"),
        reset_settings: matches.get_flag("reset_config"),
        case_sensitive: matches.get_flag("case-sensitive"),
    })
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

fn prescan_checks(app_settings: &Settings) -> bool {
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
