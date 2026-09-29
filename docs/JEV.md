# Jev as a capability

Clearpath computes a polyline. That is all it does. Cost, inflate, A*, string-pull.
Zero judgment. DitherPath keeps that kernel and puts Jev *around* it, not inside it.

Jev is a **capability any stage can register**: a named question set plus a policy
that maps Noul/Choice/Score onto an `Action`. The loop is the harness World loop:

```
observe(state) → jev.decide(questions) → policy(answers) → Action
```

`src/capability.rs` is the generic surface. Decode and path are two instances.
Approach and project are the ones clearpath cannot express.

## Catalog

| capability | when | closed actions |
| --- | --- | --- |
| `decode` | after occupancy + CRC | identified / anchored / unanchored |
| `path` | after A* polyline | skirt / hold / replan / abort |
| `approach` | before the next scan | approach / hold / orbit / depart |
| `project` | onto vision | occupancy / identity / keepout / NONE |

Adding a fifth surface is a new `Capability` impl. Do not edit the planner.

## Rails that stay in code

- Geometry (inflate, A*, snap) never calls Jev.
- CRC failure cannot become `identified`.
- `accepted` requires both the noul threshold *and* a permitted Choice.
- Live key stays on the server. 502 ≠ empty field.

## Call

```rust
let acts = ditherpath::run_all("visible=4 crc_ok=true clearance_ok=true");
// acts["decode"], acts["path"], acts["approach"], acts["project"]
```
