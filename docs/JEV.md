# Jev in DitherPath

Jev (TypeSafe System One) is the **judgment layer**. It does not decode bits
and it does not run A*. Those stay deterministic. Jev answers typed questions
about state the pipeline already measured.

Same contract as `SoMaCoSF/jev-minesweeper-harness` and the Austin gate
(gist `d3944c12d91b96eb41ac0d1a26ba5d2a`):

- One request, many independent questions.
- Choice is a **closed list**. `NONE` is a real option. The model cannot invent a layer or a landmark.
- Noul is P(yes). Thresholds live in code.
- Score is an ordered rubric, not a yes/no.
- Missing answers stay `null`. Never coerce to `0`.
- Placing a tile and minting an identity are different risks and do not share a threshold.

## Decode questions

| id | type | branch |
| --- | --- | --- |
| `commit_payload` | Noul | accept the recovered bytes as a GYST mark |
| `layer` | Choice `{L1,L2,L3,L4,NONE}` | which octave is actually resolved |
| `quality` | Score 0–3 | lattice cleanliness |

Verdicts (city feed, reused):

| verdict | meaning |
| --- | --- |
| `Identified` | CRC + placed layer + commit noul ≥ 0.80 → accept UUID |
| `Anchored` | lattice placed, identity refused → keep-out only |
| `Unanchored` | keep the observation and the numbers |

A CRC failure cannot become `Identified` no matter what Jev says. Code owns that rail.

## Path questions

| id | type | branch |
| --- | --- | --- |
| `accept_route` | Noul | is this polyline safe given keep-out |
| `maneuver` | Choice `{skirt,hold,replan,abort}` | next closed action |
| `exposure` | Score 0–3 | how tight the corridor is |

`accepted` in code = `accept_noul ≥ 0.70` **and** maneuver is `skirt`.
Hold / replan / abort never silently drive.

## Wiring

```
scan → occupancy → CRC
                 ↓
        Jev decode questions
                 ↓
     Identified | Anchored | Unanchored
                 ↓
        plan polyline (A*)
                 ↓
        Jev path questions
                 ↓
     skirt | hold | replan | abort
```

Offline: `HeuristicJev` reads flags already in the state string
(`crc_ok=true`, `visible=4`, `clearance_ok=true`). Live Jev is the same
`Jev` trait pointed at `POST /v1/systemone` or
`https://jevtypesafeai.com/api/v1/decide` with `TYPESAFE_API_KEY`.
The key stays on the server. A 502 is "we could not read," not an empty field.
