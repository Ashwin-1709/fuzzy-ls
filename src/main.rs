extern crate clap;
mod editor;
mod search;
use clap::{ArgAction, Parser};
use regex::Regex;
use std::collections::BTreeSet;

#[derive(Parser)]
#[clap(
    name = "ffs",
    version = "0.1.0",
    about = "Fuzzy file search command line tool.",
    author = "Ashwin Pugalia"
)]
struct Cli {
    /// Query string used for the search.
    #[clap(
        help = "Query used for the search. Default search mode is fuzzy search within recursive directories."
    )]
    query: String,

    /// Use query as a regex pattern.
    #[clap(short, long, action = ArgAction::SetTrue, help = "Query is a regex pattern and the search is performed using the regex.")]
    regex: bool,

    /// Use query as an exact pattern.
    #[clap(short = 'p', long, action = ArgAction::SetTrue, help = "Exact pattern matching is done for the query.")]
    exact: bool,

    /// Exclude files of specific extensions.
    #[clap(
        short = 'e',
        long,
        help = "Exclude files of specific extensions.",
        value_name = ".ext",
        num_args = 0..,
    )]
    exclude: Vec<String>,

    /// Focus search on specific set of extensions.
    #[clap(
        short = 'f',
        long,
        help = "Focus search on specific set of extensions. In case both exclude and focus are provided, focus takes precedence.",
        value_name = ".ext",
        num_args = 0..,
    )]
    focus: Vec<String>,

    /// Default code editor to open the files.
    #[clap(
        short = 'd',
        long,
        help = "Default editor to open files. By default the files are opened in neovim.",
        value_name = "nvim",
        default_value = "nvim"
    )]
    default_editor_command: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    if args.regex && args.exact {
        return Err("Both regex and exact flags cannot be set together.".into());
    }
    let mut exclude_extension_set: BTreeSet<String> = BTreeSet::new();
    let mut focus_extension_set: BTreeSet<String> = BTreeSet::new();
    args.exclude.into_iter().for_each(|ext| {
        exclude_extension_set.insert(ext.to_string());
    });
    args.focus.into_iter().for_each(|ext| {
        focus_extension_set.insert(ext.to_string());
    });
    let files = search::walk_directory(exclude_extension_set, focus_extension_set);
    let mut potential_hits: Vec<(u32, String, String)> = Vec::new();
    if args.exact {
        for (file_name, full_path) in files {
            if file_name == args.query {
                potential_hits.push((0, file_name, full_path));
            }
        }
    } else if args.regex {
        let pattern: Regex = match Regex::new(&args.query) {
            Ok(pattern) => pattern,
            Err(error) => return Err(error.into()),
        };
        for (file_name, full_path) in files {
            match pattern.captures(&file_name) {
                Some(caps) => {
                    if caps
                        .get(0)
                        .map_or(false, |matched| matched.as_str() == file_name)
                    {
                        potential_hits.push((0, file_name, full_path));
                    }
                }
                None => continue,
            }
        }
    } else {
        let mut ranked_files: Vec<(u32, String, String)> = Vec::new();
        for (file_name, full_path) in files {
            match search::score_fuzzy_search(
                args.query.clone(),
                file_name.clone(),
                search::FuzzySearchAlgorithm::DamerauLevenshtein,
            ) {
                Ok(score) => ranked_files.push((score, file_name.clone(), full_path)),
                Err(error) => return Err(error.into()),
            };
        }
        ranked_files.sort_by(|a, b| a.0.cmp(&b.0));
        let threshold: u32 = match args.query.len() {
            0..=4 => (args.query.len() as f32 * 0.25).ceil() as u32,
            5..=10 => (args.query.len() as f32 * 0.35).ceil() as u32,
            _ => (args.query.len() as f32 * 0.45).ceil() as u32,
        };
        for (score, file_name, full_path) in ranked_files {
            if score <= threshold {
                potential_hits.push((score, file_name, full_path));
            } else {
                break;
            }
        }
    }

    if potential_hits.is_empty() {
        println!("No files found.");
    } else {
        println!("{} files found:", potential_hits.len());
        let mut file_number: usize = 1;
        for (score, file_name, full_path) in potential_hits.clone() {
            if score == 0 {
                println!(
                    "{}. \x1b[32m{}\x1b[0m - {}",
                    file_number, file_name, full_path
                ); // Green color for score 0
            } else {
                println!(
                    "{}. \x1b[34m{}\x1b[0m - {}",
                    file_number, file_name, full_path
                ); // Blue color for other scores
            }
            file_number += 1;
        }

        if cfg!(feature = "open_in_editor") {
           return editor::experimental_open_files(
                args.default_editor_command,
                file_number,
                potential_hits,
            );
        }
    }
    return Ok(());
}
