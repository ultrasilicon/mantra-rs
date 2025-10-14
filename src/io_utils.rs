use anyhow::Result;
use std::{
    fs,
    path::{Path, PathBuf},
};
use time::OffsetDateTime;

pub fn now_slug() -> String {
    let t = OffsetDateTime::now_utc();
    t.format(&time::format_description::well_known::Rfc3339)
        .unwrap()
        .replace(':', "*")
        .replace('+', "*")
        .replace('-', "_")
}

pub fn read_to_string(p: &Path) -> Result<String> {
    Ok(fs::read_to_string(p)?)
}

pub fn write_string(p: &Path, s: &str) -> Result<()> {
    fs::write(p, s)?;
    Ok(())
}

pub fn temp_rs_path(original: &Path) -> Result<PathBuf> {
    let ts = now_slug();
    let filename = original.file_name().unwrap().to_string_lossy().to_string();
    let tmp = PathBuf::from(format!("/tmp/{}_{}", ts, filename));
    Ok(tmp)
}
