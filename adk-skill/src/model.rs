use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SkillFrontmatter {
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ParsedSkill {
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub body: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillDocument {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub body: String,
    pub path: PathBuf,
    pub hash: String,
    pub last_modified: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub path: PathBuf,
    pub hash: String,
    pub last_modified: Option<i64>,
}

impl From<&SkillDocument> for SkillSummary {
    fn from(value: &SkillDocument) -> Self {
        Self {
            id: value.id.clone(),
            name: value.name.clone(),
            description: value.description.clone(),
            tags: value.tags.clone(),
            path: value.path.clone(),
            hash: value.hash.clone(),
            last_modified: value.last_modified,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SkillIndex {
    skills: Vec<SkillDocument>,
}

impl SkillIndex {
    pub fn new(skills: Vec<SkillDocument>) -> Self {
        Self { skills }
    }

    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }

    pub fn len(&self) -> usize {
        self.skills.len()
    }

    pub fn skills(&self) -> &[SkillDocument] {
        &self.skills
    }

    pub fn summaries(&self) -> Vec<SkillSummary> {
        self.skills.iter().map(SkillSummary::from).collect()
    }
}

#[derive(Debug, Clone)]
pub struct SelectionPolicy {
    pub top_k: usize,
    pub min_score: f32,
    pub include_tags: Vec<String>,
    pub exclude_tags: Vec<String>,
}

impl Default for SelectionPolicy {
    fn default() -> Self {
        Self { top_k: 1, min_score: 1.0, include_tags: Vec::new(), exclude_tags: Vec::new() }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SkillMatch {
    pub score: f32,
    pub skill: SkillSummary,
}
