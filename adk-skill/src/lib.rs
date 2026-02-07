//! AgentSkills support for ADK-Rust.
//!
//! This crate provides:
//! - Skill markdown parsing and validation
//! - `.skills/` discovery and indexing
//! - Simple lexical skill matching
//! - Plugin helper for runtime skill injection

mod discovery;
mod error;
mod index;
mod injector;
mod model;
mod parser;
mod select;

pub use discovery::discover_skill_files;
pub use error::{SkillError, SkillResult};
pub use index::load_skill_index;
pub use injector::{
    SkillInjector, SkillInjectorConfig, apply_skill_injection, select_skill_prompt_block,
};
pub use model::{
    ParsedSkill, SelectionPolicy, SkillDocument, SkillFrontmatter, SkillIndex, SkillMatch,
    SkillSummary,
};
pub use parser::parse_skill_markdown;
pub use select::select_skills;
