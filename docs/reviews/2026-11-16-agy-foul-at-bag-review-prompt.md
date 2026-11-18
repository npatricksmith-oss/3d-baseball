# Review request: fair or foul as the ball passes first or third

You are an independent reviewer for a deterministic baseball simulation (Rust, `sim/`) with an Unreal client (`client/`). Read `CLAUDE.md` first: its "Invariants" section is binding. **Paths:** the repository root is `/home/natex/Projects/3D Baseball` (it contains a space). `read_file` needs the absolute path; in shell commands quote it or `cd` into it first.

## What to review

Commits on `main`: `327eed1` (sim) and `7188407` (client), range `30634cf..7188407`. See them with `git show 327eed1` and `git show 7188407`.

1. **`sim/crates/bb_game/src/live.rs`: `passed_base` and its call in `ball_facts` (highest risk).** The sim now sends the rules engine `Input::BallPassedBase { t, fair }` when a batted ball that has touched the ground crosses the line through first or third base square to that base's foul line, on that base's side of the field. Before, such a ball was judged only where it touched beyond the base or came to rest. Check against `docs/rules-spec.md` §2 (the `BallPassedBase` row) and §5.2, and the Official Baseball Rules definitions "Fair Ball" and "Foul Ball":
   - Is the geometry right: the crossing line, the side restriction (`p.x * side < 0`), the base distance `BASE_L` 27.432 m, the bisection, the airborne versus rolling test (`p.z > r_ball + 0.01`; fair if `line >= -r_ball` in the air, `line >= 0` on the ground)?
   - Can it fire wrongly: a ball that has not bounced, a ball crossing near second base, a ball behind home, a deflected or thrown ball, a ball crossing inside one tick together with a bounce or a settle (the facts are sorted by time before they go out), a crossing at `t_end`, `floor` (the last fact time) later than the crossing?
   - The `pending` check reads `self.st.play` before this tick's facts are emitted. Can a fact emitted earlier in the same tick decide the ball, and does it matter?
   - Determinism (Invariant 5): no new transcendental calls outside `bb_math::m`, no RNG, no hash iteration.
2. **`sim/crates/bb_game/src/describe.rs`:** the caption says "Foul" whenever the rules emitted `FairFoulDecided { fair: false }` and the plate appearance did not end. Is there a case where that is wrong (for example a caught foul fly with the plate appearance continuing, or a foul with two strikes)?
3. **`sim/crates/bb_game/src/timeline.rs` and `sim/crates/bb_ffi`:** dead-ball kind 4 (a foul ball out of play, presentation only; the event log still records it as 1). Does anything else read the marker's `base` as the event log's code?
4. **`client/Source/BaseballClient/BbDirector.{h,cpp}` (not compiled yet):** after a foul out of play the client skips the walk back (`bFoulOut`). The ball is drawn at 0.075 m and its shadow uses a code-built translucent material on the engine plane (`SoftDiscMaterial`). Look for compile errors by reading against the engine headers under `~/UnrealEngine/5.8.4/Engine/Source/Runtime/Engine/Public/Materials/`, and for logic errors (the replay path, `HoldAfter`).

## How to verify

Run tests and probes rather than reasoning alone:

```bash
export PATH=$HOME/.cargo/bin:$PATH CARGO_BUILD_JOBS=2
cd sim
cargo test --release -p bb_game --lib -- a_ball_rolling_past_third describe
cargo run --release -q -p bb_game --example foul_caption_probe -- 3647 15 20 62
cargo test --release -p bb_ffi
```

You may write throwaway probes under `sim/crates/bb_game/examples/` named `review_*.rs`, and must delete them before you finish. Keep cargo's target directory as it is (do not use `/tmp`). Use at most 2 build jobs: the laptop overheats.

## Rules for you

- **Edit no source, test, spec, TASKS or observations file.** Write only `docs/reviews/2026-11-16-agy-hydrogen-foul-at-bag-review.md`.
- In it: a short summary; findings ranked P1 (wrong game outcome or broken invariant), P2 (wrong in some cases, or a gap a test should cover), P3 (minor), each with file and line, a concrete failing case, and how you checked it; a **Verification** section listing what you ran and what it printed; and a final section `## Resolution` containing only the word "Open."
- Do not commit, push, or change branches.
