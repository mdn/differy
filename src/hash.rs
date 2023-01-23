use async_std::{fs, path::Path};
use sha2::Digest;
use walkdir::WalkDir;

pub(crate) async fn hash_all(
    dir: &Path,
    out: &mut Vec<(String, String)>,
    base: &Path,
) -> std::io::Result<()> {
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_dir() {
            let hash = sha2::Sha256::digest(fs::read(entry.path()).await?);
            out.push((
                format!("{:x}", hash),
                entry
                    .path()
                    .strip_prefix(base)
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
            ));
        }
    }
    Ok(())
}
