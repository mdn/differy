use async_recursion::async_recursion;
use async_std::fs::File;
use async_std::prelude::*;
use async_std::{
    fs::{self},
    path::{Path, PathBuf},
};
use clap::{App, Arg, SubCommand};
use sha2::Digest;

use std::collections::HashSet;

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
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("hash") {
        let path = matches.value_of("PATH").unwrap();
        let out = matches.value_of("out").unwrap();
        let mut out_file = File::create(out).await?;
        let path = PathBuf::from(path);
        let mut hashes = vec![];
        hash_all(&path, &mut hashes, &path).await?;
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
        let (removed, added) = diff(&old, &new).await?;
        for (hash, filename) in removed {
            out_file
                .write_all(format!("- {} {}\n", hash, filename).as_bytes())
                .await?;
        }
        for (hash, filename) in added {
            out_file
                .write_all(format!("+ {} {}\n", hash, filename).as_bytes())
                .await?;
        }
    }
    Ok(())
}

async fn diff(
    a: &Path,
    b: &Path,
) -> std::io::Result<(Vec<(String, String)>, Vec<(String, String)>)> {
    let a = fs::read_to_string(a).await?;
    let b = fs::read_to_string(b).await?;

    let a = parse_hashes(&a);
    let b = parse_hashes(&b);

    let a_set: HashSet<&str> = a.iter().map(|(hash, _)| *hash).collect();
    let b_set: HashSet<&str> = b.iter().map(|(hash, _)| *hash).collect();

    let a_not_b: HashSet<_> = a_set.difference(&b_set).collect();
    let b_not_a: HashSet<_> = b_set.difference(&a_set).collect();

    let removed: Vec<(String, String)> = a
        .iter()
        .filter(|(hash, _)| a_not_b.contains(&hash))
        .map(|(hash, file)| (hash.to_string(), file.to_string()))
        .collect();
    let added: Vec<(String, String)> = b
        .iter()
        .filter(|(hash, _)| b_not_a.contains(&hash))
        .map(|(hash, file)| (hash.to_string(), file.to_string()))
        .collect();

    Ok((removed, added))
}

fn parse_hashes(hashes: &str) -> Vec<(&str, &str)> {
    let mut out = vec![];
    for line in hashes.split('\n') {
        let mut split = line.split(' ').filter(|s| !s.is_empty());
        if let (Some(hash), Some(file)) = (split.next(), split.next()) {
            out.push((hash, file))
        }
    }
    out
}

#[async_recursion]
async fn hash_all(dir: &Path, out: &mut Vec<(String, String)>, base: &Path) -> std::io::Result<()> {
    if dir.is_dir().await {
        let mut entries = fs::read_dir(dir).await?;
        while let Some(Ok(entry)) = entries.next().await {
            let path = entry.path();
            if path.is_dir().await {
                hash_all(&path, out, base).await?;
            } else {
                let hash = sha2::Sha256::digest(&fs::read(entry.path()).await?);
                out.push((
                    format!("{:x}", hash),
                    entry.path().strip_prefix(base).unwrap().to_string_lossy().to_string(),
                ));
            }
        }
    }
    Ok(())
}
