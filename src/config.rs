use clap::{value_parser, Arg, ArgAction, Command};
use std::error::Error;

#[derive(Debug)]
pub struct Config {
    pub scan: bool,
    pub full_scan: bool,
    pub verbose: bool,
    pub root: Option<Vec<String>>,
    pub remove_roots: Option<Vec<String>>,
    pub display_root: bool,
    pub add_keywords: Option<Vec<String>>,
    pub remove_keywords: Option<Vec<String>>,
    pub display_keywords: bool,
    pub output_directory: Option<String>,
    pub print_output_directory: bool,
    pub print_settings: bool,
    pub reset_settings: bool,
    pub case_sensitive: bool,
    pub pattern_file: Option<String>,
}

pub fn get_args() -> Result<Config, Box<dyn Error>> {
    let matches = Command::new("sift")
        .author("Bryan Vinton, bryan.vinton18@gmail.com")
        .version("0.2.0")
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
                .help("prints the output directory path")
                .action(ArgAction::SetTrue),
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
        .arg(
            Arg::new("pattern_file")
                .short('f')
                .help("provide a path to a text file containing patterns (comma separated)"),
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
        print_output_directory: matches.get_flag("print_output_directory"),
        print_settings: matches.get_flag("print_settings"),
        reset_settings: matches.get_flag("reset_config"),
        case_sensitive: matches.get_flag("case-sensitive"),
        pattern_file: match matches.get_one::<String>("pattern_file") {
            Some(pf) => Some(pf.to_string()),
            None => None,
        },
    })
}
