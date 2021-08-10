use async_std::fs::{read_to_string, write, File};
use async_std::prelude::*;
use async_std::path::PathBuf;
use clap::{App, Arg, SubCommand};

mod diff;
mod hash;
mod package;

const PACKAGE_FILENAME: &str = "update.zip";
const REMOVED_FILENAME: &str = "removed";

#[async_std::main]
async fn main() -> std::io::Result<()> {
    let matches = App::new("differy")
        .version("1.0")
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
        let (removed, added) = diff::diff(&old, &new).await?;
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
    }
    if let Some(matches) = matches.subcommand_matches("package") {
        let diff = matches.value_of("diff").unwrap();
        let root = matches.value_of("root").unwrap();
        let out = matches.value_of("out").unwrap();
        let prefix = matches.value_of("prefix");
        let root = PathBuf::from(root);
        let mut package_out = PathBuf::from(out);
        if let Some(prefix) = prefix {
            package_out.push(format!("{}-{}", &prefix, PACKAGE_FILENAME));
        } else {
            package_out.push(PACKAGE_FILENAME);
        }
        let diff = PathBuf::from(diff);
        let diff = read_to_string(diff).await?;
        let mut added = vec![];
        let mut removed = vec![];
        for line in diff.split('\n') {
            if let Some(file) = line.strip_prefix("+ ") {
                added.push(file.to_string())
            }
            if let Some(file) = line.strip_prefix("- ") {
                removed.push(file.to_string())
            }
        }
        package::zip_files(&added, &root, &package_out).await?;
        let mut removed_out = PathBuf::from(out);
        if let Some(prefix) = prefix {
            removed_out.push(format!("{}-{}", &prefix, REMOVED_FILENAME));
        } else {
            removed_out.push(REMOVED_FILENAME);
        }
        write(removed_out, removed.join("\n").as_bytes()).await?;
    }
    Ok(())
}
