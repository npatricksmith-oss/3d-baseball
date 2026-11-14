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
fn seed_13_play_140_takes_a_sure_lead_force_instead_of_automatic_first() {
    // Tied sixth, loaded, one out. Main's proximity shortcut carried the ball to
    // first (touched at 4.000 s), which took the force off every other base, and the
    // go-ahead run scored. The holder must now weigh the lead forces. (Since main's
    // arm-by-position change the first baseman is 9.1 m from his bag by the time he
    // can throw, and the common comparison finds a 3-6-3 double play that ends the
    // inning; either that or the home force satisfies §5.6.)
    let (_, choices, _) = replay(include_str!("fixtures/a4/13-140.json"));
    let first = &choices[0];
    assert!(!(first.action == "carry" && first.base == 1), "{first:#?}");
    let chosen = &first.options[first.selected_option.unwrap()];
    assert!(chosen.force && chosen.base >= 2, "{first:#?}");
    // §5.6 note's "sure" screen: at least 0.25 s in hand on the first out.
    assert!(chosen.runner_at - chosen.ball_at >= 0.25, "{chosen:#?}");
}

#[test]
fn seed_70_play_157_keeps_the_force_double_play_against_a_single_home_tag() {
    // §5.6: two likely outs (second then first) beat a single home tag. With
    // continuation disabled the unchanged first-out weights instead favour home.
    let (out, choices, _) = replay(include_str!("fixtures/a4/70-157.json"));
    assert_eq!((choices[0].action, choices[0].base), ("carry", 2), "{:#?}", choices[0]);
    assert_eq!(out.calls.iter().map(|c| c.base).collect::<Vec<_>>(), vec![2, 1]);
    let params = bb_agents::throwing::ChoiceParams { continuation_weight: 0.0, ..Default::default() };
    let (_, single_out_choices, _) = replay_with(include_str!("fixtures/a4/70-157.json"), params);
    assert_eq!(single_out_choices[0].base, 4, "single-out destination weights still favour home");
}

#[test]
fn seed_17_play_203_never_carries_to_unforced_third_outside_tag_reach() {
    let (_, choices, _) = replay(include_str!("fixtures/a4/17-203.json"));
    assert_eq!((choices[0].action, choices[0].base), ("throw", 1));
    for c in &choices {
        if c.action == "carry" {
            let o = &c.options[c.selected_option.unwrap()];
            assert!(o.force || o.distance <= 1.2, "outside tag reach an unforced bag is not a carry out");
        }
    }
}
