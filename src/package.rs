use async_std::{
    fs::write,
    path::{Path, PathBuf},
};

use crate::{compress, diff::Diff};

const CONTENT_FILENAME: &str = "content.zip";
const UPDATE_FILENAME: &str = "update.zip";
const REMOVED_FILENAME: &str = "removed";
const APP_PREFIX: &str = "app";

fn build_path<I: Into<PathBuf>>(base: I, file_name: &str, prefix: &str, app: bool) -> PathBuf {
    let mut full_name = String::new();
    full_name.push_str(prefix);
    full_name.push('-');
    if app {
        full_name.push_str(APP_PREFIX);
        full_name.push('-');
    }
    full_name.push_str(file_name);
    let mut out = base.into();
    out.push(full_name);
    out
}

pub(crate) async fn package_update(
    root: &Path,
    diff: &Diff,
    out: &Path,
    prefix: &str,
) -> std::io::Result<()> {
    let update_out = build_path(out, UPDATE_FILENAME, prefix, false);
    compress::zip_files(diff.update_iter(), root, &update_out, false).await?;
    let update_out = build_path(out, UPDATE_FILENAME, prefix, true);
    compress::zip_files(diff.update_iter(), root, &update_out, true).await?;

    let removed_out = build_path(out, REMOVED_FILENAME, prefix, false);
    write(removed_out, diff.removed.join("\n").as_bytes()).await?;

    Ok(())
}

pub(crate) async fn package_content(root: &Path, out: &Path, prefix: &str) -> std::io::Result<()> {
    let content_out = build_path(out, CONTENT_FILENAME, prefix, false);
    compress::zip_dir(root, &content_out, true).await?;
    let content_out = build_path(out, CONTENT_FILENAME, prefix, true);
    compress::zip_dir(root, &content_out, true).await?;
    Ok(())
}

pub(crate) async fn package_hashes<T: AsRef<str>>(
    hashes: &[(T, T)],
    out: &Path,
    prefix: &str,
) -> std::io::Result<()> {
    let mut buf = vec![];
    for (hash, file) in hashes {
        buf.extend(format!("{} {}\n", hash.as_ref(), file.as_ref()).as_bytes())
    }
    let file_name = build_path("", "checksums", prefix, false);
    let mut out_file_name = out.to_path_buf();
    out_file_name.push(&file_name);
    out_file_name.set_extension("zip");
    compress::zip_content::<&str>(file_name.to_str().unwrap(), &buf, &out_file_name).await?;
    Ok(())
}
