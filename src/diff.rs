use std::collections::HashSet;

use async_std::{
    fs::{self, read_to_string, File},
    io::prelude::WriteExt,
    path::Path,
};

pub(crate) struct Diff {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub modified: Vec<String>,
}

impl Diff {
    #[allow(dead_code)]
    pub async fn from_path(path: &Path) -> std::io::Result<Self> {
        let diff = read_to_string(path).await?;
        let mut added = vec![];
        let mut removed = vec![];
        let mut modified = vec![];
        for line in diff.split('\n') {
            if let Some(file) = line.strip_prefix("+ ") {
                added.push(file.to_string())
            }
            if let Some(file) = line.strip_prefix("~ ") {
                modified.push(file.to_string())
            }
            if let Some(file) = line.strip_prefix("- ") {
                removed.push(file.to_string())
            }
        }
        Ok(Self {
            added,
            removed,
            modified,
        })
    }

    pub fn update_iter(&self) -> impl Iterator<Item = &String> {
        self.added.iter().chain(self.modified.iter())
    }

    pub async fn write(&self, out_file: &mut File) -> std::io::Result<()> {
        for filename in &self.removed {
            out_file
                .write_all(format!("- {}\n", filename).as_bytes())
                .await?;
        }
        for filename in &self.added {
            out_file
                .write_all(format!("+ {}\n", filename).as_bytes())
                .await?;
        }
        for filename in &self.modified {
            out_file
                .write_all(format!("~ {}\n", filename).as_bytes())
                .await?;
        }
        Ok(())
    }
}

pub(crate) fn parse_hashes(hashes: &str) -> Vec<(&str, &str)> {
    let mut out = vec![];
    for line in hashes.split('\n') {
        let mut split = line.split(' ').filter(|s| !s.is_empty());
        if let (Some(hash), Some(file)) = (split.next(), split.next()) {
            out.push((hash, file))
        }
    }
    out
}

pub(crate) fn diff<T: AsRef<str>, S: AsRef<str>>(
    a: &[(T, T)],
    b: &[(S, S)],
) -> std::io::Result<Diff> {
    let a_set: HashSet<&str> = a.iter().map(|(hash, _)| hash.as_ref()).collect();
    let b_set: HashSet<&str> = b.iter().map(|(hash, _)| hash.as_ref()).collect();

    let b_not_a: HashSet<_> = b_set.difference(&a_set).collect();

    let a_file_set: HashSet<&str> = a.iter().map(|(_, file)| file.as_ref()).collect();
    let b_file_set: HashSet<&str> = b.iter().map(|(_, file)| file.as_ref()).collect();

    let a_not_b_file: HashSet<_> = a_file_set.difference(&b_file_set).collect();
    let b_not_a_file: HashSet<_> = b_file_set.difference(&a_file_set).collect();
    let a_and_b_file: HashSet<_> = a_file_set.intersection(&b_file_set).collect();

    let removed: Vec<String> = a
        .iter()
        .filter(|(_, file)| a_not_b_file.contains(&file.as_ref()))
        .map(|(_, file)| file.as_ref().to_string())
        .collect();
    let added: Vec<String> = b
        .iter()
        .filter(|(_, file)| b_not_a_file.contains(&file.as_ref()))
        .map(|(_, file)| file.as_ref().to_string())
        .collect();

    let modified: Vec<String> = b
        .iter()
        .filter(|(hash, file)| {
            b_not_a.contains(&hash.as_ref()) && a_and_b_file.contains(&file.as_ref())
        })
        .map(|(_, file)| file.as_ref().to_string())
        .collect();

    Ok(Diff {
        removed,
        added,
        modified,
    })
}
pub(crate) async fn diff_hash_files(a: &Path, b: &Path) -> std::io::Result<Diff> {
    let a = fs::read_to_string(a).await?;
    let b = fs::read_to_string(b).await?;

    let a = parse_hashes(&a);
    let b = parse_hashes(&b);

    diff(&a, &b)
}
