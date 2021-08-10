use async_std::fs::{read_to_string, write, File};
use async_std::path::PathBuf;
use async_std::prelude::*;
use clap::{crate_version, App, Arg, SubCommand};

mod diff;
mod hash;
mod package;

const CONTENT_FILENAME: &str = "content.zip";
const UPDATE_FILENAME: &str = "update.zip";
const REMOVED_FILENAME: &str = "removed";
const APP_PREFIX: &str = "app";

#[async_std::main]
async fn main() -> std::io::Result<()> {
    let matches = App::new("differy")
        .version(crate_version!())
        .author("Florian Dieminger <me@fiji-flo.de>")
        .about("Hash and diff all the things")
        .subcommand(
            SubCommand::with_name("hash")
                .about("Hash all files")
                .arg(
                    Arg::with_name("PATH")
                        .required(true)
                        .help("Path to scan")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("out")
                        .long("out")
                        .short("o")
                        .required(true)
                        .help("Output file")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("diff")
                .about("Diff two hash files")
                .arg(
                    Arg::with_name("old")
                        .required(true)
                        .help("Old hash file")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("new")
                        .required(true)
                        .help("New hash file")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("out")
                        .long("out")
                        .short("o")
                        .required(true)
                        .help("Output file")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("package")
                .about("Package an update zip")
                .arg(
                    Arg::with_name("diff")
                        .long("diff")
                        .short("d")
                        .required(true)
                        .help("Diff file")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("root")
                        .required(true)
                        .help("Build root")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("prefix")
                        .long("prefix")
                        .short("p")
                        .required(false)
                        .help("Output prefix")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("out")
                        .long("out")
                        .short("o")
                        .required(true)
                        .help("Output folder")
                        .takes_value(true),
                ),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("hash") {
        let path = matches.value_of("PATH").unwrap();
        let out = matches.value_of("out").unwrap();
        let mut out_file = File::create(out).await?;
        let path = PathBuf::from(path);
        let mut hashes = vec![];
        hash::hash_all(&path, &mut hashes, &path).await?;
        for (hash, filename) in hashes {
            out_file
                .write_all(format!("{} {}\n", hash, filename).as_bytes())
                .await?;
        }
    }
    if let Some(matches) = matches.subcommand_matches("diff") {
        let old = matches.value_of("old").unwrap();
        let new = matches.value_of("new").unwrap();
        let out = matches.value_of("out").unwrap();
        let mut out_file = File::create(out).await?;
        let old = PathBuf::from(old);
        let new = PathBuf::from(new);
        let (removed, added, modified) = diff::diff(&old, &new).await?;
        for filename in removed {
            out_file
                .write_all(format!("- {}\n", filename).as_bytes())
                .await?;
        }
        for filename in added {
            out_file
                .write_all(format!("+ {}\n", filename).as_bytes())
                .await?;
        }
        for filename in modified {
            out_file
                .write_all(format!("~ {}\n", filename).as_bytes())
                .await?;
        }
    }
    if let Some(matches) = matches.subcommand_matches("package") {
        let diff = matches.value_of("diff").unwrap();
        let root = matches.value_of("root").unwrap();
        let out = matches.value_of("out").unwrap();
        let prefix = matches.value_of("prefix");
        let root = PathBuf::from(root);
        let diff = PathBuf::from(diff);
        let diff = read_to_string(diff).await?;
        let mut update = vec![];
        let mut remove = vec![];
        for line in diff.split('\n') {
            if let Some(file) = line.strip_prefix("+ ") {
                update.push(file.to_string())
            }
            if let Some(file) = line.strip_prefix("~ ") {
                update.push(file.to_string())
            }
            if let Some(file) = line.strip_prefix("- ") {
                remove.push(file.to_string())
            }
        }
        let update_out = build_path(out, UPDATE_FILENAME, &prefix, false);
        package::zip_files(&update, &root, &update_out, false).await?;
        let update_out = build_path(out, UPDATE_FILENAME, &prefix, true);
        package::zip_files(&update, &root, &update_out, true).await?;

        let removed_out = build_path(out, REMOVED_FILENAME, &prefix, false);
        write(removed_out, remove.join("\n").as_bytes()).await?;

        let content_out = build_path(out, CONTENT_FILENAME, &prefix, false);
        package::zip_dir(&root, &content_out, true).await?;
        let content_out = build_path(out, CONTENT_FILENAME, &prefix, true);
        package::zip_dir(&root, &content_out, true).await?;
    }
    Ok(())
}

fn build_path(base: &str, file_name: &str, prefix: &Option<&str>, app: bool) -> PathBuf {
    let mut full_name = String::new();
    if let Some(prefix) = prefix {
        full_name.push_str(prefix);
        full_name.push('-');
    }
    if app {
        full_name.push_str(APP_PREFIX);
        full_name.push('-');
    }
    full_name.push_str(file_name);
    let mut out = PathBuf::from(base);
    out.push(full_name);
    out
}
