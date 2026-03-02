use serde::{Deserialize, Serialize};
use uuid::Uuid;

// =========================
// Temporal Filtering
// =========================
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BandpassParams {
    pub low_cutoff: f32,
    pub high_cutoff: f32,
    pub order: usize,
}

// =========================
// Channel Pruning
// =========================
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum PruningMethod {
    Sci,
    Psp,
    Snr,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PruningParams {
    pub pruning_method: PruningMethod,
    pub threshold: f32,
}

// =========================
// Preprocessing Step
// =========================
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum StepKind {
    Bandpass(BandpassParams),
    Pruning(PruningParams),
}

fn label_for(kind: &StepKind) -> String {
    match kind {
        StepKind::Bandpass(_) => "Bandpass Filter".to_string(),
        StepKind::Pruning(_) => "Channel Pruning".to_string(),
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PipelineStep {
    pub id: String,
    pub label: String,
    pub step_kind: StepKind,
    pub enabled: bool,
}

impl PipelineStep {
    pub fn new(kind: StepKind) -> Self {
        let label = label_for(&kind);
        Self {
            id: Uuid::new_v4().to_string(),
            label,
            step_kind: kind,
            enabled: true,
        }
    }

    pub fn invalidate(&mut self) {
        // Will clear cached_output when caching is added in Phase 5
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Pipeline {
    pub steps: Vec<PipelineStep>,
    pub active_step_index: Option<usize>,
}

impl Pipeline {
    pub fn push(&mut self, kind: StepKind) {
        self.steps.push(PipelineStep::new(kind));
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.steps.len() {
            self.steps.remove(index);
            self.invalidate_from(index);
        }
    }

    pub fn invalidate_from(&mut self, from: usize) {
        for step in self.steps.iter_mut().skip(from) {
            step.invalidate();
        }
    }
}
