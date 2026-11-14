#!/usr/bin/env python3
"""Independent generator of game rosters (data-formats §2.4), stdlib only.

Written from the spec text, not from the Rust: Philox4x32-10 (Salmon et al., SC'11, checked
against the Random123 known-answer vector below), the key layout of §2.3 and the roster
rules of §2.4. Only the name pools are shared with the Rust (they are data, not logic); they
are read out of `sim/crates/bb_game/src/roster.rs` so there is one copy.

    python3 roster_oracle.py search      # seeds that reject a family name and a number
    python3 roster_oracle.py show SEED   # the rosters of one seed, with the rejections
"""
import re
import sys
from pathlib import Path

M0, M1 = 0xD2511F53, 0xCD9E8D57
W0, W1 = 0x9E3779B9, 0xBB67AE85
MASK = 0xFFFFFFFF


def philox4x32_10(ctr, key):
    c0, c1, c2, c3 = ctr
    k0, k1 = key
    for r in range(10):
        if r:
            k0, k1 = (k0 + W0) & MASK, (k1 + W1) & MASK
        p0, p1 = M0 * c0, M1 * c2
        c0, c1, c2, c3 = ((p1 >> 32) ^ c1 ^ k0) & MASK, p1 & MASK, ((p0 >> 32) ^ c3 ^ k1) & MASK, p0 & MASK
    return [c0, c1, c2, c3]


# Random123 known-answer test: counter 0, key 0.
assert philox4x32_10([0, 0, 0, 0], [0, 0]) == [0x6627E8D5, 0xE169C58D, 0xBC57AC4C, 0x9B00DBD8]

ROSTER_GENERATION, ROSTER_HANDS = 12, 13


class Stream:
    """data-formats §2.3: key = seed (low word first), counter = [play, agent<<16|purpose, sub, block]."""

    def __init__(self, seed, agent, purpose, sub):
        self.key = [seed & MASK, (seed >> 32) & MASK]
        self.ctr = [0, (agent << 16) | purpose, sub]
        self.block, self.buf, self.used = 0, [], 4

    def u32(self):
        if self.used == 4:
            self.buf = philox4x32_10(self.ctr + [self.block], self.key)
            self.block += 1
            self.used = 0
        v = self.buf[self.used]
        self.used += 1
        return v

    def uniform(self):
        """Open unit interval: a 64-bit word (first u32 low) -> (top 52 bits + 1/2) / 2**52."""
        lo = self.u32()
        hi = self.u32()
        return (((hi << 32 | lo) >> 12) + 0.5) / 2**52

    def index(self, n):
        return min(int(self.uniform() * n), n - 1)


def pools():
    src = (Path(__file__).resolve().parents[2] / "sim/crates/bb_game/src/roster.rs").read_text()
    body = src[src.index("pub const NAME_POOLS"):src.index("/// One player's identity")]
    out = []
    for blk in body.split("NamePool {")[1:]:
        w = float(re.search(r"weight: ([0-9.]+)", blk).group(1))
        g = re.search(r"given: &\[(.*?)\],", blk, re.S).group(1)
        f = re.search(r"family: &\[(.*?)\],\s*\}", blk, re.S).group(1)
        out.append((w, re.findall(r'"([^"]+)"', g), re.findall(r'"([^"]+)"', f)))
    return out


POOLS = pools()
THROWS_LEFT = [0.27, 0.0, 0.35, 0.0, 0.0, 0.0, 0.25, 0.20, 0.25]  # §2.4 item 4, positions 1-9
CODES = ["P", "C", "1B", "2B", "3B", "SS", "LF", "CF", "RF"]


def generate(seed):
    families, teams, log = [], [], {"family_rejections": 0, "number_rejections": 0, "given_rejections": 0}
    for side in (0, 1):
        order = list(range(2, 10))  # positions 2..9, catcher to right field
        rng = Stream(seed, side << 8 | 0xFF, ROSTER_GENERATION, 0)
        for i in range(7, 0, -1):
            j = rng.index(i + 1)
            order[i], order[j] = order[j], order[i]
        numbers, givens, players = [], [], []
        for position in range(1, 10):
            agent = side << 8 | 0x80 | position
            a = 0
            while True:
                s = Stream(seed, agent, ROSTER_GENERATION, 2 * a)
                u, acc, pool = s.uniform(), 0.0, POOLS[0]
                for p in POOLS:
                    acc += p[0]
                    if u < acc:
                        pool = p
                        break
                given, family = pool[1][s.index(len(pool[1]))], pool[2][s.index(len(pool[2]))]
                if family in families:
                    log["family_rejections"] += 1
                elif given in givens:
                    log["given_rejections"] += 1
                else:
                    break
                a += 1
            families.append(family)
            givens.append(given)
            a = 0
            while True:
                n = 1 + Stream(seed, agent, ROSTER_GENERATION, 2 * a + 1).index(99)
                if n not in numbers:
                    break
                log["number_rejections"] += 1
                a += 1
            numbers.append(n)
            h = Stream(seed, agent, ROSTER_HANDS, 0)
            left_thrower = h.uniform() < THROWS_LEFT[position - 1]
            u = h.uniform()
            lb, sw = (0.88, 0.04) if left_thrower else (0.24, 0.14)
            bats = "L" if u < lb else "S" if u < lb + sw else "R"
            slot = 9 if position == 1 else order.index(position) + 1
            players.append((slot, CODES[position - 1], n, given, family, bats + "/" + ("L" if left_thrower else "R")))
        teams.append(sorted(players))
    return teams, log


if __name__ == "__main__":
    if sys.argv[1] == "search":
        for seed in range(0, 200):
            _, log = generate(seed)
            if log["family_rejections"] and log["number_rejections"]:
                print(seed, log)
    else:
        teams, log = generate(int(sys.argv[2]))
        print(log)
        for t in teams:
            for p in t:
                print(p)
            print()
