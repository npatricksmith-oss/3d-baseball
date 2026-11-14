//! Logs written before game rosters existed still read (data-formats §2.4: the roster is an
//! optional part of the header; a log without one gets a box score with no batter lines).
//! The logs here are built by hand, not by the sim, so they cannot pick up a roster from it.

use bb_schema::cfg::{GameConfig, GameConfigArgs, TeamConfig, TeamConfigArgs};
use bb_schema::ev::{self, Event, EventArgs, EventBody, LogHeader, LogHeaderArgs};
use bb_schema::log::{self, LogWriter};
use bb_schema::{ProducerInfo, ProducerInfoArgs};
use bb_stats::box_score;
use flatbuffers::FlatBufferBuilder;

/// A header with no config at all, or (`teams`) a config whose two teams carry a name but
/// no roster, as before the roster field existed.
fn header(teams: bool) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let sv = fbb.create_string("0.1.0");
    let p = ProducerInfo::create(&mut fbb, &ProducerInfoArgs { schema_major: bb_schema::SCHEMA_MAJOR, schema_minor: bb_schema::SCHEMA_MINOR, sim_version: Some(sv), ..Default::default() });
    let config = teams.then(|| {
        let (a, h) = (fbb.create_string("Away"), fbb.create_string("Home"));
        let away = TeamConfig::create(&mut fbb, &TeamConfigArgs { team_id: 1, name: Some(a), ..Default::default() });
        let home = TeamConfig::create(&mut fbb, &TeamConfigArgs { team_id: 2, name: Some(h), ..Default::default() });
        GameConfig::create(&mut fbb, &GameConfigArgs { game_id: 1, game_seed: 5, away: Some(away), home: Some(home), ..Default::default() })
    });
    let h = LogHeader::create(&mut fbb, &LogHeaderArgs { producer: Some(p), game_id: 1, game_seed: 5, config });
    fbb.finish_size_prefixed(h, Some(log::HEADER_ID));
    fbb.finished_data().to_vec()
}

/// One play: the top of the 1st starts; batter 7 strikes out; batter 8 homers and scores
/// himself; the game ends 1–0 after one inning.
fn log(teams: bool) -> Vec<u8> {
    // Each play record holds one plate appearance, as the sim writes them.
    fn play(w: &mut LogWriter, index: u32, build: impl FnOnce(&mut Vec<(EventBody, flatbuffers::WIPOffset<flatbuffers::UnionWIPOffset>)>, &mut FlatBufferBuilder<'_>)) {
        w.write_play(index, |fbb| {
            let mut bodies = Vec::new();
            build(&mut bodies, fbb);
            let evs: Vec<_> = bodies
                .into_iter()
                .enumerate()
                .map(|(i, (body_type, body))| Event::create(fbb, &EventArgs { seq: i as u32, t: i as f64, body_type, body: Some(body) }))
                .collect();
            fbb.create_vector(&evs)
        });
    }
    let mut w = LogWriter::new(&header(teams));
    play(&mut w, 0, |bodies, fbb| {
        let h = ev::HalfInningStarted::create(fbb, &ev::HalfInningStartedArgs { inning: 1, half: ev::HalfInning::Top }).as_union_value();
        bodies.push((EventBody::HalfInningStarted, h));
        let k = ev::PlateAppearanceEnded::create(fbb, &ev::PlateAppearanceEndedArgs { batter: 7, result: ev::PaResult::Strikeout, outs_recorded: 1, ..Default::default() }).as_union_value();
        bodies.push((EventBody::PlateAppearanceEnded, k));
    });
    play(&mut w, 1, |bodies, fbb| {
        let hr = ev::PlateAppearanceEnded::create(fbb, &ev::PlateAppearanceEndedArgs { batter: 8, result: ev::PaResult::HomeRun, runs_scored: 1, ..Default::default() }).as_union_value();
        bodies.push((EventBody::PlateAppearanceEnded, hr));
        let r = ev::RunScored::create(fbb, &ev::RunScoredArgs { runner: 8, ..Default::default() }).as_union_value();
        bodies.push((EventBody::RunScored, r));
        let g = ev::GameEnded::create(fbb, &ev::GameEndedArgs { score_away: 1, score_home: 0, innings: 1, ..Default::default() }).as_union_value();
        bodies.push((EventBody::GameEnded, g));
    });
    w.finish(1, 0).0
}

#[test]
fn a_log_without_rosters_still_reads_and_counts() {
    for teams in [false, true] {
        let b = box_score(&log(teams)).unwrap_or_else(|e| panic!("teams {teams}: {e:?}"));
        // No roster, so no batter lines for either side.
        assert!(b.batting[0].is_empty() && b.batting[1].is_empty(), "teams {teams}: {:?}", b.batting);
        // The team lines come from the events alone: two plate appearances, a strikeout and
        // a home run, one run, in a one-inning game that ended 1–0.
        let t = &b.teams[0];
        assert_eq!((t.pa, t.k, t.hr, t.runs, t.batted), (2, 1, 1, 1, 0), "teams {teams}");
        assert_eq!(b.linescore[0], vec![1]);
        assert_eq!(b.final_score, Some((1, 0)));
        assert_eq!(b.innings, 1);
    }
}
