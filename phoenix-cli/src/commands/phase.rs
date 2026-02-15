use phoenix_lib::workflow::{Phase, PhaseStatus, WorkflowPhaseEvent};

pub fn emit(phase: Phase, status: PhaseStatus, detail: Option<&str>) {
    if std::env::var("PHOENIX_PHASE_EVENTS").ok().as_deref() != Some("1") {
        return;
    }

    let event = WorkflowPhaseEvent::new(phase, status, detail.map(|value| value.to_string()));

    if let Ok(payload) = serde_json::to_string(&event) {
        println!("{}", payload);
    }
}
