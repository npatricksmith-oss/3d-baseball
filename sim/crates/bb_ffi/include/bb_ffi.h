/* C ABI of the 3D Baseball sim core (data-formats §6.4). Hand-written; keep in sync with
 * sim/crates/bb_ffi/src/lib.rs. All payloads are FlatBuffers blobs owned by the library. */
#ifndef BB_FFI_H
#define BB_FFI_H
#include <stddef.h>
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif

typedef struct BbGame BbGame;

/* The ABI's version: raised whenever a struct below changes its layout. A client compares
 * bb_abi_version() with the BB_ABI_VERSION it was compiled with and refuses a mismatch (an
 * older library lacks the function). 2 (2026-11-23): BbDive gained a runner's dive back.
 * 3 (2026-11-23): BbPlayInfo gained swing_start_s and swing_contact_s; bb_timeline_sample
 * interpolates the bat between the timeline's 5 ms samples. */
#define BB_ABI_VERSION 3
uint32_t bb_abi_version(void);

/* New AI-vs-AI game with average players. */
BbGame *bb_game_create(uint64_t seed);
/* Advance one pitch: 1 = *out_blob holds a PlayTimeline ("BBTL"); 0 = game over; -1 = bad args. */
int32_t bb_game_advance(BbGame *g, uint8_t **out_blob, size_t *out_len);
/* Score: returns 1 if final, 0 if in progress, -1 on bad args. */
int32_t bb_game_score(const BbGame *g, uint16_t *away, uint16_t *home);
/* Sampling a timeline without parsing it (positions in the sim frame, metres: origin at the
 * rear point of home plate, +Y to center field, +Z up, +X to first base). */
typedef struct { uint16_t agent; uint16_t _pad; float pos[3]; float yaw; } BbActor;
#define BB_MAX_ACTORS 16
/* A player off his feet from a dive, from leaving them until he is back on them: a fielder's
 * dive for a ball (base 0), or a runner's dive back to bag `base` (1-3). since_s after he left
 * them, up_in_s before he is up; dir = the dive's direction on the ground (sim frame, unit;
 * zero if he went nowhere). A dive back also has: end_in_s, seconds until his glide ends
 * (negative after), touched = 1 if it ended with his hand on the bag (0: put out in the air or
 * the play over); hand_from and hand_to, where his hand was at take-off and at the glide's end
 * (the glide is at a constant speed: in between, on the line at since_s/(since_s+end_in_s)).
 * All 0 for a fielder. */
typedef struct { uint16_t agent; uint16_t base; float since_s; float up_in_s; float dir[2]; float end_in_s; uint32_t touched; float hand_from[2]; float hand_to[2]; } BbDive;
#define BB_MAX_DIVES 4
typedef struct {
  float ball[3]; uint32_t n_actors; BbActor actors[BB_MAX_ACTORS];
  uint32_t has_bat; float bat_knob[3]; float bat_tip[3]; /* the bat, while someone holds it */
  int32_t holder;      /* agent id of the fielder who has the ball, or -1 */
  float release_in_s;  /* seconds until that holder releases a throw, or -1 if he does not */
  uint32_t n_dives; BbDive dives[BB_MAX_DIVES]; /* players off their feet from a dive */
} BbFrame;
#define BB_SUMMARY_LEN 96
/* One out as the umpire signals it: at t_s (s from release, within a second of the out);
 * margin_s = how close the play was (the runner's time short of the base; 0 bang-bang,
 * very large for a routine catch); base 0 = the batter at the plate (a caught fly or a
 * strikeout), 1-3 a base, 4 home. */
typedef struct { float t_s; float margin_s; uint16_t runner; uint8_t base; uint8_t _pad; } BbCall;
#define BB_MAX_CALLS 4
typedef struct {
  uint32_t play_index; float duration_s;
  uint8_t inning, top, outs, balls, strikes, away_runs, home_runs;
  uint8_t bases; /* runners on after the play: bit 0 first, bit 1 second, bit 2 third */
  float release_speed_mps, exit_velocity_mps, launch_angle_rad;
  float contact_s; /* play time of contact, or -1: show contact measurements only from then */
  float call_s;    /* play time from which the result may be shown (the umpire's call; on contact, the play's end) */
  char summary[BB_SUMMARY_LEN]; /* the play in words, NUL-terminated */
  uint32_t n_calls; BbCall calls[BB_MAX_CALLS]; /* the umpire's out calls, in time order */
  uint16_t batter; uint16_t _pad; /* the batter the pitch was thrown to (his batting id) */
  /* The ball left the playing field (rules-spec 5.8): dead_s = play time it was dead, or -1
   * if it never left; dead_at = where it crossed the wall or fence (the ball's centre, sim frame); dead_reason
   * 0 never left, 1 a throw out of play, 2 home run, 3 ground-rule double, 4 a foul ball
   * out of play (the event log records it as 1).
   * The two do not describe the same instant for a batted ball over a foul-territory wall or into a
   * dugout: it is dead only once it is down in the seats (a fielder at the rail may reach over for it
   * until then, rules-spec 5.3), so dead_s is that landing time (the clip's end), after the
   * ball passed dead_at. Sampling the ball at dead_s gives its place in the seats, not dead_at. For a
   * home run, a ground-rule double and a throw over a wall the two coincide. */
  float dead_s; float dead_at[3]; uint8_t dead_reason;
  /* The pitch thrown (bb.PitchType): 1 four-seam, 2 sinker, 3 cutter, 4 slider, 5 sweeper,
   * 6 slurve, 7 curveball, 8 knuckle curve, 9 changeup, 10 splitter; 0 unknown. */
  uint8_t pitch_type; uint8_t _pad2[2];
  /* On contact, the projected distance (physics-spec 12): from home to where the batted ball
   * would have come down to field level unobstructed, m; 0 without contact. */
  float projected_distance_m;
  /* Where the pitch crossed the judgment plane (the middle of the plate): plate_x_m across
   * (sim x, + toward first base), plate_z_m up from the ground; valid when plate_crossed is 1.
   * The batter's strike zone it was judged against: bottom and top above the ground and
   * half-width across the plate, m (a strike when the ball touches it). pitch_call: 1 ball,
   * 2 called strike, 3 swinging strike, 4 foul, 5 foul tip, 6 in play, 7 hit by pitch; 0 unknown. */
  float plate_x_m, plate_z_m;
  float zone_bottom_m, zone_top_m, zone_half_width_m;
  uint8_t plate_crossed, pitch_call, _pad3[2];
  /* The batter's swing, for presentation (player-visuals 13.6): swing_start_s = play time its
   * start (the timeline's SwingStart marker), swing_contact_s = play time of its nominal contact
   * (the moment its hands and bat line up, within 0.025 s of the actual contact); each -1 if he
   * did not swing, and swing_contact_s -1 too for a timeline recorded before this field. */
  float swing_start_s, swing_contact_s;
} BbPlayInfo;
/* Agent ids: high byte = team (0 away, 1 home); bit 0x80 = fielder, low 7 bits = position 1-9;
 * without bit 0x80, a batter or runner, low 7 bits = place in the batting order 1-9. A player
 * has one id of each kind; bb_game_player resolves either. */
/* Who a player is (identity only): names are NUL-terminated ASCII; position 1 = pitcher ...
 * 9 = right field; hands 1 = left, 2 = right; bats may also be 3, a switch hitter,
 * who stands on the side opposite the pitcher's hand (his box is on the first-base side, sim
 * x > 0, when he bats left). A fielder throws with `throws` and wears his glove on the other. */
#define BB_NAME_LEN 32
typedef struct {
  uint32_t player_id; uint16_t bat_agent, field_agent;
  uint8_t side, number, position, batting_order, bats, throws, _pad[2];
  char given[BB_NAME_LEN], family[BB_NAME_LEN];
} BbPlayer;
/* One player of a side (0 away, 1 home) by place in the batting order (batting_slot 0-8):
 * 1 = *out filled, 0 = no such side or slot, -1 = bad args. */
int32_t bb_game_roster(const BbGame *g, uint32_t side, uint32_t batting_slot, BbPlayer *out);
/* The player behind an agent id of either kind: 1 = *out filled, 0 = not a player, -1 = bad args. */
int32_t bb_game_player(const BbGame *g, uint16_t agent, BbPlayer *out);
/* The box score of the game so far, a pure function of its event log, computed when asked
 * (one pass over the log: ask when it is shown, not every frame). Sides: 0 away, 1 home.
 * through_plays = how many pitches to include (BbPlayInfo.play_index is the number before that
 * pitch; UINT32_MAX = all so far), so a client a pitch behind the sim shows only what it has shown.
 * Counts are capped at 255 per batter, 65535 per side. Names are NUL-terminated ASCII.
 * Fills *out and returns 1; 0 if the log cannot be read; -1 on bad args. */
#define BB_BOX_MAX_BATTERS 12
#define BB_BOX_MAX_INNINGS 30
typedef struct {
  uint16_t agent; uint8_t number, position, batting_order, _pad[3]; /* position 1-9 */
  uint8_t pa, ab, r, h, doubles, triples, hr, rbi, bb, k, hbp, sf, sh, _pad2[3];
  char given[BB_NAME_LEN], family[BB_NAME_LEN];
} BbBatLine;
typedef struct {
  uint16_t pa, ab, r, h, doubles, triples, hr, rbi, bb, k, hbp, sf, sh;
  uint16_t lob;          /* runners left on base at the end of each half-inning so far */
  uint16_t undetermined; /* batted balls the scorer could not rule (not counted as hits) */
  uint16_t _pad;
} BbBoxTotals;
typedef struct {
  uint8_t n_lines;   /* lines[0..n_lines), in batting order */
  uint8_t n_batted;  /* innings this side has batted: runs[] and hits[] used, from first_inning */
  uint8_t _pad[2];
  BbBoxTotals totals;
  BbBatLine lines[BB_BOX_MAX_BATTERS];
  uint8_t runs[BB_BOX_MAX_INNINGS], hits[BB_BOX_MAX_INNINGS];
} BbBoxTeam;
/* A pitcher's line: pitches thrown, counted from the log. agent = the id he fields under. */
#define BB_BOX_MAX_PITCHERS 8
typedef struct {
  uint16_t agent; uint8_t number, _pad; uint16_t pitches, _pad2;
  char given[BB_NAME_LEN], family[BB_NAME_LEN];
} BbPitchLine;
typedef struct {
  uint8_t is_final;     /* 1 once the log holds the end of the game */
  uint8_t innings;      /* innings played or begun */
  uint8_t first_inning; /* inning number of runs[0]: 1 unless the game ran past BB_BOX_MAX_INNINGS */
  uint8_t _pad;
  BbBoxTeam teams[2];
  uint8_t n_pitchers[2]; /* pitchers[side][0..n_pitchers[side]), in the order they first pitched */
  uint8_t _pad2[2];
  BbPitchLine pitchers[2][BB_BOX_MAX_PITCHERS];
} BbBoxScore;
int32_t bb_game_box_score(const BbGame *g, uint32_t through_plays, BbBoxScore *out);
/* The park as the physics has it (bb_physics::ground::Field), for a client to draw. Read-only.
 * The boundary of the playing field is a closed loop of n_wall straight pieces, clockwise seen
 * from above (the field is on the right of each piece): from the left-field pole along the
 * fence to the right-field pole, down the first-base side, across the backstop and up the
 * third-base side. Piece i runs from wall[i].from to wall[(i + 1) % n_wall].from. The fence's
 * curve is cut into chords whose ends are on it. Sim frame, metres. */
typedef struct {
  float from[2];
  float top_m;  /* height of the piece's top above the field; 0 at a dugout's opening */
  uint8_t kind; /* 0 outfield fence, 1 backstop, 2 side wall, 3 stands closing in to a pole, 4 dugout opening */
  uint8_t _pad[3];
} BbWallPiece;
#define BB_PARK_MAX_WALL 96
typedef struct {
  uint32_t n_wall; BbWallPiece wall[BB_PARK_MAX_WALL];
  float poles[2][2];   /* the foot of each foul pole: left field, right field */
  float bases[3][2];   /* first, second, third: the corner of the infield square each marks */
  float rubber_m;      /* front edge of the pitcher's rubber, from the rear point of home along +Y */
  float dirt_radius_m; /* infield dirt within this distance of the rear point of home, grass beyond */
  /* Heights: the outfield fence, the stands' wall in foul ground, the screen over the walls
   * behind home, a dugout's roof (a ball under it goes into the dugout). */
  float fence_height_m, foul_wall_height_m, backstop_height_m, dugout_roof_m;
} BbPark;
/* 1 = *out filled, -1 = bad args. */
int32_t bb_game_park(const BbGame *g, BbPark *out);
/* blob/len must be a readable allocation of that length (normally one from bb_game_advance).
 * Summary of a timeline blob: 1 = ok, -1 = bad args or not a timeline. */
int32_t bb_timeline_info(const uint8_t *blob, size_t len, BbPlayInfo *out);
/* Ball and actors at play time t (s from release, clamped): 1 = ok, -1 = bad args. */
int32_t bb_timeline_sample(const uint8_t *blob, size_t len, float t, BbFrame *out);
/* Release a blob returned by bb_game_advance. */
void bb_blob_free(uint8_t *blob, size_t len);
void bb_game_free(BbGame *g);

#ifdef __cplusplus
}
#endif
#endif
