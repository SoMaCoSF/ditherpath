# DitherPath architecture

Single Rust crate. Geometry and judgment are separate.

```
GYST UUIDv8                 src/gyst.rs
        |
        | 16 raw bytes + keep-out + goal
        v
fractal occupancy pyramid   src/codec.rs
        |  payload only on NEW octaves
        v
lidar intensity snap        src/lidar.rs
        |  CRC + lens
        v
Jev capability              src/jev.rs + src/capability.rs
        |  Identified | Anchored | Unanchored
        v
AABB + A* + string-pull     src/planner.rs     (zero judgment)
        |
        v
Jev capability again        path / approach / project
        |  skirt|hold|replan|abort ; project channel
        v
RouteRequest JSON           examples/route.json
```

Clearpath (and our A*) compute a polyline. They judge nothing.
Jev is a Capability any stage can register. See `docs/JEV.md` and `docs/HANDOFF.md`.

## Carrier constraint

Dither3D's second rule: zoom in adds dots, never removes them. Encoding copies
parent sites to the even-even sublattice and writes bits only on the complementary
sites. A far scan resolves the L0 beacon; a near scan resolves L4.

## Identity

GYST UUIDv8 from
https://gist.github.com/SoMaCoSF/582fdee49c30196663b759ca9a468a5e

Type `0x610` (`DITHER_MARK`), domain `TRANSPORT` (0xD), namespace =
FNV-1a-12(`somacosf.com`). Low-42 is SHA-256 of the field seed.

## Licenses at the boundary

- This crate: MIT
- Dither3D shaders: MPL-2.0 — not vendored
- clearpath: MIT/Apache — not vendored; JSON is the joint
