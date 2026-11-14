//! Event-log evidence must reproduce physical facts captured during the live play.
use bb_game::game::Game;
use bb_rules::Input;
use bb_schema::log;

#[test]
fn live_contacts_and_throws_survive_log_round_trip() {
    let mut checked = 0;
    let mut margins = 0;
    let mut throws = 0;
    for seed in [0, 1, 7, 42] {
        let mut game = Game::new(seed, true);
        let mut steps = Vec::new();
        while let Some(step) = game.step_pitch() { steps.push(step); }
        let result = game.finish();
        let log = log::read(&result.log).unwrap();
        for (step, play) in steps.iter().zip(&log.plays) {
            let Some(e) = play.events().unwrap().iter().find_map(|e| e.body_as_scorer_evidence()) else { continue };
            checked += 1;
            let touch = step.facts.iter().find_map(|f| match f { Input::BaseTouched { t, agent, base: 1, holding_ball: false } if *agent == e.batter() => Some(*t), _ => None });
            assert_eq!(e.has_first_touch(), touch.is_some());
            if let Some(t) = touch { assert_eq!(e.first_touch_s(), t); }
            let secured = step.facts.iter().find_map(|f| match f { Input::GloveSecured { t, fielder, x, y } => Some((*t, *fielder, *x, *y)), _ => None });
            assert_eq!(e.has_secured(), secured.is_some());
            if let Some((t, player, x, y)) = secured {
                assert_eq!((e.secured_s(), e.reference_fielder()), (t, player));
                let at = e.secured_at().unwrap(); assert_eq!((at.x(), at.y()), (x, y));
                assert!(e.has_handling()); assert!(e.handling_s() <= t);
            }
            // The batter's arrival is his touch, or his estimated one when the play ended before it.
            assert!(!(e.has_batter_eta() && touch.is_some()));
            if e.has_batter_eta() { assert!(e.batter_eta_s() > 0.0 && e.batter_eta_s().is_finite()); }
            if e.has_margin() { assert!((touch.is_some() || e.has_batter_eta()) && secured.is_some()); assert!(!e.clean_play_margin_s().is_nan()); margins += 1; }
            let released: Vec<_> = e.throws().unwrap().iter().map(|t| {
                let (origin, target) = (t.origin().unwrap(), t.target().unwrap());
                assert!(origin.x().is_finite() && origin.y().is_finite() && origin.z().is_finite());
                assert!(target.x().is_finite() && target.y().is_finite());
                (t.t_release(), t.thrower())
            }).collect();
            assert_eq!(released, step.viz.released);
            throws += released.len();
            assert!(e.runners().unwrap().iter().any(|r| r.agent() == e.batter() && r.base() == 0));
        }
    }
    assert!(checked > 100 && margins > 20 && throws > 50, "{checked} plays {margins} margins {throws} throws");
}
