use clap::{value_parser, Arg, ArgAction, Command};
use std::error::Error;

#[derive(Debug)]
pub struct Args {
    pub scan: bool,
    pub full_scan: bool,
    pub verbose: bool,
    pub roots: Option<Vec<String>>,
    pub remove_roots: Option<Vec<String>>,
    pub add_keywords: Option<Vec<String>>,
    pub remove_keywords: Option<Vec<String>>,
    pub display_keywords: bool,
    pub output_directory: Option<String>,
    pub print_settings: bool,
    pub reset_settings: bool,
    pub case_sensitive: bool,
    pub pattern_file: Option<String>,
    pub config_file: String,
}

pub fn get_args() -> Result<Args, Box<dyn Error>> {
    let matches = Command::new("sift")
        .author("Bryan Vinton, bvinton@ext-defender.net")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Searches for Regex patterns in common document types")
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
                .action(ArgAction::Append)
                .num_args(1..),
        )
        .arg(
            Arg::new("remove_root")
                .short('R')
                .help("remove a directory from search")
                .action(ArgAction::Append)
                .num_args(1..),
        )
        .arg(
            Arg::new("add_keyword")
                .short('a')
                .value_name("keywords_to_add")
                .action(ArgAction::Append)
                .num_args(1..)
                .help("adds keyword(s) to config file"),
        )
        .arg(
            Arg::new("remove_keyword")
                .short('A')
                .value_name("keywords_to_remove")
                .help("removes keyword from config file")
                .action(ArgAction::Append)
                .num_args(1..),
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
            Arg::new("print_settings")
                .short('l')
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
        .arg(
            Arg::new("pattern_file")
                .short('f')
                .help("provide a path to a text file containing patterns (comma separated)")
                .default_value(None),
        )
        .arg(
            Arg::new("config_name")
                .short('c')
                .help("specify the config name you want to use")
                .default_value("Default"),
        )
        .get_matches();

    Ok(Args {
        scan: matches.get_flag("scan"),
        full_scan: matches.get_flag("full_scan"),
        verbose: matches.get_flag("verbose"),
        roots: match matches.get_many::<String>("root") {
            Some(c) => Some(c.into_iter().map(|v| v.clone()).collect()),
            None => None,
        },
        remove_roots: match matches.get_many::<String>("remove_root") {
            Some(c) => Some(c.into_iter().map(|v| v.clone()).collect()),
            None => None,
        },
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
        pattern_file: match matches.get_one::<String>("pattern_file") {
            Some(pf) => Some(pf.to_string()),
            None => None,
        },
        config_file: matches
            .get_one::<String>("config_name")
            .unwrap()
            .to_string(),
    })
}
