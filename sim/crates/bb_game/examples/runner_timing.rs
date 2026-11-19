//! One CSV row per runner on first or second on a scorer-credited outfield single.
//! Run with no arguments for direct game seeds 0–299 and 1000–1299, or
//! `-- <games> <first seed>` for one range, or `-- <seed> <play> <runner>` for
//! one runner's raw 60 Hz trace. To try different leads, append metres and m/s for
//! 0–1 outs and then two outs to the two-argument form. Times are seconds from contact.
//! Actual tracks and an independent, noise-free throw benchmark are both reported.

use bb_agents::running::LEAGUE_TAU_S;
use bb_game::game::{Game, PitchStep, simulate_game_fielders};
use bb_game::live::{
    RunnerStartParams, SecondaryLead, choice_trace,
    runner_trace::{self, Cause, Tick},
    throw_release_point,
};
use bb_game::play::{FielderAttrs, base_pos, run_time};
use bb_math::{Vec3, m, units::mps_to_mph};
use bb_physics::{FlightEnv, throw::solve_throw};
use bb_rules::{Event, Input, OutKind, Touch};
use bb_stats::scorer::Outcome;
use std::ops::Range;

const BAG_M: f64 = 27.432;
const RUNNER_MPS: f64 = 8.2;
const SECURE_RADIUS_M: f64 = 2.0;
const INFIELDER_TRANSFER_FRACTION: f64 = 0.7;

fn track(step: &PitchStep, id: u16) -> &[(f64, Vec3)] {
    step.viz
        .actors
        .iter()
        .find(|(agent, _)| *agent == id)
        .map_or(&[], |(_, samples)| samples.as_slice())
}

fn position_at(samples: &[(f64, Vec3)], t: f64) -> Vec3 {
    let i = samples.partition_point(|(at, _)| *at < t);
    if i == 0 {
        return samples[0].1;
    }
    if i == samples.len() {
        return samples[i - 1].1;
    }
    let (before, a) = samples[i - 1];
    let (after, b) = samples[i];
    a + (b - a) * ((t - before) / (after - before))
}

fn run_from_speed(distance_m: f64, initial_mps: f64) -> f64 {
    if distance_m <= 0.0 {
        return 0.0;
    }
    if initial_mps < 0.0 {
        // A runner going the wrong way must brake and make up his stopping distance.
        const BRAKE_MPS2: f64 = 4.75;
        let brake_s = -initial_mps / BRAKE_MPS2;
        return brake_s
            + run_time(
                distance_m - initial_mps * brake_s / 2.0,
                RUNNER_MPS,
                LEAGUE_TAU_S,
            );
    }
    if initial_mps == 0.0 {
        return run_time(distance_m, RUNNER_MPS, LEAGUE_TAU_S);
    }
    // Invert the same exponential curve as live.rs, with a forward velocity at contact.
    let (mut lo, mut hi) = (0.0, distance_m / RUNNER_MPS + LEAGUE_TAU_S);
    for _ in 0..50 {
        let t = 0.5 * (lo + hi);
        let d = RUNNER_MPS * t
            - (RUNNER_MPS - initial_mps) * LEAGUE_TAU_S * (1.0 - m::exp(-t / LEAGUE_TAU_S));
        if d < distance_m {
            lo = t;
        } else {
            hi = t;
        }
    }
    0.5 * (lo + hi)
}

// Each tick is charged the change in t + uninterrupted ETA to the next bag.
// The differences telescope, including ground lost to a retreat and its recovery.
fn delay_by_cause(ticks: &[Tick], runner: u16, start: u8, split: f64) -> [f64; 7] {
    let target = (start + 1) as f64 * BAG_M;
    let remaining = |tick: &Tick| run_from_speed(target - tick.s, tick.signed_mps);
    let mut ticks = ticks
        .iter()
        .filter(|t| t.runner == runner && t.t <= split + 1e-9)
        .peekable();
    let mut delay = [0.0; 7];
    while let Some(tick) = ticks.next() {
        let next = ticks.peek().map_or(split, |next| next.t + remaining(next));
        let slot = match tick.cause {
            Cause::Read => 0,
            Cause::Air => 1,
            Cause::GroundHold => 2,
            Cause::Traffic => 3,
            Cause::BagHold => 4,
            Cause::OtherHold => 5,
            Cause::Run => 6,
        };
        delay[slot] += next - (tick.t + remaining(tick));
    }
    delay
}

fn raw_trace(seed: u64, play: u32, runner: u16) {
    let mut game = Game::new(seed, true).with_fielders(core::array::from_fn(FielderAttrs::average));
    while let (Some(step), ticks) = runner_trace::capture(|| game.step_pitch()) {
        if step.play != play {
            continue;
        }
        let contact = step
            .record
            .contact_state
            .expect("no contact on this pitch")
            .0;
        assert!(ticks.iter().any(|t| t.runner == runner), "runner absent");
        println!("time_s,path_m,signed_mps,goal,cause,ball_down");
        for t in ticks.iter().filter(|t| t.runner == runner) {
            println!(
                "{:.5},{:.5},{:.5},{},{:?},{}",
                t.t - contact,
                t.s,
                t.signed_mps,
                t.goal,
                t.cause,
                u8::from(t.ball_down)
            );
        }
        return;
    }
    panic!("play {play} absent in seed {seed}");
}

#[derive(Clone, Copy)]
struct ThrowBenchmark {
    exchange_s: f64,
    flight_s: f64,
    cutoff: bool,
}

fn throw_benchmark(
    env: &FlightEnv,
    start: Vec3,
    target: Vec3,
    attrs: FielderAttrs,
) -> ThrowBenchmark {
    let flight = |from, to: Vec3, attrs: FielderAttrs| {
        let origin = throw_release_point(from, to, attrs.throws);
        let distance = m::hypot(to.x - origin.x, to.y - origin.y);
        solve_throw(env, origin.z, 1.3, distance, attrs.arm)
            .map_or(f64::INFINITY, |solution| solution.t_flight)
    };
    // Fixed setup and a ready receiver at the bag. Take the faster of a clean direct
    // throw and a clean two-leg throw through a ready shortstop at the midpoint.
    let middle = (start + target) * 0.5;
    let cutoff = FielderAttrs::average(5);
    let relay_exchange = INFIELDER_TRANSFER_FRACTION * cutoff.transfer;
    let direct = flight(start, target, attrs);
    let via = flight(start, middle, attrs) + flight(middle, target, cutoff);
    if direct <= via + relay_exchange {
        ThrowBenchmark {
            exchange_s: attrs.transfer,
            flight_s: direct,
            cutoff: false,
        }
    } else {
        ThrowBenchmark {
            exchange_s: attrs.transfer + relay_exchange,
            flight_s: via,
            cutoff: true,
        }
    }
}

struct Delivery {
    t: f64,
    release: f64,
    cutoff: bool,
}

fn actual_delivery(
    step: &PitchStep,
    throws: &[&choice_trace::Choice],
    base: u8,
) -> Option<Delivery> {
    let target = base_pos(base);
    for (i, release) in throws.iter().enumerate() {
        // A throw aimed at this base only counts once a fielder controls it near the bag.
        // Follow subsequent releases so a real relay / redirected return is counted.
        if release.base != base || release.through {
            continue;
        }
        let next_release = throws.get(i + 1).map_or(f64::INFINITY, |next| next.t);
        if let Some(t) = step.facts.iter().find_map(|fact| match *fact {
            Input::GloveSecured { t, fielder, x, y }
                if t >= release.t
                    && t <= next_release
                    && fielder != release.holder
                    && m::hypot(x - target.x, y - target.y) <= SECURE_RADIUS_M =>
            {
                Some(t)
            }
            _ => None,
        }) {
            return Some(Delivery {
                t,
                release: release.t,
                cutoff: i > 0,
            });
        }
    }
    None
}

fn release_speed(step: &PitchStep, t: f64) -> Option<f64> {
    // Recorded ball samples are roughly 1/30 s apart. The chord slightly understates
    // muzzle speed due to drag. Avoid chords that cross the release or next possession.
    let flight_end = step
        .viz
        .secured
        .iter()
        .find(|(at, _)| *at > t)
        .map_or(f64::INFINITY, |s| s.0);
    let i = step.viz.batted.partition_point(|p| p[0] < t - 1e-9);
    let a = step.viz.batted.get(i)?;
    let b = step.viz.batted.get(i + 1)?;
    (b[0] < flight_end && b[0] - t <= 0.07).then(|| {
        let d = Vec3::new(b[1] - a[1], b[2] - a[2], b[3] - a[3]);
        mps_to_mph(d.norm() / (b[0] - a[0]))
    })
}

fn movement(samples: &[(f64, Vec3)], base: u8, contact: f64) -> (Option<f64>, Option<f64>) {
    let at_contact = samples[0].1;
    // First recorded displacement in either direction, and first displacement forward.
    // The reported time is the sample at the end of the motion-onset interval.
    let forward = (base_pos(base + 1) - base_pos(base)) / BAG_M;
    let first = samples
        .iter()
        .find(|(_, p)| (*p - at_contact).norm() > 1e-6);
    let first_forward = samples
        .windows(2)
        .find(|pair| (pair[1].1 - pair[0].1).dot(forward) > 1e-6);
    (
        first.map(|p| p.0 - contact),
        first_forward.map(|p| p[1].0 - contact),
    )
}

fn row(
    seed: u64,
    step: &PitchStep,
    choices: &[choice_trace::Choice],
    outs: u8,
    bases: ([Option<u16>; 3], [Option<u16>; 3]),
    first_handling: Option<f64>,
    ticks: &[Tick],
    env: &FlightEnv,
) -> Vec<String> {
    let (runners, bases_after) = bases;
    let Some(contact) = step.facts.iter().find_map(|f| match *f {
        Input::BatContact { t } => Some(t),
        _ => None,
    }) else {
        return Vec::new();
    };
    let Some((secure, fielder)) = step.viz.secured.first().copied() else {
        return Vec::new();
    };
    let pos = (fielder & 0x7f) as usize;
    if !(7..=9).contains(&pos) {
        return Vec::new();
    }
    let Some((_, launch, _)) = step.record.batted else {
        return Vec::new();
    };
    let ball_type = match launch.to_degrees() {
        angle if angle < 10.0 => "GB",
        angle if angle < 25.0 => "LD",
        _ => "FB",
    };
    let fielding = ["LF", "CF", "RF"][pos - 7];
    let attrs = FielderAttrs::average(pos - 1);
    let field_track = track(step, fielder);
    let pickup = position_at(field_track, secure);
    let charge_m: f64 = field_track
        .windows(2)
        .take_while(|pair| pair[1].0 < secure)
        .map(|pair| (pair[1].1 - pair[0].1).norm())
        .sum::<f64>()
        + (pickup
            - field_track
                .iter()
                .rev()
                .find(|(t, _)| *t <= secure)
                .unwrap()
                .1)
            .norm();
    let landing = step.facts.iter().find_map(|f| match *f {
        Input::BallTouched {
            t,
            what: Touch::Ground { .. },
        } if t <= secure => Some(t),
        _ => None,
    });
    let throws: Vec<_> = choices
        .iter()
        .filter(|c| matches!(c.action, "throw" | "return") && c.t >= secure)
        .collect();
    assert_eq!(throws.len(), step.viz.released.len());
    let first_throw = throws.first();
    let (release, first_cutoff, speed) = first_throw.map_or((None, None, None), |c| {
        (Some(c.t), Some(c.through), release_speed(step, c.t))
    });
    let number = |x: Option<f64>| {
        x.filter(|v| v.is_finite())
            .map_or(String::new(), |v| format!("{v:.5}"))
    };
    let elapsed = |x: Option<f64>| number(x.map(|t| t - contact));
    let read = if outs == 2 { 0.25 } else { 0.50 };

    (1..=2)
        .filter_map(|base| {
            let runner = runners[(base - 1) as usize]?;
            let target = base + 2;
            let samples = track(step, runner);
            let samples = &samples[samples.partition_point(|(t, _)| *t < contact)..];
            let lead_m = (samples.first()?.1 - base_pos(base)).norm();
            let (first_move, first_forward) = movement(samples, base, contact);
            let touch = |at| {
                step.facts.iter().find_map(|f| match *f {
                    Input::BaseTouched {
                        t,
                        agent,
                        base,
                        holding_ball: false,
                    } if agent == runner && base == at => Some(t),
                    _ => None,
                })
            };
            let split = touch(base + 1);
            let arrival = touch(target);
            let third = touch(3);
            let safe = step.rules_events.iter().any(|e| match *e {
                Event::RunScored { runner: id } => id == runner && target == 4,
                Event::RunnerAdvanced { runner: id, to, .. } => {
                    id == runner && target == 3 && to >= target
                }
                _ => false,
            });
            let out = step.calls.iter().find(|call| call.runner == runner);
            let extra_path = (base_pos(target) - base_pos(base + 1)) / BAG_M;
            let past = |p: Vec3| (p - base_pos(base + 1)).dot(extra_path) > 0.05;
            let moved_past =
                samples.iter().any(|&(_, p)| past(p)) || out.is_some_and(|call| past(call.at));
            let sent = safe || arrival.is_some() || (split.is_some() && moved_past);
            let outcome = if safe {
                "safe"
            } else if out.is_some() {
                "out"
            } else if arrival.is_some() {
                "nullified"
            } else {
                "stayed"
            };
            let occupied =
                target == 3 && bases_after[2].is_some() && runners[1..].contains(&bases_after[2]);
            let eligible = split.is_some() && !occupied;

            let distance_m = 2.0 * BAG_M - lead_m;
            let from_rest = run_time(distance_m, RUNNER_MPS, LEAGUE_TAU_S);
            let contact_mps = ticks
                .iter()
                .find(|t| t.runner == runner)
                .map_or(0.0, |t| t.signed_mps);
            let coast_m = contact_mps * read;
            let from_read = run_from_speed(distance_m - coast_m, contact_mps);
            let uninterrupted = read + from_read;
            let projected = split.map_or(uninterrupted, |t| t - contact + BAG_M / RUNNER_MPS);
            let projected =
                out.filter(|call| moved_past && past(call.at))
                    .map_or(projected, |call| {
                        let remaining_m = (base_pos(target) - call.at).dot(extra_path).max(0.0);
                        call.t_out - contact + remaining_m / RUNNER_MPS
                    });
            let runner_s = arrival.map_or(projected, |t| t - contact);
            let benchmark = throw_benchmark(env, pickup, base_pos(target), attrs);
            let defense_s = secure - contact + benchmark.exchange_s + benchmark.flight_s;
            let delivery = actual_delivery(step, &throws, target);
            let actual_margin = delivery.as_ref().map(|d| runner_s - (d.t - contact));

            // Illustrative changes, relative to the current branch, with checks/traffic and
            // the defense's clock held fixed. Rolling starts begin at contact.
            let lead_add_m = if outs == 2 { 1.0 } else { 0.5 };
            let initial_mps = if outs == 2 { 4.0 } else { 2.0 };
            let lead_gain = from_rest - run_time(distance_m - lead_add_m, RUNNER_MPS, LEAGUE_TAU_S);
            let rolling_gain = from_rest - run_from_speed(distance_m, initial_mps);
            let joint_gain =
                read + from_rest - run_from_speed(distance_m - lead_add_m, initial_mps);

            let delays = split.map(|t| delay_by_cause(ticks, runner, base, t));
            Some(
                [
                    seed.to_string(),
                    step.play.to_string(),
                    runner.to_string(),
                    outs.to_string(),
                    base.to_string(),
                    fielding.into(),
                    ball_type.into(),
                    format!("{lead_m:.5}"),
                    number(first_move),
                    number(first_forward),
                    elapsed(split),
                    elapsed(third),
                    elapsed(arrival),
                    number(Some(runner_s)),
                    elapsed(landing),
                    elapsed(first_handling),
                    elapsed(Some(secure)),
                    number(Some(charge_m)),
                    elapsed(release),
                    number(release.map(|t| t - secure)),
                    number(speed),
                    first_cutoff.map_or(String::new(), |b| u8::from(b).to_string()),
                    elapsed(delivery.as_ref().map(|d| d.release)),
                    elapsed(delivery.as_ref().map(|d| d.t)),
                    delivery
                        .as_ref()
                        .map_or(String::new(), |d| u8::from(d.cutoff).to_string()),
                    number(actual_margin),
                    u8::from(sent).to_string(),
                    outcome.into(),
                    elapsed(out.map(|call| call.t_out)),
                    u8::from(eligible).to_string(),
                    u8::from(occupied).to_string(),
                    number(Some(read)),
                    number(Some((lead_m + coast_m) / RUNNER_MPS)),
                    number(Some(from_read - (distance_m - coast_m) / RUNNER_MPS)),
                    number(Some(runner_s - uninterrupted)),
                    number(Some(benchmark.exchange_s)),
                    number(Some(benchmark.flight_s)),
                    u8::from(benchmark.cutoff).to_string(),
                    number(Some(defense_s)),
                    number(Some(runner_s - defense_s)),
                    number(Some(lead_gain)),
                    number(Some(rolling_gain)),
                    number(Some(joint_gain)),
                    number(Some(contact_mps)),
                    number(Some(coast_m)),
                ]
                .into_iter()
                .chain((0..7).map(|i| number(delays.map(|d| d[i]))))
                .collect::<Vec<_>>()
                .join(","),
            )
        })
        .collect()
}

#[derive(Default)]
struct Counts {
    // Starting base (first or second), then outs. Count opportunities and advances.
    singles: [[(u64, u64); 3]; 2],
    // Launch below 10°, 10–25°, 25–50° and 50° or more. Catches with runners, plays
    // with appeals, and runners retired on those appeals.
    catches: [(u64, u64, u64); 4],
    // (seed, play, runner, launch °) for each runner doubled off a 25–50° catch.
    doubled_25_50: Vec<(u64, u32, u16, f64)>,
}

impl Counts {
    fn observe(&mut self, seed: u64, step: &PitchStep, outs: u8, runners: [Option<u16>; 3]) {
        if step
            .scorer_ruling
            .is_some_and(|r| r.outcome == Outcome::Hit(1))
        {
            for (i, runner) in runners[..2].iter().copied().enumerate() {
                let Some(id) = runner else { continue };
                let entry = &mut self.singles[i][outs as usize];
                entry.0 += 1;
                let safe = step.rules_events.iter().any(|e| match *e {
                    Event::RunScored { runner } => runner == id,
                    Event::RunnerAdvanced { runner, to, .. } => i == 0 && runner == id && to >= 3,
                    _ => false,
                });
                entry.1 += u64::from(safe);
            }
        }

        if outs >= 2
            || !step
                .rules_events
                .iter()
                .any(|e| matches!(e, Event::Catch { .. }))
        {
            return;
        }
        let Some((_, launch, _)) = step.record.batted else {
            return;
        };
        let angle = launch.to_degrees();
        let doubled_ids: Vec<u16> = step.rules_events.iter().filter_map(|e| match *e {
            Event::RunnerOut { runner, kind: OutKind::Appeal, .. } if runners.contains(&Some(runner)) => Some(runner),
            _ => None,
        }).collect();
        let doubled = doubled_ids.len() as u64;
        let bin = [10.0, 25.0, 50.0].iter().filter(|&&edge| angle >= edge).count();
        if bin == 2 {
            self.doubled_25_50.extend(doubled_ids.iter().map(|&id| (seed, step.play, id, angle)));
        }
        let entry = &mut self.catches[bin];
        entry.0 += 1;
        entry.1 += u64::from(doubled > 0);
        entry.2 += doubled;
    }

    fn add(&mut self, other: Self) {
        for (mine, theirs) in self
            .singles
            .iter_mut()
            .flatten()
            .zip(other.singles.into_iter().flatten())
        {
            mine.0 += theirs.0;
            mine.1 += theirs.1;
        }
        for (mine, theirs) in self.catches.iter_mut().zip(other.catches) {
            mine.0 += theirs.0;
            mine.1 += theirs.1;
            mine.2 += theirs.2;
        }
        self.doubled_25_50.extend(other.doubled_25_50);
    }
}

fn run(seeds: Range<u64>, params: Option<RunnerStartParams>) -> (Vec<String>, Counts) {
    let env = FlightEnv::default_env();
    let mut rows = Vec::new();
    let mut counts = Counts::default();
    for seed in seeds {
        let mut game =
            Game::new(seed, true).with_fielders(core::array::from_fn(FielderAttrs::average));
        if let Some(params) = params {
            game = game.with_runner_start(params);
        }
        let mut singles = Vec::new();
        loop {
            let (outs, runners) = (game.st.outs, game.st.bases);
            if runners.iter().all(Option::is_none) {
                if game.step_pitch().is_none() {
                    break;
                }
                continue;
            }
            let ((step, choices), ticks) =
                runner_trace::capture(|| choice_trace::capture(|| game.step_pitch()));
            let Some(step) = step else { break };
            counts.observe(seed, &step, outs, runners);
            if step
                .scorer_ruling
                .is_some_and(|r| r.outcome == Outcome::Hit(1))
            {
                singles.push((step, choices, outs, (runners, game.st.bases), ticks));
            }
        }
        let result = game.finish();
        let log = bb_schema::log::read(&result.log).unwrap();
        for (step, choices, outs, runners, ticks) in singles {
            let first_handling = log.plays[step.play as usize]
                .events()
                .and_then(|events| events.iter().find_map(|e| e.body_as_scorer_evidence()))
                .filter(|e| e.has_handling())
                .map(|e| e.handling_s());
            rows.extend(row(
                seed,
                &step,
                &choices,
                outs,
                runners,
                first_handling,
                &ticks,
                &env,
            ));
        }
        if seed == 0 && params.is_none() {
            let standard =
                simulate_game_fielders(seed, core::array::from_fn(FielderAttrs::average));
            assert_eq!(standard.digest, result.digest, "tracing changed seed 0");
            let digest: String = result.digest.iter().map(|b| format!("{b:02x}")).collect();
            eprintln!("seed 0 digest {digest}");
        }
    }
    (rows, counts)
}

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if let [seed, play, runner] = args.as_slice() {
        raw_trace(
            seed.parse().unwrap(),
            play.parse().unwrap(),
            runner.parse().unwrap(),
        );
        return;
    }
    let range = |games: &str, first: &str| {
        let first: u64 = first.parse().unwrap();
        first..first + games.parse::<u64>().unwrap()
    };
    let lead = |extra: &str, speed: &str| {
        let extra_m: f64 = extra.parse().unwrap();
        let speed_mps: f64 = speed.parse().unwrap();
        assert!((0.5..=1.5).contains(&extra_m) && (1.0..=6.0).contains(&speed_mps));
        SecondaryLead { extra_m, speed_mps }
    };
    let (ranges, params) = match args.as_slice() {
        [] => (vec![0..300, 1000..1300], None),
        [n, first] => (vec![range(n, first)], None),
        [n, first, d1, v1, d2, v2] => (
            vec![range(n, first)],
            Some(RunnerStartParams {
                fewer_than_two: lead(d1, v1),
                two_out: lead(d2, v2),
            }),
        ),
        _ => panic!(
            "usage: runner_timing [games first_seed [lead01 speed01 lead2 speed2] | seed play runner]"
        ),
    };
    let games: u64 = ranges.iter().map(|r| r.end - r.start).sum();
    let (rows, counts) = std::thread::scope(|scope| {
        let jobs: Vec<_> = ranges
            .into_iter()
            .flat_map(|r| {
                let len = r.end - r.start;
                (0..6).map(move |i| r.start + len * i / 6..r.start + len * (i + 1) / 6)
            })
            .collect();
        // At most six games at a time, including outside the cargo build.
        let mut all = Vec::new();
        let mut counts = Counts::default();
        for batch in jobs.chunks(6) {
            let handles: Vec<_> = batch
                .iter()
                .map(|range| {
                    let range = range.clone();
                    scope.spawn(move || run(range, params))
                })
                .collect();
            for handle in handles {
                let (rows, totals) = handle.join().unwrap();
                all.extend(rows);
                counts.add(totals);
            }
        }
        (all, counts)
    });
    eprintln!("{games} games. All singles (opportunities, advances) by outs 0/1/2:");
    eprintln!("  first to third: {:?}", counts.singles[0]);
    eprintln!("  second to home: {:?}", counts.singles[1]);
    eprintln!("Caught balls with runners and 0–1 out (catches, plays with appeals, doubled off):");
    eprintln!("  launch below 10°: {:?}", counts.catches[0]);
    eprintln!("  launch 10–25°: {:?}", counts.catches[1]);
    eprintln!("  launch 25–50°: {:?}", counts.catches[2]);
    eprintln!("  launch 50° and above: {:?}", counts.catches[3]);
    for (seed, play, runner, angle) in &counts.doubled_25_50 {
        eprintln!("  doubled off at 25–50°: seed {seed} play {play} runner {runner} launch {angle:.1}°");
    }
    println!(concat!(
        "seed,play,runner,outs,start,field,ball,lead_m,move_s,forward_s,split_s,third_s,",
        "actual_runner_s,race_runner_s,land_s,first_touch_s,secure_s,charge_m,first_release_s,",
        "exchange_s,release_mph,first_throw_cutoff,target_release_s,actual_ball_s,actual_cutoff,",
        "actual_margin_s,sent,outcome,out_s,eligible,occupied,read_s,lead_saved_s,accel_cost_s,",
        "extra_runner_s,benchmark_exchange_s,benchmark_flight_s,benchmark_cutoff,",
        "benchmark_ball_s,margin_s,lead_gain_s,rolling_gain_s,joint_gain_s,contact_mps,coast_m,",
        "read_debt_s,air_debt_s,ground_hold_debt_s,traffic_debt_s,bag_hold_debt_s,other_hold_debt_s,run_debt_s"
    ));
    for line in rows {
        println!("{line}");
    }
}
