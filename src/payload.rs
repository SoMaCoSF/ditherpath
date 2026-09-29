//! Surface payload: raw GYST UUID + keep-out + optional relative goal.

use crate::gyst::GystUuid;

#[derive(Clone, Debug, PartialEq)]
pub struct SurfacePayload {
    pub uuid: [u8; 16],
    pub keepout_m: f64,
    pub goal_rel: Option<(f64, f64)>,
}

impl SurfacePayload {
    pub fn from_uuid(uuid: &GystUuid, keepout_m: f64, goal_rel: Option<(f64, f64)>) -> Self {
        Self {
            uuid: uuid.bytes,
            keepout_m,
            goal_rel,
        }
    }

    pub fn pack(&self) -> Vec<u8> {
        let mut out = self.uuid.to_vec();
        let keep = (self.keepout_m * 100.0).round() as u16;
        out.extend_from_slice(&keep.to_be_bytes());
        match self.goal_rel {
            None => out.push(0),
            Some((gx, gy)) => {
                out.push(1);
                let xb = ((gx * 100.0).round() as i16).to_be_bytes();
                let yb = ((gy * 100.0).round() as i16).to_be_bytes();
                out.extend_from_slice(&xb);
                out.extend_from_slice(&yb);
            }
        }
        out
    }

    pub fn unpack(bytes: &[u8]) -> Result<Self, PayloadError> {
        if bytes.len() < 19 {
            return Err(PayloadError::Short);
        }
        let mut uuid = [0u8; 16];
        uuid.copy_from_slice(&bytes[..16]);
        let keepout_m = u16::from_be_bytes([bytes[16], bytes[17]]) as f64 / 100.0;
        let goal_rel = if bytes[18] & 1 != 0 {
            if bytes.len() < 23 {
                return Err(PayloadError::Short);
            }
            Some((
                i16::from_be_bytes([bytes[19], bytes[20]]) as f64 / 100.0,
                i16::from_be_bytes([bytes[21], bytes[22]]) as f64 / 100.0,
            ))
        } else {
            None
        };
        Ok(Self {
            uuid,
            keepout_m,
            goal_rel,
        })
    }

    pub fn uuid_hyphenated(&self) -> String {
        GystUuid::decode(self.uuid)
            .map(|u| u.hyphenated())
            .unwrap_or_else(|_| self.uuid.iter().map(|b| format!("{b:02x}")).collect())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PayloadError {
    Short,
}

impl std::fmt::Display for PayloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "surface payload too short")
    }
}
impl std::error::Error for PayloadError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gyst::{GystFields, GystUuid};

    #[test]
    fn pack_roundtrip() {
        let u = GystUuid::encode(GystFields::mark("pad-00a1", 1_700_000_000, 0.4));
        let p = SurfacePayload::from_uuid(&u, 0.40, Some((8.0, 0.5)));
        let back = SurfacePayload::unpack(&p.pack()).unwrap();
        assert_eq!(back.uuid, u.bytes);
        assert!((back.keepout_m - 0.40).abs() < 0.001);
        assert_eq!(back.goal_rel, Some((8.0, 0.5)));
    }
}
