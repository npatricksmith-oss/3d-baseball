# Data Formats Specification

**Last updated:** 2026-10-31
**Status:** Draft v1. Gates milestone M0 (task M0.5 builds the `.fbs` files from this document).
**Parent:** [PLAN.md](../PLAN.md) §2, §3.1, §4, §10 · **Siblings:** [physics-spec.md](physics-spec.md), [agent-models.md](agent-models.md), `rules-spec.md` (to be written)

This document defines every data format that crosses a boundary: the **event log** (source of truth for stats), the **`PlayTimeline`** (what the client plays back, including agent belief snapshots), **manager commands**, and **game-state snapshots**. It also fixes identifiers, the RNG key layout, the FlatBuffers conventions, versioning rules, and the mapping from the sim's spin vector to Baseball Savant's spin-axis and tilt conventions.

Status tags follow the physics spec: **[S]** sourced, **[D]** derived, **[E]** estimate, plus **[C] Choice**: a design decision made here that the owner may revisit without touching any other decision.

---

## 1. Principles

1. **Two outputs, two jobs.** The event log is small, complete, and permanent. Stats come from it (PLAN §4.3). The timeline is large, visual, and optional. Playback, replays, and explanations come from it (D8, D9). No stat may read a timeline.
2. **Same build + seed + command log ⇒ same game.** Every manager command, human or AI, is recorded in the event log at the play where it took effect. Together with the game seed and the build, that log is enough to re-run a game bit for bit (PLAN §4.2). Replays still use stored timelines, because a new build will drift.
3. **SI units, sim frame, everywhere in stored data.** Field names carry units as suffixes (`_m`, `_mps`, `_s`, `_rad`, `_radps`, `_kg`). Imperial units and Savant conventions appear only in exporters (§9) and UI. The Unreal frame conversion lives only in the client plugin (PLAN §5).
4. **Facts, not prose.** Explanations are stored as typed facts with numbers. The client turns them into sentences. This keeps the sim language-agnostic and lets explanation wording change without a schema change.
5. **The world/agent split shows in the data.** Fields that describe what an agent *believed* are grouped under `belief` or `intent` tables and never mixed into physical-fact tables. A reader can always tell a measurement from an opinion.

---

## 2. Identifiers and Time

### 2.1 Identifiers

| ID | Type | Scope | Definition |
| :--- | :--- | :--- | :--- |
| `PlayerId` | u32 | League, permanent | Assigned by the generator. Never reused |
| `TeamId` | u16 | League, permanent | |
| `GameId` | u32 | League, permanent | Schedule order within a season, plus a season prefix: `season × 100 000 + n` |
| `AgentId` | u16 | One game | `side << 8 \| slot`. `side`: 0 away, 1 home, 2 officials. `slot` 0–39 is the roster slot fixed at game start; 0xFF is the team's manager. Officials: slot 0 home-plate umpire, 1–3 base umpires |
| `PlayIndex` | u32 | One game | Increments for every **play**: each pitch, and each non-pitch action that can change game state (pickoff throw, step-off, balk, mound visit, substitution batch). Starts at 0. Replaces "pitchIndex" in PLAN §4.2's RNG key |
| `PitchNumber` | u32 | One game | Counts pitches only. Stored on pitch events for convenience |
| `EventSeq` | u32 | One game | Strictly increasing across all events in the log |

`AgentId`s are stable for the whole game. A substitute takes a new, previously unused slot, so an `AgentId` never refers to two players. The roster table in `GameStarted` and in each `Substitution` event maps `AgentId → PlayerId`.

**Fielding positions** use the scorer's numbering: P=1, C=2, 1B=3, 2B=4, 3B=5, SS=6, LF=7, CF=8, RF=9, plus DH=10, PH=11, PR=12. **Bases**: 0 home (batter), 1, 2, 3, and 4 = scored.

### 2.2 Time

- **Play time** `t` (seconds) restarts at 0 at the start of each play. For a pitch, `t = 0` is the start of the pitcher's delivery. Release, plate crossing, contact, and every later moment are offsets from that.
- Stored as **f64 in the event log** (exact event times for determinism checks) and **f32 in timelines** (f32 resolves about 4 µs at 30 s, far finer than rendering needs).
- There is no game clock between plays (PLAN §9). Bullpen warm-up uses **elapsed game time**, an f64 total of play durations plus fixed between-play gaps [E: 20 s between pitches, 2 min between half-innings, 30 s per mound visit]. These gaps are rules/game parameters, so warm-up and fatigue never depend on how fast the human watches.

### 2.3 RNG key layout

PLAN §4.2 requires a keyed, counter-based RNG. The **key layout** is fixed here because changing it changes every game. The generator algorithm is chosen in M0.4 and recorded in `CLAUDE.md`.

```
key = (game_seed: u64, play_index: u32, agent_id: u16, purpose: u16, sub_index: u32)
```

`sub_index` separates repeated draws within one play: the observation number for perception, the bounce number for surfaces, the throw number for throws. A runner's advance decisions: decision *n* (from 1) uses 2*n* for its timing error and 2*n* + 1 for its read of a free ball; 1 is the first read at contact. The `purpose` registry is append-only. **Never renumber or reuse a value.**

| Purpose | Value | Used by |
| :--- | :--- | :--- |
| `PitchRelease` | 1 | Pitcher release-angle, speed, spin noise |
| `PitchSelection` | 2 | Sampling from the pitcher's mixed strategy (§8.4 entropy term) |
| `BatterPerception` | 3 | Batter observation noise |
| `SwingExecution` | 4 | Swing-placement and timing noise |
| `FielderPerception` | 5 | Landing/intercept estimate noise |
| `GlovePlacement` | 6 | Capture error |
| `ThrowExecution` | 7 | Throw direction and speed noise |
| `RunnerPerception` | 8 | Jump reads, arrival-versus-throw estimates |
| `SurfaceBounce` | 9 | Bounce-normal tilt (physics spec §9.3, the only world-side use) |
| `AgentPolicy` | 10 | Other sampled agent decisions (manager AI, tie-breaks) |
| `FielderAnticipation` | 11 | A fielder's reads before he reads the ball (agent-models §5.10): sub-index 0, the pitch location before contact; sub-index 1, the bat's angle at contact; sub-index 2, the swing's lead (when the bat started forward against the pitch's arrival) |
| `RosterGeneration` | 12 | Names, uniform numbers and the batting order of a generated game roster (§2.4). Not a world draw: it happens before the first pitch and decides who the players are, never what happens |
| `RosterHands` | 13 | The throwing hand and batting side of a generated player (§2.4). Like `RosterGeneration`, drawn before the first pitch |

The game seed is `derive(season_seed, game_id)` (PLAN §4.2). Because the RNG is counter-based, **no RNG state is ever saved**. Snapshots (§7) need only the seed and the indices.

### 2.4 Game rosters (owner backlog A2; interim until the league generator, M7)

Every game has a **roster** for each side: who the players are. A roster is identity only. It gives no player any ability; attributes stay where they are (average) until the league generator assigns them.

**One team, nine players.** Until substitutions and the designated hitter exist, a side has exactly nine players, one per fielding position 1–9, and the pitcher bats. Each player has:

| Field | Type | Rule |
| :--- | :--- | :--- |
| `player_id` | `PlayerId` | Unique in the game. Without a league, away players are 1–9 and home players 10–18, in position order |
| `given`, `family` | text | From the name pools below. ASCII only for now (C buffers and the client's caption check assume it); diacritics wait for the client's fonts [Proposed] |
| `origin` | name-pool id | Which pool the name came from, kept so that a player's look can match his name later (backlog L2) |
| `number` | 1–99 | Uniform number, unique within the team |
| `position` | 1–9 | Scorer's numbering (§2.1); every position is filled exactly once per team |
| `batting_order` | 1–9 | Unique within the team; the pitcher bats ninth |
| `throws` | `Hand` (left, right) | The hand he throws with; his glove is on the other. Drawn from his position (below) |
| `bats` | left, right or switch | The side of the plate he bats from. A switch hitter stands on the side opposite the pitcher's throwing hand. Drawn from his throwing hand (below) |

**Two agent ids per player (interim).** The code predates this section and gives a player one id while he bats and runs, and another while he fields: the **batting id** `side << 8 | batting_order` and the **fielding id** `side << 8 | 0x80 | position`. Both are RNG key components, so merging them would change every game; that waits for the substitution work, which needs §2.1's roster slots anyway. Until then the roster is the only place that knows the two ids are one man, and it resolves either one.

**Generation.** With no league, the roster is generated from the game seed alone, so the same seed always gives the same players. All draws use `RosterGeneration` (§2.3; the hands use `RosterHands`) at `play_index` 0, and a draw of an index below *n* is `floor(u · n)` for a uniform `u` in (0, 1). Teams are generated away first, players in position order 1–9.

1. **Name.** Stream `agent_id` = the player's fielding id, `sub_index` = 2*a* for attempt *a* = 0, 1, …. Three draws in order: the pool (by the pool weights), the given name's index in that pool, the family name's index. If an earlier player **in the game** (either team) already has that family name, or an earlier teammate that given name, the attempt is rejected and the next one drawn; so a family name identifies one player in a game, and play-by-play can use it alone.
2. **Number.** Same stream id, `sub_index` = 2*a* + 1, one draw: `1 + floor(u · 99)`. Rejected if a teammate has it.
3. **Batting order.** Stream `agent_id` = the team's manager (`side << 8 | 0xFF`), `sub_index` 0. A Fisher–Yates shuffle of the eight position players, listed catcher to right field (positions 2–9): for *i* = 7 down to 1, swap entries *i* and `floor(u · (i + 1))`. The result bats first to eighth; the pitcher bats ninth. With every player average the order carries no meaning yet; the manager AI sets it from ability later (M6).

4. **Hands (owner backlog A1).** Stream `agent_id` = the player's fielding id, purpose `RosterHands`, `sub_index` 0. Two draws in order. *Throwing hand:* left if `u` is below his position's share of left-handed throwers, `THROWS_LEFT` [E]: pitcher 0.27, first base 0.35, left field 0.25, center field 0.20, right field 0.25, and 0 at catcher, second, third and shortstop (a left-hander there has to turn his body to throw across the infield, so teams do not play one). *Batting side:* from his throwing hand [E]: a right-handed thrower bats left if `u` < 0.24, switch if `u` < 0.38, else right; a left-handed thrower bats left if `u` < 0.88, switch if `u` < 0.92, else right. Together these put 32% of position players on the left side, 13% switch and 55% on the right, so by these tables 45% of position players' turns against a right-handed pitcher are taken from the left side (32.4% left plus 12.7% switch) (MLB: 44% in 2012, 49% in 2026, switch hitters batting left included; switch hitters 14% of plate appearances). The names and numbers of every seed are unchanged by this step, because it has its own purpose.

**What the hands do in a game.** The pitcher of the fielding side throws with his hand (a left-hander is the right-hander's mirror image: release point and spin, agent-models §2.2). The batter stands on his side of the plate, a switch hitter on the side opposite that pitcher's hand; his eyes, swing and body are there, and the fielders read his swing as a left- or right-handed one (agent-models §5.10). A fielder releases his throws from his throwing-arm side (agent-models §5.5). Abilities are still average: two players differ only in their hands.

**Name pools [E].** Four pools, each a list of given names and a list of family names, with weights near the make-up of professional baseball: English 0.62, Spanish 0.28, Japanese 0.06, Korean 0.04. No family name appears in two pools. The lists live in `bb_game::roster`; **reordering or editing them changes every generated roster**, so they are append-only in spirit once the owner has seen names he likes. Any full name is a chance pairing of two common names; none is taken from a real player.

**Where the roster goes.**

- `Game` carries it, and the rules state's lineups are built from it.
- **Event log:** the `LogHeader` holds a `GameConfig` whose two `TeamConfig.roster` lists carry every field above (`RosterEntry.agent_id` is the batting id, `field_agent_id` the fielding id). The header is outside the digest (§4.4), so adding rosters changed no stored digest. Statistics read names only from here (PLAN §4.3).
- **Play-by-play:** a line that ends a plate appearance starts with the batter's family name ("Ramirez: Single to left").
- **Timeline:** `PlayTimeline.batter` is the batting id of the batter the pitch was thrown to.
- **C ABI:** `bb_game_roster(game, side, batting_slot)` and `bb_game_player(game, agent_id)` fill a `BbPlayer` (ids, name, number, position, batting order, hands); `BbPlayInfo.batter` gives the batter of a play.

---

## 3. FlatBuffers Conventions

All four formats are FlatBuffers. Schemas live in `sim/schemas/` and are compiled for Rust (sim) and C++ (Unreal client).

| File | Namespace | Root type | `file_identifier` |
| :--- | :--- | :--- | :--- |
| `common.fbs` | `bb` | (none: shared structs, enums, `ProducerInfo`) | |
| `events.fbs` | `bb.ev` | `PlayRecord` (and `LogHeader`, `LogFooter`) | `BBEV` (play frames); `BBEH` header and `BBEF` footer frames (§4.1) |
| `timeline.fbs` | `bb.tl` | `PlayTimeline` | `BBTL` |
| `commands.fbs` | `bb.cmd` | `CommandEnvelope`, `CommandResult` | `BBCM` |
| `snapshot.fbs` | `bb.ss` | `SaveState`, `ViewState` | `BBSS` |
| `config.fbs` | `bb.cfg` | `GameConfig` (rosters, park, rules options, seed) | `BBCF` |

Rules for every schema:
- **Every table field has an explicit `(id: N)`**, so reordering source lines can never change the binary layout.
- **Every enum has explicit values** and starts with `Unknown = 0`.
- **Structs** for fixed small groups: `Vec3d` (3×f64), `Vec3f` (3×f32), `Quatf` (4×f32, x,y,z,w). Event logs use `Vec3d`; timelines use `Vec3f`.
- **No defaults that carry meaning.** A field whose absence and zero differ is a table or uses an explicit `has_` flag.
- **Strings** only for human text (names, park name). Never for keys or types.

Common types (sketch; M0.5 writes the real file):

```fbs
namespace bb;

struct Vec3d { x: double; y: double; z: double; }
struct Vec3f { x: float;  y: float;  z: float;  }
struct Quatf { x: float;  y: float;  z: float;  w: float; }

enum Side : ubyte { Unknown = 0, Away = 1, Home = 2, Officials = 3 }
enum Hand : ubyte { Unknown = 0, Left = 1, Right = 2, Switch = 3 }

table ProducerInfo {
  schema_major: ushort (id: 0);
  schema_minor: ushort (id: 1);
  sim_version: string (id: 2);          // crate semver
  git_commit: string (id: 3);           // "dirty" suffix if uncommitted changes
  physics_params_hash: [ubyte] (id: 4); // BLAKE3 of the canonical parameter set
  rules_version: ushort (id: 5);
  scorer_version: ushort (id: 6);
  belief_tables_hash: [ubyte] (id: 7);  // learned tables in use (PLAN §8.5)
}
```

---

## 4. Event Log

### 4.1 Container

One log per game. A log is a sequence of **size-prefixed** FlatBuffers:

```
LogHeader  PlayRecord(0)  PlayRecord(1)  …  PlayRecord(n)  LogFooter
```

- `LogHeader`: `ProducerInfo`, `GameId`, game seed, and the full `GameConfig` (rosters, lineups, park, rules options). Everything needed to re-run the game except the commands, which are in the plays.
- `PlayRecord`: `play_index` and an ordered vector of `Event`s for that play. One record per play allows streaming, random access by play, and a per-play hash.
- `LogFooter`: final score, the per-play digest list, and the game digest (§4.4).
- **Frame identifiers [C, M0.5]:** header frames use `BBEH`, play frames `BBEV`, footer frames `BBEF`, so a reader never infers a frame's type from its position. A log without a `BBEF` frame is unfinished and is rejected by the finished-log reader. The footer must list exactly one digest per play, in order, with matching `play_index`.
- **Storage:** files use the `.bbev` extension. In a league save (SQLite, M7), each game's log is a row with the whole container as one zstd-compressed blob. Compression is outside the hashed bytes.
- **Size budget [E]:** about 300 bytes per pitch on average, roughly 100 KB per game, roughly 100 MB per 972-game season. A season keeps every event log.

### 4.2 Event envelope

```fbs
namespace bb.ev;

table Event {
  seq: uint (id: 0);         // EventSeq
  t: double (id: 1);         // play time, s
  body_type / body: EventBody (id: 2, 3);  // union
}
```

### 4.3 Event types

Grouped by producer. "Physical" events record what the world did. "Rules" events record what the rules engine decided. "Bookkeeping" events are the scorer's rulings and the manager's commands. The union is append-only (§8).

**Game flow (rules)**

| Event | Key fields |
| :--- | :--- |
| `GameStarted` | roster table (`AgentId → PlayerId`, position, batting order), starting pitchers, park id, rules options |
| `HalfInningStarted` | inning, top/bottom |
| `PlateAppearanceStarted` | batter, pitcher, catcher, base-out state, score, batter height (sets the zone) |
| `PlateAppearanceEnded` | result code (§4.5), outs recorded, runs scored, RBI |
| `HalfInningEnded` | runs, left on base |
| `GameEnded` | final score, innings, reason (regulation, walk-off, extra innings) |

**Pitch (physical, then rules)**

| Event | Key fields |
| :--- | :--- |
| `PitchReleased` | pitcher, `PitchNumber`, pitch type thrown, release `r`/`v`/`ω` (Vec3d, SI), `t_release`. Also the pitcher's **intent**: type, target (x, z) at the plate plane, directive in force |
| `PitchCrossed` | crossing time, position and velocity at the judgment plane (physics spec §10), zone top/bottom used, `in_zone` (geometric), or `bounced_before_plate` with the bounce point |
| `SwingRecorded` | `swung`, `check_swing` and whether it was judged a swing, commit time, swing placement θ actually executed, bat speed, attack angle, and **contact** or **miss**: contact time, offset (vertical, along-axis), u*; on a miss, closest approach. Also `batter_read`: dominant pitch-type hypothesis and its weight at commit (the one belief kept in the log, for perception analytics) |
| `PitchResult` | ball, called strike, swinging strike, foul, foul tip, in play, hit by pitch, hit umpire. New count |

**Ball in play (physical)**

| Event | Key fields |
| :--- | :--- |
| `BattedBall` | exit velocity vector `v′`, spin `ω′`, launch angle, spray angle, projected distance (physics spec §12) |
| `BallContact` | what the ball touched: `Ground` (surface type, point), `Wall` (segment, height), `Pole`, `Base`, `Body` (agent), `Glove` (agent). Point and post-contact velocity |
| `FielderAction` | agent, kind: `Catch`, `Deflection`, `Bobble`, `Pickup`, `Block`, `Miss`. Glove pose error, ball speed at capture |
| `Throw` | thrower, intended receiver and base, release `r`/`v`, transfer time, outcome (caught, short hop, wide, past) |
| `BallDead` | reason (out of play, home run, ground-rule double, time called) |

**Runners and outs (rules)**

| Event | Key fields |
| :--- | :--- |
| `RunnerAdvanced` | runner, from base, to base, arrival time, cause (hit, walk, steal, error, wild pitch, …) |
| `RunnerOut` | runner, base, kind (force, tag, fly-out, strikeout, …), putout agent, assist agents |
| `RunScored` | runner, time, batter credited with the RBI if any |
| `StolenBaseAttempt` | runner, jump time after the pitcher's first move, result |

**Bookkeeping**

| Event | Key fields |
| :--- | :--- |
| `ScorerRuling` | hit kind (1B/2B/3B/HR) or error (agent, kind: fielding/throwing) or fielder's choice, earned/unearned run marks, wild pitch / passed ball, sacrifice. `scorer_version` |
| `ScorerEvidence` | the numbers behind each ruling, such as the counterfactual time margin of an ordinary clean play. Lets a later scorer version re-rule from the log without re-simulating (PLAN §4.3) |

| `ManagerCommandApplied` | the full `Command` (§6) as accepted, the issuing manager, human or AI |
| `Substitution` | out agent, in agent (new slot), new position, batting-order slot; pitching changes included |
| `AlignmentChanged` | team, infield and outfield alignment, each fielder's assigned home position |
| `MoundVisit` | team, visits used |
| `WarmupState` | pitcher, state (cold, warming, ready, overwarmed), pitches thrown in the pen. Emitted on change |
| `PhysicalLoad` | agent, kind (pitch effort, sprint, dive, collision, HBP energy), amount in J or a defined load unit. Feeds fatigue now and injuries later (PLAN §11, D18) |

For each live ball `ScorerEvidence` records the batter and starting runners, the first
handling (including a glove deflection), the first secure possession and its location,
the batter's actual first-base touch when present, and the origin, target, base and time
of each throw. `has_margin` distinguishes a computed reference margin from absent evidence;
`clean_play_margin_s` is actual first touch minus the arrival of the set grade-50 reference
throw. It is computed after the play from the first secure possession, not used by agents.
No margin is asserted when the batter never touched first or the reference cannot be solved.
Runner outs and their destination bases are recorded by `RunnerOut`. Errors are not yet ruled.


### 4.4 Determinism digest

- `digest_n = BLAKE3(digest_{n−1} ‖ bytes(PlayRecord_n))`, with `digest_{−1}` = 32 zero bytes.
- The **game digest** is the last `digest_n`. The header is excluded, so a build that only changes `git_commit` but not behavior still produces the same digest.
- The per-play list in the footer finds the first divergent play in one comparison pass.
- **M0 exit test:** the same game run twice gives the same game digest. Later, a CI job compares digests across platforms (PLAN §4.2 goal, not guarantee).
- This requires the writer to build buffers deterministically: fixed field-write order and no map iteration. The FlatBuffers Rust builder is deterministic for an identical call sequence.

### 4.5 Plate-appearance result codes

`Single, Double, Triple, HomeRun, Walk, IntentionalWalk, HitByPitch, Strikeout, StrikeoutDroppedThirdReached, GroundOut, FlyOut, LineOut, PopOut, ForceOut, FieldersChoice, DoublePlay, TriplePlay, ReachedOnError, SacrificeBunt, SacrificeFly, CatcherInterference`. Append-only like every enum. Accounting tests (`PA = AB + BB + HBP + SF + SH + CI`) are written against these codes.

---

## 5. PlayTimeline

### 5.1 Purpose and lifetime

One `PlayTimeline` per play, returned by the sim when the client advances (PLAN §2 live loop). It contains everything needed to animate the play, overlay telemetry, and explain it, without calling the sim again.

**Retention [C]:**
- Live game: the client keeps all timelines of the current game in memory for instant replay.
- Saved: timelines are written to disk only for games the human managed or watched, and only for plays flagged as highlights (balls in play, strikeouts, scoring plays, any play the human bookmarks). Background games keep event logs only.
- Size budget [E]: about 5 KB per pitch without a ball in play and 50–100 KB per ball in play. About 3–7 MB per fully stored game before zstd.

### 5.2 Structure

```fbs
namespace bb.tl;

table PlayTimeline {
  producer: bb.ProducerInfo (id: 0);
  game_id: uint (id: 1);
  play_index: uint (id: 2);
  duration_s: float (id: 3);
  ball: BallTrack (id: 4);
  bat: BatTrack (id: 5);               // absent if no swing
  actors: [ActorTrack] (id: 6);        // every agent that moved
  markers: [Marker] (id: 7);           // discrete moments
  beliefs: Beliefs (id: 8);            // agent belief snapshots (§5.5)
  facts: [ExplanationFact] (id: 9);    // §5.6
  telemetry: Telemetry (id: 10);       // Statcast-style numbers for overlays
  view_after: bb.ss.ViewState (id: 11);// scoreboard state after the play (§7.2)
}
```

### 5.3 Tracks

**Ball track: Hermite knots, not frames.** The sim already represents flight as cubic Hermite segments between integrator steps (physics spec §5.3). The track stores those knots, so the client reconstructs the exact physical path at any render rate.

```fbs
struct BallKnot { t: float; r: bb.Vec3f; v: bb.Vec3f; }   // 28 bytes
table BallSegment {                 // one ballistic or rolling phase
  mode: BallMode (id: 0);           // Flight, Rolling, Held, Rest
  holder: ushort (id: 1);           // AgentId when Held (ball moves with that glove/hand)
  knots: [BallKnot] (id: 2);
  spin_radps: bb.Vec3f (id: 3);     // at segment start (for visual spin)
}
table BallTrack { segments: [BallSegment] (id: 0); }
```

- A new segment starts at every discontinuity: release, contact, bounce, catch, throw.
- **Knot thinning [C]:** the recorder drops interior knots while the Hermite reconstruction stays within **1 mm** of the full-resolution path. Event knots are never dropped.
- In `Held` segments the ball follows the holder's hand. Knots are still stored, so the client never has to infer it.

**Bat track:** sampled pose every **2 ms** over the swing [E], plus one exact sample at contact (or closest approach).

*(M4, 2026-11-12, owner watch: "the batting outcome is already settled" before the pitch.)* Every pitch's track opens at t = 0 with the same stance sample (bat upright at the back shoulder), swing or take, because a client holds the first sample until the next one, and a swing's first sample (bat laid back) gave the decision away before the release. A swing leaves the stance at most `LOAD_S` = 0.05 s [E] before it starts and never before the batter's decision (`PitchRecord::t_decide`, his commit tick). The executed swing can start less than his latency after the decision (timing noise and the in-flight aim adjustment move it), and then the load is shorter (Codex batter-presentation review #1). A `SwingStart` marker marks the swing's start. Built samples are every 5 ms, not 2.

**Awarded bases** (walk, hit batter, home run, ball out of play): the runners' root tracks hold still until they go, then jog at 6 m/s. On a walk or a hit batter they go at the umpire's call (the pitch's end, when the catcher has it, plus `T_signal` = 0.2 s, rules-spec §1.3) plus `AWARD_REACTION_S` = 0.4 s [E]; on a home run or a ball out of play, 0.4 s after the play ends. Root tracks interpolate linearly, so without the hold knot they drifted toward the base path from the release. A root knot at the same spot as a neighbour (the start or end of a hold) has zero velocity, and so the stationary facing, so it does not turn toward where he goes next.

**When a client may show results** (`bb_timeline_info`): a complete timeline carries the future (telemetry, summary, view after the play), so the client must gate it. `contact_s` is the play time of contact (−1 if none): show contact measurements only from then on. `call_s` is when the result may be shown: on a pitch not hit, the umpire's call (pitch end + `T_signal`); on contact, the end of the play.

```fbs
struct BatSample { t: float; knob: bb.Vec3f; rot: bb.Quatf; }   // bat axis = rot × (+X local)
table BatTrack { agent: ushort (id: 0); model_id: ushort (id: 1); samples: [BatSample] (id: 2); }
```

Bat geometry (`model_id`) comes from the config's bat table, so the client draws the same profile the physics used.

**Actor tracks:** root motion for every agent that moved during the play.

```fbs
struct RootKnot { t: float; p: bb.Vec3f; v: bb.Vec3f; yaw: float; }  // Hermite in p
struct GloveKnot { t: float; p: bb.Vec3f; normal: bb.Vec3f; }
table ActorTrack {
  agent: ushort (id: 0);
  root: [RootKnot] (id: 1);     // 30 Hz [E], thinned like the ball (5 cm tolerance [E])
  glove: [GloveKnot] (id: 2);   // 100 Hz [E], only around reach/capture windows
  body_pose: BodyPose (id: 3);  // capsule set for HBP/blocking (batter, catcher); optional
}
```

- Root velocity is stored, not differentiated by the client. That is what Motion Matching consumes (PLAN §12).
- Agents that did not move are absent. The client keeps them at their `ViewState` positions.

### 5.4 Markers

Discrete moments that animation is **time-warped** to (PLAN §12) and that cameras and audio cue off.

`DeliveryStart, Release, PlateCross, SwingStart, CheckSwingBrake, Contact, Bounce, WallHit, FirstStep, Dive, Catch, Transfer, ThrowRelease, BaseTouch, SlideStart, Tag, RunScores, BallDead, UmpireCall, DiveUp`

```fbs
table Marker { t: float (id: 0); kind: MarkerKind (id: 1); agent: ushort (id: 2); base: ubyte (id: 3); p: bb.Vec3f (id: 4); margin_s: float (id: 5); }
```

`Dive` and `DiveUp` (emitted since 2026-11-13) bracket a fielder's dive: `Dive` when he leaves his feet (`agent`, `p` where he left them), `DiveUp` when he is back on them, the get-up time later (`p` where his body landed: the sim carries him there over the 0.3 s lunge, agent-models §5.3). A play that ends during a dive (the deciding fact can come on the dive's own tick) lasts until the diver is up: his track finishes the lunge and holds at the landing point, and `duration_s` covers the `DiveUp`. This is presentation only. The FFI reports a dive under way in `BbFrame.dives` (`since_s`, `up_in_s`, `dir`), pairing each `Dive` with the fielder's first `DiveUp` strictly after it, so a client can lay the figure out and get it up on the sim's times. At most `BB_MAX_DIVES` (4) are reported at once; any more are left out.

`UmpireCall` (added 2026-11-12) is the umpire signalling an out, at the time he signals it (rules-spec §1.3): `agent` is the runner retired, `base` where (0: the batter at the plate, a caught fly or a strikeout; 4 home), `p` the runner's position, `margin_s` how close the play was (s; 0 bang-bang, `FLT_MAX` for a routine catch). One per out, so a client can advance the outs on the scoreboard as each is called. The play's `duration_s` covers the last call. `bb_timeline_info` copies them into `BbPlayInfo.calls` (`sim/crates/bb_ffi/include/bb_ffi.h`).

### 5.5 Agent belief snapshots

These make causal explanations possible (PLAN §10.3). They are **beliefs**, stored apart from physical tracks (principle 5). Each agent model in `agent-models.md` must emit the matching snapshot.

| Snapshot | When | Contents |
| :--- | :--- | :--- |
| `BatterBelief` | Every observation step (~10 ms) from release to commit, plus one at commit | Per pitch-type hypothesis k: weight `π_k`, predicted plate (x, z, t) mean, 3×3 covariance (6 floats). At commit: aim point (the dominant mode), P(strike), the swing/take decision value, directive in force |
| `BatterAdjust` | Commit to cutoff | Updated aim point and the executed placement θ (shows "he adjusted, but not enough") |
| `PitcherIntent` | Before release | Chosen type and target, the top alternatives with believed values, the pitcher's model of the batter's prior (`PitchCraft` output) |
| `FielderRead` | 20–50 Hz re-plan steps, from reaction to capture or give-up | Estimated landing/intercept point and σ, planned route target, assigned role (primary, backup, cover, cutoff) |
| `ThrowDecision` | At transfer | Chosen target, believed P(out) per option |
| `RunnerDecision` | Each decision point | Option (advance, hold, tag, go back), believed arrival margin versus throw, base-coach signal |
| `ManagerNote` | Between plays | AI manager's believed win probability and the reason for a move (for the human's opponent) |

Storage: the full `BatterBelief` series is about 40 steps × up to 6 hypotheses × 40 bytes ≈ 10 KB. Acceptable for watched plays. Background games never build timelines, so belief recording costs nothing there. **The recorder is a no-op unless a timeline is requested**, and recording must never change sim behavior (tested by digest equality with recording on and off).

### 5.6 Explanation facts

Typed facts computed by the sim at the end of the play from the snapshots and the physical record. The client picks and words them (PLAN §10.3).

| Fact | Fields | Example wording (client side) |
| :--- | :--- | :--- |
| `PitchRead` | believed type, its weight at commit, actual type | "Read fastball (78% at commit)" |
| `SwingMiss` | vertical miss of the swing plane at the ball, cm; timing error, ms | "The slider broke 11 cm under his swing plane" |
| `ContactQuality` | offset from sweet spot, cm; exit velocity; launch angle; contact class | "Topped to short, 71 mph" |
| `FirstStep` | fielder, first-step direction (in/out/lateral), initial read error, m | "First step was in; the depth read was 9 m short" |
| `RouteCost` | fielder, time lost versus an ideal route, s | "cost about 0.4 s" |
| `ReachMiss` | fielder, distance from glove reach to ball, m | "landed 1.2 m past his reach" |
| `ThrowRace` | runner arrival time, ball arrival time, margin | "Safe by 0.08 s" |
| `DirectiveEffect` | directive in force and the decision it changed | "Taking: let a hittable 2-0 fastball go" |

Each fact carries an importance score [E]. The client shows the top one or two.

### 5.7 Telemetry block

Per-play Statcast-style numbers for overlays, all in SI and all already in the event log (the timeline copy is a convenience): release speed, plate speed, spin rate, spin efficiency, spin vector, induced vertical and horizontal break (internal full-flight definition, physics spec §12), plate location, exit velocity, launch and spray angle, contact offset, bat speed, attack angle, projected distance, hang time, apex, sprint speeds, throw speeds, route efficiency. Conversion to mph/ft/rpm and to Savant conventions (§9) happens in the client's display layer.

---

## 6. Manager Commands

### 6.1 Decision points and timing

- The sim stops at a **decision point** before every play. The client (or AI manager) may submit commands; the sim applies them in submission order, then resolves the next play.
- Commands submitted **while a timeline plays** queue for the next decision point (PLAN §2).
- Each command carries `expected_play_index`. A command whose index does not match the current decision point is rejected as stale. This prevents a command typed during a replay from landing on the wrong pitch.
- Some commands are **only legal at certain decision points** (a pitching change between batters, a pinch runner when the runner is on base). The rules engine validates them (`rules-spec.md`, including the three-batter minimum, D17).

### 6.2 Envelope and result

```fbs
namespace bb.cmd;

table CommandEnvelope {
  issuer: ushort (id: 0);              // manager AgentId
  is_ai: bool (id: 1);
  expected_play_index: uint (id: 2);
  client_seq: uint (id: 3);            // echoed back for UI matching
  command_type / command: Command (id: 4, 5);   // union
}

table CommandResult {
  client_seq: uint (id: 0);
  status: Status (id: 1);              // Accepted, Rejected
  reason: RejectReason (id: 2);        // Stale, IllegalNow, ThreeBatterMinimum,
                                       // PlayerUnavailable, DhRule, NotYourTeam, Malformed, …
  detail: string (id: 3);              // debug text, never parsed
}
```

### 6.3 Command catalog

Each directive has a **scope**: `NextPitch`, `PlateAppearance`, or `UntilChanged`. Every command maps to a PLAN §10.1 lever and adjusts **agent policies only**.

| Group | Command | Parameters |
| :--- | :--- | :--- |
| Pitching | `PitchingDirective` | mode: Normal, AttackZone, WorkEdges, PitchAround, InduceGrounder; scope |
| | `IntentionalWalk` | (none; awards first base under current rules) |
| | `PickoffOrStepOff` | base, kind: Pickoff, StepOff, VaryHold |
| | `MoundVisit` | (none) |
| Batting | `BattingDirective` | mode: Normal, Take, Patient, Ambush (+ pitch type and zone region), Protect, Elevate; scope |
| | `Bunt` | kind: Sacrifice, ForHit, SafetySqueeze, SuicideSqueeze |
| | `HitAndRun` | runner(s) |
| Running | `Steal` | runner, kind: Straight, Delayed |
| | `RunningAggressiveness` | team setting, base-coach send threshold [0–1] |
| Defense | `InfieldAlignment` | Normal, CornersIn, AllIn, DoublePlayDepth, GuardLines, or `Custom` per-fielder (x, y) |
| | `OutfieldAlignment` | Shallow, Normal, Deep, NoDoubles, ShadeLeft, ShadeRight, or `Custom` |
| Personnel | `Substitution` | out agent, in player, position, batting slot (pinch hitter, pinch runner, defensive replacement, double switch as one batch) |
| | `PitchingChange` | out pitcher, in pitcher |
| | `BullpenWarmup` | pitcher, action: Start, Stop |
| Control | `DelegateToAi` | scope: None, UntilHalfInningEnd, UntilGameEnd, PlayersOnly. Lets the AI manager run the team while the human skips ahead (PLAN §10.2) [C] |

Pacing (watch every pitch, skip to the end of the inning) is **not** a sim command. It is the client calling advance with a different stop condition (§6.4). Every pacing mode runs the same simulation.

### 6.4 C ABI shape (informative)

The real API is specified in M0 by the spike (`bb_ffi`). It is expected to look like this, with all payloads as FlatBuffers blobs:

| Call | Purpose |
| :--- | :--- |
| `bb_game_create(config_blob) → handle` | New game from `GameConfig` |
| `bb_game_load(savestate_blob) → handle` | Resume from `SaveState` |
| `bb_game_submit(handle, command_blob) → result_blob` | Validate and queue one command |
| `bb_game_advance(handle, stop_condition, want_timeline) → job` | Resolve plays on a worker thread until the stop condition (next play, next ball in play or high-leverage play, end of half-inning, inning N, game end) |
| `bb_job_poll(job) → status`, `bb_job_take(job) → timeline_blobs + view_state` | Non-blocking for the game thread (PLAN §3.4) |
| `bb_game_save(handle) → savestate_blob` | Only at a decision point |
| `bb_blob_free(blob)` | Rust owns every blob it returns |

---

## 7. Game-State Snapshots

### 7.1 `SaveState` (resume)

A complete, sim-internal state at a **decision point**. It is opaque to the client.

- `ProducerInfo`, `GameConfig`, `game_seed`, next `PlayIndex`, `PitchNumber`, `EventSeq`, elapsed game time
- Rules state: inning, half, outs, count, bases (runner `AgentId`s), score, lineups and batting order position, used players, DH status, the current reliever's batters-faced count (D17), mound visits used, pending queued commands
- Agent state: each pitcher's fatigue and pitch count, bullpen warm-up states, each batter's **adaptation memory** for the opposing pitchers (PLAN §8.6), standing manager settings and directives with their scopes, AI manager internal state
- Accumulated `PhysicalLoad` totals per agent
- Learned belief tables are referenced **by hash**, not embedded. Loading with a different table set is an error unless the caller overrides it

There is no RNG state (§2.3). **Required test:** play N plays, save, load, continue; the game digest must equal an uninterrupted run with the same commands. This test catches any state left out of `SaveState`.

Version policy: a save from a different **major** schema version is not loadable. A different sim build may load a save, but determinism against the original run is not promised (D9).

### 7.2 `ViewState` (client display)

What the client needs to draw the scoreboard and HUD **without** reading sim internals. Returned after every play and embedded in each `PlayTimeline`.

- Score by inning, hits, errors; inning, half, outs, count; runners on base; current batter and pitcher; due-up list
- Each team's lineup with positions and the fielders' current standing positions (x, y)
- Per player, **displayed values only**: pitch count, a fatigue indicator band, warm-up state, today's line
- The human's standing directives and alignment, and which commands are legal right now (so the HUD can grey out illegal options)

`ViewState` deliberately contains only what a real manager could know or see. It never exposes hidden attributes or exact fatigue numbers (D1's spirit carried to the UI; scouting uncertainty later).

---

## 8. Versioning

| Change | Version bump | Requirement |
| :--- | :--- | :--- |
| Add a field to the end of a table (new `id`) | minor | Old readers ignore it; new readers handle its absence |
| Add a union member or enum value (appended) | minor | Readers skip unknown union members and treat unknown enum values as `Unknown`; stats code warns once |
| Deprecate a field | minor | Mark `(deprecated)`; never reuse its `id` |
| Remove, reorder, retype, or rename the meaning of a field | **major** | Needs a migration tool or an archived reader |
| Change the **meaning or unit** of a field, even if bytes are identical | **major** | Same |
| Change the RNG key layout or a `purpose` value | **major** | And every stored digest is invalidated |

- `schema_major.schema_minor` is one number pair shared by all six schema files [C]. It is simpler than tracking each file separately and the files evolve together.
- Readers check the `file_identifier` and reject a newer major version with a clear error.
- **Compatibility corpus:** `sim/schemas/compat/` holds one sample file per format from every released minor version. CI parses all of them with the current reader.
- `rules_version`, `scorer_version`, and `physics_params_hash` are separate from schema versions. They identify *behavior*, so they are what tells a reader whether two logs are comparable.

---

## 9. Spin Axis and Savant Mapping

The sim stores spin only as the vector **ω** in the world frame (physics spec §1.3). Spin axis, tilt, and Savant-style movement are **derived for display and export** and never stored as primary data.

### 9.1 Definitions

At release, with release velocity unit vector **v̂**:
- Transverse spin **ω⊥** = **ω** − (**ω**·**v̂**)**v̂**. Gyro spin = **ω**·**v̂**. Spin efficiency = |**ω⊥**| / |**ω**| (physics spec §12).
- **Magnus direction** **m̂** = normalize(**ω⊥** × **v̂**). This is the direction the spin pushes the ball.
- Project **m̂** onto the X–Z plane (the plane the catcher looks through): (m_x, m_z).

### 9.2 Savant `spin_axis` (degrees, 0–360)

Savant's convention: **180° is pure backspin and 0° is pure topspin** [S: Baseball Savant glossary]. A right-handed pitcher's four-seam fastball typically reads about 200–230° [S, used as the check below].

```
spin_axis_deg = (180 + atan2(−m_x, m_z) in degrees) mod 360
```

Checks, for a pitch traveling −Y:
- Pure backspin, **ω** ∥ −X: **m̂** = +Z → (m_x, m_z) = (0, 1) → **180°**.
- Pure topspin (12–6 curveball), **ω** ∥ +X: **m̂** = −Z → **0°**.
- Right-handed pitcher's four-seamer, with backspin plus arm-side run toward third base (−X): **m̂** ≈ (−0.5, 0.87) → **210°**. That matches the typical range.
- A left-handed pitcher's mirror image: (+0.5, 0.87) → **150°**.

### 9.3 Tilt (clock face)

Tilt is the clock-hand direction of **m̂** **as seen from behind the pitcher** (the pitcher's perspective), with 12:00 = pure backspin. A right-handed four-seamer is about 1:00; a left-handed one about 11:00. [E: this is the common public usage and agrees with §9.2; R8 confirms it.]

```
tilt_hours = ((spin_axis_deg − 180) / 30) mod 12      # 0 is shown as 12
display    = h:mm, rounded to 15 minutes
```

*Correction to physics-spec §12:* its row said "as seen by the batter". From the batter's or catcher's view the clock is mirrored (a right-hander's fastball would read 11:00). The pitcher's perspective is the convention, and §12 now points here.

**Required unit test (M1):** encode §9.2's four checks and the matching tilts. Also round-trip: generate **ω** from a (tilt, efficiency, rate, gyro sign) and recover tilt and efficiency.

**Verification item (new R8):** download a small public Savant sample and confirm the sign of `spin_axis` and of `pfx_x` for right- and left-handed four-seamers against §9.2. Our derivation uses the Magnus direction from spin, as Savant's "spin-based" axis does. Savant also publishes an observed (movement-inferred) axis. With seam-shifted wake deferred, the sim's two axes are identical, so differences in real data are expected and are not a bug.

### 9.4 Savant-compatible export

`sim-cli export --savant` writes a CSV for comparison with public data. The column names are Savant's; the conversions happen only here.

| Savant column | From sim | Conversion |
| :--- | :--- | :--- |
| `release_speed` | \|v\| at release | m/s → mph |
| `release_pos_x`, `release_pos_z` | release r.x, r.z | m → ft (Savant uses catcher's perspective, same X sign as the sim) |
| `release_extension` | 18.44 m − release r.y | m → ft |
| `release_spin_rate` | \|ω\| | rad/s → rpm |
| `spin_axis` | §9.2 | degrees |
| `pfx_x`, `pfx_z` | Savant-window movement (physics spec §12, M1 variant) | m → ft; catcher's perspective, so a right-hander's arm-side run is **negative** `pfx_x` |
| `plate_x`, `plate_z` | crossing point at the **middle** of the plate (Savant changed this in 2026 to align with ABS; earlier seasons used the front edge) | m → ft |
| `sz_top`, `sz_bot` | zone bounds | m → ft |
| `vx0, vy0, vz0, ax, ay, az` | constant-acceleration fit to the trajectory, evaluated at y = 50 ft | SI → ft, ft/s, ft/s² [S: Kagan & Nathan 2017 frame] |
| `launch_speed`, `launch_angle` | exit velocity, launch angle | mph, degrees |
| `hit_distance_sc` | projected distance | ft. Both are projections, but Statcast's projection method differs from our unobstructed flight, so compare distributions, not single balls |
| `p_throws`, `stand`, `pitch_type`, `balls`, `strikes`, `outs_when_up`, `inning` | log | Savant codes; pitch types map from the arsenal (FF, SI, SL, CU, CH, FC, ST, SV, FS, KC) |

`plate_x` is evaluated at the plate's midpoint plane, which matches both Statcast from 2026 on and the strike call (physics spec §10). For comparisons with pre-2026 Savant data, use the front-edge crossing, which is also in the log.

---

## 10. Open Items

1. **R8** (§9.3): confirm Savant `spin_axis` and `pfx_x` signs against a real sample. Before M1 golden tests.
2. The between-play time gaps (§2.2) belong in `rules-spec.md` as rules parameters once it exists.
3. ~~Exact contents of each belief snapshot~~ Filled by [agent-models.md §9](agent-models.md).
4. Highlight-retention policy (§5.1) and the explanation-fact importance scores are first guesses. Revisit at M6.
5. The `bb_ffi` call list (§6.4) is confirmed or changed by the M0.8 Unreal spike.
