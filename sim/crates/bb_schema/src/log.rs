//! Event-log container (data-formats §4.1) and determinism digest (§4.4).
//!
//! A log is a sequence of size-prefixed FlatBuffers frames, all with file identifier `BBEV`:
//!
//! ```text
//! LogHeader  PlayRecord(0)  …  PlayRecord(n)  LogFooter
//! ```
//!
//! Frames are told apart by position: the first is the header, the last of a finished log is
//! the footer. `digest_n = BLAKE3(digest_{n−1} ‖ frame_n)` over each PlayRecord frame's bytes
//! as written (including its 4-byte size prefix), with `digest_{−1}` = 32 zero bytes. The
//! header is excluded, so changing only `git_commit` keeps the game digest.

use flatbuffers::{FlatBufferBuilder, WIPOffset};

use crate::ev::{
    Event, LogFooter, LogFooterArgs, LogHeader, PlayDigest, PlayDigestArgs, PlayRecord,
    PlayRecordArgs,
};

pub const FILE_ID: &str = "BBEV";
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

    /// Append an already-built, size-prefixed `PlayRecord` frame.
    pub fn push_play_frame(&mut self, play_index: u32, frame: &[u8]) -> Digest {
        self.out.extend_from_slice(frame);
        self.digest = chain(&self.digest, frame);
        self.play_digests.push((play_index, self.digest));
        self.digest
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
        fbb.finish_size_prefixed(f, Some(FILE_ID));
        self.out.extend_from_slice(fbb.finished_data());
        (self.out, self.digest)
    }
}

#[derive(Debug)]
pub enum LogError {
    Truncated,
    BadIdentifier,
    Invalid(flatbuffers::InvalidFlatbuffer),
    DigestMismatch { play_index: u32 },
}

impl From<flatbuffers::InvalidFlatbuffer> for LogError {
    fn from(e: flatbuffers::InvalidFlatbuffer) -> Self {
        LogError::Invalid(e)
    }
}

/// Split a log into size-prefixed frames (each slice includes its prefix).
pub fn frames(bytes: &[u8]) -> Result<Vec<&[u8]>, LogError> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if i + 4 > bytes.len() {
            return Err(LogError::Truncated);
        }
        let n = u32::from_le_bytes(bytes[i..i + 4].try_into().unwrap()) as usize;
        let end = i + 4 + n;
        if end > bytes.len() {
            return Err(LogError::Truncated);
        }
        let f = &bytes[i..end];
        if !flatbuffers::buffer_has_identifier(f, FILE_ID, true) {
            return Err(LogError::BadIdentifier);
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

/// Parse a finished log, verify every frame, and recompute the digest chain against the footer.
pub fn read(bytes: &[u8]) -> Result<LogView<'_>, LogError> {
    let fr = frames(bytes)?;
    if fr.len() < 2 {
        return Err(LogError::Truncated);
    }
    let header = flatbuffers::size_prefixed_root::<LogHeader>(fr[0])?;
    let footer = flatbuffers::size_prefixed_root::<LogFooter>(fr[fr.len() - 1])?;
    let mut plays = Vec::with_capacity(fr.len() - 2);
    let mut d: Digest = [0; 32];
    let stored = footer.play_digests();
    for (k, f) in fr[1..fr.len() - 1].iter().enumerate() {
        let p = flatbuffers::size_prefixed_root::<PlayRecord>(f)?;
        d = chain(&d, f);
        if let Some(s) = stored {
            let ok = k < s.len() && s.get(k).digest().map(|x| x.bytes() == d).unwrap_or(false);
            if !ok {
                return Err(LogError::DigestMismatch { play_index: p.play_index() });
            }
        }
        plays.push(p);
    }
    if footer.game_digest().map(|g| g.bytes() != d).unwrap_or(true) && !plays.is_empty() {
        return Err(LogError::DigestMismatch { play_index: u32::MAX });
    }
    Ok(LogView { header, plays, footer, game_digest: d })
}
