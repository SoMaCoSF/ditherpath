//! Full loop: GYST -> encode -> lidar -> decode -> plan.

use crate::codec::{encode, EncodedPatch};
use crate::gyst::{GystFields, GystUuid};
use crate::lidar::{decode_hits, simulate};
use crate::payload::SurfacePayload;
use crate::planner::{plan, Aabb, PlanError, Point, Route, RouteRequest};
use crate::texture::{write_pgm_stack, write_svg_preview};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct DemoConfig {
    pub content_key: String,
    pub timestamp_sec: u32,
    pub signal: f64,
    pub lens_id: u8,
    pub max_layer: u32,
    pub keepout_m: f64,
    pub tile_m: f64,
    pub origin: Point,
    pub start: Point,
    pub workspace: Aabb,
    pub goal_rel: Option<(f64, f64)>,
}

impl Default for DemoConfig {
    fn default() -> Self {
        Self {
            content_key: "pad-00a1".into(),
            timestamp_sec: 1_746_000_000,
            signal: 0.5,
            lens_id: 0x5A,
            max_layer: 4,
            keepout_m: 0.40,
            tile_m: 2.0,
            origin: Point::new(2.0, 3.0),
            start: Point::new(0.5, 4.0),
            workspace: Aabb {
                min: Point::new(0.0, 0.0),
                max: Point::new(12.0, 8.0),
                id: "ws".into(),
            },
            goal_rel: Some((8.0, 0.5)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PipelineOut {
    pub uuid: GystUuid,
    pub payload: SurfacePayload,
    pub patch: EncodedPatch,
    pub request: RouteRequest,
    pub route: Route,
}

pub fn run(cfg: &DemoConfig) -> Result<PipelineOut, Box<dyn std::error::Error>> {
    let uuid = GystUuid::encode(GystFields::mark(
        &cfg.content_key,
        cfg.timestamp_sec,
        cfg.signal,
    ));
    let payload = SurfacePayload::from_uuid(&uuid, cfg.keepout_m, cfg.goal_rel);
    let mut patch = encode(&payload.pack(), cfg.lens_id, cfg.max_layer)?;
    patch.tile_m = cfg.tile_m;
    let hits = simulate(&patch, cfg.max_layer, cfg.tile_m / 16.0, 0.03, 7);
    let recovered = decode_hits(&hits, cfg.lens_id, cfg.max_layer, cfg.tile_m)?;
    let decoded = SurfacePayload::unpack(&recovered)?;
    if decoded.uuid != uuid.bytes {
        return Err("decoded UUID mismatch".into());
    }
    let half = cfg.tile_m / 2.0 + decoded.keepout_m;
    let cx = cfg.origin.x + cfg.tile_m / 2.0;
    let cy = cfg.origin.y + cfg.tile_m / 2.0;
    let obstacle = Aabb {
        min: Point::new(cx - half, cy - half),
        max: Point::new(cx + half, cy + half),
        id: decoded.uuid_hyphenated(),
    };
    let goal = match decoded.goal_rel {
        Some((gx, gy)) => Point::new(cfg.origin.x + gx, cfg.origin.y + gy),
        None => Point::new(
            cfg.workspace.max.x - 1.0,
            (cfg.workspace.min.y + cfg.workspace.max.y) / 2.0,
        ),
    };
    let request = RouteRequest {
        workspace: cfg.workspace.clone(),
        start: cfg.start,
        goal,
        margin: 0.0,
        obstacles: vec![obstacle],
    };
    let route = plan(&request, 0.25).map_err(|e: PlanError| e.to_string())?;
    Ok(PipelineOut {
        uuid,
        payload: decoded,
        patch,
        request,
        route,
    })
}

pub fn write_artifacts(out: &PipelineOut, dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    write_pgm_stack(&out.patch, &dir.join("dither_stack.pgm"), 8)?;
    let route_pts: Vec<(f64, f64)> = out.route.polyline.iter().map(|p| (p.x, p.y)).collect();
    let obs: Vec<(f64, f64, f64, f64)> = out
        .request
        .obstacles
        .iter()
        .map(|o| (o.min.x, o.min.y, o.max.x, o.max.y))
        .collect();
    write_svg_preview(
        &out.patch,
        &route_pts,
        (
            out.request.workspace.min.x,
            out.request.workspace.min.y,
            out.request.workspace.max.x,
            out.request.workspace.max.y,
        ),
        &obs,
        &dir.join("route.svg"),
    )?;
    std::fs::write(dir.join("route.json"), out.request.to_json(Some(&out.route)))?;
    std::fs::write(dir.join("uuid.txt"), format!("{}
", out.uuid.hyphenated()))?;
    Ok(())
}
