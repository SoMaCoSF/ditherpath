//! DitherPath — surface-stable encoded dither, GYST identity, deterministic pathing.

pub mod codec;
pub mod gyst;
pub mod lidar;
pub mod payload;
pub mod pipeline;
pub mod planner;
pub mod texture;

pub use codec::{capacity_bits, encode, EncodedPatch};
pub use gyst::{GystFields, GystUuid};
pub use payload::SurfacePayload;
pub use pipeline::{run, write_artifacts, DemoConfig, PipelineOut};
pub use planner::{plan, Route, RouteRequest};
