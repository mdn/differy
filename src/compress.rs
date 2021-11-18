use async_std::fs::{read, read_to_string};
use async_std::path::Path;
use std::io::{Read, Write};
use walkdir::WalkDir;
use zip::result::ZipResult;
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

const APP_REPLACEMENTS: &[(&str, &str)] = &[(
    "https://interactive-examples.mdn.mozilla.net",
    "mdn-app://examples/examples",
), (
    "https://yari-demos.prod.mdn.mozit.cloud",
    "mdn-app://yari-demos"
)];

pub fn replace(input: String, replace: &[(&str, &str)]) -> String {
    let mut result = String::new();
    let mut last_end = 0;
    let mut matches = vec![];
    for (from, _) in replace {
        matches.extend(input.match_indices(from));
    }
    if matches.is_empty() {
        return input;
    }
    matches.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
    for (start, part) in matches {
        result.push_str(unsafe { input.get_unchecked(last_end..start) });
        let to = replace
            .iter()
            .find_map(|(from, to)| if *from == part { Some(to) } else { None })
            .unwrap();
        result.push_str(to);
        last_end = start + part.len();
    }
    result.push_str(unsafe { input.get_unchecked(last_end..input.len()) });
    result
}

pub(crate) fn zip_content(file_name: &str, content: &[u8], out_file: &Path) -> ZipResult<()> {
    let out_path = Path::new(out_file);
    let file = std::fs::File::create(&out_path)?;

    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(CompressionMethod::DEFLATE)
        .unix_permissions(0o644);

    zip.start_file(file_name, options)?;
    zip.write_all(content)?;
    zip.finish()?;
    Ok(())
}

pub(crate) fn unzip_content(zip_file: &Path, file_name: &str) -> ZipResult<String> {
    let zipfile = std::fs::File::open(zip_file)?;
    let mut archive = ZipArchive::new(zipfile).unwrap();
    let mut file = archive.by_name(file_name)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

pub(crate) fn zip_append_buf<T: AsRef<str>, B: AsRef<[u8]>>(
    zip_file_path: &Path,
    files: &[(T, B)],
) -> ZipResult<()> {
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(zip_file_path)?;

    let mut zip = ZipWriter::new_append(file)?;
    let options = FileOptions::default()
        .compression_method(CompressionMethod::DEFLATE)
        .unix_permissions(0o644);

    for (file_name, buf) in files {
        zip.start_file(file_name.as_ref(), options)?;
        zip.write_all(buf.as_ref())?;
    }
    zip.finish()?;
    Ok(())
}

pub(crate) async fn zip_files<T: AsRef<str>>(
    files: impl Iterator<Item = T>,
    src_dir: &Path,
    out_file: &Path,
    app: bool,
) -> ZipResult<()> {
    let out_path = Path::new(out_file);
    let file = std::fs::File::create(&out_path)?;

    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default();

    for path in files {
        let full_path = src_dir.join(&path.as_ref());

        if full_path.is_file().await {
            zip.start_file(path.as_ref(), options)?;

            if path.as_ref().ends_with("index.json") {
                let mut buf = read_to_string(full_path).await?;
                if app {
                    buf = replace_all(buf);
                }
                zip.write_all(buf.as_bytes())?;
            } else {
                let buf = read(full_path).await?;
                zip.write_all(&buf)?;
            }
        } else {
            zip.add_directory(path.as_ref(), options)?;
        }
    }
    zip.finish()?;
    Ok(())
}

fn replace_all(input: String) -> String {
    replace(input, APP_REPLACEMENTS)
}

pub(crate) async fn zip_dir(src_dir: &Path, out_file: &Path, app: bool) -> ZipResult<()> {
    let path = Path::new(out_file);
    let file = std::fs::File::create(&path)?;

    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default();

    for entry in WalkDir::new(src_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = path.strip_prefix(src_dir).unwrap().to_str().unwrap();

        if path.is_file() {
            zip.start_file(name, options)?;
            if name.ends_with("index.json") {
                let mut buf = read_to_string(path).await?;
                if app {
                    buf = replace_all(buf);
                }
                zip.write_all(buf.as_bytes())?;
            } else {
                let buf = read(path).await?;
                zip.write_all(&buf)?;
            }
        } else if !name.is_empty() {
            zip.add_directory(name, options)?;
        }
    }
    zip.finish()?;
    Ok(())
}
