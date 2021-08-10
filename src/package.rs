use async_std::fs::read;
use async_std::path::Path;
use std::io::Write;
use walkdir::WalkDir;
use zip::result::ZipResult;
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipWriter};

pub(crate) async fn zip_files<T: AsRef<str>>(
    files: &[T],
    src_dir: &Path,
    out_file: &Path,
) -> ZipResult<()> {
    let path = Path::new(out_file);
    let file = std::fs::File::create(&path)?;

    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(CompressionMethod::DEFLATE)
        .unix_permissions(0o644);

    for path in files {
        let full_path = src_dir.join(&path.as_ref());

        if full_path.is_file().await {
            zip.start_file(path.as_ref(), options)?;

            let buf = read(full_path).await?;
            zip.write_all(&buf)?;
        }
    }
    zip.finish()?;
    Ok(())
}

pub(crate) async fn zip_dir(src_dir: &Path, out_file: &Path) -> ZipResult<()> {
    let path = Path::new(out_file);
    let file = std::fs::File::create(&path)?;

    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(CompressionMethod::DEFLATE)
        .unix_permissions(0o644);

    for entry in WalkDir::new(src_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = path.strip_prefix(src_dir).unwrap().to_str().unwrap();

        if path.is_file() {
            zip.start_file(name, options)?;
            let buf = read(path).await?;
            zip.write_all(&buf)?;
        } else if !name.is_empty() {
            zip.add_directory(name, options)?;
        }
    }
    zip.finish()?;
    Ok(())
}
