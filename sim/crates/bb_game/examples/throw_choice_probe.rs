//! A4 measuring only. `-- [games=256] [first_seed=0] [records.jsonl]`.
//! JSONL includes EVERY throw/carry commitment, its original estimates, and outcome.
//! Consecutive carry ticks coalesce; options are from the FIRST tick. Since the A4
//! fix every option is one the holder actually weighed (carries and throws in one
//! table); the `unweighed_throw` shadow of the old nearby-carry shortcut is retired.
//! Each seed is replayed without tracing and both log bytes and digest must match.
use bb_game::game::{Game, PitchStep};
use bb_game::live::choice_trace::{Choice, OptionEstimate, capture};
use bb_rules::{Event, Input};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::io::{BufWriter, Write};

// Diagnostic judgments, NOT simulation parameters. A quarter second is the existing
// §5.5 safe margin; 0.15 s is the umpire's close-play boundary. Keep borderline cases
// separate from stronger flags, and never infer a bad decision from bad execution.
const SURE_S: f64 = 0.25;
const CLOSE_S: f64 = 0.15;
const CLASSES: [&str; 10] = [
    "two_out_home_over_force",
    "behind_sure_lead",
    "risky_lead_over_sure_out",
    "home_gamble_early_or_leading",
    "late_needed_home_out",
    "no_chance_throw",
    "speculative_throw",
    "slower_cutoff_route",
    "nonforce_carry_over_sure_force",
    "ahead_of_stopped_runner",
];

fn margin(o: &OptionEstimate) -> f64 {
    o.runner_at - o.ball_at
}
fn chosen(c: &Choice) -> Option<&OptionEstimate> {
    if let Some(i) = c.selected_option {
        return c.options.get(i);
    }
    if c.action == "return" {
        None
    } else {
        // Two runners can head toward the same bag. The real best loop retains the
        // first in a value tie, so associate the evidence with that runner as well.
        c.options.iter().filter(|o| o.base == c.base && o.kind == "throw").reduce(|a, b| if b.value > a.value { b } else { a })
    }
}
fn alternatives(c: &Choice) -> impl Iterator<Item = &OptionEstimate> {
    c.options.iter().filter(|o| o.base != c.base)
}
fn flags(c: &Choice) -> Vec<&'static str> {
    if c.action == "return" {
        return Vec::new();
    } // a control throw, not an out attempt
    let Some(s) = chosen(c) else { return Vec::new() };
    let mut f = Vec::new();
    let lead = c.score[1 - c.batting] as i32 - c.score[c.batting] as i32;
    let sure_force = alternatives(c).any(|o| o.force && margin(o) >= SURE_S);
    if c.outs >= 2 && c.base == 4 && !s.force && alternatives(c).any(|o| o.force && o.base <= 2 && margin(o) >= CLOSE_S) {
        f.push(CLASSES[0]); // close-to-sure force; requires watching, not a proof
    }
    if c.outs < 2 && margin(s) >= SURE_S && alternatives(c).any(|o| o.base > c.base && margin(o) >= SURE_S) {
        f.push(CLASSES[1]);
    }
    if margin(s) < CLOSE_S && alternatives(c).any(|o| o.base < c.base && margin(o) >= SURE_S) {
        f.push(CLASSES[2]);
    }
    if c.outs < 2 && c.base == 4 && !s.force && margin(s) < CLOSE_S && sure_force && (c.inning <= 6 || lead >= 2) {
        f.push(CLASSES[3]);
    }
    if c.outs < 2 && c.base != 4 && c.inning >= 8 && (0..=1).contains(&lead) && alternatives(c).any(|o| o.base == 4 && margin(o) >= SURE_S) {
        f.push(CLASSES[4]);
    }
    if c.action == "throw" && margin(s) <= -SURE_S {
        f.push(CLASSES[5]);
    }
    if c.action == "throw" && margin(s) < 0.0 {
        f.push(CLASSES[6]);
    }
    if c.action == "throw" && s.direct_at.zip(s.via_at).is_some_and(|(d, v)| if c.through { v > d + 0.05 } else { d > v + 0.05 }) {
        f.push(CLASSES[7]);
    }
    if c.action.contains("carry") && s.distance > 1.2 && !s.force && sure_force {
        f.push(CLASSES[8]);
    }
    // Retrospective state audit: the holder sees the previous tick's residual speed
    // at a bag even though that runner decided to stop there. Internal intent is
    // copied only for this diagnostic; this is not an available tactical oracle.
    if c.action == "throw" && c.runners.iter().any(|r| r.id == s.runner && r.intended_goal < c.base && (r.s - r.intended_goal as f64 * 27.432).abs() < 1e-6) {
        f.push(CLASSES[9]);
    }
    f
}

fn option_json(o: &OptionEstimate) -> Value {
    json!({"runner":o.runner,"base":o.base,"kind":o.kind,"force":o.force,
        "runner_at_s":o.runner_at,"ball_at_s":o.ball_at,"margin_s":margin(o),
        "direct_at_s":o.direct_at,"via_at_s":o.via_at,"set_ball_at_s":o.set_at,
        "receiver":o.receiver,"receiver_at_s":o.receiver_at,"p_out":o.p_out,
        "weight":o.weight,"value":o.value,"through":o.through,"distance_m":o.distance,
        "second_out_p":o.second_out_p,"second_ball_at_s":o.second_ball_at})
}

fn record(seed: u64, c: &Choice, step: &PitchStep, labels: &[&str]) -> Value {
    // Outcomes are evidence only. A carry's geometric out may occur on its following
    // tick; a relay's final out can occur after another holder has made a decision.
    let selected = chosen(c).map(|o| o.runner);
    let target_out = selected.and_then(|r| step.calls.iter().find(|call| call.runner == r && call.t_out >= c.t - 1e-9));
    let next = step.viz.secured.iter().find(|s| s.0 > c.t + 1e-9);
    let touches: Vec<_> = step
        .facts
        .iter()
        .filter_map(|f| match *f {
            Input::BaseTouched { t, agent, base, holding_ball } if t >= c.t => Some(json!({"t_s":t,"agent":agent,"base":base,"ball":holding_ball})),
            _ => None,
        })
        .collect();
    let outs: Vec<_> = step.calls.iter().filter(|o| o.t_out >= c.t - 1e-9).map(|o| json!({"t_s":o.t_out,"runner":o.runner,"base":o.base,"margin_s":o.margin_s})).collect();
    let run_scorers: Vec<_> = step
        .rules_events
        .iter()
        .filter_map(|e| match e {
            Event::RunScored { runner } => Some(*runner),
            _ => None,
        })
        .collect();
    let runners: Vec<_> = c
        .runners
        .iter()
        .map(|r| json!({"id":r.id,"start":r.start,"s_m":r.s,"speed_mps":r.speed,"goal":r.goal,"audit_intended_goal":r.intended_goal,"out":r.out,"scored":r.scored,"protected_overrun":r.protected_overrun}))
        .collect();
    let options: Vec<_> = c.options.iter().map(option_json).collect();
    json!({"seed":seed,"play":c.play,"t_s":c.t,"until_s":c.until,"ticks":c.ticks,
        "holder":c.holder,"from_m":c.from.to_array(),"possession_s":c.possession,
        "inning":c.inning,"batting":c.batting,"score":c.score,"outs":c.outs,
        "action":c.action,"base":c.base,"selected_runner":selected,"through":c.through,"runners":runners,
        "options":options,"flags":labels,
        "outcome":{"selected_runner_out_later":target_out.is_some(),
            "next_secured":next.map(|s|json!({"t_s":s.0,"holder":s.1})),
            "outs_after":outs,"touches_after":touches,"play_run_scorers":run_scorers}})
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let games: u64 = a.first().map(|s| s.parse().expect("games")).unwrap_or(256);
    let first: u64 = a.get(1).map(|s| s.parse().expect("first seed")).unwrap_or(0);
    assert!(games > 0);
    let mut writer = a.get(2).map(|path| BufWriter::new(std::fs::File::create(path).expect("JSONL output")));
    let mut counts = BTreeMap::<&str, u64>::new();
    let mut examples = BTreeMap::<&str, Vec<String>>::new();
    let (mut bip, mut commitments, mut flagged, mut runs, mut pitches, mut outs) = (0u64, 0u64, 0u64, 0u64, 0u64, 0u64);
    let mut actions = BTreeMap::<&str, u64>::new();
    let (mut via, mut concession, mut two_out_third, mut target_outs) = (0u64, 0u64, 0u64, 0u64);
    // Sub-counts the note reports: the cutoff share of ahead-of-stopped throws; how
    // many behind-lead choices the holder weighed with a double-play continuation and
    // how many of those plays got two outs; non-force (tag-plan) carries and their tags.
    let (mut stopped_via, mut behind_cont, mut behind_two_outs, mut tag_carries, mut tag_carry_outs) = (0u64, 0u64, 0u64, 0u64, 0u64);
    for seed in first..first + games {
        let mut g = Game::new(seed, true);
        loop {
            let (step, choices) = capture(|| g.step_pitch());
            let Some(step) = step else { break };
            bip += step.batted.is_some() as u64;
            assert_eq!(
                choices.iter().filter(|c| c.action == "throw" || c.action == "return").count(),
                step.viz.released.len(),
                "every release must have evidence: seed {seed} play {}",
                step.play
            );
            for c in &choices {
                commitments += 1;
                *actions.entry(c.action).or_default() += 1;
                via += c.through as u64;
                let labels = flags(c);
                flagged += (!labels.is_empty()) as u64;
                let row = record(seed, c, &step, &labels);
                target_outs += row["outcome"]["selected_runner_out_later"].as_bool().unwrap() as u64;
                if c.outs >= 2 && c.runners.iter().any(|r| r.start == 3 && !r.out && !r.scored) {
                    two_out_third += 1;
                }
                if c.outs < 2 && c.base < 4 && chosen(c).is_some_and(|o| margin(o) >= SURE_S) && alternatives(c).any(|o| o.base == 4 && margin(o) < 0.0) {
                    concession += 1;
                }
                let s = chosen(c);
                if labels.contains(&"ahead_of_stopped_runner") && c.through {
                    stopped_via += 1;
                }
                if labels.contains(&"behind_sure_lead") {
                    behind_cont += s.is_some_and(|o| o.second_out_p.is_some_and(|p| p > 0.3)) as u64;
                    behind_two_outs += (row["outcome"]["outs_after"].as_array().unwrap().len() >= 2) as u64;
                }
                if c.action.contains("carry") && s.is_some_and(|o| !o.force && o.distance > 1.2) {
                    tag_carries += 1;
                    // The rules record a tag at the base after the runner's last touch, so a
                    // runner going back to the carry's bag is out "at" the next one (Gemini A4
                    // review P2-5: those tags were not counted).
                    tag_carry_outs += step.calls.iter().any(|call| Some(call.runner) == s.map(|o| o.runner) && (call.base == c.base || call.base == c.base + 1) && call.t_out >= c.t - 1e-9) as u64;
                }
                for &class in &labels {
                    *counts.entry(class).or_default() += 1;
                    let ex = examples.entry(class).or_default();
                    if ex.len() < 8 {
                        let s = chosen(c).unwrap();
                        ex.push(format!(
                            "seed {seed} play {} t {:.3} {} {}->{} outs {} score {:?} margin {:.3} target_out {}",
                            c.play,
                            c.t,
                            c.action,
                            c.holder,
                            c.base,
                            c.outs,
                            c.score,
                            margin(s),
                            row["outcome"]["selected_runner_out_later"]
                        ));
                    }
                }
                if let Some(w) = &mut writer {
                    serde_json::to_writer(&mut *w, &row).unwrap();
                    writeln!(w).unwrap();
                }
            }
        }
        let measured = g.finish();
        let plain = bb_game::game::simulate_game(seed);
        assert_eq!(measured.digest, plain.digest, "trace changes digest for seed {seed}");
        assert_eq!(measured.log, plain.log, "trace changes log for seed {seed}");
        assert!(measured.complete && plain.complete, "incomplete game {seed}");
        runs += measured.runs.iter().map(|&r| r as u64).sum::<u64>();
        pitches += measured.pitches as u64;
        outs += measured.events.iter().filter(|e| matches!(e, Event::RunnerOut { .. })).count() as u64;
        if games <= 8 {
            println!("digest {seed}: {:?}", measured.digest);
        }
    }
    if let Some(w) = &mut writer {
        w.flush().unwrap();
    }
    println!("{games} games seeds {first}..{}: {bip} BIP; {commitments} commitments; {flagged} flagged; {target_outs} selected runners retired later", first + games - 1);
    println!("all {games} traced/untraced logs and digests identical; {runs} runs, {pitches} pitches, {outs} outs");
    println!("actions {actions:?}; {via} through cutoff; {concession} sure outs conceding unavailable home play; {two_out_third} commitments with two out and runner from third");
    println!("ahead_of_stopped through cutoff {stopped_via}; behind_sure_lead weighed with a continuation {behind_cont}, plays with two outs {behind_two_outs}; non-force carries from beyond 1.2 m {tag_carries}, tagged at that bag {tag_carry_outs}");
    for name in CLASSES {
        let n = counts.get(name).copied().unwrap_or(0);
        println!("{name:<36} {n:>6} {:>9.4}/game", n as f64 / games as f64);
        for ex in examples.get(name).into_iter().flatten() {
            println!("  {ex}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn option(base: u8, force: bool, margin: f64) -> OptionEstimate {
        OptionEstimate {
            runner: base as u16,
            base,
            kind: "throw",
            force,
            runner_at: 5.0,
            ball_at: 5.0 - margin,
            direct_at: Some(4.0),
            via_at: Some(5.0),
            set_at: None,
            receiver: None,
            receiver_at: None,
            p_out: None,
            weight: None,
            value: None,
            through: false,
            distance: 20.0,
            second_out_p: None,
            second_ball_at: None,
        }
    }
    fn choice() -> Choice {
        Choice {
            play: 0,
            t: 3.0,
            until: 3.0,
            ticks: 1,
            holder: 1,
            possession: 2.0,
            from: bb_math::Vec3::ZERO,
            outs: 2,
            inning: 3,
            batting: 0,
            score: [0, 3],
            action: "throw",
            base: 4,
            through: false,
            runners: vec![],
            selected_option: None,
            options: vec![option(4, false, 0.1), option(1, true, 0.4)],
        }
    }
    #[test]
    fn owner_case_and_sensible_concession() {
        // Owner's independent expectation: with two outs, prefer the sure force
        // at first over the close tag home. Early with a lead, concede for that out.
        let mut c = choice();
        assert!(flags(&c).contains(&"two_out_home_over_force"));
        c.outs = 0;
        assert!(flags(&c).contains(&"home_gamble_early_or_leading"));
        c.base = 1;
        c.options[0] = option(4, false, -1.0);
        assert!(flags(&c).is_empty());
    }
    #[test]
    fn throw_into_control_is_not_a_no_chance_out_attempt() {
        let mut c = choice();
        c.options[0] = option(4, false, -1.0);
        assert!(flags(&c).contains(&"no_chance_throw"));
        c.action = "return";
        assert!(flags(&c).is_empty());
    }
}
