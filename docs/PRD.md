# DitherPath — Product Requirements Document

| | |
| --- | --- |
| Product | DitherPath |
| Version | 0.1.0-draft |
| Date | 2026-09-29 |
| Owner | SoMaCoSF |
| Repo | https://github.com/SoMaCoSF/ditherpath |
| Status | Prototype crate; compile-from-clone is **not** green (see §14) |
| Companion docs | `docs/HANDOFF.md`, `AGENTS.md`, `docs/JEV.md`, `docs/ARCHITECTURE.md` |

This PRD is the product contract. HANDOFF is the engineering shift brief. If they disagree, this file wins on *what* and *why*; HANDOFF wins on *where the files are this week*.

---

## 1. Problem

Surfaces, pads, and keep-outs in the physical world are either dumb (paint, tape, cones) or they require a network and a database to mean anything. A drone or ground vehicle that sees a mark cannot, from the mark alone, answer: what is this; how far to stay away; where the next goal is; whether the read is good enough to act on; whether to close range, hold, or leave; whether to paint that knowledge into vision.

Existing pieces solve fragments. Dither3D is surface-stable Bayer with no payload. clearpath is inflate+A*+string-pull with zero judgment (and rustc 1.85). GYST is portable identity with no physical carrier. Jev is typed Noul/Choice/Score and is neither a decoder nor a planner.

The product is the **joint**: a mark that *is* the record, lidar-readable on approach, judged by Jev, routed by deterministic geometry, projectable from the same generator.

## 2. Thesis

**Encode identity and constraint into a surface-stable dither field. Read it with occupancy. Judge the read. Plan around what you actually recovered. Project the same field.**

Making context portable and cheap is the north star. Identity is infrastructure. The mark should decode without a lookup table.

## 3. Goals

1. One Rust crate, rustc 1.75, that a second agent can clone and extend from the docs.
2. Encode a GYST UUIDv8 + keep-out + goal into a fractal Bayer occupancy pyramid.
3. Recover that payload from a sparse intensity scan (simulated lidar now; real returns later).
4. Progressive readout: coarse octaves carry header; fine octaves carry the rest.
5. Jev as a **capability**, not a special case of pathing.
6. Deterministic pathing that emits clearpath-shaped `RouteRequest` JSON with `margin = 0`.
7. Same generator feeds a projection / vision overlay. RGB is the twin channel, not the lidar alphabet.
8. Honest verdicts: identified / anchored / unanchored. Refuse identity rather than mint garbage.

## 4. Non-goals (v0.1)

Vendoring Dither3D or clearpath. Flight stack / ROS2. On-device neural detectors. Jev inventing waypoints or UUIDs. Multi-vehicle mesh or Locus on the mark. rustc 1.85-only features. A public marketing site.

## 5. Users and jobs

| actor | job | success |
| --- | --- | --- |
| Protocol owner | ship a lidar-honest primitive | tests name the rails; docs carry the next agent |
| Next coding agent | take a shift | PRD + HANDOFF + green `cargo test` |
| Planner process | consume RouteRequest JSON | polyline skirts keep-out; margin 0 |
| Vision process | paint recovered field | channel is a closed Choice; NONE is valid |
| Human reviewer | see why a mark was or was not identified | verdict + noul + layer printed |

No end-user GUI in v0.1. The CLI demo is the product surface.

## 6. Product loop

GYST UUIDv8 (type 0x610, domain TRANSPORT) + keep-out + goal → occupancy pyramid (payload only on NEW sites) → surface → intensity scan → lattice snap → magic/lens/CRC → Jev decode (Identified/Anchored/Unanchored) → A*+string-pull (geometry only, margin 0) → Jev path/approach/project → artifacts.

## 7. Functional requirements

### 7.1 Identity `gyst` — P0

Pack/unpack RFC 9562 v8 per gist `582fdee49c30196663b759ca9a468a5e`. High 64: type(12)|ns(12)|ts(24)|ver=8(4)|fractal(12). Low 64: var=0b10(2)|prov(4)|signal(16)|random(42). Defaults: type `0x610`, domain `0xD`, ns = FNV-1a-12(`somacosf.com`), low-42 = SHA-256 truncated. Hyphenated round-trip. Gist wins if code disagrees.

### 7.2 Codec `codec` — P0

`MAGIC = 0xD3`. Header `[magic, lens_id, payload_len, crc8]`. L0 is `(0,0)` occupancy 1, not payload. Layer L parent-copies to `(x*2,y*2)` and writes bits only on `new_sites(L)`, Bayer-ranked then lens-permuted. Wrong lens fails. Right lens + CRC recovers exact bytes. Cap payload length (current impl 255).

### 7.3 Lidar `lidar` — P0

Alphabet is occupancy, not RGB. `simulate` / `recover_occupancy` / `estimate_visible` / `decode_hits`. Noisy round-trip at noise 0.03 and spacing tile/16 at L4. Physical bound: ~7 cm L0 at 20 m / 0.2°. Real PCD ingest is P2.

### 7.4 Payload `payload` — P0

Field bytes = GYST 16 + keep-out + relative goal. Keep-out becomes an already-inflated AABB. Goal is workspace coordinates.

### 7.5 Planner `planner` — P0

Deterministic A* + string-pull. **Does not call Jev.** JSON: workspace, start, goal, margin 0, obstacles[], route.{cost,polyline}. Obstacle `id` is the UUID only when Identified. Optional clearpath handoff via JSON when rustc ≥ 1.85 exists — no compile dep.

### 7.6 Jev `jev` + `capability` — P0

Primitives: Noul, Choice, Score only. Trait: name / questions / policy. Catalog: decode, path, approach, project.

- Identified = CRC AND layer≠NONE AND commit noul ≥ 0.80
- Path accepted = accept noul ≥ 0.70 AND maneuver = skirt
- Missing answers stay None
- HeuristicJev offline; live client same trait; key from env; 502 ≠ empty field
- Fifth surface = new impl, not a planner edit

Decode questions: commit_payload Noul; layer Choice {L1,L2,L3,L4,NONE}; quality Score 0–3.
Path: accept_route Noul; maneuver Choice {skirt,hold,replan,abort}; exposure Score.
Approach: closer Noul; stance Choice {approach,hold,orbit,depart}; urgency Score.
Project: emit Noul; channel Choice {occupancy,identity,keepout,NONE}; opacity Score.

### 7.7 Pipeline / CLI

`ditherpath demo <dir>` encode→scan→decode→plan→write. Must call `run_all` after CRC and after plan (not wired yet). `ditherpath uuid <key>` prints hyphenated GYST.

### 7.8 Projection — P1

Same generator emits a visual stack. Channel is the project Choice. Dither3D shaders are reference, not a dependency.

## 8. Non-functional

rustc 1.75; `sha2` only extra crate unless a later PRD names another. `cargo test` is the gate. Deterministic demo. Docs in repo. Commit per module. Repo intended private. License boundary in NOTICE. Print refusals.

## 9. Data contracts

RouteRequest: see `examples/route.json`. `margin` always 0.0.

Heuristic state flags: `visible=` `crc_ok=` `hits=` `snr_high=` `clearance_ok=` `blocked=` `cost=`.

Action: `{ kind, target, accepted, notes }`. `accepted` is the code rail, not raw noul.

## 10. Success metrics (clean clone)

1. `cargo test` green on 1.75.
2. `demo` writes uuid.txt + route.json whose polyline misses the keep-out AABB.
3. Wrong lens fails. Right lens + noise 0.03 recovers payload.
4. CRC-fail fixture cannot print Identified.
5. `run_all` returns four named actions.
6. HANDOFF + this PRD match the tree that exists.

Not metrics: stars, slogans, token poetry.

## 11. Phases

| phase | ships | exit |
| --- | --- | --- |
| 0 compile | codec, gyst, planner, texture on main | cargo test on clean clone |
| 1 joint demo | pipeline calls run_all; CLI prints verdicts | Identified on happy path |
| 2 live Jev | HttpJev + 502 handling | one recorded live turn with token count |
| 3 real scan | PCD / intensity fixture | fixture in examples/ |
| 4 projection | visual stack + project cap wired | NONE is a tested outcome |
| 5 vehicle | external process consumes JSON | out of this crate's runtime |

Do not start phase N+1 while N is red.

## 12. Risks

Payload on inherited sites (field crawls). Shared place/identify threshold (undedupable entities). Jev inside A* (nondeterministic routes). rustc 1.85 split. Session timeout. Public repo vs patent-adjacent encoding. Jev outage read as empty world. L0 smaller than lidar pitch.

## 13. Open questions (owner, not agent)

1. Freeze demo UUID timestamps for bit-stable uuid.txt, or keep wall-clock?
2. Keep-out units: cm on the wire vs meters in JSON — pick one and test it.
3. Is lens_id a published pad-class constant, or derived from the UUID?
4. Single-tile v0.1 only, or a strip with shared L0 beacons?
5. When Anchored, omit obstacle id or mint a cell-only GYST (type 0x340 style)?

Agents do not answer these by inventing a default in code without writing the decision here.

## 14. Implementation gap (2026-09-29)

On GitHub: jev, capability, lidar, payload, pipeline, CLI, this docs set.
Missing on main: codec, gyst, planner, texture.
Local copies of codec/gyst: `/home/workdir/artifacts/ditherpath/src/`.
Pipeline does not call `run_all`.
Phase 0 is the next shift, not a fifth capability.

## 15. PR acceptance

Names a phase from §11. Does not put Jev in planner.rs. Does not write payload onto inherited sites. Adds or updates a test for any rail it touches. Updates HANDOFF source map if files moved. Leaves margin at 0. No API key in examples except the env var name.
