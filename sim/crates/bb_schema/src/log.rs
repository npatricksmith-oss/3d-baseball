//! Event-log container (data-formats §4.1) and determinism digest (§4.4).
//!
//! A log is a sequence of size-prefixed FlatBuffers frames:
//!
//! ```text
//! LogHeader (BBEH)  PlayRecord(0) (BBEV)  …  PlayRecord(n) (BBEV)  LogFooter (BBEF)
//! ```
//!
//! Each frame type has its own file identifier, so a reader never has to guess a frame's
//! type from its position (a missing footer is detected, not misparsed). `digest_n = BLAKE3(digest_{n−1} ‖ frame_n)` over each PlayRecord frame's bytes
//! as written (including its 4-byte size prefix), with `digest_{−1}` = 32 zero bytes. The
//! header is excluded, so changing only `git_commit` keeps the game digest.

use flatbuffers::{FlatBufferBuilder, WIPOffset};

use crate::ev::{
    Event, LogFooter, LogFooterArgs, LogHeader, PlayDigest, PlayDigestArgs, PlayRecord,
    PlayRecordArgs,
};

/// File identifiers per frame type (data-formats §3, §4.1).
pub const HEADER_ID: &str = "BBEH";
pub const FILE_ID: &str = "BBEV";
pub const FOOTER_ID: &str = "BBEF";

/// Smallest valid size-prefixed frame: prefix, root offset, identifier.
const MIN_FRAME: usize = 12;
pub type Digest = [u8; 32];

pub fn chain(prev: &Digest, frame: &[u8]) -> Digest {
    let mut h = blake3::Hasher::new();
    h.update(prev);
    h.update(frame);
    *h.finalize().as_bytes()
}

/// Streams a game's log into memory, maintaining the digest chain.
pub struct LogWriter {
    out: Vec<u8>,
    digest: Digest,
    play_digests: Vec<(u32, Digest)>,
}

impl LogWriter {
    /// Start a log with an already-finished, size-prefixed `LogHeader` frame.
    pub fn new(header_frame: &[u8]) -> Self {
        let mut out = Vec::with_capacity(64 * 1024);
        out.extend_from_slice(header_frame);
        Self { out, digest: [0; 32], play_digests: Vec::new() }
    }

    /// Append one play. `build` receives a fresh builder and returns the finished events
    /// vector; the writer wraps it in a `PlayRecord` and finishes it size-prefixed.
    pub fn write_play<F>(&mut self, play_index: u32, build: F) -> Digest
    where
        F: for<'b> FnOnce(
            &mut FlatBufferBuilder<'b>,
        ) -> WIPOffset<flatbuffers::Vector<'b, flatbuffers::ForwardsUOffset<Event<'b>>>>,
    {
        let mut fbb = FlatBufferBuilder::with_capacity(1024);
        let events = build(&mut fbb);
        let rec = PlayRecord::create(&mut fbb, &PlayRecordArgs { play_index, events: Some(events) });
        fbb.finish_size_prefixed(rec, Some(FILE_ID));
        self.push_play_frame(play_index, fbb.finished_data())
    }

    /// Append an already-built, size-prefixed `PlayRecord` frame. Panics if the frame is not
    /// a valid `PlayRecord` for `play_index` (a programming error in the producer).
    pub fn push_play_frame(&mut self, play_index: u32, frame: &[u8]) -> Digest {
        let rec = check_frame(frame, FILE_ID)
            .and_then(|f| flatbuffers::size_prefixed_root::<PlayRecord>(f).map_err(LogError::from))
            .expect("valid PlayRecord frame");
        assert_eq!(rec.play_index(), play_index, "PlayRecord index disagrees with the writer");
        self.out.extend_from_slice(frame);
        self.digest = chain(&self.digest, frame);
        self.play_digests.push((play_index, self.digest));
        self.digest
    }

    /// Decode the most recently written play for presentation, without finalizing the log.
    pub fn latest_play(&self) -> Option<PlayRecord<'_>> {
        let frame = frames(&self.out).ok()?.into_iter().last()?;
        if !flatbuffers::buffer_has_identifier(frame, FILE_ID, true) { return None; }
        flatbuffers::size_prefixed_root::<PlayRecord>(frame).ok()
    }

    pub fn digest(&self) -> Digest {
        self.digest
    }

    /// Write the footer and return the whole log.
    pub fn finish(mut self, score_away: u8, score_home: u8) -> (Vec<u8>, Digest) {
        let mut fbb = FlatBufferBuilder::with_capacity(256 + 48 * self.play_digests.len());
        let pds: Vec<_> = self
            .play_digests
            .iter()
            .map(|(i, d)| {
                let dv = fbb.create_vector(d);
                PlayDigest::create(&mut fbb, &PlayDigestArgs { play_index: *i, digest: Some(dv) })
            })
            .collect();
        let pds = fbb.create_vector(&pds);
        let gd = fbb.create_vector(&self.digest);
        let f = LogFooter::create(
            &mut fbb,
            &LogFooterArgs { score_away, score_home, play_digests: Some(pds), game_digest: Some(gd) },
        );
        fbb.finish_size_prefixed(f, Some(FOOTER_ID));
        self.out.extend_from_slice(fbb.finished_data());
        (self.out, self.digest)
    }
}

#[derive(Debug)]
pub enum LogError {
    Truncated,
    BadIdentifier { expected: &'static str },
    MissingFooter,
    Invalid(flatbuffers::InvalidFlatbuffer),
    DigestMismatch { play_index: u32 },
    BadFooter(&'static str),
}

impl From<flatbuffers::InvalidFlatbuffer> for LogError {
    fn from(e: flatbuffers::InvalidFlatbuffer) -> Self {
        LogError::Invalid(e)
    }
}

fn check_frame<'a>(f: &'a [u8], id: &'static str) -> Result<&'a [u8], LogError> {
    if f.len() < MIN_FRAME {
        return Err(LogError::Truncated);
    }
    if !flatbuffers::buffer_has_identifier(f, id, true) {
        return Err(LogError::BadIdentifier { expected: id });
    }
    Ok(f)
}

/// Split a log into size-prefixed frames (each slice includes its prefix). Frame types are
/// not checked here; [`read`] does that.
pub fn frames(bytes: &[u8]) -> Result<Vec<&[u8]>, LogError> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        let pre = bytes.get(i..i.checked_add(4).ok_or(LogError::Truncated)?).ok_or(LogError::Truncated)?;
        let n = u32::from_le_bytes(pre.try_into().unwrap()) as usize;
        let end = i.checked_add(4).and_then(|x| x.checked_add(n)).ok_or(LogError::Truncated)?;
        let f = bytes.get(i..end).ok_or(LogError::Truncated)?;
        if f.len() < MIN_FRAME {
            return Err(LogError::Truncated);
        }
        out.push(f);
        i = end;
    }
    Ok(out)
}

/// A parsed, verified, finished log.
pub struct LogView<'a> {
    pub header: LogHeader<'a>,
    pub plays: Vec<PlayRecord<'a>>,
    pub footer: LogFooter<'a>,
    pub game_digest: Digest,
}

/// Parse a finished log, verify every frame and its type, and check the footer's per-play
/// digest list (exactly one entry per play, matching index and digest) and game digest.
pub fn read(bytes: &[u8]) -> Result<LogView<'_>, LogError> {
    let fr = frames(bytes)?;
    if fr.is_empty() {
        return Err(LogError::Truncated);
    }
    let header = flatbuffers::size_prefixed_root::<LogHeader>(check_frame(fr[0], HEADER_ID)?)?;
    let last = fr[fr.len() - 1];
    if fr.len() < 2 || !flatbuffers::buffer_has_identifier(last, FOOTER_ID, true) {
        return Err(LogError::MissingFooter);
    }
    let footer = flatbuffers::size_prefixed_root::<LogFooter>(last)?;
    let stored = footer.play_digests().ok_or(LogError::BadFooter("no per-play digest list"))?;
    let body = &fr[1..fr.len() - 1];
    if stored.len() != body.len() {
        return Err(LogError::BadFooter("per-play digest count differs from play count"));
    }
    let mut plays = Vec::with_capacity(body.len());
    let mut d: Digest = [0; 32];
    for (k, f) in body.iter().enumerate() {
        let p = flatbuffers::size_prefixed_root::<PlayRecord>(check_frame(f, FILE_ID)?)?;
        d = chain(&d, f);
        let e = stored.get(k);
        let ok = e.play_index() == p.play_index() && e.digest().map(|x| x.bytes() == d).unwrap_or(false);
        if !ok {
            return Err(LogError::DigestMismatch { play_index: p.play_index() });
        }
        plays.push(p);
    }
    match footer.game_digest() {
        Some(g) if g.bytes() == d => {}
        _ => return Err(LogError::DigestMismatch { play_index: u32::MAX }),
    }
    Ok(LogView { header, plays, footer, game_digest: d })
}
