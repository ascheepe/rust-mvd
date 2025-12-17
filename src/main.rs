use chrono::{DateTime, Utc};
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

struct Config {
    format_string: String,
    source_dir: PathBuf,
}

struct ParseError;

fn parse_config(args: &[String]) -> Result<Config, ParseError> {
    let mut format_string = "%Y%m".to_string();
    let source_dir: PathBuf;

    match args.len() {
        1 => {
            source_dir = args[0].clone().into();
        }
        3 => {
            if args[0] != "-f" {
                return Err(ParseError);
            }
            format_string = args[1].clone();
            source_dir = args[2].clone().into();
        }
        _ => {
            return Err(ParseError);
        }
    }

    Ok(Config {
        format_string,
        source_dir,
    })
}

fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

fn move_files(config: &Config) -> io::Result<()> {
    let source_dir = &config.source_dir;

    for entry in fs::read_dir(&source_dir)? {
        let entry = entry?;

        if !entry.file_type()?.is_file() {
            continue;
        }

        let metadata = entry.metadata()?;
        let mtime = metadata.modified()?;

        // Convert SystemTime -> chrono::DateTime<Utc>
        let datetime: DateTime<Utc> = mtime.into();
        let raw_dir = datetime.format(&config.format_string).to_string();
        let destdir = sanitize_filename(&raw_dir);

        fs::create_dir_all(&destdir)?;

        // Build destination path
        let mut destination = PathBuf::from(&destdir);
        if let Some(filename) = entry.path().file_name() {
            destination.push(filename);
        }

        // println!("mv {:?} -> {:?}", entry.path(), destination);

        fs::rename(entry.path(), destination)?;
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = match parse_config(&args[1..]) {
        Ok(config) => config,
        Err(_) => {
            eprintln!("usage: mvd [-f fmt] directory");
            return;
        }
    };

    match move_files(&config) {
        Ok(_) => (),
        Err(err) => {
            eprintln!("Error moving files: {err}");
        }
    };
}
