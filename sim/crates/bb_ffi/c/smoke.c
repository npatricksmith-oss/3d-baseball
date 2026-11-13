/* C smoke test of the bb_ffi ABI: play a full game and check every timeline blob. */
#include <stdio.h>
#include <string.h>
#include "../include/bb_ffi.h"

int main(void) {
    BbGame *g = bb_game_create(3);
    uint8_t *blob; size_t len; int n = 0, calls = 0;
    while (bb_game_advance(g, &blob, &len) == 1) {
        if (len < 8 || memcmp(blob + 4, "BBTL", 4) != 0) { fprintf(stderr, "bad blob\n"); return 1; }
        /* The header's BbPlayInfo layout must match the library's: the umpire's calls land
         * inside the clip, in order, at a base 0-4. */
        BbPlayInfo info;
        if (bb_timeline_info(blob, len, &info) != 1 || info.n_calls > BB_MAX_CALLS) { fprintf(stderr, "bad info\n"); return 1; }
        for (uint32_t i = 0; i < info.n_calls; i++) {
            const BbCall *c = &info.calls[i];
            if (c->t_s <= 0.f || c->t_s > info.duration_s || c->base > 4 || (i > 0 && c->t_s < info.calls[i - 1].t_s)) { fprintf(stderr, "bad call\n"); return 1; }
        }
        calls += info.n_calls;
        bb_blob_free(blob, len);
        n++;
    }
    uint16_t a, h;
    int fin = bb_game_score(g, &a, &h);
    bb_game_free(g);
    printf("C client: %d pitches, %d out calls, final=%d, score %u-%u\n", n, calls, fin, a, h);
    return fin == 1 && calls >= 51 ? 0 : 1;
}
