use crate::schema::{AgentSchema, AgentType, ProjectSchema};
use adk_agent::{Agent, LlmAgentBuilder, SequentialAgent};
use adk_model::gemini::GeminiModel;
use anyhow::{anyhow, Result};
use std::sync::Arc;

/// Compile an AgentSchema into a runnable Agent
pub fn compile_agent(name: &str, schema: &AgentSchema, api_key: &str, project: &ProjectSchema) -> Result<Arc<dyn Agent>> {
    match schema.agent_type {
        AgentType::Llm => compile_llm_agent(name, schema, api_key),
        AgentType::Sequential => compile_sequential_agent(name, schema, api_key, project),
        _ => Err(anyhow!("Agent type {:?} not yet supported", schema.agent_type)),
    }
}

fn compile_llm_agent(name: &str, schema: &AgentSchema, api_key: &str) -> Result<Arc<dyn Agent>> {
    let model_name = schema.model.as_deref().unwrap_or("gemini-2.0-flash");
    let model = Arc::new(GeminiModel::new(api_key, model_name)?);

    let mut builder = LlmAgentBuilder::new(name).model(model);

    if !schema.instruction.is_empty() {
        builder = builder.instruction(&schema.instruction);
    }

    Ok(Arc::new(builder.build()?))
}

fn compile_sequential_agent(name: &str, schema: &AgentSchema, api_key: &str, project: &ProjectSchema) -> Result<Arc<dyn Agent>> {
    let mut sub_agents: Vec<Arc<dyn Agent>> = Vec::new();
    
    for sub_id in &schema.sub_agents {
        let sub_schema = project.agents.get(sub_id)
            .ok_or_else(|| anyhow!("Sub-agent {} not found", sub_id))?;
        sub_agents.push(compile_agent(sub_id, sub_schema, api_key, project)?);
    }
    
    if sub_agents.is_empty() {
        return Err(anyhow!("Sequential agent {} has no sub-agents", name));
    }
    
    Ok(Arc::new(SequentialAgent::new(name, sub_agents)))
}
