//! Execution isolation (AGENTS.md invariant 3, agent-models §1.1): decision code never
//! receives an `Executed*` value.
//!
//! Why a source scan and not a compile-fail test: `motor` and its `Executed*` types are `pub`
//! because `bb_game` and the lab tests need them, and Rust cannot hide a `pub` item from the
//! sibling modules of its own crate. A compile-fail (`trybuild`) test would therefore compile
//! fine when `pitcher.rs` named `ExecutedRelease`. The only compiler-enforced version moves
//! `motor` into its own crate; that changes the crate layout in AGENTS.md invariant 3 and is the
//! owner's call. Until then this test enforces the rule on the source text.
//!
//! The rule: every file under `bb_agents/src/` except `motor.rs` (and `motor/`) must, once comments
//! and string literals are stripped,
//!   1. contain no identifier starting with `Executed` or `execute_`;
//!   2. contain no identifier `motor` (this covers `crate::motor`, `super::motor`,
//!      `bb_agents::motor`, `use ... as m`, re-exports, `motor::*` and
//!      `use crate::{.., motor}`), except the one declaration `mod motor;` in `lib.rs`;
//!   3. not use `#[path]`, `include!`, `include_str!`, `include_bytes!`, `concat_idents` or
//!      `paste`, which could pull in or build a name the scan cannot see.
//!
//! And `motor.rs` itself must not leak its output through a name-free route: no `trait`
//! (a blanket impl would put a method on every type), no `macro_rules`, and every `impl`
//! must be for a type defined in `motor.rs` (an `impl IntendedRelease { fn run(..) -> Executed.. }`
//! would let `pitcher.rs` call it without naming `motor`). Importing decision types into
//! `motor.rs` is fine: that is the intended direction.
//!
//! Known limits (the scan is textual, not a type checker):
//!   - A procedural macro or a macro exported by `bb_math`/`bb_physics`/`bb_observe` that builds
//!     these names from pieces is not seen; `bb_agents` may not depend on such a crate without
//!     review. Only `paste` and `concat_idents` are flagged by name.
//!   - The scan covers `bb_agents/src/` only. Other crates (`bb_game`, tests, examples) may use
//!     `motor` freely; whether `bb_game` hands an `Executed*` value to an agent is for review.
//!   - A type that merely contains an `Executed*` value (a field) must name it, so rule 1
//!     catches it; a `pub` function in `motor.rs` that returns one is reached only via `motor`.
//!   - Files outside `src/` pulled in by `#[path]` or `include!` are refused by rule 3.
//!   - The scan trusts `lib.rs`'s one `mod motor;` line; another `mod` line there that
//!     re-declares the file is a rule-3 or rule-2 hit.

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Violation {
    line: usize,
    rule: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Tok {
    Ident(String),
    Punct(char),
}

fn is_ident_start(c: char) -> bool {
    c == '_' || c.is_alphabetic()
}

fn is_ident_char(c: char) -> bool {
    c == '_' || c.is_alphanumeric()
}

/// Replace comments and string/char literals with spaces (newlines kept, so line numbers
/// survive). Raw identifiers lose their `r#` prefix.
fn strip(src: &str) -> String {
    let c: Vec<char> = src.chars().collect();
    let n = c.len();
    let mut out = String::with_capacity(src.len());
    let blank = |out: &mut String, ch: char| out.push(if ch == '\n' { '\n' } else { ' ' });
    let mut i = 0;
    while i < n {
        let ch = c[i];
        let next = c.get(i + 1).copied();
        if ch == '/' && next == Some('/') {
            while i < n && c[i] != '\n' {
                out.push(' ');
                i += 1;
            }
        } else if ch == '/' && next == Some('*') {
            let mut depth = 1;
            out.push_str("  ");
            i += 2;
            while i < n && depth > 0 {
                if c[i] == '/' && c.get(i + 1) == Some(&'*') {
                    depth += 1;
                    out.push_str("  ");
                    i += 2;
                } else if c[i] == '*' && c.get(i + 1) == Some(&'/') {
                    depth -= 1;
                    out.push_str("  ");
                    i += 2;
                } else {
                    blank(&mut out, c[i]);
                    i += 1;
                }
            }
        } else if ch == '"' {
            out.push(' ');
            i += 1;
            while i < n && c[i] != '"' {
                if c[i] == '\\' && i + 1 < n {
                    blank(&mut out, c[i]);
                    blank(&mut out, c[i + 1]);
                    i += 2;
                } else {
                    blank(&mut out, c[i]);
                    i += 1;
                }
            }
            if i < n {
                out.push(' ');
                i += 1;
            }
        } else if ch == '\'' {
            if next == Some('\\') {
                // '\n', '\'', '\u{..}': blank up to the closing quote after the escaped char.
                let mut j = i + 3;
                while j < n && c[j] != '\'' {
                    j += 1;
                }
                let end = j.min(n - 1);
                for &x in &c[i..=end] {
                    blank(&mut out, x);
                }
                i = end + 1;
            } else if c.get(i + 2) == Some(&'\'') {
                // 'x', including '"'.
                for &x in &c[i..i + 3] {
                    blank(&mut out, x);
                }
                i += 3;
            } else {
                out.push('\''); // a lifetime
                i += 1;
            }
        } else if is_ident_start(ch) {
            let mut j = i;
            while j < n && is_ident_char(c[j]) {
                j += 1;
            }
            let id: String = c[i..j].iter().collect();
            if matches!(id.as_str(), "r" | "br" | "cr") {
                let mut k = j;
                let mut hashes = 0;
                while c.get(k) == Some(&'#') {
                    k += 1;
                    hashes += 1;
                }
                if c.get(k) == Some(&'"') {
                    // raw string: blank through the closing quote and hashes
                    for &x in &c[i..=k] {
                        blank(&mut out, x);
                    }
                    i = k + 1;
                    while i < n {
                        if c[i] == '"' && (0..hashes).all(|h| c.get(i + 1 + h) == Some(&'#')) {
                            for &x in &c[i..i + 1 + hashes] {
                                blank(&mut out, x);
                            }
                            i += 1 + hashes;
                            break;
                        }
                        blank(&mut out, c[i]);
                        i += 1;
                    }
                    continue;
                }
                if id == "r" && hashes == 1 && c.get(k).is_some_and(|&x| is_ident_start(x)) {
                    out.push_str("  "); // raw identifier r#name
                    i = k;
                    continue;
                }
            }
            out.push_str(&id);
            i = j;
        } else {
            out.push(ch);
            i += 1;
        }
    }
    out
}

fn tokens(stripped: &str) -> Vec<(Tok, usize)> {
    let c: Vec<char> = stripped.chars().collect();
    let mut toks = Vec::new();
    let (mut i, mut line) = (0, 1);
    while i < c.len() {
        let ch = c[i];
        if ch == '\n' {
            line += 1;
            i += 1;
        } else if ch.is_whitespace() {
            i += 1;
        } else if is_ident_char(ch) {
            let mut j = i;
            while j < c.len() && is_ident_char(c[j]) {
                j += 1;
            }
            toks.push((Tok::Ident(c[i..j].iter().collect()), line));
            i = j;
        } else {
            toks.push((Tok::Punct(ch), line));
            i += 1;
        }
    }
    toks
}

fn ident(t: &(Tok, usize)) -> Option<&str> {
    match &t.0 {
        Tok::Ident(s) => Some(s),
        Tok::Punct(_) => None,
    }
}

fn is_punct(t: Option<&(Tok, usize)>, p: char) -> bool {
    t.is_some_and(|t| t.0 == Tok::Punct(p))
}

/// Rules for every `bb_agents/src` file except `motor.rs`. `is_lib`: the file is `lib.rs`, which may
/// declare `mod motor;` once.
fn scan_decision(src: &str, is_lib: bool) -> Vec<Violation> {
    let toks = tokens(&strip(src));
    let mut v = Vec::new();
    let mut push = |line: usize, rule: &str| v.push(Violation { line, rule: rule.to_string() });
    for (k, t) in toks.iter().enumerate() {
        let Some(id) = ident(t) else { continue };
        let line = t.1;
        let prev = k.checked_sub(1).map(|p| &toks[p]);
        let next = toks.get(k + 1);
        if id.starts_with("Executed") {
            push(line, &format!("names the execution-noise output type {id} (invariant 3: decision code never receives an Executed* value)"));
        } else if id.starts_with("execute_") {
            push(line, &format!("calls or names {id}, which belongs to bb_agents::motor (invariant 3)"));
        } else if id == "motor" {
            let declaration = is_lib && prev.and_then(ident) == Some("mod") && is_punct(next, ';');
            if !declaration {
                push(line, "refers to the motor module (use, path, alias, re-export or glob); only lib.rs may declare it (invariant 3)");
            }
        } else if matches!(id, "include" | "include_str" | "include_bytes") && is_punct(next, '!') {
            push(line, &format!("{id}! can pull motor code or names past the scan (rule 3)"));
        } else if matches!(id, "concat_idents" | "paste") {
            push(line, &format!("{id} can build an Executed*/motor name the scan cannot see (rule 3)"));
        } else if id == "path" && is_punct(prev, '[') {
            let before = k.checked_sub(2).map(|p| &toks[p]);
            if is_punct(before, '#') || is_punct(before, '!') {
                push(line, "#[path] can re-declare motor.rs under another name (rule 3)");
            }
        }
    }
    v
}

/// Rules for `motor.rs`: it may import decision types, but must not hand its output to decision
/// code without a name.
fn scan_motor(src: &str) -> Vec<Violation> {
    let toks = tokens(&strip(src));
    let mut v = Vec::new();
    let mut push = |line: usize, rule: &str| v.push(Violation { line, rule: rule.to_string() });
    let mut locals: Vec<&str> = Vec::new();
    for (k, t) in toks.iter().enumerate() {
        if matches!(ident(t), Some("struct" | "enum" | "type" | "union")) {
            if let Some(name) = toks.get(k + 1).and_then(ident) {
                locals.push(name);
            }
        }
    }
    for (k, t) in toks.iter().enumerate() {
        let Some(id) = ident(t) else { continue };
        match id {
            "trait" => push(t.1, "motor.rs defines a trait: a blanket impl would put a method on decision types without naming motor (motor.rs rule)"),
            "macro_rules" => push(t.1, "motor.rs defines a macro, which can be used without naming motor (motor.rs rule)"),
            "impl" => {
                let mut j = k + 1;
                if is_punct(toks.get(j), '<') {
                    let mut depth = 0;
                    while j < toks.len() {
                        match toks[j].0 {
                            Tok::Punct('<') => depth += 1,
                            Tok::Punct('>') => {
                                depth -= 1;
                                if depth == 0 {
                                    j += 1;
                                    break;
                                }
                            }
                            _ => {}
                        }
                        j += 1;
                    }
                }
                // Self type: after "for" if this is a trait impl, else the first type name.
                let mut start = j;
                let mut m = j;
                while m < toks.len() && !is_punct(toks.get(m), '{') && ident(&toks[m]) != Some("where") {
                    if ident(&toks[m]) == Some("for") {
                        start = m + 1;
                    }
                    m += 1;
                }
                let self_ty = toks[start..m.min(toks.len())]
                    .iter()
                    .filter_map(ident)
                    .find(|s| !matches!(*s, "mut" | "dyn" | "const"));
                if !self_ty.is_some_and(|s| locals.contains(&s)) {
                    push(t.1, &format!("motor.rs impl for {}, which motor.rs does not define: it would hang a method (or an Executed* result) on a decision type (motor.rs rule)", self_ty.unwrap_or("?")));
                }
            }
            _ => {}
        }
    }
    v
}

fn rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<_> = fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            rs_files(&p, out);
        } else if p.extension().is_some_and(|e| e == "rs") {
            out.push(p);
        }
    }
}

fn report(rel: &str, vs: &[Violation], failures: &mut Vec<String>) {
    for x in vs {
        failures.push(format!("bb_agents/src/{rel}:{}: {}", x.line, x.rule));
    }
}

#[test]
fn decision_code_never_names_executed_values() {
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rs_files(&src_dir, &mut files);
    let rels: Vec<String> = files
        .iter()
        .map(|p| p.strip_prefix(&src_dir).unwrap().to_string_lossy().replace('\\', "/"))
        .collect();
    // The scan must actually see the decision modules, or it would pass by finding nothing.
    for must in ["lib.rs", "motor.rs", "pitcher.rs", "batter.rs", "swing.rs", "running.rs", "throwing.rs", "attributes.rs"] {
        assert!(rels.iter().any(|r| r == must), "scan did not find src/{must}; found {rels:?}");
    }
    let mut failures = Vec::new();
    for (p, rel) in files.iter().zip(&rels) {
        let text = fs::read_to_string(p).unwrap();
        if rel == "motor.rs" || rel.starts_with("motor/") {
            report(rel, &scan_motor(&text), &mut failures);
        } else {
            report(rel, &scan_decision(&text, rel == "lib.rs"), &mut failures);
        }
    }
    assert!(failures.is_empty(), "execution isolation broken:\n{}", failures.join("\n"));
}

// ---- self-test: the scanner must flag each violation kind and ignore clean mentions ----

fn flags(src: &str, is_lib: bool) -> Vec<usize> {
    scan_decision(src, is_lib).iter().map(|x| x.line).collect()
}

#[test]
fn scanner_flags_each_planted_violation() {
    let bad: &[(&str, &str)] = &[
        ("named type", "fn f(x: ExecutedSwing) {}"),
        ("type in return position", "fn f() -> ExecutedRelease { todo!() }"),
        ("any Executed prefix", "struct S { t: ExecutedThrowLater }"),
        ("call execute_release", "let e = execute_release(&i, &n, &mut r);"),
        ("call execute_swing", "let e = execute_swing(&i, &n, &mut r, &mut q);"),
        ("call execute_throw", "let e = execute_throw(30.0, &n, &mut r);"),
        ("crate path import", "use crate::motor::ReleaseNoise;"),
        ("super path import", "use super::motor::SwingNoise;"),
        ("extern-name path import", "use bb_agents::motor::TRUNC_SIGMA;"),
        ("glob import", "use crate::motor::*;"),
        ("renamed import", "use crate::motor as m;"),
        ("renamed type", "use crate::motor::ExecutedSwing as Harmless;"),
        ("grouped import", "use crate::{pitcher, motor};"),
        ("nested group", "use crate::{pitcher::X, motor::{ThrowNoise}};"),
        ("re-export", "pub use crate::motor::ExecutedThrow;"),
        ("module path use after crate glob", "use crate::*;\nfn f() { let _ = motor::TRUNC_SIGMA; }"),
        ("module declared outside lib.rs", "pub mod motor;"),
        ("module aliased in lib.rs", "pub use motor as m;"),
        ("path attribute", "#[path = \"motor.rs\"]\nmod sneaky;"),
        ("inner path attribute", "#![path = \"motor.rs\"]"),
        ("include!", "include!(\"motor.rs\");"),
        ("include_str!", "const S: &str = include_str!(\"motor.rs\");"),
        ("paste", "paste! { type T = [<Executed Swing>]; }"),
        ("raw identifier", "fn f(x: r#ExecutedSwing) {}"),
        ("after a block comment", "/* harmless */ fn f(x: ExecutedSwing) {}"),
        ("after an unfinished-looking nested comment", "/* a /* b */ still comment */ fn f(x: ExecutedSwing) {}"),
        ("after an escaped quote in a string", "let s = \"a\\\"b\"; let e: ExecutedSwing;"),
        ("after a quote char literal", "let c = '\"'; let e: ExecutedSwing;"),
        ("after an escaped quote char literal", "let c = '\\''; let e: ExecutedSwing;"),
        ("after a lifetime", "fn f<'a>(x: &'a str) -> ExecutedSwing { todo!() }"),
        ("after a raw string", "let s = r#\"a \" b\"#; let e: ExecutedSwing;"),
        ("after a line comment", "// note\nlet e: ExecutedSwing;"),
    ];
    for (what, src) in bad {
        assert!(!flags(src, false).is_empty(), "scanner missed: {what}: {src:?}");
    }
    // lib.rs: the one declaration is allowed, any other use is not.
    assert!(flags("pub mod motor;", true).is_empty(), "lib.rs may declare mod motor;");
    assert!(!flags("pub mod motor;\npub use motor::*;", true).is_empty(), "re-export in lib.rs");
    assert!(!flags("mod motor { }", true).is_empty(), "inline module named motor in lib.rs");
}

#[test]
fn scanner_reports_file_line_and_rule() {
    let v = scan_decision("fn a() {}\n\n// ok\nfn b(x: ExecutedSwing) {}\n", false);
    assert_eq!(v.len(), 1, "{v:?}");
    assert_eq!(v[0].line, 4);
    assert!(v[0].rule.contains("ExecutedSwing") && v[0].rule.contains("invariant 3"), "{}", v[0].rule);
    let mut failures = Vec::new();
    report("pitcher.rs", &v, &mut failures);
    assert!(failures[0].starts_with("bb_agents/src/pitcher.rs:4: "), "{}", failures[0]);
}

#[test]
fn scanner_ignores_clean_mentions() {
    let clean: &[(&str, &str)] = &[
        ("line comment", "// ExecutedRelease comes from crate::motor::execute_release\nfn f() {}"),
        ("doc comment", "/// Fed to [`crate::motor::execute_swing`] later; never sees ExecutedSwing.\nfn f() {}"),
        ("inner doc comment", "//! ExecutedThrow is bb_game's.\nfn f() {}"),
        ("block comment", "/* motor::execute_throw(ExecutedThrow) */ fn f() {}"),
        ("nested block comment", "/* a /* ExecutedSwing */ motor */ fn f() {}"),
        ("string literal", "let s = \"ExecutedRelease motor execute_throw\";"),
        ("string with escaped quote", "let s = \"say \\\"ExecutedSwing\\\" twice\";"),
        ("raw string", "let s = r#\"use crate::motor::ExecutedSwing; \" still inside\"#;"),
        ("byte string", "let s = b\"ExecutedSwing\";"),
        ("doc attribute", "#[doc = \"see motor::ExecutedSwing\"]\nfn f() {}"),
        ("similar identifiers", "let motor_latency_s = 0.1; let executed = 1; let execution_noise = 2; fn execute() {}"),
        ("field and variable names that merely contain the words", "struct S { motor_lag: f64, executed_count: u32 }"),
        ("lifetime then comment", "fn f<'a>(x: &'a str) -> &'a str { x } // ExecutedSwing"),
    ];
    for (what, src) in clean {
        assert!(flags(src, false).is_empty(), "scanner flagged a clean mention: {what}: {:?}", scan_decision(src, false));
    }
}

#[test]
fn motor_scan_flags_name_free_leaks_and_accepts_the_real_shape() {
    let ok = "use crate::pitcher::IntendedRelease;\n\
              pub struct ReleaseNoise { pub angle: f64 }\n\
              impl ReleaseNoise { pub fn new() -> Self { Self { angle: 0.0 } } }\n\
              impl Default for ReleaseNoise { fn default() -> Self { Self::new() } }\n\
              pub struct ExecutedRelease { pub yaw: f64 }\n\
              pub fn execute_release(i: &IntendedRelease) -> ExecutedRelease { todo!() }\n";
    assert!(scan_motor(ok).is_empty(), "{:?}", scan_motor(ok));
    let bad: &[(&str, &str)] = &[
        ("impl on a decision type", "use crate::pitcher::IntendedRelease;\nimpl IntendedRelease { pub fn run(&self) -> ExecutedRelease { todo!() } }"),
        ("impl on a reference to a decision type", "impl<'a> Foo for &'a IntendedRelease { }"),
        ("trait impl for a decision type", "use crate::swing::IntendedSwing;\nimpl Clone for IntendedSwing { fn clone(&self) -> Self { todo!() } }"),
        ("blanket impl", "pub struct L;\nimpl<T> Run for T { fn run(&self) {} }"),
        ("trait definition", "pub trait Execute { fn go(&self) -> ExecutedSwing; }"),
        ("macro definition", "macro_rules! go { () => {}; }"),
    ];
    for (what, src) in bad {
        assert!(!scan_motor(src).is_empty(), "motor scan missed: {what}");
    }
    // Reports the line of the offending impl.
    let v = scan_motor("pub struct A;\n\nimpl B { }\n");
    assert_eq!(v.iter().map(|x| x.line).collect::<Vec<_>>(), vec![3], "{v:?}");
}
