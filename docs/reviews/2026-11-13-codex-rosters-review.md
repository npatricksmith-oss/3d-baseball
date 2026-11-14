# Independent review: game rosters (owner backlog A2)

Reviewed `328e51b..6264c50` at head `6264c50babc58f7196bebe3a1e8b9b4e7840bb48`.

No P1/P2 or functional defect found. The four requested game digests are unchanged, the release workspace tests and C smoke test pass, and independent generator and event-log probes agree with the implementation. Two P3 findings concern missing permanent regression coverage. Full FlatBuffers regeneration could not run because the sandbox cannot start the existing Podman runtime; generated field definitions were checked directly instead.

## Findings

### P3 — No independent roster fixture protects the specified draw contract

**Location:** `sim/crates/bb_game/src/roster.rs:280` (also the distribution test at line 311).

**What is wrong:** Comparing `generate(7)` with another call to `generate(7)` checks repeatability, but does not pin the specified stream keys, retry sub-indices, draw order, shuffle direction, or name-pool ordering. A deterministic implementation with a different purpose or draw sequence could still satisfy the structural and distribution assertions. The integration and ABI tests also obtain their expected identities from the same generator. Those are useful transport tests, but cannot independently establish generator correctness.

**Verification:** Read all new roster tests and independently implemented Philox, stream construction, rejection, and the §2.4 shuffle in Python, using the checked-in pool data. All 18,000 players over seeds 0–999 matched, including retries (up to six rejected name attempts and three rejected number attempts). This establishes correctness for those inputs, but the independent expectations are not checked into the test suite. For example, seed 7's away leadoff player is CF Yuki Kobayashi, number 28, player id 8; its pitcher is Daniel Morgan, number 6, batting ninth.

**Suggested fix:** Add literal, independently derived full-roster fixtures for several seeds, including seeds exercising both rejection paths. Keep the structural and distribution tests as complementary checks. Assert identities, ids, numbers, hands, positions, and batting orders without computing expectations through `Rosters::generate`.

### P3 — Batting tests do not independently check who scored each run

**Location:** `sim/crates/bb_stats/tests/accounting.rs:69`.

**What is wrong:** The new test checks player totals against team totals, plus plate-appearance order and category sums. It does not assert any individual player's runs against `RunScored.runner`. Crediting a run to a different teammate would preserve every current run assertion. The four selected games also do not explicitly require walk-off, extra-inning, or rosterless-log coverage.

**Verification:** Inspected the assertions and ran an additional audit over seeds 0–199. That audit counted events by the batter/runner id's team and slot, independently of `box_score`'s current-half tracking, and compared all seven player counters. All 16,060 PAs and 2,220 runs matched, including 13 extra-inning games and 12 walk-offs. Replacing seed 7's header with a header lacking config/rosters also preserved all team statistics and produced empty batting vectors. The implementation passes these stronger checks; the gap is permanent regression coverage.

**Suggested fix:** Add a small log fixture with distinct named runners scoring across a half-inning boundary and at a walk-off, asserting specific player lines. Add an explicit rosterless-header case and/or retain an independent per-id event audit over representative real games.

## Review assessment

1. **Replay and invariants.** The requested digest example matches at both commits:

   | Seed | Digest at both commits |
   | --- | --- |
   | 1 | `11b47dd84bbda5dcca8e59badca0a50292187647cbf985e1dfe16c017de9b224` |
   | 2 | `63ef32e7702b1f4497366af8ac35738c2c7eaab32231ca394dfd37c09490c4b2` |
   | 3 | `9d05f16745cf88752a18398a9a820dd6923ed4ed77f1ec5e970692778dffa52a` |
   | 42 | `7ca0f0f3b0fd2f8e2345a01addb30364459f18974101325ab2a8605bcbdd4540` |

   `rosters.lineups()` still yields away batting ids 1–9 and home ids `0x101`–`0x109`, exactly the old fixture. No physical or decision path reads player identity or roster handedness. The roster read inside `step_pitch` formats the lab label; the other consumers serialize or describe identity. As an additional check, changing every roster's names, numbers, and hands after initialization, with recording enabled, left the same four game digests unchanged. Header bytes remain outside the digest chain. `Purpose::RosterGeneration = 12` is appended; values 1–11 are unchanged. Its purpose occupies the low 16 bits of counter word 1, separate from the agent id, so matching seed/play/agent ids cannot alias an existing-purpose stream. No new outcome rolls, hash iteration, or direct transcendental calls were introduced. `bb_agents` still depends only on math, physics, and observation crates; `bb_stats` has only `bb_schema` as a production dependency.

2. **Generation and §2.4.** Fielding ids, play index 0, purpose 12, even name-attempt sub-indices, odd number-attempt sub-indices, three name draws, and one number draw agree with the spec. Families are rejected game-wide, givens and numbers team-wide. Player ids and right-handed slots agree. The shuffle starts with positions 2–9, runs from 7 down to 1, uses the manager id and sub-index 0, and puts the pitcher ninth. The implementation computes this independent manager stream before the names rather than after them; this has no observable draw-order effect because the streams are disjoint. No substantive spec/code disagreement found.

   This is the correct Fisher–Yates algorithm. It is unbiased with ideal uniform bounded indices. The prescribed 52-bit `floor(u*n)` mapping has the minute quantization bias inherent in that spec for non-power-of-two bounds; this is not an algorithmic shuffle error. Both uniform endpoints are excluded. For every used bound, including 99, the smallest/largest input maps to 0/`n-1`; the clamp is harmless and current pools are nonempty.

   The pools cannot exhaust all acceptable combinations: even at the final player, the English pool retains at least 40/48 given names and 48/65 families. Under the uniform-draw model, that pool alone supplies an acceptance probability of at least about 0.3815; number acceptance is at least 91/99. No stuck loop occurred in the probes. This is not a formal all-seed termination proof: retries are unbounded, and `2*a` would overflow after `2^31` unsuccessful attempts (wrapping/repeating streams in release, panicking with overflow checks). The test comment at `roster.rs:302` overstates its proof when it says the loop “always ends”; pool capacity establishes available choices, not a bound on a deterministic rejection sequence. No practical failing seed was found.

3. **Agent ids and displayed names.** Exhaustively checked all 65,536 ids against the independent valid ranges: exactly 36 resolve, and 65,500 reject, including officials, managers, invalid slots, and sides ≥2. Both player ids resolve to the same identity. Eight watched games checked 1,857 pitch batter ids and lab labels, and 609 PA-ending name prefixes against separately counted batting order. The batter is captured before stepping the pitch; `describe` reads `PlateAppearanceEnded.batter`. Half-inning changes did not shift the named player.

4. **C ABI and Unreal.** Independently compiled C and Rust layout probes agree exactly on the reviewed x86-64 target. `BbPlayer`: size 80, alignment 4; player id offset 0, batting/fielding ids 4/6, six byte fields 8–13, explicit pad 14, given/family buffers 16/48. `BbPlayInfo`: size 188, alignment 4; summary offset 36, call count 132, calls 136, appended batter 184, explicit pad 186. New functions reject null inputs and invalid indices; non-null allocation/lifetime/alignment validity remains the caller's documented obligation. Name and summary arrays start zeroed and copy at most capacity minus one, retaining NUL termination. No new undefined behavior found for valid callers.

   The Unreal source includes this shared header, uses normal struct storage/copies, and displays `summary` directly at `client/Source/BaseballClient/BbDirector.cpp:1606`; it does not parse the old caption prefix. The Rust tests now split the name off before checking `Walk`/`Home run`. A seed-3 Walker probe included six Walker PAs that were not walks. Appending the field is source-compatible with a rebuilt client, not binary-compatible with an old 184-byte `BbPlayInfo` allocation. `client/run.sh` rebuilds both library and Unreal client, which addresses that requirement in the normal launch path. Unreal was not built during this review.

5. **Caption capacity.** The longest family names are nine bytes (`Dominguez`, `Fernandez`, `Gutierrez`, `Hernandez`, `Rodriguez`, `Kobayashi`, `Matsumoto`, `Takahashi`). A conservative bound, even combining clauses that need not be jointly reachable, is 89 bytes: 11 for family plus `: `, 24 for `Inside-the-park home run`, 15 for ` to the pitcher`, 25 for `, runner out on the bases`, and 14 for the run suffix. The longer multi-play/fielder's-choice alternatives bound at 88. Thus every legal current-game caption fits the 95-byte payload with room for NUL. The calculation assumes baseball's maximum four scoring runners, not arbitrary malformed event slices with unlimited `RunScored` entries.

6. **Box score and tests.** Current-half tracking is updated from log `HalfInningStarted` events, after preceding-half scoring events; individual run credit uses the runner's batting id. The independent 200-game audit and rosterless-header probe passed. The legal-roster tests, batting-order PA inequalities, and independently counted integration batting order are meaningful expectations. Header/ABI comparisons against regenerated rosters are meaningful serialization checks, but do not protect exact generator draws. Same-seed equality alone is a weak oracle; pool histograms test gross bias, not the complete shuffle distribution. There is also no committed exhaustive caption-bound or explicit Walker non-walk regression test. These are coverage observations, not evidence of incorrect current output.

7. **Schema and names.** New `RosterEntry` ids 9–12 and `PlayTimeline.batter` id 13 are appended without changing old ids or defaults. Generated vtable offsets, getters, argument fields, builders, and verifiers agree with those additions. Shared schema version 0.2 is the additive minor bump required by §8; old absent fields have defaults. Full regeneration remains unverified as described below. The pools contain ASCII common given/family names with no duplicate family across pools; manual inspection found no inappropriate or clearly misfiled entry. Spanish names without accents and Korean given-name hyphens follow the stated interim ASCII policy.

## Verification

Commands ran from the repository root unless noted. Build/probe executables stayed under this checkout's `sim/target`; temporary archives, probe sources, logs, and Zig caches used `/tmp`. No source, test, spec, or TASKS file was edited by this review.

- `cat CLAUDE.md`; `git log --stat 328e51b..6264c50`; `git diff 328e51b..6264c50`; focused `cat`, `sed`, `nl`, and `rg` reads of the changed files, RNG, log container, crate manifests, schema version policy, and Unreal consumers: completed.
- `git archive 328e51b | tar -x -C /tmp/rosters-review-base`: created an isolated baseline without switching branches, stashing, or changing commits.
- In the archived baseline's `sim`, `CARGO_TARGET_DIR='<checkout>/sim/target' cargo run --release -q -p bb_game --example digest`; in this checkout's `sim`, `cargo run --release -q -p bb_game --example digest`; `diff -u /tmp/rosters-review-base-digests.txt /tmp/rosters-review-head-digests.txt`: all four digests identical.
- Initial baseline linking failed because Zig attempted to write its default read-only cache. Subsequent builds exported `PATH=$HOME/.cargo/bin:$PATH`, `ZIG_GLOBAL_CACHE_DIR=/tmp/rosters-review-zig-global`, and `ZIG_LOCAL_CACHE_DIR=/tmp/rosters-review-zig-local`. Sharing the target directory between archives then exposed stale Cargo artifacts in an initial smoke attempt. Ran `cargo clean --release -p bb_game -p bb_schema -p bb_math -p bb_stats -p bb_ffi` and repeated the head digest, workspace suite, and smoke test. Results below are from the rebuilt artifacts.
- In `sim`, `cargo test --workspace --release > /tmp/rosters-review-workspace-tests.log 2>&1`: passed, 167 tests across 41 unit/integration/doc-test targets, zero failures or ignored tests.
- In `sim`, `bash crates/bb_ffi/c/run_smoke.sh`: passed. Seed 3: 238 pitches, 54 out calls, final score 5–4; both rosters and both agent-id lookups passed.
- `python3 /tmp/rosters-review-build-probe.py`; `sim/target/rosters-review-probe`: passed exhaustive ids, independent player accounting over 200 games, rosterless-header compatibility, watched-game name/order/timeline checks, and altered-identity digest checks. Output: `/tmp/rosters-review-probe.log`.
- `sim/target/rosters-review-probe rosters > /tmp/rosters-review-rosters.tsv`; `python3 /tmp/rosters-review-reference.py`: independent Python Philox/§2.4 implementation matched all 18,000 players across 1,000 seeds. Pool contents were read as data; RNG and generation logic were implemented independently.
- Compiled `/tmp/rosters-review-layout.rs` with `rustc --edition=2024`, the configured Zig linker, `-L dependency=sim/target/release/deps`, and `--extern bb_ffi=sim/target/release/libbb_ffi.rlib`; compiled `/tmp/rosters-review-layout.c` with `$HOME/.local/bin/zigcc -I sim/crates/bb_ffi/include`. Ran both executables under `sim/target` and diffed their output: layouts identical.
- Python pool/phrase-length and uniform-endpoint calculations: longest family nine bytes; conservative caption bound 89 bytes; all used index endpoints valid. Direct schema consistency assertions checked new ids, vtable offsets, getters, argument fields, builders, and verifiers: passed.
- `git archive 6264c50 sim/schemas sim/crates/bb_schema/src/generated | tar -x -C /tmp/rosters-review-schema`; `bash /tmp/rosters-review-schema/sim/schemas/gen.sh`: failed with `Error: failed to get the Podman version`. `toolbox run -c fedora-toolbox-45 flatc --version` failed likewise. `podman --version` reported a stale pause-process pid and inability to change `/run/user/1000/libpod` on the read-only filesystem. No installed host `flatc` was available. The regeneration attempt touched only the temporary archive; committed generated code was untouched.
- In `sim`, `cargo run --release -q -p bb_cli -- game --seed 7`: passed; printed 18 named batting lines and totals, final score 5–2, 79 PA, 223 pitches. Output: `/tmp/rosters-review-cli.log`.
- `git diff --check 328e51b..6264c50` and final `git diff --check`: passed. HEAD stayed at `6264c50`. `CLAUDE.md` and `TASKS.md` acquired external edits during the review and were left untouched.

## Resolution

Resolved 2026-11-13. No P1 or P2; both P3s fixed in the commit that adds this file.

- **P3, no independent roster fixture: fixed.** `roster::tests::seed_7_gives_these_players` pins both of seed 7's nines as literals (batting order, position, number, given and family name, hands, player ids). The values are the ones this review's independent Python Philox and §2.4 generator reproduced, so they do not restate the Rust code. Seed 7 exercises the rejection of a teammate's given name twice. A fixture that also exercises a rejected family name and a rejected number was not added; the review's 1,000-seed comparison covered those paths, and the legal-roster test checks their result on 500 seeds.
- **P3, runs not checked per player: fixed.** `batting_lines_name_the_roster_and_follow_the_order` now runs seeds 0–39 (it asserts they include a walk-off and an extra-inning game) and checks every player's runs, plate appearances and home runs against the sim's rules events counted by the id on each event, independently of the box score's half-inning tracking. **Deferred:** a committed rosterless-log case (the review's probe passed; `bb_stats` would need a hand-built header, which fits better with the compatibility corpus of data-formats §8).
- The assessment's note that the pool test's comment overstated termination: the comment now says the pools leave acceptable names to draw, not that the loop is bounded. Retries stay unbounded, as the spec describes them.
- Schema regeneration could not be re-run in the review's sandbox; it was run when the change was made (`sim/schemas/gen.sh`, flatc 25.12.19) and touched only the two generated files for the edited tables.
- Not done here: the Unreal client was not rebuilt against the new header (`client/run.sh` rebuilds both).
