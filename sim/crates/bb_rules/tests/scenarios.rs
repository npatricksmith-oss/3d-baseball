//! Scenario corpus harness (rules-spec §12): every `scenarios/**/*.toml` is a scripted fact
//! sequence with expected events and final state. No physics involved.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bb_rules::*;
use toml::Value;

fn agent(name: &str) -> AgentId {
    let (team, rest) = name.split_once('.').unwrap_or_else(|| panic!("bad agent {name}"));
    let side: u16 = match team {
        "A" => 0,
        "H" => 1,
        _ => panic!("bad team in {name}"),
    };
    if let Ok(slot) = rest.parse::<u16>() {
        assert!((1..=9).contains(&slot), "slot {name}");
        return (side << 8) | slot;
    }
    let pos = ["P", "C", "1B", "2B", "3B", "SS", "LF", "CF", "RF"]
        .iter()
        .position(|p| *p == rest)
        .unwrap_or_else(|| panic!("bad position in {name}")) as u16;
    (side << 8) | 0x80 | (pos + 1)
}

fn name_of(a: AgentId) -> String {
    let team = if a >> 8 == 0 { "A" } else { "H" };
    let lo = a & 0xFF;
    if lo & 0x80 != 0 {
        let pos = ["P", "C", "1B", "2B", "3B", "SS", "LF", "CF", "RF"][(lo & 0x7F) as usize - 1];
        format!("{team}.{pos}")
    } else {
        format!("{team}.{lo}")
    }
}

fn f(v: &Value, k: &str) -> f64 {
    v.get(k).and_then(|x| x.as_float().or_else(|| x.as_integer().map(|i| i as f64))).unwrap_or_else(|| panic!("missing number {k} in {v}"))
}
fn s<'a>(v: &'a Value, k: &str) -> &'a str {
    v.get(k).and_then(|x| x.as_str()).unwrap_or_else(|| panic!("missing string {k} in {v}"))
}
fn b(v: &Value, k: &str) -> bool {
    v.get(k).and_then(|x| x.as_bool()).unwrap_or_else(|| panic!("missing bool {k} in {v}"))
}

fn check_keys(v: &Value, allowed: &[&str], ctx: &str) {
    if let Some(t) = v.as_table() {
        for k in t.keys() {
            assert!(allowed.contains(&k.as_str()), "{ctx}: unknown key `{k}`");
        }
    }
}

/// A scenario step: a discrete fact, or a runner position sample (rules-spec §2).
enum Parsed {
    Input(Input),
    Position(RunnerPosition),
}

fn parse_input(v: &Value) -> Parsed {
    let t = v.get("t").map(|_| f(v, "t")).unwrap_or(0.0);
    if s(v, "fact") == "RunnerPosition" {
        let p = v["pos"].as_array().unwrap();
        return Parsed::Position(RunnerPosition { t, runner: agent(s(v, "runner")), x: p[0].as_float().unwrap(), y: p[1].as_float().unwrap() });
    }
    Parsed::Input(match s(v, "fact") {
        "StartGame" => Input::StartGame,
        "PitchDelivered" => Input::PitchDelivered { t },
        "PitchCrossed" => Input::PitchCrossed { t, in_zone: b(v, "in_zone"), bounced: v.get("bounced").and_then(|x| x.as_bool()).unwrap_or(false) },
        "SwingMeasured" => Input::SwingMeasured { t, swung: b(v, "swung") },
        "BatContact" => Input::BatContact { t },
        "BallTouched" => {
            let what = match s(v, "what") {
                "Ground" => {
                    let p = v["pos"].as_array().unwrap();
                    Touch::Ground { x: p[0].as_float().unwrap(), y: p[1].as_float().unwrap() }
                }
                "OutOfPlay" => {
                    let p = v["pos"].as_array().unwrap();
                    Touch::OutOfPlay { x: p[0].as_float().unwrap(), y: p[1].as_float().unwrap(), in_flight: b(v, "in_flight") }
                }
                "Batter" => Touch::Batter,
                "Wall" => {
                    let p = v["pos"].as_array().unwrap();
                    Touch::Wall { x: p[0].as_float().unwrap(), y: p[1].as_float().unwrap() }
                }
                "FoulPole" => Touch::FoulPole,
                w => panic!("unknown touch {w}"),
            };
            Input::BallTouched { t, what }
        }
        "BallPassedBase" => Input::BallPassedBase { t, fair: b(v, "fair") },
        "BallSettled" => {
            let p = v["pos"].as_array().unwrap();
            Input::BallSettled { t, x: p[0].as_float().unwrap(), y: p[1].as_float().unwrap() }
        }
        "GloveSecured" => {
            let p = v["pos"].as_array().unwrap();
            Input::GloveSecured { t, fielder: agent(s(v, "fielder")), x: p[0].as_float().unwrap(), y: p[1].as_float().unwrap() }
        }
        "BallReleasedFromGlove" => Input::BallReleasedFromGlove {
            t,
            fielder: agent(s(v, "fielder")),
            kind: match v.get("kind").and_then(|x| x.as_str()).unwrap_or("Throw") {
                "Throw" => ReleaseKind::Throw,
                "Drop" => ReleaseKind::Drop,
                "Knocked" => ReleaseKind::Knocked,
                k => panic!("unknown release kind {k}"),
            },
            infielder: b(v, "infielder"),
        },
        "BaseTouched" => Input::BaseTouched {
            t,
            agent: agent(s(v, "agent")),
            base: v["base"].as_integer().unwrap() as u8,
            holding_ball: v.get("holding_ball").and_then(|x| x.as_bool()).unwrap_or(false),
        },
        "BaseLeft" => Input::BaseLeft { t, agent: agent(s(v, "agent")), base: v["base"].as_integer().unwrap() as u8 },
        "TagContact" => Input::TagContact { t, fielder: agent(s(v, "fielder")), runner: agent(s(v, "runner")), holding_ball: b(v, "holding_ball") },
        "PlayEndCheck" => Input::PlayEndCheck { t },
        "PlayTimeout" => Input::PlayTimeout { t },
        x => panic!("unknown fact {x}"),
    })
}

/// Render an event as a key→string map for matching.
fn render(e: &Event) -> BTreeMap<&'static str, String> {
    let mut m = BTreeMap::new();
    let name = format!("{e:?}");
    let ev = name.split([' ', '{', '(']).next().unwrap().to_string();
    m.insert("event", ev);
    match e {
        Event::PitchResult { call, balls, strikes } => {
            m.insert("call", format!("{call:?}"));
            m.insert("balls", balls.to_string());
            m.insert("strikes", strikes.to_string());
        }
        Event::RunnerOut { runner, base, kind } => {
            m.insert("runner", name_of(*runner));
            m.insert("base", base.to_string());
            m.insert("kind", format!("{kind:?}"));
        }
        Event::RunnerAdvanced { runner, from, to, cause } => {
            m.insert("runner", name_of(*runner));
            m.insert("from", from.to_string());
            m.insert("to", to.to_string());
            m.insert("cause", format!("{cause:?}"));
        }
        Event::RunnerPassed { runner, passed } => {
            m.insert("runner", name_of(*runner));
            m.insert("passed", name_of(*passed));
        }
        Event::RunScored { runner } => {
            m.insert("runner", name_of(*runner));
        }
        Event::PlateAppearanceEnded { batter, result } => {
            m.insert("batter", name_of(*batter));
            m.insert("result", format!("{result:?}"));
        }
        Event::FairFoulDecided { fair } => {
            m.insert("fair", fair.to_string());
        }
        Event::GameEnded { away, home, walk_off, .. } => {
            m.insert("away", away.to_string());
            m.insert("home", home.to_string());
            m.insert("walk_off", walk_off.to_string());
        }
        Event::PlateAppearanceStarted { batter } => {
            m.insert("batter", name_of(*batter));
        }
        Event::HalfInningStarted { inning, half } => {
            m.insert("inning", inning.to_string());
            m.insert("half", format!("{half:?}").to_lowercase());
        }
        Event::Catch { fielder } => {
            m.insert("fielder", name_of(*fielder));
        }
        Event::RulesWarning { code } => {
            m.insert("code", code.to_string());
        }
        Event::HalfInningEnded { runs, left_on_base } => {
            m.insert("runs", runs.to_string());
            m.insert("left_on_base", left_on_base.to_string());
        }
        Event::BallDead { reason } => {
            m.insert("reason", format!("{reason:?}"));
        }
        _ => {}
    }
    m
}

fn val_str(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Integer(i) => i.to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Float(x) => x.to_string(),
        other => other.to_string(),
    }
}

fn setup(init: Option<&Value>) -> RulesState {
    let mut st = RulesState::default_fixture();
    let Some(init) = init else { return st };
    check_keys(init, &["inning", "half", "outs", "count", "bases", "score", "batter", "innings"], "initial");
    if let Some(n) = init.get("innings") {
        st.options.innings = n.as_integer().unwrap() as u8;
    }
    st.status = Status::InProgress;
    st.inning = init.get("inning").map(|v| v.as_integer().unwrap() as u8).unwrap_or(1);
    st.half = match init.get("half").map(|v| v.as_str().unwrap()).unwrap_or("top") {
        "top" => Half::Top,
        "bottom" => Half::Bottom,
        h => panic!("half {h}"),
    };
    // Runs by inning consistent with the current half; the given score goes in inning 1.
    let n_away = st.inning as usize;
    let n_home = if st.half == Half::Bottom { st.inning as usize } else { st.inning as usize - 1 };
    st.runs_by_inning = [vec![0; n_away], vec![0; n_home]];
    if let Some(sc) = init.get("score").and_then(|v| v.as_array()) {
        st.runs_by_inning[0][0] = sc[0].as_integer().unwrap() as u8;
        if n_home > 0 {
            st.runs_by_inning[1][0] = sc[1].as_integer().unwrap() as u8;
        } else {
            assert_eq!(sc[1].as_integer().unwrap(), 0, "home cannot have runs before batting");
        }
    }
    st.outs = init.get("outs").map(|v| v.as_integer().unwrap() as u8).unwrap_or(0);
    if let Some(c) = init.get("count").and_then(|v| v.as_array()) {
        st.balls = c[0].as_integer().unwrap() as u8;
        st.strikes = c[1].as_integer().unwrap() as u8;
    }
    if let Some(bs) = init.get("bases") {
        check_keys(bs, &["first", "second", "third"], "initial.bases");
        for (i, k) in ["first", "second", "third"].iter().enumerate() {
            st.bases[i] = bs.get(*k).map(|v| agent(v.as_str().unwrap()));
        }
    }
    let side = st.half.batting();
    if let Some(bt) = init.get("batter") {
        let a = agent(bt.as_str().unwrap());
        st.next_up[side] = ((a & 0xFF) - 1) as usize;
    }
    st.batter = st.lineup[side][st.next_up[side]];
    st
}

fn run_scenario(path: &Path) -> Result<(), String> {
    let text = std::fs::read_to_string(path).unwrap();
    let doc: Value = toml::from_str(&text).map_err(|e| format!("parse: {e}"))?;
    check_keys(&doc, &["id", "title", "rules", "milestone", "tags", "initial", "step", "expect"], "top level");
    let mut st = setup(doc.get("initial"));
    let empty = Vec::new();
    for (i, stv) in doc.get("step").and_then(|v| v.as_array()).unwrap_or(&empty).iter().enumerate() {
        check_keys(
            stv,
            &["fact", "t", "in_zone", "bounced", "swung", "what", "pos", "in_flight", "fair", "fielder", "agent", "base", "holding_ball", "runner", "kind", "infielder", "expect_events"],
            &format!("step {i}"),
        );
        let evs = match parse_input(stv) {
            Parsed::Input(i) => step(&mut st, &i),
            Parsed::Position(p) => step(&mut st, &p),
        };
        // Facts must be chronological within a play (rules-spec §1.2; Codex M4 review #22).
        if evs.iter().any(|e| matches!(e, Event::RulesWarning { code: "time_reversed" })) {
            return Err(format!("step {i}: fact time goes backward"));
        }
        if let Some(exp) = stv.get("expect_events").and_then(|v| v.as_array()) {
            let rendered: Vec<_> = evs.iter().map(render).collect();
            let mut j = 0;
            for e in exp {
                let want: Vec<(String, String)> = e.as_table().unwrap().iter().map(|(k, v)| (k.clone(), val_str(v))).collect();
                let ok = loop {
                    if j >= rendered.len() {
                        break false;
                    }
                    let r = &rendered[j];
                    j += 1;
                    if want.iter().all(|(k, v)| r.get(k.as_str()) == Some(v)) {
                        break true;
                    }
                };
                if !ok {
                    return Err(format!("step {i}: expected {e} in {evs:?}"));
                }
            }
        }
    }
    if let Some(exp) = doc.get("expect") {
        check_keys(exp, &["outs", "bases", "score", "status", "inning", "half", "balls", "strikes", "batter", "play"], "expect");
        let mut errs = Vec::new();
        if let Some(o) = exp.get("outs") {
            if st.outs as i64 != o.as_integer().unwrap() {
                errs.push(format!("outs {} != {}", st.outs, o));
            }
        }
        if let Some(bs) = exp.get("bases") {
            let want: [Option<AgentId>; 3] = ["first", "second", "third"].map(|k| bs.get(k).map(|v| agent(v.as_str().unwrap())));
            if st.bases != want {
                errs.push(format!("bases {:?} != {:?}", st.bases.map(|b| b.map(name_of)), want.map(|b| b.map(name_of))));
            }
        }
        if let Some(sc) = exp.get("score").and_then(|v| v.as_array()) {
            let want = [sc[0].as_integer().unwrap() as u16, sc[1].as_integer().unwrap() as u16];
            if st.score() != want {
                errs.push(format!("score {:?} != {:?}", st.score(), want));
            }
        }
        if let Some(s) = exp.get("status") {
            if format!("{:?}", st.status) != s.as_str().unwrap() {
                errs.push(format!("status {:?} != {}", st.status, s));
            }
        }
        if let Some(n) = exp.get("inning") {
            if st.inning as i64 != n.as_integer().unwrap() {
                errs.push(format!("inning {} != {}", st.inning, n));
            }
        }
        if let Some(h) = exp.get("half") {
            let want = if h.as_str() == Some("top") { Half::Top } else { Half::Bottom };
            if st.half != want {
                errs.push(format!("half {:?} != {}", st.half, h));
            }
        }
        for (k, v) in [("balls", st.balls), ("strikes", st.strikes)] {
            if let Some(e) = exp.get(k) {
                if v as i64 != e.as_integer().unwrap() {
                    errs.push(format!("{k} {v} != {e}"));
                }
            }
        }
        if let Some(bt) = exp.get("batter") {
            if st.batter != agent(bt.as_str().unwrap()) {
                errs.push(format!("batter {} != {}", name_of(st.batter), bt));
            }
        }
        if let Some(p) = exp.get("play") {
            let dead = st.play == PlayState::Dead;
            if (p.as_str() == Some("Dead")) != dead {
                errs.push(format!("play {:?} != {}", st.play, p));
            }
        }
        if !errs.is_empty() {
            return Err(errs.join("; "));
        }
    }
    Ok(())
}

fn files(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            files(&p, out);
        } else if p.extension().map(|x| x == "toml").unwrap_or(false) {
            out.push(p);
        }
    }
}

#[test]
fn scenario_corpus() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("scenarios");
    let mut fs = Vec::new();
    files(&root, &mut fs);
    fs.sort();
    assert!(!fs.is_empty(), "no scenarios found");
    let mut fails = Vec::new();
    for p in &fs {
        if let Err(e) = run_scenario(p) {
            fails.push(format!("{}: {e}", p.strip_prefix(&root).unwrap().display()));
        }
    }
    println!("{} scenarios, {} failed", fs.len(), fails.len());
    assert!(fails.is_empty(), "\n{}", fails.join("\n"));
}
