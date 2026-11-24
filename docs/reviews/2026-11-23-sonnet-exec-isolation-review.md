# Review: execution-isolation scan test (work/exec-isolation, dd2b098)

Reviewer: rev-exec-iso (Sonnet, medium). Scope: `sim/crates/bb_agents/tests/exec_isolation.rs`, agent-models §1.1 and the test-list line near 985.

## Summary

The scan holds against every bypass in the brief except two undocumented ones (P2, P3). The comment and string stripper is correct on everything I tried. The self-tests are independent of the scanner. The `motor.rs` impl rule has one hole (type alias) and one false-positive class (`impl Trait`). The spec text is accurate, with one gap noted in P3-3.

## Findings

### P2-1. `#[cfg_attr(.., path = "motor.rs")]` is missed (undocumented)
Rule 3 looks for `path` directly preceded by `[` and then `#`/`!`. In `#[cfg_attr(all(), path = "motor.rs")] mod sneaky;` `path` follows `,`. The scanner returned no violation. This compiles in real Rust. Decision code could then write `sneaky::ExecutedSwing` without ever naming `motor`. It does need a deliberate edit, but the header claims `#[path]` is refused. Fix: flag any ident `path` followed by `=` inside an attribute, or flag `cfg_attr` outright.

### P2-2. Type alias in `motor.rs` lets an `impl` on a decision type through (undocumented)
`scan_motor` counts `type X = ..` names as local. `type X = IntendedRelease; impl X { fn run(&self) -> ExecutedRelease {..} }` returned no violation. That is the very leak the impl rule exists to stop, because `pitcher.rs` could call `.run()`. Fix: collect only `struct`/`enum`/`union` names as locals, or reject `type` aliases whose right-hand side is not local.

### P3-1. `motor.rs` `impl` rule false-positives on `impl Trait`
The rule fires on every `impl` token, including argument and return position (`fn execute_swing(r: &mut impl Rng)`, `-> impl Iterator`). Both probes were flagged. Current `motor.rs` has only three inherent impls, so the test passes today, but the first `impl Rng` argument added there will fail the test with a misleading "impl for ..." message. Related: `impl<'a> Foo for &'a Local` is flagged (the lifetime ident `a` is taken as the self type), and a tuple impl `impl Tr for (Local, Decision)` is not flagged (the first ident is taken). Fix: only treat `impl` as an item when the previous token is `;`, `}`, `{`, `]` or start of file, and skip lifetimes when picking the self type.

### P3-2. `include` can be renamed (undocumented, contrived)
`use core::include as inc; inc!("motor.rs")` is not flagged: rule 3 needs `include` directly followed by `!`. Other aliases (`use core::include as inc`) need a `use` line that mentions `include`. Cheap fix: flag the ident `include` anywhere. Also unscanned: `build.rs` and `OUT_DIR` output, but those reach the crate only through `include!`, which is flagged.

### P3-3. Spec wording
agent-models §1.1 says the scan "fails on ... `#[path]` or `include!`"; that is true only for the plain forms (P2-1, P3-2). It does say what enforces isolation and states the scan covers `bb_agents/src` only and that `bb_game`'s hand-off is review-checked, so it is otherwise true. The line near 985 is accurate. §1.1 does not mention the `motor.rs` rule (no trait, no macro, impl only on local types). Add a clause once P2-2 is settled.

## Answers to the brief

1. Bypass attempts (planted via a scratch copy of the test file, since removed):
   - path across lines `crate::\n motor::\n ExecutedSwing`: **caught** (both tokens, lines 2 and 3).
   - `use crate::{motor::ExecutedSwing}`, nested groups, `use crate::{pitcher, motor}`: **caught** (self-tests; `motor` is an ident wherever it sits).
   - `use crate::motor as m` then `m::X`: **caught** at the `use`.
   - `self::`, `super::super::motor`: **caught**.
   - `pub use` re-export in `lib.rs`: **caught**. `pub(crate) mod motor;` in `lib.rs`: allowed, correctly.
   - type alias in `motor.rs` named by decision code: **caught**, since reaching it needs `motor` or a `lib.rs` re-export.
   - `r#motor`: **caught** (strip removes `r#`).
   - macro taking `motor` as an argument: **caught**. Macro defined in `motor.rs`: **caught** by the motor rule. Name-building macros: only `paste`/`concat_idents`, documented.
   - plain `#[path]`: **caught**. `cfg_attr` path: **missed, undocumented** (P2-1). Renamed `include`: **missed, undocumented** (P3-2).
   - subdirectories of `src/`: **caught**, because `rs_files` recurses and sorts. `src/foo/lib.rs` does not get the `lib.rs` exemption.
2. Stripping: nested block comments, raw strings with hashes (`r##"a"# b"##`), byte strings, `b'"'`, `'"'`, `'\''`, `'\u{1F600}'`, non-ASCII chars, lifetimes (`<'a>`, `&'a str`, `&'static str`), and a backslash-newline continuation in a string all behaved correctly. A planted `ExecutedSwing` after each was still flagged, and mentions inside comments, strings, raw strings and `#[doc = ".."]` were ignored. No false negatives found. One false-positive class: any identifier or field named exactly `motor` (`let motor = 1`) is flagged; acceptable and conservative.
3. Self-tests: a flag-nothing scanner fails `scanner_flags_each_planted_violation` and the line/rule test; a flag-everything scanner fails `scanner_ignores_clean_mentions` and the `lib.rs may declare mod motor;` assertion. The `must` file list guards against scanning an empty directory. They are not tautological. Gaps: no test for `impl Trait` or the alias case (P2-2, P3-1).
4. Spec: see P3-3.
5. Runtime is about 0.01 s (plus the usual compile). No `HashMap` is used; files are sorted, so output is deterministic. The failure message gives file, line, rule and the invariant. Clear.

## Verification

- Read `exec_isolation.rs`, agent-models §1.1 and line 985.
- Ran `bin/job.sh run -- 'cd sim && cargo test -p bb_agents --test exec_isolation --test zz_probe -- --nocapture'` (job 145): 5 existing tests passed; a temporary `zz_probe.rs` (a copy of the test file plus a probe printing violation lines for the 15 decision snippets and 6 motor snippets listed above) was deleted afterwards.
- `git status` is clean apart from the untracked JOB/REVIEW files.
- Not run: the full workspace tests (no sim code changed).

## Resolution

All five fixed in `695bb43` by the original worker (cm-trial-2): `cfg_attr` paths and a renamed `include` are flagged (P2-1, P3-2); only `struct`/`enum`/`union` names count as local in `motor.rs`, so an alias impl is flagged (P2-2); `impl` counts as an item only in item position, lifetimes are skipped and tuple self types are checked (P3-1); agent-models §1.1 names the `motor.rs` rule and the forms it covers (P3-3). Each hole has a self-test case and was planted against the real source through the new `EXEC_ISOLATION_SRC` override. No re-review: a small, test-covered fix to a test file (CLAUDE.md, Pace). Coordinator re-ran `cargo test -p bb_agents`, `crate_graph` and check-docs (queue job 150, passed).
