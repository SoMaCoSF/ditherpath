# DitherPath — agent handoff

**Date:** 2026-09-29  
**Repo:** https://github.com/SoMaCoSF/ditherpath  
**Owner:** SoMaCoSF  
**HEAD at time of writing:** `e97d23b` (docs) / lidar snapshot `338b6d4`  
**Language:** Rust 2021, `rust-version = "1.75"`  
**License:** MIT. See `NOTICE` for Dither3D (MPL-2.0) and clearpath (MIT) attribution. This is **not** a git merge of those trees.

Read this file first. Then `AGENTS.md`, `docs/JEV.md`, `docs/ARCHITECTURE.md`. Do not invent a second architecture.

---

## What this crate is

One loop:

```
GYST UUIDv8
  → fractal Bayer occupancy (payload only on NEW sites per octave)
  → sparse intensity / lidar snap
  → CRC + lens check
  → Jev capability (judgment)
  → deterministic A* / string-pull (geometry)
  → RouteRequest JSON (clearpath-shaped)
  → optional vision projection of the same field
```

Intent from the owner, in order:

1. Merge the *ideas* of `somacosf/dither3d` and `mrorigo/clearpath` into **one** repo named DitherPath.
2. Encode data into the dither pattern (the thing he was patenting). A drone lidar-reads the field in flight.
3. Same generator feeds a data-rich projection / vision overlay.
4. Entire thing in **Rust**, one crate.
5. Include **Jev** (TypeSafe System One: Noul / Choice / Score) as a capability on pathing **and** decode — then more broadly on any stage. Clearpath itself does **zero** judgment; keep it that way.
6. Commit often. Make the repo **private** (connector cannot flip visibility; owner clicks Settings → Change visibility).
7. Document so the next agent can carry load. That is this file.

---

## Hard rails (do not violate)

Treat these as tests.

1. **Payload only on new sites per octave.** Parent sites copy down. Dither3D surface-stability. Payload on inherited sites makes the field crawl on approach.
2. **Code owns the branch. Jev answers questions about measured state.** Jev does not decode bits and does not run A*.
3. **Choice is a closed list. `NONE` is a real option.** The model cannot invent a layer, a landmark, or a UUID.
4. **Noul is P(yes), not a disguised scale.** Thresholds live in Rust.
5. **Missing answers stay `None`. Never coerce to `0.0`.**
6. **Placing a tile and minting an identity are different risks.** Do not share one threshold. City-feed bug: two vague questions about one street produced two vague answers.
7. **CRC failure cannot become `Identified`**, no matter what Jev returns.
8. **`accepted` requires noul threshold AND a permitted Choice.** Hold / replan / abort / NONE never silently drive.
9. **`RouteRequest.margin = 0`.** Keep-out is already baked into the AABB.
10. **Live Jev key stays on the server.** A 502 is "we could not read," not an empty field.
11. **rustc 1.75.** Do not vendor `mrorigo/clearpath` (wants 1.85). Reimplement A* + string-pull here.
12. **Commit after every module.** Session timeout wipes local scratch. GitHub is source of truth.
13. **Do not sycophant. Precision over polish.**

---

## Source map

### On GitHub `main` (timeout-safe)

`src/lib.rs`, `src/jev.rs`, `src/capability.rs`, `src/lidar.rs`, `src/payload.rs`, `src/pipeline.rs`, `src/bin/ditherpath.rs`, `docs/JEV.md`, `docs/ARCHITECTURE.md`, `docs/HANDOFF.md`, `AGENTS.md`, `examples/route.json`, `examples/uuid.txt`, crate meta.

### Declared in `lib.rs` but missing from GitHub — crate does not compile

| path | local copy | job |
| --- | --- | --- |
| `src/codec.rs` | `/home/workdir/artifacts/ditherpath/src/codec.rs` (~244 lines) | fractal encode/decode, Bayer rank, lens permute, CRC8 |
| `src/gyst.rs` | `/home/workdir/artifacts/ditherpath/src/gyst.rs` (~267 lines) | UUIDv8 pack/unpack, FNV-1a-12, SHA-256 low-42 |
| `src/planner.rs` | not in artifacts this session | grid A* + string-pull + JSON |
| `src/texture.rs` | not in artifacts this session | PGM stack + SVG |

**P0:** push `codec.rs` and `gyst.rs` from artifacts immediately. Rebuild `planner.rs` and `texture.rs` from the contracts below. Then `cargo test`. No new features until a clean clone is green.

Local `jev.rs` may differ from GitHub. Prefer GitHub unless local is a verified superset.

---

## Module contracts

### `gyst`

Spec: https://gist.github.com/SoMaCoSF/582fdee49c30196663b759ca9a468a5e

```
High 64: type(12) | ns(12) | ts(24) | ver=8(4) | fractal(12)
Low  64: var=0b10(2) | prov(4) | signal(16) | random(42)
fractal = depth(4) | domain(4) | generation(4)
```

Defaults: `type_code = 0x610`, namespace = FNV-1a-12(`somacosf.com`), `Domain::Transport = 0xD`, low-42 = SHA-256 truncated. Demo key `pad-00a1`. If gist and code disagree, gist wins.

### `codec`

`MAGIC = 0xD3`. Header `[MAGIC, lens_id, payload_len, crc8(...)]` then payload bits. L0 is site `(0,0)` occupancy 1 (anchor). Layer L copies parents to `(x*2,y*2)` and writes payload only on `new_sites(L)`, Bayer-ranked, then `lens_permute`. Wrong lens must fail.

### `lidar`

Alphabet is occupancy, not RGB. `simulate` / `recover_occupancy` / `estimate_visible` / `decode_hits`. At 20 m and 0.2°, L0 dots need ~7 cm.

### `payload`

GYST bytes + keep-out cm + relative goal. Keep-out → inflated AABB. RouteRequest margin stays 0.

### `planner` — geometry only

Do not call Jev from this file. Grid A*, string-pull, JSON: `start`, `goal`, `obstacles: [{min,max}]`, `margin: 0`, plus polyline + cost. See `examples/route.json`.

### `texture`

PGM stack, coarse layer at the bottom (Dither3D convention). SVG of workspace, patch, polyline.

### `pipeline`

encode → simulate → decode → plan → artifacts. **Gap:** does not yet call `capability::run_all`. Thread Jev after CRC and after A*, not inside those modules.

```
cargo test
cargo run --release -- demo examples
cargo run --release -- uuid pad-00a1
```

---

## Jev capability

Clearpath computes a polyline. Zero judgment.

Same loop as `SoMaCoSF/jev-minesweeper-harness`:

```
observe(state) → jev.decide(questions) → policy(answers) → Action
```

`Capability` in `src/capability.rs`. Catalog:

| name | questions | closed actions | accepted when |
| --- | --- | --- | --- |
| `decode` | commit Noul, layer Choice {L1..L4,NONE}, quality Score | identified / anchored / unanchored | CRC + layer≠NONE + noul≥0.80 |
| `path` | accept Noul, maneuver Choice {skirt,hold,replan,abort}, exposure Score | those four | noul≥0.70 and skirt |
| `approach` | closer Noul, stance Choice {approach,hold,orbit,depart}, urgency Score | those four | closer≥0.70 and approach |
| `project` | emit Noul, channel Choice {occupancy,identity,keepout,NONE}, opacity Score | those four | emit≥0.70 and channel≠NONE |

`approach` and `project` are the questions geometry cannot ask.

`HeuristicJev` reads flags in the state string. Live Jev is the same trait at `https://api.typesafe.ai/v1/system-one` or `https://jevtypesafeai.com/api/v1/decide`, key `TYPESAFE_API_KEY`. Never ship the key to a browser.

City-feed verdicts (gist `d3944c12`): Identified = placed and CRC-ok; Anchored = placed, identity refused; Unanchored = keep the numbers.

A fifth surface is a new `impl Capability`. Do not edit the planner to add judgment.

```rust
let acts = ditherpath::run_all("visible=4 crc_ok=true clearance_ok=true");
```

---

## Upstream (read, do not vendor)

- https://github.com/somacosf/dither3d — surface-stable fractal Bayer
- https://github.com/mrorigo/clearpath — RouteRequest shape only
- https://gist.github.com/SoMaCoSF/582fdee49c30196663b759ca9a468a5e — GYST bits
- https://github.com/SoMaCoSF/jev-minesweeper-harness — World loop + mock client
- https://gist.github.com/SoMaCoSF/d3944c12d91b96eb41ac0d1a26ba5d2a — gate + 502≠empty

---

## Known gaps (ordered)

1. Crate does not compile from GitHub until codec/gyst/planner/texture are on main.
2. pipeline does not call `run_all`.
3. No live HTTP Jev client in Rust.
4. No CI.
5. Repo still public until the owner flips Settings.
6. README claimed PGM/SVG examples that may be absent.
7. Keep `run` vs `run_pipeline` names; do not collide.

---

## How to take a shift

1. Push or recreate the four missing modules. One file per commit if the push tool is flaky.
2. `cargo test` on rustc 1.75 until green.
3. Wire `run_all` into pipeline after CRC and after plan. Print verdicts in the CLI.
4. Optional `HttpJev` behind the trait, 502 as error not empty answers.
5. Stop. Do not add a fifth capability unless the owner names the stage.

If you are tempted to let the model pick a path, you have left the design.
