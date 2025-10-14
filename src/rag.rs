use anyhow::Result;
use std::{fs, path::Path};

/// Load up to three small few-shot examples from a directory.
/// Files can be `.txt` or `.md` containing before/after snippets.
pub fn load_few_shot(rag_dir: &Path) -> Result<Vec<String>> {
    if !rag_dir.exists() {
        return Ok(vec![]);
    }
    let mut out = vec![];
    let mut files: Vec<_> = fs::read_dir(rag_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .collect();
    files.sort();
    for p in files.into_iter().take(3) {
        if let Ok(s) = fs::read_to_string(&p) {
            out.push(s);
        }
    }
    Ok(out)
}
