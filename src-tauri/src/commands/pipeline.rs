use serde::Serialize;
use tauri::State;

use crate::domain::pipeline::{Pipeline, StepKind};
use crate::state::AppState;

// =============================================================================
// Response types
// =============================================================================

#[derive(Serialize, Clone, Debug)]
pub struct PipelineStepSummary {
    pub id: String,
    pub label: String,
    pub enabled: bool,
    pub is_active: bool,
    pub has_cache: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct PipelineSummary {
    pub steps: Vec<PipelineStepSummary>,
    pub active_index: Option<usize>,
}

impl PipelineSummary {
    pub fn from(pipeline: &Pipeline) -> Self {
        let steps = pipeline
            .steps
            .iter()
            .enumerate()
            .map(|(i, step)| PipelineStepSummary {
                id: step.id.clone(),
                label: step.label.clone(),
                enabled: step.enabled,
                is_active: pipeline.active_step_index == Some(i),
                has_cache: false, // No caching until Phase 5
            })
            .collect();
        Self {
            steps,
            active_index: pipeline.active_step_index,
        }
    }
}

// =============================================================================
// Commands
// =============================================================================

#[tauri::command]
pub fn add_pipeline_step(
    kind: StepKind,
    state: State<AppState>,
) -> Result<PipelineSummary, String> {
    #[cfg(debug_assertions)]
    println!("[pipeline] add_pipeline_step: {:?}", kind);

    let mut session = state.session.write().map_err(|e| e.to_string())?;
    session.pipeline.push(kind);

    #[cfg(debug_assertions)]
    println!(
        "[pipeline]   → pipeline now has {} step(s)",
        session.pipeline.steps.len()
    );

    Ok(PipelineSummary::from(&session.pipeline))
}

#[tauri::command]
pub fn remove_pipeline_step(
    index: usize,
    state: State<AppState>,
) -> Result<PipelineSummary, String> {
    #[cfg(debug_assertions)]
    println!("[pipeline] remove_pipeline_step: {:?}", index);

    let mut session = state.session.write().map_err(|e| e.to_string())?;
    if index >= session.pipeline.steps.len() {
        return Err(format!(
            "Index {} out of range (pipeline has {} steps)",
            index,
            session.pipeline.steps.len()
        ));
    }
    session.pipeline.remove(index);

    #[cfg(debug_assertions)]
    println!(
        "[pipeline]   → pipeline now has {} step(s)",
        session.pipeline.steps.len()
    );

    Ok(PipelineSummary::from(&session.pipeline))
}

#[tauri::command]
pub fn move_pipeline_step(
    index: usize,
    direction: i32,
    state: State<AppState>,
) -> Result<PipelineSummary, String> {
    let mut session = state.session.write().map_err(|e| e.to_string())?;
    let len = session.pipeline.steps.len();
    if index >= len {
        return Err(format!(
            "Index {} out of range (pipeline has {} steps)",
            index, len
        ));
    }
    let new_index = index as i32 + direction;
    if new_index < 0 || new_index >= len as i32 {
        return Err(format!(
            "Cannot move step {} by {}: would be out of bounds",
            index, direction
        ));
    }
    session.pipeline.steps.swap(index, new_index as usize);
    let from = index.min(new_index as usize);
    session.pipeline.invalidate_from(from);
    Ok(PipelineSummary::from(&session.pipeline))
}

#[tauri::command]
pub fn get_pipeline_summary(state: State<AppState>) -> Result<PipelineSummary, String> {
    let session = state.session.read().map_err(|e| e.to_string())?;
    Ok(PipelineSummary::from(&session.pipeline))
}
