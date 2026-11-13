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
typedef struct {
  float ball[3]; uint32_t n_actors; BbActor actors[BB_MAX_ACTORS];
  uint32_t has_bat; float bat_knob[3]; float bat_tip[3]; /* the bat, while someone holds it */
  int32_t holder;      /* agent id of the fielder who has the ball, or -1 */
  float release_in_s;  /* seconds until that holder releases a throw, or -1 if he does not */
} BbFrame;
#define BB_SUMMARY_LEN 96
typedef struct {
  uint32_t play_index; float duration_s;
  uint8_t inning, top, outs, balls, strikes, away_runs, home_runs;
  uint8_t bases; /* runners on after the play: bit 0 first, bit 1 second, bit 2 third */
  float release_speed_mps, exit_velocity_mps, launch_angle_rad;
  float contact_s; /* play time of contact, or -1: show contact measurements only from then */
  float call_s;    /* play time from which the result may be shown (the umpire's call; on contact, the play's end) */
  char summary[BB_SUMMARY_LEN]; /* the play in words, NUL-terminated */
} BbPlayInfo;
/* Agent ids: high byte = team (0 away, 1 home); bit 0x80 = fielder, low 7 bits = position 1-9. */
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
