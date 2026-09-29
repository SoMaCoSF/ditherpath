//! DitherPath — surface-stable encoded dither, GYST identity,
//! Jev capability layer, deterministic pathing.

pub mod capability;
pub mod codec;
pub mod gyst;
pub mod jev;
pub mod lidar;
pub mod payload;
pub mod pipeline;
pub mod planner;
pub mod texture;

pub use capability::{run, run_all, Action, ApproachCap, Capability, DecodeCap, PathCap, ProjectCap};
pub use codec::{capacity_bits, encode, EncodedPatch};
pub use gyst::{GystFields, GystUuid};
pub use jev::{apply_decode, apply_path, DecodeVerdict, HeuristicJev, Jev, Maneuver};
pub use payload::SurfacePayload;
pub use pipeline::{run as run_pipeline, write_artifacts, DemoConfig, PipelineOut};
pub use planner::{plan, Route, RouteRequest};
