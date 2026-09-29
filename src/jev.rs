//! Jev System One types and DitherPath decision policy.
//!
//! Jev is a classifier (Noul / Choice / Score), not a planner and not a decoder.
//! Code owns the branch. Contract: SoMaCoSF/jev-minesweeper-harness and gist d3944c12.
//! Choice is a closed list. NONE is real. Missing answers stay None.

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Question {
    Noul { instructions: String },
    Choice { instructions: String, criteria: BTreeMap<String, String> },
    Score { instructions: String, criteria: Vec<String> },
}

impl Question {
    pub fn noul(instructions: impl Into<String>) -> Self {
        Question::Noul { instructions: instructions.into() }
    }
    pub fn choice(instructions: impl Into<String>, criteria: &[(&str, &str)]) -> Self {
        Question::Choice {
            instructions: instructions.into(),
            criteria: criteria.iter().map(|(k, v)| ((*k).into(), (*v).into())).collect(),
        }
    }
    pub fn score(instructions: impl Into<String>, criteria: &[&str]) -> Self {
        Question::Score {
            instructions: instructions.into(),
            criteria: criteria.iter().map(|s| (*s).into()).collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Answer {
    pub kind: &'static str,
    pub noul: Option<f64>,
    pub choice: Option<String>,
    pub score: Option<f64>,
    pub confidence: Option<f64>,
    pub probabilities: BTreeMap<String, f64>,
}

impl Answer {
    pub fn noul_val(p: f64) -> Self {
        Self {
            kind: "noul",
            noul: Some(p.clamp(0.0, 1.0)),
            choice: None,
            score: None,
            confidence: None,
            probabilities: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DecideResult {
    pub answers: BTreeMap<String, Answer>,
    pub model: String,
    pub mocked: bool,
}

pub trait Jev {
    fn decide(&self, state: &str, questions: &BTreeMap<String, Question>) -> DecideResult;
}

/// Offline stand-in. Reads flags already in `state`. Live Jev implements the same trait.
pub struct HeuristicJev;

impl Jev for HeuristicJev {
    fn decide(&self, state: &str, questions: &BTreeMap<String, Question>) -> DecideResult {
        let mut answers = BTreeMap::new();
        for (name, q) in questions {
            answers.insert(name.clone(), heuristic(state, q));
        }
        DecideResult { answers, model: "heuristic".into(), mocked: true }
    }
}

fn heuristic(state: &str, q: &Question) -> Answer {
    match q {
        Question::Noul { .. } => {
            let p = if state.contains("crc_ok=true") {
                0.92
            } else if state.contains("crc_ok=false") || state.contains("hits=0") {
                0.08
            } else if state.contains("clearance_ok=true") {
                0.88
            } else if state.contains("blocked=true") {
                0.12
            } else {
                0.45
            };
            Answer::noul_val(p)
        }
        Question::Choice { criteria, .. } => {
            let keys: Vec<String> = criteria.keys().cloned().collect();
            let pick = if state.contains("crc_ok=true") && keys.iter().any(|k| k == "commit") {
                "commit"
            } else if state.contains("visible=4") && keys.iter().any(|k| k == "L4") {
                "L4"
            } else if state.contains("visible=3") && keys.iter().any(|k| k == "L3") {
                "L3"
            } else if state.contains("clearance_ok=true") && keys.iter().any(|k| k == "skirt") {
                "skirt"
            } else if keys.iter().any(|k| k == "NONE") {
                "NONE"
            } else {
                keys.first().map(|s| s.as_str()).unwrap_or("NONE")
            };
            let mut probabilities = BTreeMap::new();
            let rest = (keys.len().max(2) - 1) as f64;
            for k in &keys {
                probabilities.insert(k.clone(), if k == pick { 0.86 } else { 0.14 / rest });
            }
            Answer {
                kind: "choice",
                noul: None,
                choice: Some(pick.into()),
                score: None,
                confidence: Some(0.86),
                probabilities,
            }
        }
        Question::Score { criteria, .. } => {
            let n = criteria.len().max(2) as f64;
            let level = if state.contains("snr_high=true") { n - 1.0 } else if state.contains("snr_high=false") { 1.0 } else { (n - 1.0) * 0.5 };
            Answer {
                kind: "score",
                noul: None,
                choice: None,
                score: Some(level),
                confidence: Some(0.7),
                probabilities: BTreeMap::new(),
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecodeVerdict { Identified, Anchored, Unanchored }

#[derive(Clone, Debug, PartialEq)]
pub struct DecodeDecision {
    pub verdict: DecodeVerdict,
    pub commit_noul: Option<f64>,
    pub layer: Option<String>,
    pub quality: Option<f64>,
    pub reason: String,
}

pub fn decode_questions() -> BTreeMap<String, Question> {
    let mut q = BTreeMap::new();
    q.insert("commit_payload".into(), Question::noul(
        "Is the recovered occupancy a trustworthy encoding of a GYST surface mark? Yes only if magic, lens, and CRC all hold.",
    ));
    q.insert("layer".into(), Question::choice(
        "Which finest octave is actually resolved in this scan? NONE if the lattice is not readable.",
        &[("L1", "header-only, far scan"), ("L2", "coarse payload"), ("L3", "most of payload"), ("L4", "full field"), ("NONE", "do not claim a layer")],
    ));
    q.insert("quality".into(), Question::score(
        "How clean is the snapped occupancy lattice?",
        &["unreadable speckle", "partial lattice", "usable with noise", "clean Bayer field"],
    ));
    q
}

pub fn apply_decode(answers: &BTreeMap<String, Answer>, crc_ok: bool) -> DecodeDecision {
    let commit_noul = answers.get("commit_payload").and_then(|a| a.noul);
    let layer = answers.get("layer").and_then(|a| a.choice.clone());
    let quality = answers.get("quality").and_then(|a| a.score);
    let commit = commit_noul.unwrap_or(0.0);
    let placed = layer.as_deref().map(|s| s != "NONE").unwrap_or(false);
    let (verdict, reason) = if crc_ok && placed && commit >= 0.80 {
        (DecodeVerdict::Identified, "CRC held, layer chosen, commit noul >= 0.80 — accept UUID")
    } else if placed && commit >= 0.40 {
        (DecodeVerdict::Anchored, "lattice placed, identity refused — keep-out only")
    } else {
        (DecodeVerdict::Unanchored, "observation kept; no place, no identity")
    };
    DecodeDecision { verdict, commit_noul, layer, quality, reason: reason.into() }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Maneuver { Skirt, Hold, Replan, Abort }

#[derive(Clone, Debug, PartialEq)]
pub struct PathDecision {
    pub accept_noul: Option<f64>,
    pub maneuver: Maneuver,
    pub exposure: Option<f64>,
    pub accepted: bool,
}

pub fn path_questions() -> BTreeMap<String, Question> {
    let mut q = BTreeMap::new();
    q.insert("accept_route".into(), Question::noul(
        "Is this polyline a safe transit given the decoded keep-out and remaining clearance?",
    ));
    q.insert("maneuver".into(), Question::choice(
        "Which closed action should the vehicle take now?",
        &[("skirt", "follow the planned detour"), ("hold", "stop short of keep-out"), ("replan", "invalidate polyline"), ("abort", "abandon goal")],
    ));
    q.insert("exposure".into(), Question::score(
        "How exposed is this corridor to the marked obstacle?",
        &["wide berth", "comfortable margin", "tight but legal", "grazing the keep-out"],
    ));
    q
}

pub fn apply_path(answers: &BTreeMap<String, Answer>) -> PathDecision {
    let accept_noul = answers.get("accept_route").and_then(|a| a.noul);
    let exposure = answers.get("exposure").and_then(|a| a.score);
    let label = answers.get("maneuver").and_then(|a| a.choice.as_deref()).unwrap_or("hold");
    let maneuver = match label {
        "skirt" => Maneuver::Skirt,
        "replan" => Maneuver::Replan,
        "abort" => Maneuver::Abort,
        _ => Maneuver::Hold,
    };
    let accepted = accept_noul.unwrap_or(0.0) >= 0.70 && matches!(maneuver, Maneuver::Skirt);
    PathDecision { accept_noul, maneuver, exposure, accepted }
}

pub fn decode_state(visible: u32, crc_ok: bool, hits: usize, snr_high: bool) -> String {
    format!("visible={visible} crc_ok={crc_ok} hits={hits} snr_high={snr_high}")
}

pub fn path_state(clearance_ok: bool, blocked: bool, cost: f64) -> String {
    format!("clearance_ok={clearance_ok} blocked={blocked} cost={cost:.3}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identified_only_when_crc_and_commit() {
        let ans = HeuristicJev.decide(&decode_state(4, true, 200, true), &decode_questions()).answers;
        let d = apply_decode(&ans, true);
        assert_eq!(d.verdict, DecodeVerdict::Identified);
        assert_eq!(d.layer.as_deref(), Some("L4"));
    }

    #[test]
    fn crc_fail_refuses_identity() {
        let ans = HeuristicJev.decide(&decode_state(2, false, 40, false), &decode_questions()).answers;
        assert_ne!(apply_decode(&ans, false).verdict, DecodeVerdict::Identified);
    }

    #[test]
    fn path_skirts_when_clear() {
        let ans = HeuristicJev.decide(&path_state(true, false, 10.4), &path_questions()).answers;
        let p = apply_path(&ans);
        assert_eq!(p.maneuver, Maneuver::Skirt);
        assert!(p.accepted);
    }
}
