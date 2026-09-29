# DitherPath

Rust crate that turns a **GYST UUIDv8** into a **surface-stable fractal dither**,
reads that field back from a sparse intensity scan, judges the read with **Jev**,
and plans a collision-free path around the decoded keep-out.

**Next agent:** start at [`docs/HANDOFF.md`](docs/HANDOFF.md) and [`AGENTS.md`](AGENTS.md).

```
GYST identity  →  encoded Bayer pyramid  →  lidar snap  →  CRC
               →  Jev decode (identified | anchored | unanchored)
               →  RouteRequest JSON  →  A* / string-pull   (geometry only)
               →  Jev path / approach / project
```

Not a git-merge of [Dither3D](https://github.com/somacosf/dither3d) and
[clearpath](https://github.com/mrorigo/clearpath). Those trees do not share a
runtime. This crate is the joint. See `NOTICE`.

GYST layout: https://gist.github.com/SoMaCoSF/582fdee49c30196663b759ca9a468a5e

## Build

```
cargo test
cargo run --release -- demo examples
cargo run --release -- uuid pad-00a1
```

`demo` writes `examples/uuid.txt`, `examples/route.json`, and (when texture is on main) a PGM stack + SVG.

## Modules

| module | job |
| --- | --- |
| `gyst` | RFC 9562 v8 pack/unpack, FNV-1a-12 namespace, SHA-256 low-42 |
| `codec` | fractal occupancy encode/decode, lens permutation |
| `lidar` | sparse intensity sample + lattice snap |
| `payload` | UUID + keep-out cm + relative goal |
| `jev` | Noul / Choice / Score types + decode/path policy |
| `capability` | generic stage catalog: decode, path, approach, project |
| `planner` | deterministic grid A* + string-pull; JSON emitter |
| `texture` | PGM stack + SVG preview |
| `pipeline` | the loop |

## Rails

- Payload only on *new* sites per octave (surface-stability).
- Clearpath-class geometry judges nothing. Jev is the capability around it.
- `RouteRequest.margin` is 0. Keep-out is already in the AABB.
- rustc 1.75. Do not vendor clearpath (1.85).
- Live Jev key stays on the server.

## Physical bound

L0 dots have to beat lidar angular spacing at the farthest decode range.
At 20 m and 0.2° that is ~7 cm. Occupancy is the lidar alphabet. RGB is the
projection twin of the same generator.
