use async_std::{
    fs::write,
    path::{Path, PathBuf},
};

use crate::{
    compress::{self, zip_append_buf},
    diff::Diff,
};

const CONTENT_FILENAME: &str = "content.zip";
const UPDATE_FILENAME: &str = "update.zip";
const REMOVED_FILENAME: &str = "removed";
const DIFF_LIST_FILENAME: &str = "diff.json";
const CONTENT_LIST_FILENAME: &str = "content.json";
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
    zip_append_buf(
        &update_out,
        &[(REMOVED_FILENAME, diff.removed.join("\n").as_bytes())],
    )?;

    let update_app_out = build_path(out, UPDATE_FILENAME, prefix, true);
    compress::zip_files(diff.update_iter(), root, &update_app_out, true).await?;
    zip_append_buf(
        &update_app_out,
        &[(REMOVED_FILENAME, diff.removed.join("\n").as_bytes())],
    )?;

    let removed_out = build_path(out, REMOVED_FILENAME, prefix, false);
    write(removed_out, diff.removed.join("\n").as_bytes()).await?;

    let diff_list_out = build_path(out, DIFF_LIST_FILENAME, prefix, false);
    write(diff_list_out, serde_json::to_string(diff)?).await?;

    Ok(())
}

pub(crate) async fn package_content<T: AsRef<str>>(
    root: &Path,
    out: &Path,
    prefix: &str,
    hashes: &[(T, T)],
) -> std::io::Result<()> {
    let content_out = build_path(out, CONTENT_FILENAME, prefix, false);
    compress::zip_dir(root, &content_out, false).await?;
    let content_app_out = build_path(out, CONTENT_FILENAME, prefix, true);
    compress::zip_dir(root, &content_app_out, true).await?;

    let content_list_out = build_path(out, CONTENT_LIST_FILENAME, prefix, false);
    let list = hashes
        .iter()
        .map(|(_, f)| f.as_ref())
        .collect::<Vec<&str>>();
    write(content_list_out, serde_json::to_string(&list)?).await?;
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
    compress::zip_content(file_name.to_str().unwrap(), &buf, &out_file_name)?;
    Ok(())
}
