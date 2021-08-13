use async_std::fs::File;
use async_std::path::PathBuf;
use async_std::prelude::*;
use chrono::Utc;
use clap::{crate_version, App, Arg, SubCommand};

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
                    Arg::with_name("root")
                        .required(true)
                        .help("Build root")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("from")
                        .long("from")
                        .short("f")
                        .help("Old update.json")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("num_updates")
                        .long("num")
                        .short("n")
                        .help("how many version to support")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("rev")
                        .long("rev")
                        .short("r")
                        .required(false)
                        .help("Current rev")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("out")
                        .long("out")
                        .short("o")
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
        let diff = diff::diff_hash_files(&old, &new).await?;
        diff.write(&mut out_file).await?;
    }
    if let Some(matches) = matches.subcommand_matches("package") {
        let root = matches.value_of("root").unwrap();
        let out = matches.value_of("out").unwrap_or(".");
        let current_rev = matches.value_of("rev").unwrap();
        let root = PathBuf::from(root);
        let out = PathBuf::from(out);
        let num_versions = matches
            .value_of("num_version")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(7);

        let from = matches.value_of("from").unwrap_or("update.json");
        let update_json = std::path::PathBuf::from(from);
        let Update {
            mut updates,
            latest,
            ..
        } = Update::from_file(&update_json)?;
        updates.push(latest);
        let updates: Vec<String> = updates.into_iter().rev().take(num_versions).collect();
        for version in &updates {
            let checksum_file = format!("{}-checksums", &version);
            let checksum_zip_file = PathBuf::from(&checksum_file).with_extension("zip");
            println!("packaging update {} → {}", current_rev, version);
            let old_hashes_raw = unzip_content(&checksum_zip_file, &checksum_file)?;
            let update_prefix = format!("{}-{}", current_rev, &version);
            let mut new_hashes = vec![];
            hash::hash_all(&root, &mut new_hashes, &root).await?;
            package_hashes(&new_hashes, &out, current_rev).await?;
            let diff = diff(&parse_hashes(&old_hashes_raw), new_hashes.as_slice())?;

            package_update(&root, &diff, &out, &update_prefix).await?;
        }
        println!("building content for {}", current_rev);
        package_content(&root, &out, current_rev).await?;

        let update = Update {
            date: Utc::now().naive_utc(),
            latest: current_rev.into(),
            updates,
        };
        update.save(&update_json)?;
    }
    Ok(())
}
