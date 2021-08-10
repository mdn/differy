use std::collections::HashSet;

use async_std::{fs, path::Path};

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

pub(crate) async fn diff(
    a: &Path,
    b: &Path,
) -> std::io::Result<(Vec<String>, Vec<String>, Vec<String>)> {
    let a = fs::read_to_string(a).await?;
    let b = fs::read_to_string(b).await?;

    let a = parse_hashes(&a);
    let b = parse_hashes(&b);

    let a_set: HashSet<&str> = a.iter().map(|(hash, _)| *hash).collect();
    let b_set: HashSet<&str> = b.iter().map(|(hash, _)| *hash).collect();

    let b_not_a: HashSet<_> = b_set.difference(&a_set).collect();

    let a_file_set: HashSet<&str> = a.iter().map(|(_, file)| *file).collect();
    let b_file_set: HashSet<&str> = b.iter().map(|(_, file)| *file).collect();

    let a_not_b_file: HashSet<_> = a_file_set.difference(&b_file_set).collect();
    let b_not_a_file: HashSet<_> = b_file_set.difference(&a_file_set).collect();
    let a_and_b_file: HashSet<_> = a_file_set.intersection(&b_file_set).collect();

    let removed: Vec<String> = a
        .iter()
        .filter(|(_, file)| a_not_b_file.contains(&file))
        .map(|(_, file)| file.to_string())
        .collect();
    let added: Vec<String> = b
        .iter()
        .filter(|(_, file)| b_not_a_file.contains(&file))
        .map(|(_, file)| file.to_string())
        .collect();

    let modified: Vec<String> = b
        .iter()
        .filter(|(hash, file)| b_not_a.contains(&hash) && a_and_b_file.contains(&file))
        .map(|(_, file)| file.to_string())
        .collect();

    Ok((removed, added, modified))
}
