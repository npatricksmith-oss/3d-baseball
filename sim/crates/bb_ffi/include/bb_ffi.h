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
/* Release a blob returned by bb_game_advance. */
void bb_blob_free(uint8_t *blob, size_t len);
void bb_game_free(BbGame *g);

#ifdef __cplusplus
}
#endif
#endif
