# DitherPath

Rust crate that turns a **GYST UUIDv8** into a **surface-stable fractal dither**,
reads that field back from a sparse intensity scan, and plans a collision-free
path around the decoded keep-out.

```
GYST identity  →  encoded Bayer pyramid  →  lidar snap
               →  decoded UUID + keep-out + goal
               →  RouteRequest JSON  →  A* / string-pull
```

Not a git-merge of [Dither3D](https://github.com/somacosf/dither3d) and
[clearpath](https://github.com/mrorigo/clearpath). Those trees do not share a
runtime. This crate is the joint. See `NOTICE`.

GYST layout is the published spec:
https://gist.github.com/SoMaCoSF/582fdee49c30196663b759ca9a468a5e

## Build

```
cargo test
cargo run --release -- demo examples
```

`demo` writes:

- `examples/uuid.txt` — hyphenated GYST UUIDv8
- `examples/dither_stack.pgm` — occupancy layers, bottom to top (Dither3D convention)
- `examples/route.json` — clearpath-shaped `RouteRequest` plus polyline
- `examples/route.svg` — workspace, marked patch, path

```
cargo run --release -- uuid pad-00a1
```

## Modules

| module | job |
| --- | --- |
| `gyst` | RFC 9562 v8 pack/unpack, FNV-1a-12 namespace, SHA-256 low-42 |
| `codec` | fractal occupancy encode/decode, lens permutation |
| `lidar` | sparse intensity sample + lattice snap |
| `payload` | UUID + keep-out cm + relative goal |
| `planner` | deterministic grid A* + string-pull; JSON emitter |
| `texture` | PGM stack + SVG preview |
| `pipeline` | the loop |

## Physical bound

L0 dots have to beat lidar angular spacing at the farthest decode range.
At 20 m and 0.2° that is ~7 cm. Occupancy (retroreflective vs matte) is the
alphabet. RGB Dither3D modes are the projection/vision twin of the same
generator, not the lidar channel.

## Clearpath handoff

`RouteRequest.margin` is 0. Keep-out is already baked into the AABB so
`MarginUnsupportedGeometry` is not triggered. Point `route_smooth` at the JSON
when rustc ≥ 1.85 is on the machine.
