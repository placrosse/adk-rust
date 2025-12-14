use crate::compiler::compile_agent;
use crate::schema::ProjectSchema;
use adk_agent::Agent;
use anyhow::{anyhow, Result};
use std::sync::Arc;

/// Compile a project into a runnable agent
pub fn compile_project(project: &ProjectSchema, api_key: &str) -> Result<Arc<dyn Agent>> {
    let agent_order = get_agent_order(project)?;
    
    if agent_order.is_empty() {
        return Err(anyhow!("Project has no agents"));
    }
    
    // For now, just compile the first top-level agent in the workflow
    // (which may be a SequentialAgent containing sub-agents)
    let name = &agent_order[0];
    let schema = project.agents.get(name)
        .ok_or_else(|| anyhow!("Agent {} not found", name))?;
    compile_agent(name, schema, api_key, project)
}

/// Get top-level agent order from workflow edges (START → agent → END)
fn get_agent_order(project: &ProjectSchema) -> Result<Vec<String>> {
    let mut order = Vec::new();
    let mut current = "START".to_string();
    
    for _ in 0..100 {
        let next = project.workflow.edges.iter()
            .find(|e| e.from == current)
            .map(|e| e.to.clone());
        
        match next {
            Some(ref n) if n == "END" => break,
            Some(n) => {
                order.push(n.clone());
                current = n;
            }
            None => break,
        }
    }
    
    Ok(order)
}
