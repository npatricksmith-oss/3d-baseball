//! A4 regression cases frozen from main 9fdeec1, before any gameplay fix. They
//! replay the contact and public starting situation, not a changed game's prefix.
use bb_game::{
    game::Game,
    live::{
        LiveInput, LiveOutput,
        choice_trace::{Choice, capture},
        simulate_live,
    },
    play::{FielderAttrs, RunnerIn},
};
use bb_math::Vec3;
use bb_observe::Hand;
use bb_physics::{BallState, FlightEnv, ground::Field};
use bb_rules::{Half, Input, PlayState, RulesState, step};
use serde_json::Value;

fn vec3(v: &Value) -> Vec3 {
    Vec3::new(v[0].as_f64().unwrap(), v[1].as_f64().unwrap(), v[2].as_f64().unwrap())
}

fn replay(text: &str) -> (LiveOutput, Vec<Choice>, RulesState) {
    replay_with(text, bb_agents::throwing::ChoiceParams::default())
}

fn replay_with(text: &str, params: bb_agents::throwing::ChoiceParams) -> (LiveOutput, Vec<Choice>, RulesState) {
    let v: Value = serde_json::from_str(text).unwrap();
    let seed = v["seed"].as_u64().unwrap();
    let g = Game::new(seed, false);
    let mut st = g.st.clone();
    st.inning = v["inning"].as_u64().unwrap() as u8;
    st.half = if v["batting"] == 0 { Half::Top } else { Half::Bottom };
    st.outs = v["outs"].as_u64().unwrap() as u8;
    st.balls = v["balls"].as_u64().unwrap() as u8;
    st.strikes = v["strikes"].as_u64().unwrap() as u8;
    st.batter = v["batter"].as_u64().unwrap() as u16;
    st.next_up = core::array::from_fn(|i| v["next_up"][i].as_u64().unwrap() as usize);
    st.bases = core::array::from_fn(|i| v["bases"][i].as_u64().map(|n| n as u16));
    st.runs_by_inning = core::array::from_fn(|i| v["runs_by_inning"][i].as_array().unwrap().iter().map(|n| n.as_u64().unwrap() as u8).collect());
    st.play = PlayState::Dead;
    step(&mut st, &Input::PitchDelivered { t: 0.0 });
    let cross = &v["pitch_crossing"];
    if !cross.is_null() {
        step(&mut st, &Input::PitchCrossed { t: cross["t"].as_f64().unwrap(), in_zone: cross["in_zone"].as_bool().unwrap(), bounced: cross["bounced"].as_bool().unwrap() });
    }
    let tc = v["t_contact"].as_f64().unwrap();
    step(&mut st, &Input::BatContact { t: tc });
    let side = st.half.batting();
    let fattrs = core::array::from_fn(|i| FielderAttrs { throws: g.rosters.teams[1 - side].players[i].throws, throw_choice: params, ..FielderAttrs::average(i) });
    let mut runners = vec![RunnerIn { id: st.batter, base: 0, speed: 8.2 }];
    runners.extend(st.bases.iter().enumerate().filter_map(|(i, id)| id.map(|id| RunnerIn { id, base: i as u8 + 1, speed: 8.2 })));
    let env = FlightEnv::default_env();
    let field = Field::default();
    let inp = LiveInput {
        env: &env,
        field: &field,
        ball: BallState { r: vec3(&v["ball_r"]), v: vec3(&v["ball_v"]), w: vec3(&v["ball_w"]) },
        t_contact: tc,
        runners,
        fielders: core::array::from_fn(|i| ((1 - side) as u16) << 8 | 0x80 | (i as u16 + 1)),
        fattrs,
        batter_box: vec3(&v["batter_box"]),
        bats: if v["bats"] == "R" { Hand::Right } else { Hand::Left },
        pitch_x: v["pitch_x"].as_f64(),
        bat_yaw: v["bat_yaw"].as_f64(),
        swing_lead: v["swing_lead"].as_f64(),
        seed,
        play: v["play"].as_u64().unwrap() as u32,
        record: true,
        tag_up: Default::default(),
    };
    let (out, choices) = capture(|| simulate_live(&inp, &mut st));
    (out, choices, st)
}

#[test]
fn seed_1_play_64_never_projects_the_stopped_third_base_runner_home() {
    // §5.6 and the evidence note: runner 5 stops at third. Original play made a
    // fictitious home attempt at 3.824 s, p(out)=1. There is no visible home leg.
    let (_, choices, _) = replay(include_str!("fixtures/a4/1-64.json"));
    assert!(!choices.is_empty());
    assert!(choices.iter().all(|c| c.action != "throw" || c.base != 4), "{choices:#?}");
}

#[test]
fn seed_23_play_109_takes_a_sure_lead_force_instead_of_automatic_first() {
    // Fourth inning, loaded, one out. A carry to first is an option, which would
    // remove the force at every other base, so the holder must compare the sure
    // lead forces and throw home first for the 3-2-3 / lead-force double play.
    let (out, choices, _) = replay(include_str!("fixtures/a4/23-109.json"));
    let first = &choices[0];
    assert!(!(first.action == "carry" && first.base == 1), "{first:#?}");
    assert!(first.options.iter().any(|o| o.base == 1 && o.kind == "carry" && o.force), "carry first was compared: {first:#?}");
    let chosen = &first.options[first.selected_option.unwrap()];
    assert!(chosen.force && chosen.base >= 2, "{first:#?}");
    // §5.6 note's "sure" screen: at least 0.25 s in hand on the first out.
    assert!(chosen.runner_at - chosen.ball_at >= 0.25, "{chosen:#?}");
    // The feed home has a force at first behind it, so it goes at full effort however
    // unlikely the first baseman reads that second out (here p 0.11; Opus infield-fixes
    // review P2-1: easing it to 0.85 cost the catcher 0.18 s and the double play), and the
    // 3-2-3 is completed.
    assert_eq!(out.effort.first(), Some(&1.0), "feed home at full effort: {:?}", out.effort);
    assert_eq!(out.calls.iter().map(|c| c.base).collect::<Vec<_>>(), vec![4, 1]);
}

#[test]
fn seed_13_play_140_takes_a_sure_lead_force_instead_of_automatic_first() {
    // Tied sixth, loaded, one out. Main's proximity shortcut (frozen 9fdeec1) carried the
    // ball to first, which took the force off every other base, and the go-ahead run
    // scored. The holder must weigh the lead forces. (Dropped with the Savant alignment,
    // when the first baseman held near his bag with the bases loaded and the chopper
    // skipped past him; with him at Standard depth whenever second is occupied it is his
    // ball again, and he throws for a lead force with time in hand. Restored 2026-11-14, Opus
    // infield-fixes review P3-2. What happens after the throw is not asserted: on the
    // infield merge it crossed the plate high and deflected off the catcher.)
    let (_, choices, _) = replay(include_str!("fixtures/a4/13-140.json"));
    let first = &choices[0];
    assert!(!(first.action == "carry" && first.base == 1), "{first:#?}");
    let chosen = &first.options[first.selected_option.unwrap()];
    assert!(chosen.force && chosen.base >= 2, "{first:#?}");
    // §5.6 note's "sure" screen: at least 0.25 s in hand on the first out.
    assert!(chosen.runner_at - chosen.ball_at >= 0.25, "{chosen:#?}");
}

#[test]
fn seed_3_play_140_keeps_the_force_double_play_against_a_single_home_tag() {
    // §5.6: tied sixth, runners first and third, one out: two likely outs (second
    // then first) beat a single home attempt; the throw is weighed with its likely
    // second out. (Since a tag takes 0.25 s and runners start with τ 0.68 s, §6.4,
    // home is no longer the single-out favourite either, so that counterfactual is gone.)
    // (Replaces frozen 9fdeec1 fixture `70-157`, whose 102 mph +11.0° LHB grounder
    // now shoots cleanly between Savant LHB DP SS -5.7° and 2B +19.0° into CF.)
    let (out, choices, _) = replay(include_str!("fixtures/a4/3-140.json"));
    assert_eq!((choices[0].action, choices[0].base), ("throw", 2), "{:#?}", choices[0]);
    assert_eq!(out.calls.iter().map(|c| c.base).collect::<Vec<_>>(), vec![2, 1]);
    let chosen = &choices[0].options[choices[0].selected_option.unwrap()];
    assert!(chosen.second_out_p.is_some_and(|p| p > 0.5), "{chosen:#?}");
}

#[test]
fn seed_17_play_203_does_not_run_at_third_for_a_runner_who_is_not_moving_toward_it() {
    // Main's third baseman ran a tick toward third, 2.56 m away, for the runner from second,
    // who was already back on second with his arrival speed still live. Nothing visible
    // moves toward third: no option there; take first. (Repinned 2026-11-15: runners now look
    // again and turn round physically, so this play no longer has him back on second; he goes
    // on to third, visibly moving that way, and the carry to third is the right play. What
    // the test keeps is the rule: a base is an option only if its runner is visibly moving
    // toward it.)
    let (_, choices, _) = replay(include_str!("fixtures/a4/17-203.json"));
    let c = &choices[0];
    for o in c.options.iter().filter(|o| o.base == 3) {
        let r = c.runners.iter().find(|r| r.id == o.runner).expect("the runner of the option");
        assert!(r.speed > 0.5, "an option at third for a runner moving at {:.2} m/s: {c:#?}", r.speed);
    }
}
