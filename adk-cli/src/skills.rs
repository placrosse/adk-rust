use crate::cli::SkillsCommands;
use adk_skill::{SelectionPolicy, load_skill_index, select_skills};
use anyhow::{Result, anyhow};
use serde_json::json;
use std::path::PathBuf;

pub fn run(command: SkillsCommands) -> Result<()> {
    match command {
        SkillsCommands::List { path, json: as_json } => list(&path, as_json),
        SkillsCommands::Validate { path, json: as_json } => validate(&path, as_json),
        SkillsCommands::Match {
            query,
            path,
            top_k,
            min_score,
            include_tags,
            exclude_tags,
            json: as_json,
        } => match_skills(&query, &path, top_k, min_score, include_tags, exclude_tags, as_json),
    }
}

fn list(path: &str, as_json: bool) -> Result<()> {
    let root = PathBuf::from(path);
    let index = load_skill_index(&root).map_err(|e| anyhow!(e.to_string()))?;

    if as_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "count": index.len(),
                "skills": index.summaries(),
            }))?
        );
    } else {
        println!("Found {} skill(s)", index.len());
        for skill in index.summaries() {
            println!("- {}: {} ({})", skill.name, skill.description, skill.path.display());
        }
    }

    Ok(())
}

fn validate(path: &str, as_json: bool) -> Result<()> {
    let root = PathBuf::from(path);
    match load_skill_index(&root) {
        Ok(index) => {
            if as_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "valid": true,
                        "count": index.len(),
                        "skills": index.summaries(),
                    }))?
                );
            } else {
                println!("Skills validation succeeded ({} skill(s))", index.len());
            }
            Ok(())
        }
        Err(err) => {
            if as_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "valid": false,
                        "error": err.to_string(),
                    }))?
                );
            } else {
                eprintln!("Skills validation failed: {}", err);
            }
            Err(anyhow!(err.to_string()))
        }
    }
}

fn match_skills(
    query: &str,
    path: &str,
    top_k: usize,
    min_score: f32,
    include_tags: Vec<String>,
    exclude_tags: Vec<String>,
    as_json: bool,
) -> Result<()> {
    let root = PathBuf::from(path);
    let index = load_skill_index(&root).map_err(|e| anyhow!(e.to_string()))?;
    let policy = SelectionPolicy { top_k, min_score, include_tags, exclude_tags };
    let matches = select_skills(&index, query, &policy);

    if as_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "query": query,
                "count": matches.len(),
                "matches": matches,
            }))?
        );
    } else {
        println!("Matched {} skill(s) for query: {}", matches.len(), query);
        for item in matches {
            println!("- {} (score {:.2})", item.skill.name, item.score);
        }
    }

    Ok(())
}
