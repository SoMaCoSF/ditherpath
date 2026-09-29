# Agents

You are continuing **DitherPath** for SoMaCoSF.

1. Read `docs/PRD.md` (what/why) then `docs/HANDOFF.md` (where the files are) before writing code.
2. GitHub `main` is source of truth. Commit after every module. Timeouts wipe scratch.
3. Crate target is rustc **1.75**. Do not vendor `mrorigo/clearpath`.
4. `lib.rs` declares `codec`, `gyst`, `planner`, `texture` that may still be missing on GitHub. Fix compile before new features. That is Phase 0 in the PRD.
5. Jev is a capability (`src/capability.rs`), not a planner. Clearpath-class geometry judges nothing.
6. Rails: payload only on new octave sites; closed Choice + real NONE; Noul ≠ scale; missing ≠ 0; CRC fail ≠ Identified; margin=0 on RouteRequest; Jev key never in a client.
7. Repo should be **private**. This connector cannot flip visibility. Do not re-public it.
8. Do not invent a second GYST layout. Gist `582fdee49c30196663b759ca9a468a5e` wins.
9. Precision, no sycophancy, no extra capabilities the owner did not name.
10. Do not answer PRD §13 open questions by silently picking a default.
