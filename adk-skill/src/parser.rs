use crate::error::{SkillError, SkillResult};
use crate::model::{ParsedSkill, SkillFrontmatter};
use std::path::Path;

pub fn parse_skill_markdown(path: &Path, content: &str) -> SkillResult<ParsedSkill> {
    let normalized = content.replace("\r\n", "\n");
    let mut lines = normalized.lines();

    let first = lines.next().unwrap_or_default().trim();
    if first != "---" {
        return Err(SkillError::InvalidFrontmatter {
            path: path.to_path_buf(),
            message: "missing opening frontmatter delimiter (`---`)".to_string(),
        });
    }

    let mut frontmatter_lines = Vec::new();
    let mut found_end = false;
    for line in lines.by_ref() {
        if line.trim() == "---" {
            found_end = true;
            break;
        }
        frontmatter_lines.push(line);
    }

    if !found_end {
        return Err(SkillError::InvalidFrontmatter {
            path: path.to_path_buf(),
            message: "missing closing frontmatter delimiter (`---`)".to_string(),
        });
    }

    let frontmatter_raw = frontmatter_lines.join("\n");
    let fm: SkillFrontmatter = serde_yaml::from_str(&frontmatter_raw)?;

    let name = fm.name.trim().to_string();
    if name.is_empty() {
        return Err(SkillError::MissingField { path: path.to_path_buf(), field: "name" });
    }

    let description = fm.description.trim().to_string();
    if description.is_empty() {
        return Err(SkillError::MissingField { path: path.to_path_buf(), field: "description" });
    }

    let tags = fm
        .tags
        .into_iter()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>();

    let body = lines.collect::<Vec<_>>().join("\n").trim().to_string();

    Ok(ParsedSkill { name, description, tags, body })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_skill() {
        let content = r#"---
name: repo_search
description: Search the codebase quickly
tags:
  - code
  - search
---
Use ripgrep first.
"#;
        let parsed = parse_skill_markdown(Path::new(".skills/repo_search.md"), content).unwrap();
        assert_eq!(parsed.name, "repo_search");
        assert_eq!(parsed.description, "Search the codebase quickly");
        assert_eq!(parsed.tags, vec!["code", "search"]);
        assert!(parsed.body.contains("Use ripgrep first."));
    }

    #[test]
    fn rejects_missing_required_fields() {
        let content = r#"---
name: ""
description: ""
---
body
"#;
        let err = parse_skill_markdown(Path::new(".skills/bad.md"), content).unwrap_err();
        assert!(matches!(err, SkillError::MissingField { .. }));
    }
}
