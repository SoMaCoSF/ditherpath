//! Sparse intensity sampling and lattice snap.

use crate::codec::{decode, CodecError, EncodedPatch, Site};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug)]
pub struct LidarHit {
    pub x: f64,
    pub y: f64,
    pub intensity: f64,
}

fn lcg(state: &mut u32) -> f64 {
    *state = state.wrapping_mul(1664525).wrapping_add(1013904223);
    (*state as f64) / (u32::MAX as f64)
}

pub fn simulate(
    patch: &EncodedPatch,
    layer_visible: u32,
    spacing_m: f64,
    noise: f64,
    seed: u32,
) -> Vec<LidarHit> {
    let l = layer_visible.min(patch.layers.len().saturating_sub(1) as u32);
    let occ = &patch.layers[l as usize];
    let n = 1u16 << l;
    let pitch = patch.tile_m / n as f64;
    let extent = patch.tile_m;
    let steps = ((extent / spacing_m).ceil() as i32 + 1).max(2);
    let mut rng = seed;
    let mut hits = Vec::new();
    for iy in 0..steps {
        for ix in 0..steps {
            let wx = ix as f64 * spacing_m;
            let wy = iy as f64 * spacing_m;
            if wx < -pitch || wy < -pitch || wx > extent + pitch || wy > extent + pitch {
                continue;
            }
            let sx = ((wx / pitch).round() as i32).rem_euclid(n as i32) as u16;
            let sy = ((wy / pitch).round() as i32).rem_euclid(n as i32) as u16;
            let cx = sx as f64 * pitch;
            let cy = sy as f64 * pitch;
            let dx = wx - cx;
            let dy = wy - cy;
            if dx * dx + dy * dy > (0.45 * pitch) * (0.45 * pitch) {
                continue;
            }
            let bit = *occ.get(&(sx, sy)).unwrap_or(&0);
            let inten = if bit == 1 { 0.85 } else { 0.12 };
            hits.push(LidarHit {
                x: cx,
                y: cy,
                intensity: (inten + (lcg(&mut rng) - 0.5) * 2.0 * noise).clamp(0.0, 1.0),
            });
        }
    }
    hits
}

pub fn recover_occupancy(
    hits: &[LidarHit],
    layer: u32,
    tile_m: f64,
    thresh: f64,
) -> BTreeMap<Site, u8> {
    let n = 1u16 << layer;
    let pitch = tile_m / n as f64;
    let mut votes: BTreeMap<Site, (f64, u32)> = BTreeMap::new();
    for h in hits {
        let sx = ((h.x / pitch).round() as i32).rem_euclid(n as i32) as u16;
        let sy = ((h.y / pitch).round() as i32).rem_euclid(n as i32) as u16;
        let e = votes.entry((sx, sy)).or_insert((0.0, 0));
        e.0 += h.intensity;
        e.1 += 1;
    }
    votes
        .into_iter()
        .map(|(k, (s, c))| (k, u8::from(s / c as f64 >= thresh)))
        .collect()
}

pub fn estimate_visible(hits: &[LidarHit], max_layer: u32, tile_m: f64) -> u32 {
    let mut xs: Vec<f64> = hits.iter().map(|h| (h.x * 1e6).round() / 1e6).collect();
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    xs.dedup();
    if xs.len() < 2 {
        return 0;
    }
    let mut pitch = f64::MAX;
    for w in xs.windows(2) {
        let d = w[1] - w[0];
        if d > 0.0 && d < pitch {
            pitch = d;
        }
    }
    let mut visible = 0u32;
    for l in 0..=max_layer {
        if tile_m / ((1u32 << l) as f64) >= pitch * 0.6 {
            visible = l;
        }
    }
    visible
}

pub fn decode_hits(
    hits: &[LidarHit],
    lens_id: u8,
    max_layer: u32,
    tile_m: f64,
) -> Result<Vec<u8>, CodecError> {
    if hits.is_empty() {
        return Err(CodecError::NoHits);
    }
    let visible = estimate_visible(hits, max_layer, tile_m).min(max_layer);
    let fine = recover_occupancy(hits, visible, tile_m, 0.5);
    decode(&fine, visible, lens_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::encode;

    #[test]
    fn noisy_roundtrip() {
        let payload = b"GYST-PAD-00A1\x00\x00\x00";
        let patch = encode(payload, 0x5A, 4).unwrap();
        let hits = simulate(&patch, 4, patch.tile_m / 16.0, 0.03, 7);
        assert_eq!(decode_hits(&hits, 0x5A, 4, patch.tile_m).unwrap(), payload);
    }

    #[test]
    fn wrong_lens_fails() {
        let patch = encode(b"hello-world-pad!!", 0x5A, 4).unwrap();
        let hits = simulate(&patch, 4, patch.tile_m / 16.0, 0.0, 1);
        assert!(decode_hits(&hits, 0x11, 4, patch.tile_m).is_err());
    }
}
