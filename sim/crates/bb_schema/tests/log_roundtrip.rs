//! M0.5: event-log write/read round trip and the determinism digest (data-formats §4.4).

use bb_math::{KeyedRng, Purpose, RngKey};
use bb_schema::ev::{
    Event, EventArgs, EventBody, LogHeader, LogHeaderArgs, PitchReleased, PitchReleasedArgs,
    PitchResult, PitchResultArgs, PitchOutcome,
};
use bb_schema::log::{self, LogWriter};
use bb_schema::{ProducerInfo, ProducerInfoArgs, vec3d};
use flatbuffers::FlatBufferBuilder;

fn header(git_commit: &str, seed: u64) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let gc = fbb.create_string(git_commit);
    let sv = fbb.create_string("0.1.0");
    let p = ProducerInfo::create(
        &mut fbb,
        &ProducerInfoArgs {
            schema_major: bb_schema::SCHEMA_MAJOR,
            schema_minor: bb_schema::SCHEMA_MINOR,
            sim_version: Some(sv),
            git_commit: Some(gc),
            ..Default::default()
        },
    );
    let h = LogHeader::create(
        &mut fbb,
        &LogHeaderArgs { producer: Some(p), game_id: 1, game_seed: seed, config: None },
    );
    fbb.finish_size_prefixed(h, Some(log::HEADER_ID));
    fbb.finished_data().to_vec()
}

/// A fake "game": each play is a pitch whose release state carries keyed-RNG noise.
fn fake_game(seed: u64, git_commit: &str, plays: u32) -> (Vec<u8>, log::Digest) {
    let mut w = LogWriter::new(&header(git_commit, seed));
    let mut seq = 0u32;
    for play in 0..plays {
        let mut rng = KeyedRng::new(RngKey {
            game_seed: seed,
            play_index: play,
            agent_id: 0x0100,
            purpose: Purpose::PitchRelease,
            sub_index: 0,
        });
        let v = [rng.gauss(0.0, 0.3), rng.gauss(-41.0, 0.5), rng.gauss(-1.5, 0.3)];
        w.write_play(play, |fbb| {
            let rv = vec3d([0.0, 16.76, 1.8]);
            let vv = vec3d(v);
            let ww = vec3d([-230.0, 0.0, 0.0]);
            let pr = PitchReleased::create(
                fbb,
                &PitchReleasedArgs {
                    pitcher: 0x0100,
                    pitch_number: play,
                    t_release: 1.2,
                    r: Some(&rv),
                    v: Some(&vv),
                    w: Some(&ww),
                    ..Default::default()
                },
            );
            let e0 = Event::create(
                fbb,
                &EventArgs {
                    seq,
                    t: 1.2,
                    body_type: EventBody::PitchReleased,
                    body: Some(pr.as_union_value()),
                },
            );
            let res = PitchResult::create(
                fbb,
                &PitchResultArgs { outcome: PitchOutcome::Ball, balls: 1, strikes: 0 },
            );
            let e1 = Event::create(
                fbb,
                &EventArgs {
                    seq: seq + 1,
                    t: 1.65,
                    body_type: EventBody::PitchResult,
                    body: Some(res.as_union_value()),
                },
            );
            fbb.create_vector(&[e0, e1])
        });
        seq += 2;
    }
    w.finish(0, 0)
}

#[test]
fn same_seed_same_digest() {
    let (a, da) = fake_game(7, "abc", 300);
    let (b, db) = fake_game(7, "abc", 300);
    assert_eq!(da, db);
    assert_eq!(a, b);
}

#[test]
fn header_is_excluded_from_digest() {
    let (_, da) = fake_game(7, "abc", 50);
    let (_, db) = fake_game(7, "abc-dirty", 50);
    assert_eq!(da, db);
}

#[test]
fn different_seed_different_digest() {
    let (_, da) = fake_game(7, "abc", 50);
    let (_, db) = fake_game(8, "abc", 50);
    assert_ne!(da, db);
}

#[test]
fn round_trip_and_verify() {
    let (bytes, d) = fake_game(11, "abc", 40);
    let v = log::read(&bytes).expect("valid log");
    assert_eq!(v.game_digest, d);
    assert_eq!(v.plays.len(), 40);
    assert_eq!(v.header.game_seed(), 11);
    let ev = v.plays[5].events().unwrap().get(0);
    let pr = ev.body_as_pitch_released().unwrap();
    assert_eq!(pr.pitch_number(), 5);
    assert!(pr.v().unwrap().y() < -38.0);
    // ~ data-formats §4.1 size budget: a pitch-only play stays well under 300 bytes.
    let frames = log::frames(&bytes).unwrap();
    let play_bytes: usize = frames[1..frames.len() - 1].iter().map(|f| f.len()).sum();
    assert!(play_bytes / 40 < 300, "{} bytes/play", play_bytes / 40);
}

#[test]
fn tampering_is_detected() {
    let (mut bytes, _) = fake_game(11, "abc", 10);
    let frames = log::frames(&bytes).unwrap();
    // Flip a byte inside the third play's payload (after header + 2 plays).
    let off: usize = frames[..3].iter().map(|f| f.len()).sum::<usize>() + 40;
    bytes[off] ^= 0x01;
    assert!(log::read(&bytes).is_err());
}

#[test]
fn missing_footer_is_rejected() {
    let (bytes, _) = fake_game(3, "abc", 2);
    let frames = log::frames(&bytes).unwrap();
    let cut: usize = frames[..frames.len() - 1].iter().map(|f| f.len()).sum();
    assert!(matches!(log::read(&bytes[..cut]), Err(log::LogError::MissingFooter)));
    // Header plus a single play and no footer: the play must not be taken for a footer.
    let cut2: usize = frames[..2].iter().map(|f| f.len()).sum();
    assert!(matches!(log::read(&bytes[..cut2]), Err(log::LogError::MissingFooter)));
}

#[test]
fn zero_play_log_verifies_digest() {
    let (bytes, d) = fake_game(3, "abc", 0);
    assert_eq!(d, [0u8; 32]);
    let v = log::read(&bytes).unwrap();
    assert!(v.plays.is_empty());
}

#[test]
fn short_and_oversized_frames_error_without_panicking() {
    for n in 0..12u32 {
        let mut b = n.to_le_bytes().to_vec();
        b.extend(std::iter::repeat_n(0u8, n as usize));
        assert!(log::read(&b).is_err(), "len {n}");
    }
    let mut b = u32::MAX.to_le_bytes().to_vec();
    b.extend([0u8; 16]);
    assert!(log::read(&b).is_err());
    assert!(log::read(&[1, 2]).is_err());
}

#[test]
#[should_panic(expected = "disagrees")]
fn writer_rejects_misindexed_frame() {
    let mut w = LogWriter::new(&header("abc", 1));
    let mut fbb = FlatBufferBuilder::new();
    let ev = fbb.create_vector::<flatbuffers::WIPOffset<Event>>(&[]);
    let rec = bb_schema::ev::PlayRecord::create(&mut fbb, &bb_schema::ev::PlayRecordArgs { play_index: 0, events: Some(ev) });
    fbb.finish_size_prefixed(rec, Some(log::FILE_ID));
    w.push_play_frame(99, fbb.finished_data());
}
