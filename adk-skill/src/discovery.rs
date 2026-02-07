use crate::error::{SkillError, SkillResult};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn discover_skill_files(root: impl AsRef<Path>) -> SkillResult<Vec<PathBuf>> {
    let skill_root = root.as_ref().join(".skills");
    if !skill_root.exists() {
        return Ok(Vec::new());
    }
    if !skill_root.is_dir() {
        return Err(SkillError::InvalidSkillsRoot(skill_root));
    }

    let mut files = WalkDir::new(&skill_root)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "md"))
        .map(|entry| entry.into_path())
        .collect::<Vec<_>>();

    files.sort();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn discovers_only_markdown_skill_files() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        fs::create_dir_all(root.join(".skills/nested")).unwrap();

        fs::write(root.join(".skills/a.md"), "---\nname: a\ndescription: a\n---\n").unwrap();
        fs::write(root.join(".skills/nested/b.md"), "---\nname: b\ndescription: b\n---\n").unwrap();
        fs::write(root.join(".skills/notes.txt"), "ignore").unwrap();

        let files = discover_skill_files(root).unwrap();
        assert_eq!(files.len(), 2);
        assert!(files.iter().all(|p| p.extension().is_some_and(|ext| ext == "md")));
    }
}
