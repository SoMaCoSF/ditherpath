//! Generic Jev capability.
//!
//! Clearpath (and our A* kernel) compute a polyline. They judge nothing.
//! This module is the missing layer: any stage can register a named set of
//! Noul/Choice/Score questions plus a policy that turns answers into an Action.
//! Same loop as jev-minesweeper-harness World: observe → decide → policy → step.

use std::collections::BTreeMap;

use crate::jev::{Answer, HeuristicJev, Jev, Question};

#[derive(Clone, Debug, PartialEq)]
pub struct Action {
    pub kind: String,
    pub target: String,
    pub accepted: bool,
    pub notes: BTreeMap<String, String>,
}

impl Action {
    pub fn new(kind: impl Into<String>, target: impl Into<String>, accepted: bool) -> Self {
        Self {
            kind: kind.into(),
            target: target.into(),
            accepted,
            notes: BTreeMap::new(),
        }
    }
    pub fn note(mut self, k: impl Into<String>, v: impl Into<String>) -> Self {
        self.notes.insert(k.into(), v.into());
        self
    }
}

/// A named judgment surface. Geometry modules do not implement this.
pub trait Capability {
    fn name(&self) -> &'static str;
    fn questions(&self) -> BTreeMap<String, Question>;
    fn policy(&self, answers: &BTreeMap<String, Answer>, state: &str) -> Action;
}

/// Run any capability against a Jev backend (heuristic or live).
pub fn run<C: Capability, J: Jev>(cap: &C, jev: &J, state: &str) -> Action {
    let result = jev.decide(state, &cap.questions());
    cap.policy(&result.answers, state)
}

/// Built-in surfaces. Add more without touching the planner.

pub struct DecodeCap;
impl Capability for DecodeCap {
    fn name(&self) -> &'static str { "decode" }
    fn questions(&self) -> BTreeMap<String, Question> { crate::jev::decode_questions() }
    fn policy(&self, answers: &BTreeMap<String, Answer>, state: &str) -> Action {
        let crc_ok = state.contains("crc_ok=true");
        let d = crate::jev::apply_decode(answers, crc_ok);
        Action::new(
            format!("{:?}", d.verdict).to_lowercase(),
            d.layer.clone().unwrap_or_else(|| "NONE".into()),
            d.verdict == crate::jev::DecodeVerdict::Identified,
        )
        .note("reason", d.reason)
    }
}

pub struct PathCap;
impl Capability for PathCap {
    fn name(&self) -> &'static str { "path" }
    fn questions(&self) -> BTreeMap<String, Question> { crate::jev::path_questions() }
    fn policy(&self, answers: &BTreeMap<String, Answer>, _state: &str) -> Action {
        let p = crate::jev::apply_path(answers);
        Action::new(format!("{:?}", p.maneuver).to_lowercase(), "route", p.accepted)
    }
}

/// Approach / hold / scan-again — the drone question clearpath cannot ask.
pub struct ApproachCap;
impl Capability for ApproachCap {
    fn name(&self) -> &'static str { "approach" }
    fn questions(&self) -> BTreeMap<String, Question> {
        let mut q = BTreeMap::new();
        q.insert("closer".into(), Question::noul(
            "Would dropping one octave closer recover a cleaner lattice without entering keep-out?",
        ));
        q.insert("stance".into(), Question::choice(
            "What should the vehicle do relative to this mark?",
            &[("approach", "move in one octave"), ("hold", "stay and integrate more hits"), ("orbit", "circle for a second aspect"), ("depart", "leave the mark")],
        ));
        q.insert("urgency".into(), Question::score(
            "How time-critical is a better read?",
            &["can wait", "this pass", "before the next waypoint", "now"],
        ));
        q
    }
    fn policy(&self, answers: &BTreeMap<String, Answer>, _state: &str) -> Action {
        let closer = answers.get("closer").and_then(|a| a.noul).unwrap_or(0.0);
        let stance = answers.get("stance").and_then(|a| a.choice.clone()).unwrap_or_else(|| "hold".into());
        let accepted = closer >= 0.70 && stance == "approach";
        Action::new(stance, "octave", accepted)
    }
}

/// Project / paint / withhold — vision side of the same field.
pub struct ProjectCap;
impl Capability for ProjectCap {
    fn name(&self) -> &'static str { "project" }
    fn questions(&self) -> BTreeMap<String, Question> {
        let mut q = BTreeMap::new();
        q.insert("emit".into(), Question::noul(
            "Should we project the decoded field (identity + keep-out) onto the vehicle vision layer?",
        ));
        q.insert("channel".into(), Question::choice(
            "Which vision channel carries the projection?",
            &[("occupancy", "raw lattice"), ("identity", "UUID overlay"), ("keepout", "inflated AABB only"), ("NONE", "do not project")],
        ));
        q.insert("opacity".into(), Question::score(
            "How hard should the overlay read?",
            &["hint", "readable", "operational", "cannot miss"],
        ));
        q
    }
    fn policy(&self, answers: &BTreeMap<String, Answer>, _state: &str) -> Action {
        let emit = answers.get("emit").and_then(|a| a.noul).unwrap_or(0.0);
        let channel = answers.get("channel").and_then(|a| a.choice.clone()).unwrap_or_else(|| "NONE".into());
        let accepted = emit >= 0.70 && channel != "NONE";
        Action::new(channel, "vision", accepted)
    }
}

pub fn catalog() -> Vec<Box<dyn Capability>> {
    vec![
        Box::new(DecodeCap),
        Box::new(PathCap),
        Box::new(ApproachCap),
        Box::new(ProjectCap),
    ]
}

/// Convenience: run the whole catalog on one state string with HeuristicJev.
pub fn run_all(state: &str) -> BTreeMap<String, Action> {
    let jev = HeuristicJev;
    let mut out = BTreeMap::new();
    for cap in catalog() {
        out.insert(cap.name().into(), run(cap.as_ref(), &jev, state));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_four_surfaces() {
        assert_eq!(catalog().len(), 4);
    }

    #[test]
    fn good_scan_identifies_and_may_project() {
        let acts = run_all("visible=4 crc_ok=true hits=200 snr_high=true clearance_ok=true blocked=false cost=10.4");
        assert_eq!(acts["decode"].kind.contains("identified") || acts["decode"].accepted, true);
        assert!(acts.contains_key("path"));
        assert!(acts.contains_key("approach"));
        assert!(acts.contains_key("project"));
    }
}
