"""Where hitters meet the ball, and how fast the bat is, by where the ball then goes (PLAN D44; body-motion 4.15).

    python3 -I analysis/statcast/savant_contact_by_spray.py <dir with sc_*.csv>        (fetch lines: README.md)
    python3 -I analysis/statcast/savant_contact_by_spray.py --check                    the script's own arithmetic on made-up rows

THE CHECK OF D44, NEVER GIVEN TO THE PRACTICE SEARCH. Balls in play from Baseball Savant's per-pitch files, split by
spray direction, left-handed batters mirrored so that every batter reads as a right-hander (negative = pulled, toward
left field; positive = the other way).

What each column is:
- spray: the direction from home plate to where the ball was fielded or landed (hc_x, hc_y), atan2(hc_x - 125.42,
  198.27 - hc_y), the convention of savant_spray.py. It is where the ball ended up, not the direction it left the bat
  (a ground ball is fielded where a fielder got to it), and the two differ by a few degrees for sliced and hooked balls.
- the ball's place: intercept_ball_minus_batter_pos_y_inches. Savant's CSV documentation: "the distance between the
  bat/ball intercept point and the batter's center of mass", Y from the mound to the plate; positive is toward the
  pitcher. THE BATTER'S POSITION IS WHERE HIS CENTRE OF MASS WAS AS HE STOOD BEFORE THE PITCH (research note 15 reads
  the same figure on the leaderboard as from the hips in the stance, a second before release). WHERE HIS HIPS ARE AT
  CONTACT IS NOT IN THE DATA: a hitter strides and shifts toward the pitcher before contact, so the ball is nearer his
  hips at contact than this column says, by an amount the files do not carry.
- bat speed: Statcast's, the sweet spot (6 in from the head) at the intercept, mph.

Swings counted: balls in play (description hit_into_play...) with spray, the ball's place and bat speed present, bunts
left out (the event's text says bunt), and only COMPETITIVE swings by Statcast's own rule (each batter's fastest 90 %
of his tracked swings, the cut taken over all his swings in the files): a checked swing that dribbles a ball in play is
not a swing in the sense the body practises.

Tables: every ball in play; hard-hit balls (95 mph or more off the bat); fastballs (FF, SI, FC) in the middle of the
zone (the pitch the body practises on); and each swing against ITS OWN BATTER'S mean (batters with 15 or more balls in
play), which takes out that pull hitters and opposite-field hitters are different men who stand differently.
Each cell: n, mean and SD (the SD of the swings, with the mean's standard error SD / sqrt(n) beside it).
"""
import csv
import glob
import math
import statistics as st
import sys
from collections import defaultdict

IN = 2.54                      # cm in an inch
BANDS = ((-90.0, -27.0, "pulled hard (past -27)"), (-27.0, -9.0, "pulled (-27 to -9)"), (-9.0, 9.0, "up the middle (-9 to +9)"),
         (9.0, 27.0, "the other way (+9 to +27)"), (27.0, 90.0, "far the other way (past +27)"))
HARD = 95.0                    # mph off the bat: Statcast's hard-hit line
FAST = ("FF", "SI", "FC")
MID_X, MID_Z = (-0.55, 0.55), (1.9, 3.1)      # ft: the middle of the zone, across (plate_x) and up (plate_z)
COMPETITIVE = 0.10             # each batter's slowest tenth of tracked swings is left out (Statcast's competitive-swing rule)
MIN_OWN = 15                   # balls in play a batter needs before a swing is read against his own mean


def spray_deg(hc_x, hc_y):
    return math.degrees(math.atan2(hc_x - 125.42, 198.27 - hc_y))


def num(r, k):
    try:
        return float(r[k])
    except (ValueError, KeyError, TypeError):
        return None


def read(rows_in):
    """(balls in play as dicts: spray (right-hander's frame), y_in, bat, ev, batter key, fast_mid; how many were dropped and why)."""
    swings = defaultdict(list)
    rows_in = list(rows_in)
    for r in rows_in:
        b = num(r, "bat_speed")
        if b is not None:
            swings[r.get("batter")].append(b)
    cut = {k: sorted(v)[int(COMPETITIVE * len(v))] for k, v in swings.items()}
    out, drop = [], defaultdict(int)
    for r in rows_in:
        if not r.get("description", "").startswith("hit_into_play"):
            continue
        if "bunt" in (r.get("des") or "").lower() or "bunt" in (r.get("events") or "").lower():
            drop["bunts"] += 1
            continue
        hx, hy, y, b = num(r, "hc_x"), num(r, "hc_y"), num(r, "intercept_ball_minus_batter_pos_y_inches"), num(r, "bat_speed")
        if hx is None or hy is None:
            drop["no spray"] += 1
            continue
        if y is None or b is None:
            drop["no bat tracking"] += 1
            continue
        if b < cut[r.get("batter")]:
            drop["not a competitive swing"] += 1
            continue
        mir = 1.0 if r.get("stand") == "R" else -1.0
        px, pz = num(r, "plate_x"), num(r, "plate_z")
        out.append(dict(spray=mir * spray_deg(hx, hy), y=y, bat=b, ev=num(r, "launch_speed"), who=(r.get("batter"), r.get("stand")), stand=r.get("stand"),
                        fast_mid=r.get("pitch_type") in FAST and px is not None and pz is not None and MID_X[0] <= px <= MID_X[1] and MID_Z[0] <= pz <= MID_Z[1]))
    return out, dict(drop)


def cell(v):
    """(n, mean, SD of the values, the mean's standard error)."""
    n = len(v)
    if n < 2:
        return n, (v[0] if v else float("nan")), float("nan"), float("nan")
    sd = st.stdev(v)
    return n, st.mean(v), sd, sd / math.sqrt(n)


def table(rows, title, y_of=lambda r: r["y"], bat_of=lambda r: r["bat"], rel=False, out=print):
    out("")
    out(title)
    if rel:
        out("%-30s %6s  %22s  %22s" % ("where the ball went", "n", "the ball's place, in (cm)", "bat speed, mph"))
    else:
        out("%-30s %6s  %30s  %22s" % ("where the ball went", "n", "the ball's place, in (cm)", "bat speed, mph"))
    got = []
    for lo, hi, name in BANDS:
        sel = [r for r in rows if lo <= r["spray"] < hi]
        ny, my, sy, ey = cell([y_of(r) for r in sel])
        nb, mb, sb, eb = cell([bat_of(r) for r in sel])
        got.append((name, ny, my, sy, ey, mb, sb, eb))
        sign = "%+" if rel else "%"
        out(("%-30s %6d  " + sign + "6.1f +- %4.1f (se %.2f)  [" + sign + "6.1f cm]   " + sign + "6.2f +- %4.2f (se %.2f)") % (name, ny, my, sy, ey, my * IN, mb, sb, eb))
    return got


def report(rows, drop, out=print):
    out("balls in play read: %d (left out: %s); right-handers %d, left-handers %d (mirrored)" % (
        len(rows), ", ".join("%s %d" % kv for kv in sorted(drop.items())) or "none", sum(r["stand"] == "R" for r in rows), sum(r["stand"] != "R" for r in rows)))
    out("THE BALL'S PLACE IS FROM HIS CENTRE OF MASS AS HE STOOD BEFORE THE PITCH, toward the pitcher; his hips at contact are not in the data.")
    res = {"all": table(rows, "EVERY BALL IN PLAY", out=out)}
    res["hard"] = table([r for r in rows if r["ev"] is not None and r["ev"] >= HARD], "HARD-HIT (%.0f mph or more off the bat)" % HARD, out=out)
    res["fast_mid"] = table([r for r in rows if r["fast_mid"]], "FASTBALLS (%s) IN THE MIDDLE OF THE ZONE (plate_x %.2f to %.2f ft, plate_z %.1f to %.1f ft)" % ((", ".join(FAST),) + MID_X + MID_Z), out=out)
    by = defaultdict(list)
    for r in rows:
        by[r["who"]].append(r)
    own = {k: (st.mean(x["y"] for x in v), st.mean(x["bat"] for x in v)) for k, v in by.items() if len(v) >= MIN_OWN}
    sel = [r for r in rows if r["who"] in own]
    res["own"] = table(sel, "EACH SWING AGAINST ITS OWN BATTER'S MEAN (%d batters with %d or more balls in play; + = further out, faster)" % (len(own), MIN_OWN),
                       y_of=lambda r: r["y"] - own[r["who"]][0], bat_of=lambda r: r["bat"] - own[r["who"]][1], rel=True, out=out)
    res["own_hard"] = table([r for r in sel if r["ev"] is not None and r["ev"] >= HARD], "THE SAME, HARD-HIT ONLY (against the batter's mean over all his balls in play)",
                            y_of=lambda r: r["y"] - own[r["who"]][0], bat_of=lambda r: r["bat"] - own[r["who"]][1], rel=True, out=out)
    # the slopes within a batter: the ball's place and the bat's speed against spray, least squares on the deviations
    xs = [r["spray"] - st.mean(x["spray"] for x in by[r["who"]]) for r in sel]
    for name, ys in (("the ball's place, in", [r["y"] - own[r["who"]][0] for r in sel]), ("bat speed, mph", [r["bat"] - own[r["who"]][1] for r in sel])):
        sxx = sum(x * x for x in xs)
        slope = sum(x * y for x, y in zip(xs, ys)) / sxx
        resid = [y - slope * x for x, y in zip(xs, ys)]
        se = math.sqrt(sum(e * e for e in resid) / (len(xs) - 2) / sxx)
        res["slope " + name] = (slope, se)
        out("within a batter, %s per 10 deg of spray toward the opposite field: %+.2f (se %.2f)" % (name, 10 * slope, 10 * se))
    return res


def check():
    """the script's arithmetic on rows made by hand: the mirroring, the bands, the cut, a cell's mean and SD."""
    def row(batter, stand, hx, hy, y, bat, ev=100.0, desc="hit_into_play", des="", pt="FF"):
        return dict(batter=batter, stand=stand, hc_x=str(hx), hc_y=str(hy), intercept_ball_minus_batter_pos_y_inches=str(y), bat_speed=str(bat),
                    launch_speed=str(ev), description=desc, des=des, events="", pitch_type=pt, plate_x="0.0", plate_z="2.5")
    # a ball fielded 50 units to the left-field side and 100 up the field: atan2(-50, 100) = -26.57 deg for a right-hander (pulled),
    # and the same spot is +26.57 (the other way) for a left-hander
    assert abs(spray_deg(125.42 - 50, 198.27 - 100) + 26.565) < 1e-3
    rows = [row("a", "R", 125.42 - 50, 98.27, 34.0, 72.0), row("a", "R", 125.42, 98.27, 30.0, 70.0), row("a", "R", 125.42 + 50, 98.27, 26.0, 68.0),
            row("b", "L", 125.42 + 50, 98.27, 36.0, 74.0),                                   # a left-hander's ball to right field: pulled
            row("a", "R", 125.42, 98.27, 30.0, 40.0, des="Batter bunts"),                    # a bunt: out
            row("a", "R", 125.42, 98.27, "", 70.0)]                                          # no tracking: out
    rows += [dict(batter="a", stand="R", bat_speed=str(60.0 + i), description="swinging_strike") for i in range(20)]   # his other swings
    # batter a has 26 tracked swings (the bunt's 40 and the untracked ball's 70 among them); a tenth of 26 is 2.6, so the cut is
    # his third slowest, 60 (after 40 and 59), and the two under it are not competitive swings
    rows.append(row("a", "R", 125.42, 98.27, 10.0, 59.0))                                    # a ball in play on a swing under his cut: out
    got, drop = read(rows)
    assert drop == {"bunts": 1, "no bat tracking": 1, "not a competitive swing": 1}, drop
    assert len(got) == 4
    by = {name: [r for r in got if lo <= r["spray"] < hi] for lo, hi, name in BANDS}
    assert [len(by[b[2]]) for b in BANDS] == [0, 2, 1, 1, 0], [len(by[b[2]]) for b in BANDS]
    n, m, sd, se = cell([r["y"] for r in by["pulled (-27 to -9)"]])               # 34 and 36: mean 35, SD sqrt(2), se 1
    assert (n, m) == (2, 35.0) and abs(sd - math.sqrt(2.0)) < 1e-12 and abs(se - 1.0) < 1e-12
    print("check: the mirroring, the bands, the bunt, the tracking and the competitive-swing cuts and a cell's figures are as worked by hand: PASS")
    return True


if __name__ == "__main__":
    if "--check" in sys.argv:
        sys.exit(0 if check() else 1)
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(2)
    files = sorted(glob.glob(sys.argv[1] + "/sc_*.csv"))
    allrows = []
    for f in files:
        with open(f, newline="", encoding="utf-8-sig") as fh:
            allrows += list(csv.DictReader(fh))
    print("%d files, %d pitches: %s" % (len(files), len(allrows), ", ".join(f.split("sc_")[-1][:-4] for f in files)))
    rows, drop = read(allrows)
    report(rows, drop)
