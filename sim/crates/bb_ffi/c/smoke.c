/* C smoke test of the bb_ffi ABI: play a full game and check every timeline blob. */
#include <stdio.h>
#include <string.h>
#include "../include/bb_ffi.h"

int main(void) {
    BbGame *g = bb_game_create(3);
    uint8_t *blob; size_t len; int n = 0, calls = 0, dead = 0;
    /* The header's BbPlayer layout must match the library's: both lineups read back in
     * order with printable names, and each player is found again under both of his ids. */
    for (uint32_t side = 0; side < 2; side++) {
        for (uint32_t slot = 0; slot < 9; slot++) {
            BbPlayer p, q;
            if (bb_game_roster(g, side, slot, &p) != 1 || p.side != side || p.batting_order != slot + 1 || p.position < 1 || p.position > 9
                || p.number < 1 || p.number > 99 || strlen(p.given) == 0 || strlen(p.family) == 0 || strlen(p.family) >= BB_NAME_LEN) { fprintf(stderr, "bad roster\n"); return 1; }
            if (p.bat_agent != ((side << 8) | (slot + 1)) || p.field_agent != ((side << 8) | 0x80 | p.position)) { fprintf(stderr, "bad agent ids\n"); return 1; }
            if (bb_game_player(g, p.bat_agent, &q) != 1 || memcmp(&p, &q, sizeof p) != 0 || bb_game_player(g, p.field_agent, &q) != 1 || memcmp(&p, &q, sizeof p) != 0) { fprintf(stderr, "bad lookup\n"); return 1; }
            if (slot == 0) printf("C client: %s leads off with #%u %s %s\n", side ? "home" : "away", p.number, p.given, p.family);
        }
    }
    { BbPlayer p; if (bb_game_roster(g, 2, 0, &p) != 0 || bb_game_roster(g, 0, 9, &p) != 0 || bb_game_player(g, 0x0200, &p) != 0) { fprintf(stderr, "bad miss\n"); return 1; } }
    while (bb_game_advance(g, &blob, &len) == 1) {
        if (len < 8 || memcmp(blob + 4, "BBTL", 4) != 0) { fprintf(stderr, "bad blob\n"); return 1; }
        /* The header's BbPlayInfo layout must match the library's: the umpire's calls land
         * inside the clip, in order, at a base 0-4. */
        BbPlayInfo info;
        if (bb_timeline_info(blob, len, &info) != 1 || info.n_calls > BB_MAX_CALLS) { fprintf(stderr, "bad info\n"); return 1; }
        /* The batter is a player of the side at bat (the half shown is the one after the play,
         * so only check that he is a batter). */
        { BbPlayer b; if (bb_game_player(g, info.batter, &b) != 1 || b.bat_agent != info.batter) { fprintf(stderr, "bad batter\n"); return 1; } }
        for (uint32_t i = 0; i < info.n_calls; i++) {
            const BbCall *c = &info.calls[i];
            if (c->t_s <= 0.f || c->t_s > info.duration_s || c->base > 4 || (i > 0 && c->t_s < info.calls[i - 1].t_s)) { fprintf(stderr, "bad call\n"); return 1; }
        }
        calls += info.n_calls;
        /* A ball that left the field: inside the clip, a known reason, a ground point. */
        if (info.dead_s >= 0.f) {
            if (info.dead_s > info.duration_s + 1e-4f || info.dead_reason < 1 || info.dead_reason > 3 || info.dead_at[2] != 0.f) { fprintf(stderr, "bad dead ball\n"); return 1; }
            dead++;
        } else if (info.dead_reason != 0) { fprintf(stderr, "bad dead reason\n"); return 1; }
        bb_blob_free(blob, len);
        n++;
    }
    uint16_t a, h;
    int fin = bb_game_score(g, &a, &h);
    bb_game_free(g);
    printf("C client: %d pitches, %d out calls, %d balls out of play, final=%d, score %u-%u\n", n, calls, dead, fin, a, h);
    return fin == 1 && calls >= 51 && dead >= 3 ? 0 : 1;
}
