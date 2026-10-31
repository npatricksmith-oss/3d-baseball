/* C smoke test of the bb_ffi ABI: play a full game and check every timeline blob. */
#include <stdio.h>
#include <string.h>
#include "../include/bb_ffi.h"

int main(void) {
    BbGame *g = bb_game_create(3);
    uint8_t *blob; size_t len; int n = 0;
    while (bb_game_advance(g, &blob, &len) == 1) {
        if (len < 8 || memcmp(blob + 4, "BBTL", 4) != 0) { fprintf(stderr, "bad blob\n"); return 1; }
        bb_blob_free(blob, len);
        n++;
    }
    uint16_t a, h;
    int fin = bb_game_score(g, &a, &h);
    bb_game_free(g);
    printf("C client: %d pitches, final=%d, score %u-%u\n", n, fin, a, h);
    return fin == 1 ? 0 : 1;
}
