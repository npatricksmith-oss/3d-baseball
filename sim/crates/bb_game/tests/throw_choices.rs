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
    let mut runners = vec![RunnerIn { id: st.batter, base: 0, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S }];
    runners.extend(st.bases.iter().enumerate().filter_map(|(i, id)| id.map(|id| RunnerIn { id, base: i as u8 + 1, speed: 8.2, tau_s: bb_agents::running::LEAGUE_TAU_S })));
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
        runner_start: Default::default(),
        tag_up: Default::default(), advance: Default::default(),
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
    // catcher goes on to first for the 3-2-3. (Until 2026-11-15 it was completed; the batter
    // is left-handed, and charged his own 26.58 m from the box instead of 27.43 m he beats
    // the relay by about 0.05 s. The out at home stands.)
    assert_eq!(out.effort.first(), Some(&1.0), "feed home at full effort: {:?}", out.effort);
    assert_eq!(out.calls.first().map(|c| c.base), Some(4));
    assert!(choices.get(1).is_some_and(|c| c.action == "throw" && c.base == 1), "{:#?}", choices.get(1));
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

#[test]
fn seed_65147_play_10_an_outfielder_takes_a_grounder_through_the_hole_both_left_to_the_other() {
    // Agent-models §5.7, the owner's saved play (seed 65147 play 10): a chopper off the plate
    // goes through the right side. The centre and right fielders each read it as the other's
    // and both stayed; the right fielder now cuts it off in front of the warning track.
    // (Frozen 2026-11-20 from `4f18c47`, where the game still reached this ball: it was a
    // whole-game replay in live.rs, and the catch-throw review fixes changed earlier plays.)
    let text = include_str!("fixtures/a4/65147-10.json");
    let (out, _, _) = replay(text);
    let contact = serde_json::from_str::<Value>(text).unwrap()["t_contact"].as_f64().unwrap();
    let (t, who, x, y) = out.facts.iter().find_map(|f| match f {
        Input::GloveSecured { t, fielder, x, y } => Some((*t, *fielder, *x, *y)),
        _ => None,
    }).expect("someone fields it");
    assert_eq!(who & 0x7F, 9, "fielder {who} secured it first at ({x:.1}, {y:.1})");
    assert!(x.hypot(y) < 100.0 && t - contact < 6.0, "fielded at ({x:.1}, {y:.1}), {t:.2} s");
}

/// The first glove possession and whether the ball had touched the ground or a wall before it.
fn first_secured(out: &LiveOutput) -> (f64, u16, bool) {
    let (t, who) = out.facts.iter().find_map(|f| match f {
        Input::GloveSecured { t, fielder, .. } => Some((*t, *fielder)),
        _ => None,
    }).expect("someone fields it");
    let down = out.facts.iter().any(|f| matches!(f, Input::BallTouched { t: tt, what: bb_rules::Touch::Ground { .. } | bb_rules::Touch::Wall { .. } } if *tt < t));
    (t, who, down)
}

#[test]
fn seed_155_play_23_the_centre_fielder_setting_up_still_catches_the_deep_fly() {
    // Agent-models §5.5, the set-up behind a fly: a deep fly to left-centre, a runner on
    // second, none out. The centre fielder catches it on main; on `8621f54` he went for a spot
    // behind the catch point at full speed, a late re-read put the catch point under him, he
    // ran on past it toward the new spot, could no longer get back, and the left fielder took
    // it on the bounce (approach note §5–§6). The catch comes first: he is set up only while he
    // can be settled on the spot and still come through in time.
    let (out, _, _) = replay(include_str!("fixtures/a4/155-23.json"));
    let (t, who, down) = first_secured(&out);
    assert!(!down && who & 0x7F == 8, "fielder {} secured it at {t:.2} s, after it came down: {down}", who & 0x7F);
}

#[test]
fn seed_264_play_109_the_left_fielder_still_catches_the_sinking_liner() {
    // Agent-models §5.5, the set-up behind a fly: a sinking liner to left, a runner on second,
    // none out. Main's left fielder catches it at his shoetops; on `8621f54` his first read had
    // it high and deep, he started for a spot behind it, and when the read came in he was a
    // step behind and took it on the short hop, 0.06 s late (approach note §5–§6).
    let (out, _, _) = replay(include_str!("fixtures/a4/264-109.json"));
    let (t, who, down) = first_secured(&out);
    assert!(!down && who & 0x7F == 7, "fielder {} secured it at {t:.2} s, after it came down: {down}", who & 0x7F);
}

/// The first glove possession, in seconds after contact, and by whom.
fn first_taken(text: &str) -> (f64, u16, bool) {
    let v: Value = serde_json::from_str(text).unwrap();
    let (out, _, _) = replay(text);
    let (t, who, down) = first_secured(&out);
    (t - v["t_contact"].as_f64().unwrap(), who & 0x7F, down)
}

#[test]
fn seed_1048_play_193_the_pitcher_takes_the_comebacker_in_front_of_him() {
    // Agent-models §5.2, the approach (Opus approach review P2-3): a comebacker, a runner on
    // first, none out. On main the pitcher steps to his left and takes it 0.65 s after contact in front
    // of him. On `af5825d` he backed away toward a deeper point on the ball's path to take it
    // with a better throw, the ball passed him and he picked it up 3.9 s after contact. A ball
    // coming at him is taken in front of him.
    let (t, who, _) = first_taken(include_str!("fixtures/a4/1048-193.json"));
    assert!(who == 1 && t < 1.0, "fielder {who} first took it {t:.2} s after contact");
}

#[test]
fn seed_136_play_13_the_second_baseman_does_not_let_the_grounder_by() {
    // Agent-models §5.2, the approach (Opus approach review P2-3): a grounder to the second
    // baseman, nobody on, one out. On `af5825d`, paced to meet it exactly at a later hop, he let
    // it pass 1.1 m from him and picked it up 4.1 s after contact. With the plan timed as the
    // movement runs it, he fields it 1.65 s after contact.
    let (t, who, _) = first_taken(include_str!("fixtures/a4/136-13.json"));
    assert!(who == 4 && t < 2.0, "fielder {who} first took it {t:.2} s after contact");
}

#[test]
fn seed_207_play_66_the_left_fielder_catches_the_pop_up() {
    // Agent-models §5.5, the set-up behind a fly (Opus approach review P2-2): a pop-up to
    // shallow left, a runner on third, one out. On `af5825d` the left fielder chose
    // the set-up on his first read, headed for a spot behind a catch point his read had wrong,
    // and came up short; main catches it. He sets up only once his read has settled.
    let (t, who, down) = first_taken(include_str!("fixtures/a4/207-66.json"));
    assert!(!down && who == 7, "fielder {who} secured it {t:.2} s after contact, after it came down: {down}");
}

#[test]
fn seed_1042_play_28_the_right_fielder_catches_the_fly() {
    // Agent-models §5.5, the set-up behind a fly (Opus approach review P2-2): a fly to
    // right-centre, a runner on second, none out. On `af5825d` the right fielder
    // drifted deep for the set-up from 1.7 s on an unsettled read and the ball landed 1.3 m from
    // him; main catches it.
    let (t, who, down) = first_taken(include_str!("fixtures/a4/1042-28.json"));
    assert!(!down && who == 9, "fielder {who} secured it {t:.2} s after contact, after it came down: {down}");
}

/// Seed 51001 play 66, the owner's saved play (calibration note 2026-11-20-saved-plays-51001
/// §1; frozen from main `667762a`): one out, a runner on first, a ground ball to the shortstop,
/// who fields it drifting away from second with the second baseman 4 m from the bag and running
/// to it. Main took the sure out at first (6-3, the runner to second): the shortstop believed
/// the second baseman would get there as from a standing start. From the man's visible motion
/// (agent-models §5.5, the receiver's estimate) he takes the lead runner, the force at second
/// with time to spare (the note's counterfactual: 0.43 s). No double play: the second baseman
/// takes the bag moving away from first.
#[test]
fn seed_51001_play_66_the_shortstop_takes_the_lead_runner_at_second() {
    let (out, choices, _) = replay(include_str!("fixtures/a4/51001-66.json"));
    assert_eq!((choices[0].action, choices[0].base), ("throw", 2), "{:#?}", choices[0]);
    let calls: Vec<_> = out.calls.iter().map(|c| (c.base, c.margin_s)).collect();
    assert_eq!(calls.len(), 1, "{calls:?}");
    assert!(calls[0].0 == 2 && calls[0].1 > 0.3, "the force at second, with time to spare: {calls:?}");
}

/// Seed 51001 play 168, the owner's saved play (calibration note §1; frozen from main
/// `667762a`): one out, a runner on first, a ground ball to the second baseman 7 m from second,
/// the shortstop covering from 13 m. On main the shortstop braked to a stop on the bag
/// (3.0 s after contact), the feed waited for him, and the pivot had no chance at first. A man
/// covering a force base who is late for the throw crosses it (agent-models §5.3, §5.5, the
/// note's Q-A (b)): a 4-6-3 double play (the note's counterfactual: 0.12–0.18 s at first).
#[test]
fn seed_51001_play_168_the_covering_shortstop_turns_the_double_play() {
    let (out, choices, _) = replay(include_str!("fixtures/a4/51001-168.json"));
    assert_eq!((choices[0].action, choices[0].base), ("throw", 2), "{:#?}", choices[0]);
    let calls: Vec<_> = out.calls.iter().map(|c| (c.base, c.margin_s)).collect();
    assert_eq!(calls.iter().map(|c| c.0).collect::<Vec<_>>(), vec![2, 1], "4-6-3: {calls:?}");
    assert!(calls[1].1 > 0.05, "the batter out at first with a margin: {calls:?}");
}

/// Seed 12 play 105 (Opus dp-cover review P1-1; frozen from the branch at `37bec49`): one out,
/// a runner on first, a comebacker to the pitcher, who waits for the second baseman coming
/// across to second from the first-base side. Released to arrive as the man came within 1.5 m
/// of the bag, the feed passed him 1.75 m away and rolled into centre field (the runner took
/// third). Held until it reaches the bag `RECEIVE_MARGIN_S` after he can take it there
/// (agent-models §5.5), the throw is taken by the man it was thrown to, at second.
#[test]
fn seed_12_play_105_the_pitchers_feed_is_taken_by_the_covering_second_baseman() {
    let (out, choices, _) = replay(include_str!("fixtures/a4/12-105.json"));
    let feed = choices.iter().find(|c| c.holder & 0x7f == 1 && c.action == "throw").expect("the pitcher throws");
    assert_eq!(feed.base, 2, "{feed:#?}");
    let receiver = feed.options[feed.selected_option.unwrap()].receiver.expect("a receiver");
    let next = out.facts.iter().find_map(|f| match f {
        Input::GloveSecured { t, fielder, x, y } if *t > feed.t => Some((*fielder, *x, *y)),
        _ => None,
    });
    let (who, x, y) = next.expect("someone takes the throw");
    assert_eq!(who, receiver, "the feed was taken by {who} at ({x:.1}, {y:.1}), not by the receiver {receiver}");
    assert!((x * x + (y - 38.795).powi(2)).sqrt() < 3.0, "taken at ({x:.1}, {y:.1}), not at second");
}

/// The fixture mirrored across the line from home through second (x → −x): the ball, its spin
/// (an axial vector: ω → (ω_x, −ω_y, −ω_z)), the batter's side and the bat; then the ball's
/// position, velocity and spin turned `turn_deg` about the vertical toward +x. The other side's
/// Savant alignment is close to the mirror image (agent-models §5.9: a left-handed batter's
/// second baseman at 148.2 ft / +19.1°, a right-handed batter's shortstop at 148.3 ft / −18.3°),
/// but the bounces' scatter is not mirrored.
fn mirrored(text: &str, turn_deg: f64) -> String {
    let mut v: Value = serde_json::from_str(text).unwrap();
    let neg = |v: &mut Value, k: usize| v[k] = (-v[k].as_f64().unwrap()).into();
    for k in ["ball_r", "ball_v", "batter_box"] {
        neg(&mut v[k], 0);
    }
    neg(&mut v["ball_w"], 1);
    neg(&mut v["ball_w"], 2);
    for k in ["pitch_x", "bat_yaw"] {
        v[k] = (-v[k].as_f64().unwrap()).into();
    }
    v["bats"] = if v["bats"] == "L" { "R" } else { "L" }.into();
    turned(&v.to_string(), turn_deg, None)
}

/// The fixture's ball (position, velocity, spin) turned `turn_deg` about the vertical toward
/// +x (the first-base side), and, with `on_first`, that batting id on first base.
fn turned(text: &str, turn_deg: f64, on_first: Option<u16>) -> String {
    let mut v: Value = serde_json::from_str(text).unwrap();
    let (s, c) = turn_deg.to_radians().sin_cos();
    for k in ["ball_r", "ball_v", "ball_w"] {
        let a = vec3(&v[k]);
        v[k] = serde_json::json!([a.x * c + a.y * s, -a.x * s + a.y * c, a.z]);
    }
    if let Some(id) = on_first {
        v["bases"][0] = id.into();
    }
    v.to_string()
}

/// Replays a fixture with the jobs traced: the play and every tick's jobs from the first read.
fn replay_jobs(text: &str) -> (LiveOutput, Vec<bb_game::live::job_trace::Tick>) {
    bb_game::live::job_trace::take();
    bb_game::live::job_trace::ON.with(|x| x.set(true));
    let (out, _, _) = replay(text);
    bb_game::live::job_trace::ON.with(|x| x.set(false));
    (out, bb_game::live::job_trace::take())
}

/// A ground single through the infield, as on the owner's saved play 102 (below): the middle
/// infielder on the ball's side (`out_man`) goes out as the cutoff man for second from the
/// first call and never stands with no job while the ball is loose; his partner covers second
/// from the first call; the outfielder fields it in front of him, so it is not a deep ball and
/// nobody goes out toward third. Checked against the geometry: when the ball is fielded the
/// cutoff man is out from where he started and on the line from the ball to second, between
/// them (within `CUT_LINE_M` = 5 m, agent-models §5.5).
fn check_ground_single_jobs(text: &str, out_man: usize, cover_man: usize, fielder: u16) {
    const SECOND: (f64, f64) = (0.0, 38.795);
    let (out, ticks) = replay_jobs(text);
    let &(t_fielded, by) = out.secured.first().expect("the ball is fielded");
    assert_eq!(by & 0x7f, fielder, "fielded by {by:#x}");
    let first = &ticks[0];
    assert_eq!((first.job[out_man], first.job[cover_man]), (22, 12), "the first call: {:?}", first.job);
    for k in ticks.iter().filter(|k| k.t < t_fielded) {
        assert_eq!((k.job[out_man], k.job[cover_man]), (22, 12), "at {:.2} s, before the ball is fielded: {:?}", k.t, k.job);
        assert!(!k.job.contains(&23), "a relay toward third at {:.2} s: {:?}", k.t, k.job);
    }
    let at = ticks.iter().rev().find(|k| k.t <= t_fielded).unwrap();
    let ball = out.facts.iter().find_map(|f| match *f {
        Input::GloveSecured { x, y, .. } => Some((x, y)),
        _ => None,
    }).unwrap();
    let p = at.pos[out_man];
    let (lx, ly) = (ball.0 - SECOND.0, ball.1 - SECOND.1);
    let len = (lx * lx + ly * ly).sqrt();
    let along = ((p.x - SECOND.0) * lx + (p.y - SECOND.1) * ly) / len;
    let off = ((p.x - SECOND.0) * ly - (p.y - SECOND.1) * lx).abs() / len;
    assert!(off < 5.0 && along > 0.0 && along < len, "the cutoff man at ({:.1}, {:.1}) is off the line from second to the ball at ({:.1}, {:.1}): {off:.1} m off, {along:.1} m along", p.x, p.y, ball.0, ball.1);
    assert!(p.y.hypot(p.x) > ticks[0].pos[out_man].y.hypot(ticks[0].pos[out_man].x), "he went out: from {:?} to {p:?}", ticks[0].pos[out_man]);
}

/// Seed 51001 play 102, the owner's saved play (calibration note 2026-11-20-saved-plays-51001
/// §3; frozen from main `904c1ea`, a left-handed batter): none out, bases empty, a 98 mph
/// grounder up the middle on the second baseman's side, out of his reach, fielded by the centre
/// fielder at 88 m. On main the first call gave second to the shortstop and the second baseman
/// nothing, so he stood for 0.75 s; then the deep-ball rule sent the shortstop out toward third
/// and second fell back to the second baseman. Now (agent-models §5.7, a ground ball through
/// the infield) the second baseman goes out as the cutoff for second at the first call and the
/// shortstop covers second.
#[test]
fn seed_51001_play_102_the_second_baseman_goes_out_as_the_cutoff_and_the_shortstop_covers() {
    check_ground_single_jobs(include_str!("fixtures/a4/51001-102.json"), 3, 5, 8);
}

/// The mirror image of play 102, on the shortstop's side (a right-handed batter): the shortstop
/// goes out and the second baseman covers second. Turned 3° toward the middle (spray −11° to
/// −8°): the exact mirror bounces differently and the shortstop fields it in the infield.
#[test]
fn mirrored_play_102_the_shortstop_goes_out_and_the_second_baseman_covers() {
    check_ground_single_jobs(&mirrored(include_str!("fixtures/a4/51001-102.json"), 3.0), 5, 3, 8);
}


/// Batting id put on first in the variants below (any of the batting side's ids).
const ON_FIRST: u16 = 5;

/// Jobs from the first call until the ball is first secured: (ticks, who secured it).
fn loose_jobs(text: &str) -> (Vec<bb_game::live::job_trace::Tick>, f64, u16, LiveOutput) {
    let (out, ticks) = replay_jobs(text);
    let &(t, by) = out.secured.first().expect("the ball is fielded");
    (ticks.into_iter().filter(|k| k.t < t).collect(), t, by & 0x7f, out)
}

/// Opus mid-infield review P1-1: play 102 turned 3° toward first with a runner on first (none
/// out, double-play depth). The second baseman goes for it and fields it 51 m out. With a
/// runner on first the shortstop holds second for the force until the ball is plainly through
/// (agent-models §5.7): he covers second at every tick from the first call to the catch and is
/// never out on a cutoff. (Before the fix he went out toward third at 1.74 s, came back,
/// went out again at 2.32 s: second empty for 1.45 s with the runner going.)
#[test]
fn runner_on_first_the_shortstop_holds_second_while_the_second_baseman_fields_it() {
    let (ticks, _, by, _) = loose_jobs(&turned(include_str!("fixtures/a4/51001-102.json"), 3.0, Some(ON_FIRST)));
    assert_eq!(by, 4, "the premise: the second baseman fields it");
    assert!(ticks.iter().any(|k| k.job[3] == 1), "the second baseman goes for it");
    for k in &ticks {
        assert_eq!(k.job[5], 12, "at {:.2} s the shortstop is not on second: {:?}", k.t, k.job);
    }
}

/// Opus mid-infield review P1-1 and P2-2: a comebacker the pitcher fields (play 102 turned 8°
/// toward third, at the mound), with nobody on and with a runner on first. No middle infielder
/// goes out on a cutoff, and second is covered at every tick from the first call to the catch.
#[test]
fn a_comebacker_sends_nobody_out_and_leaves_second_covered() {
    for on_first in [None, Some(ON_FIRST)] {
        let (ticks, _, by, _) = loose_jobs(&turned(include_str!("fixtures/a4/51001-102.json"), -8.0, on_first));
        assert_eq!(by, 1, "the premise: the pitcher fields it ({on_first:?})");
        for k in &ticks {
            assert!(!(20..30).contains(&k.job[3]) && !(20..30).contains(&k.job[5]), "a cutoff at {:.2} s ({on_first:?}): {:?}", k.t, k.job);
            assert!(k.job.contains(&12), "second open at {:.2} s ({on_first:?}): {:?}", k.t, k.job);
        }
    }
}

/// Opus mid-infield review P1-1 (the flicker): over play 102 turned from 14° toward third to
/// 14° toward first, bases empty and a runner on first, no middle infielder goes out on a
/// cutoff, comes off it and goes out again while the ball is loose; and with a runner on first
/// second is never empty while one of them is out on a cutoff.
#[test]
fn the_through_cutoff_does_not_flicker() {
    let base = include_str!("fixtures/a4/51001-102.json");
    for turn in (-7..=7).map(|k| 2.0 * k as f64) {
        for on_first in [None, Some(ON_FIRST)] {
            let (ticks, ..) = loose_jobs(&turned(base, turn, on_first));
            for i in [3, 5] {
                let cut: Vec<bool> = ticks.iter().map(|k| (20..30).contains(&k.job[i])).collect();
                let starts = cut.windows(2).filter(|w| !w[0] && w[1]).count() + usize::from(cut.first() == Some(&true));
                assert!(starts <= 1, "turn {turn}, {on_first:?}: fielder {i} went out {starts} times: {:?}", ticks.iter().map(|k| (k.t, k.job[i])).collect::<Vec<_>>());
            }
            if on_first.is_some() {
                for k in &ticks {
                    let out = (20..30).contains(&k.job[3]) || (20..30).contains(&k.job[5]);
                    assert!(!out || k.job.contains(&12), "turn {turn}: second open with a man on a cutoff at {:.2} s: {:?}", k.t, k.job);
                }
            }
        }
    }
}
