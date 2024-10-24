use async_std::fs::File;
use async_std::path::PathBuf;
use async_std::prelude::*;
use chrono::Utc;
use clap::{crate_version, Arg, Command};

use crate::compress::unzip_content;
use crate::diff::{diff, parse_hashes};
use crate::package::package_hashes;
use crate::package::{package_content, package_update};
use crate::update::Update;

mod compress;
mod diff;
mod hash;
mod package;
mod update;

const NUM_VERSION_DEFAULT: usize = 14;

fn cli() -> Command {
    Command::new("differy")
        .version(crate_version!())
        .author("Florian Dieminger <me@fiji-flo.de>")
        .about("Hash and diff all the things")
        .subcommand(
            Command::new("hash")
                .about("Hash all files")
                .arg(Arg::new("PATH").required(true).help("Path to scan"))
                .arg(
                    Arg::new("out")
                        .long("out")
                        .short('o')
                        .required(true)
                        .help("Output file"),
                ),
        )
        .subcommand(
            Command::new("diff")
                .about("Diff two hash files")
                .arg(Arg::new("old").required(true).help("Old hash file"))
                .arg(Arg::new("new").required(true).help("New hash file"))
                .arg(
                    Arg::new("out")
                        .long("out")
                        .short('o')
                        .required(true)
                        .help("Output file"),
                ),
        )
        .subcommand(
            Command::new("package")
                .about("Package an update zip")
                .arg(Arg::new("root").required(true).help("Build root"))
                .arg(
                    Arg::new("from")
                        .long("from")
                        .short('f')
                        .help("Old update.json"),
                )
                .arg(
                    Arg::new("num_updates")
                        .long("num")
                        .short('n')
                        .help("how many version to support"),
                )
                .arg(
                    Arg::new("rev")
                        .long("rev")
                        .short('r')
                        .required(false)
                        .help("Current rev"),
                )
                .arg(Arg::new("out").long("out").short('o').help("Output folder")),
        )
}

#[async_std::main]
async fn main() -> std::io::Result<()> {
    let matches = cli().get_matches();

    if let Some(matches) = matches.subcommand_matches("hash") {
        let path = matches.get_one::<String>("PATH").unwrap();
        let out = matches.get_one::<String>("out").unwrap();
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
        let old = matches.get_one::<String>("old").unwrap();
        let new = matches.get_one::<String>("new").unwrap();
        let out = matches.get_one::<String>("out").unwrap();
        let mut out_file = File::create(out).await?;
        let old = PathBuf::from(old);
        let new = PathBuf::from(new);
        let diff = diff::diff_hash_files(&old, &new).await?;
        diff.write(&mut out_file).await?;
    }
    if let Some(matches) = matches.subcommand_matches("package") {
        let root = matches.get_one::<String>("root").unwrap();
        let out = matches
            .get_one::<String>("out")
            .map(|s| s.as_str())
            .unwrap_or(".");
        let current_rev = matches.get_one::<String>("rev").unwrap();
        let root = PathBuf::from(root);
        let out = PathBuf::from(out);
        let num_versions = matches
            .get_one::<String>("num_updates")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(NUM_VERSION_DEFAULT);

        let from = matches
            .get_one::<String>("from")
            .map(|s| s.as_str())
            .unwrap_or("update.json");
        let update_json = std::path::PathBuf::from(from);
        let Update {
            updates, latest, ..
        } = Update::from_file(&update_json).unwrap_or_default();

        let mut to_be_updated = Vec::new();
        if let Some(latest) = latest {
            if *current_rev != latest {
                to_be_updated.push(latest);
            }
        }
        let take_versions = num_versions - to_be_updated.len();
        to_be_updated.extend(updates.into_iter().take(take_versions));

        let mut new_hashes = vec![];
        hash::hash_all(&root, &mut new_hashes, &root).await?;
        package_hashes(&new_hashes, &out, current_rev).await?;

        let mut updated = vec![];
        for version in to_be_updated {
            let checksum_file = format!("{}-checksums", &version);
            let checksum_zip_file = PathBuf::from(&checksum_file).with_extension("zip");
            println!("packaging update {} → {}", current_rev, version);
            let old_hashes_raw = match unzip_content(&checksum_zip_file, &checksum_file) {
                Ok(r) => r,
                Err(e) => {
                    println!("Error unpacking: {:?}.zip: {}", checksum_file, e);
                    continue;
                }
            };
            let update_prefix = format!("{}-{}", current_rev, &version);
            let diff = diff(&parse_hashes(&old_hashes_raw), new_hashes.as_slice())?;

            package_update(&root, &diff, &out, &update_prefix).await?;
            updated.push(version);
        }
        println!("building content for {}", current_rev);
        package_content(&root, &out, current_rev, &new_hashes).await?;

        let update = Update {
            date: Some(Utc::now().naive_utc()),
            latest: Some(current_rev.into()),
            updates: updated,
        };
        update.save(&update_json)?;
    }
    Ok(())
}

#[test]
fn verify_cli() {
    cli().debug_assert();
}
