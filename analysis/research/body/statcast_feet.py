"""Where major leaguers' feet are when the bat gets to the ball, from the foot points Baseball Savant's batting-stance
page carries (body-motion 4.13 rung 3; research note 15 A).  Stdlib only; no search, no body model.

    python3 analysis/research/body/statcast_feet.py <saved page .html, or its vizData as .json>
    python3 analysis/research/body/statcast_feet.py <file> --check     # exits 1 unless the figures the body stands on come out
    python3 analysis/research/body/statcast_feet.py --fetch <out.html> [--season 2026]

THE PAGE.  https://baseballsavant.mlb.com/visuals/batting-stance?type=batter&gameType=Regular&seasonStart=2026&seasonEnd=2026&minSwings=q&minGroupSwings=10
("MLB Batting Stance Visual"), read 2026-11-26: season 2026, regular season, qualified hitters, 231 hitter-sides.  The
page's script holds `vizData = [...]`, one object a hitter-side.  The page's data are MLB's and are not committed; the copy
read that day is kept (gitignored) under sim/target/codex-probes/body3d/s11/statcast/.  The page is live: a later season or
a changed minimum gives other rows.

WHAT THE DATA HOLD, AND WHAT IS MY CONSTRUCTION (flagged in note 15):
  - per hitter-side, three points of each foot (big toe, small toe, heel) at three instants: suffix 0 the stance (one
    second before release), 1 the release, 2 the bat's intercept (nearest the ball: about contact); averages over his swings.
  - units are FEET in the plate's frame (origin the plate's rear apex, +y to the pitcher, +x to first base): inferred, and
    checked two ways below: the stance's separation rebuilt from the points against the published column (in inches), and
    `avg_intercept_y` * 12 against `avg_intercept_y_in`.
  - `avg_batter_y_position` is the published "depth in the box", inches behind the plate's front edge (17 in ahead of the
    origin), of his centre of mass ("the midpoint of their hips") one second before release: the hips IN THE STANCE.
  - MINE: the toe = the midpoint of the two toe points; the ANKLE JOINT a quarter of the way from the heel to the toe (where
    the body model has it, body.py's foot); left-handers mirrored onto right-handers (x -> -x); the foot's direction heel
    to toe, 0 = square to the plate, + = turned toward the pitcher.

THE FIGURES IT PRINTED on the copy read 2026-11-26 (n = 231; mean +- SD over the hitter-sides), which the body stands on
(body.py STANCE_APART, STANCE_ACROSS, STANCE_FOOT; practice3d.MEET_FROM_REAR_ANKLE):
    the ankle joints apart along the line to the pitcher, at the intercept   102.59 +- 7.94 cm     -> 1.03 m
    the lead ankle toward the plate from the rear one                          1.28 +- 9.89 cm     -> 0.0 (level across)
    the front foot turned toward the pitcher from square                      43.81 +- 10.15 deg   -> 44 deg
    the ball (intercept) toward the pitcher from the rear ankle              102.24 +- 8.81 cm     -> 1.022 m
    the ball toward the pitcher from the lead ankle                           -0.36 +- 9.38 cm     (level with it)
    the stance's separation, published against rebuilt from the points: correlation 0.9864, the points 3.51 cm more
(note 15 quotes the last two ball figures as 102.2 +- 8.8 and -0.4 +- 9.4.)
And, for where the ball is met relative to the HIPS (4.13, "After the independent review of session 11", P1-1):
    the ball ahead of the hips in the stance (the published column)           79.00 +- 9.42 cm  (31.1 in)
    the hips in the stance, ahead of the stance's rear ankle                  33.67 +- 4.57 cm, 0.46 +- 0.10 of the way to the lead ankle
    the rear ankle's travel toward the pitcher, stance to intercept           10.43 +- 6.11 cm;  the lead ankle's 37.14 +- 16.70 cm
    the ball ahead of the hips at the intercept IF the hips keep their share of the way between the ankles:  55.21 cm
                                               IF the hips travel as the ankles' midpoint does:              55.22 cm
                                               IF the hips do not travel at all:                             79.00 cm
(the hips' place at the intercept is NOT in the data; the three lines are assumptions, labelled as that).
The 2025 page (253 hitter-sides) gives 103.14, 0.52, 41.35 deg, 102.31, -0.83; 78.13; 54.76 and 54.83.
"""
import json
import math
import statistics as st
import sys

URL = "https://baseballsavant.mlb.com/visuals/batting-stance?type=batter&gameType=Regular&seasonStart=%d&seasonEnd=%d&minSwings=q&minGroupSwings=10"
FT = 30.48                 # cm in a foot
IN = 2.54                  # cm in an inch
PLATE_FRONT_IN = 17.0      # the plate's front edge ahead of its rear apex
ANKLE = 0.25               # the ankle joint's place, heel to toe (body.py's foot)

# what the body stands on, with how far the page's mean may lie from it (the rounding the spec states)
STANDS_ON = (("ankles apart, along (cm)", 103.0, 0.5), ("lead ankle across (cm)", 0.0, 1.5), ("front foot's turn (deg)", 44.0, 0.5),
             ("ball ahead of the rear ankle (cm)", 102.2, 0.05))


def load(path):
    """the page's rows: from a saved copy of the page (its `vizData = [...]`) or from that array saved as JSON."""
    s = open(path, encoding="utf-8").read()
    if s.lstrip().startswith("["):
        return json.loads(s)
    i = s.find("vizData")
    if i < 0:
        raise SystemExit("no vizData in %s: not the batting-stance visual's page" % path)
    return json.JSONDecoder().raw_decode(s[s.find("[", i):])[0]


def feet(r):
    """one hitter-side's feet as a right-hander's, in cm in the plate's frame: {(which, instant): {"heel", "toe", "ankle",
    "centre"}} for which in ("lead", "rear") and instant in (0, 1, 2); and the foot's direction (deg; 0 = toe at the plate,
    + = turned toward the pitcher) as ["turn"]."""
    m = 1.0 if r["bat_side"] == "R" else -1.0
    lead, rear = ("l", "r") if r["bat_side"] == "R" else ("r", "l")
    out = {}
    for which, f in (("lead", lead), ("rear", rear)):
        for t in (0, 1, 2):
            P = lambda part: (FT * m * r["avg_%s%s_x%d" % (f, part, t)], FT * r["avg_%s%s_y%d" % (f, part, t)])
            (bx, by), (sx, sy), heel = P("bigtoe"), P("smalltoe"), P("heel")
            toe = ((bx + sx) / 2, (by + sy) / 2)
            at = lambda k: (heel[0] + k * (toe[0] - heel[0]), heel[1] + k * (toe[1] - heel[1]))
            out[which, t] = {"heel": heel, "toe": toe, "ankle": at(ANKLE), "centre": at(0.5),
                             "turn": math.degrees(math.atan2(toe[1] - heel[1], toe[0] - heel[0]))}
    return out


def measures(rows):
    """every figure, one list a quantity, over the hitter-sides that have the intercept's points."""
    o = {}
    add = lambda k, v: o.setdefault(k, []).append(v)
    for r in rows:
        if r.get("avg_rheel_x2") is None or r.get("avg_intercept_y") is None:
            continue
        F = feet(r)
        la, ra = F["lead", 2]["ankle"], F["rear", 2]["ankle"]
        la0, ra0 = F["lead", 0]["ankle"], F["rear", 0]["ankle"]
        ball = FT * r["avg_intercept_y"]
        hips0 = IN * (PLATE_FRONT_IN - r["avg_batter_y_position"])          # the hips in the stance, along the line to the pitcher
        add("ankles apart, along (cm)", la[1] - ra[1])
        add("lead ankle across (cm)", la[0] - ra[0])
        add("front foot's turn (deg)", F["lead", 2]["turn"])
        add("rear foot's turn at the intercept (deg)", F["rear", 2]["turn"])
        add("feet's centres apart at the intercept (cm)", math.dist(F["lead", 2]["centre"], F["rear", 2]["centre"]))
        add("ball ahead of the rear ankle (cm)", ball - ra[1])
        add("ball ahead of the lead ankle (cm)", ball - la[1])
        add("stance separation, published (cm)", IN * r["avg_foot_sep0"])
        add("stance separation, from the points' centres (cm)", math.dist(F["lead", 0]["centre"], F["rear", 0]["centre"]))
        add("intercept_y in feet * 12 less the page's inches (in)", 12 * r["avg_intercept_y"] - r["avg_intercept_y_in"])
        # where the ball is met relative to the hips
        add("ball ahead of the hips in the stance (cm)", ball - hips0)
        add("the same, the page's own column (cm)", IN * r["avg_intercept_y_vs_batter"])
        add("hips in the stance ahead of the stance's rear ankle (cm)", hips0 - ra0[1])
        share = (hips0 - ra0[1]) / (la0[1] - ra0[1])
        add("hips' share of the way rear ankle -> lead ankle, in the stance", share)
        add("rear ankle's travel toward the pitcher, stance to intercept (cm)", ra[1] - ra0[1])
        add("lead ankle's travel toward the pitcher, stance to intercept (cm)", la[1] - la0[1])
        add("IF the hips keep their share between the ankles: ball ahead of them at the intercept (cm)", ball - (ra[1] + share * (la[1] - ra[1])))
        add("IF the hips travel as the ankles' midpoint: ball ahead of them at the intercept (cm)", ball - (hips0 + (la[1] + ra[1]) / 2 - (la0[1] + ra0[1]) / 2))
        add("hips in the stance ahead of the intercept's rear ankle (cm)", hips0 - ra[1])
    return o


def corr(x, y):
    mx, my = st.mean(x), st.mean(y)
    return sum((a - mx) * (b - my) for a, b in zip(x, y)) / math.sqrt(sum((a - mx) ** 2 for a in x) * sum((b - my) ** 2 for b in y))


def pct(v, p):
    v = sorted(v)
    k = (len(v) - 1) * p
    f = math.floor(k)
    c = min(f + 1, len(v) - 1)
    return v[f] + (v[c] - v[f]) * (k - f)


def main(argv):
    if "--fetch" in argv:
        import urllib.request
        out = argv[argv.index("--fetch") + 1]
        y = int(argv[argv.index("--season") + 1]) if "--season" in argv else 2026
        req = urllib.request.Request(URL % (y, y), headers={"User-Agent": "Mozilla/5.0 (research script)"})
        open(out, "wb").write(urllib.request.urlopen(req, timeout=60).read())
        print("saved", URL % (y, y), "->", out)
        return 0
    path = [a for a in argv if not a.startswith("--")][0]
    o = measures(load(path))
    n = len(o["ankles apart, along (cm)"])
    print("%d hitter-sides with the intercept's foot points (%s)" % (n, path))
    for k, v in o.items():
        print("  %-92s mean %8.2f  sd %6.2f  10th-90th %7.2f to %7.2f" % (k, st.mean(v), st.stdev(v), pct(v, 0.1), pct(v, 0.9)))
    a, b = o["stance separation, published (cm)"], o["stance separation, from the points' centres (cm)"]
    print("  the stance's separation, published against rebuilt from the points: correlation %.4f; the points %.2f cm more (so the points are in feet)"
          % (corr(a, b), st.mean(b) - st.mean(a)))
    if "--check" in argv:
        bad = 0
        for k, want, tol in STANDS_ON:
            got = st.mean(o[k])
            ok = abs(got - want) <= tol
            bad += not ok
            print("  CHECK %-36s the page's mean %8.2f; the body stands on %7.2f (within %.2f): %s" % (k, got, want, tol, "ok" if ok else "DIFFERS"))
        print("PASS" if not bad else "FAIL")
        return 1 if bad else 0
    return 0


if __name__ == "__main__":
    if len(sys.argv) < 2:
        raise SystemExit(__doc__)
    sys.exit(main(sys.argv[1:]))
