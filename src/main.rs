use async_std::fs::{read_to_string, File};
use async_std::path::PathBuf;
use async_std::prelude::*;
use clap::{crate_version, App, Arg, SubCommand};

use crate::diff::{diff, parse_hashes};
use crate::package::package_hashes;
use crate::{
    package::{package_content, package_update},
};

mod compress;
mod diff;
mod hash;
mod package;

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
                    Arg::with_name("from")
                        .long("from")
                        .short("f")
                        .required(true)
                        .help("Old ref")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("root")
                        .required(true)
                        .help("Build root")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("content")
                        .long("content")
                        .short("c")
                        .required(false)
                        .help("Package full content"),
                )
                .arg(
                    Arg::with_name("ref")
                        .long("ref")
                        .short("r")
                        .required(false)
                        .help("Current ref")
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
        let diff = diff::diff_hash_files(&old, &new).await?;
        diff.write(&mut out_file).await?;
    }
    if let Some(matches) = matches.subcommand_matches("package") {
        let root = matches.value_of("root").unwrap();
        let out = matches.value_of("out").unwrap();
        let current_ref = matches.value_of("ref").unwrap();
        let content = matches.is_present("content");
        let root = PathBuf::from(root);
        let out = PathBuf::from(out);

        let old_ref = matches.value_of("from").unwrap();
        let old_hashes_raw =
            read_to_string(&PathBuf::from(format!("{}-checksums", old_ref))).await?;
        let update_prefix = format!("{}-{}", current_ref, old_ref);
        let mut new_hashes = vec![];
        hash::hash_all(&root, &mut new_hashes, &root).await?;
        package_hashes(&new_hashes, &out, current_ref).await?;
        let diff = diff(&parse_hashes(&old_hashes_raw), new_hashes.as_slice())?;

        package_update(&root, &diff, &out, &update_prefix).await?;
        if content {
            package_content(&root, &out, current_ref).await?;
        }
    }
    Ok(())
}
