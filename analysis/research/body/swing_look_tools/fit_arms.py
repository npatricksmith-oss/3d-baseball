"""A helper for writing swing_author.py's arm rows: for each instant, the hands' place and the barrel's direction NEAR
what was written by hand at which this body's wrists, forearms and shoulders are inside their ranges, with the bat's
roll and the elbows' swivels that strain them least. Prints rows to paste. Not part of the motion."""
import sys, math
sys.path.insert(0, "analysis/research/body")
import swing_author as A
E, B = A.E, A.B
D2R = A.D2R

INTENT = [  # t, hand_f, hand_l, hand_u, bat_az, bat_el
    (A.T_START, 0.25, -0.18, -0.03, -170, 55), (A.T_TOP, 0.22, -0.24, 0.00, -185, 55), (A.T_TOUCH, 0.21, -0.27, 0.00, -195, 55),
    (A.T_HEEL, 0.21, -0.27, -0.01, -195, 52), (-0.120, 0.22, -0.27, -0.03, -186, 45), (-0.100, 0.23, -0.26, -0.07, -170, 32),
    (-0.080, 0.25, -0.25, -0.12, -150, 15), (-0.060, 0.28, -0.22, -0.19, -122, 0), (-0.045, 0.30, -0.19, -0.26, -98, -12),
    (-0.030, 0.32, -0.16, -0.31, -70, -20), (-0.015, 0.33, -0.13, -0.34, -36, -24), (0.0, 0.335, -0.10, -0.36, 1, -24),
    (0.015, 0.38, -0.07, -0.34, 36, -19), (0.030, 0.42, -0.03, -0.32, 69, -9), (0.050, 0.46, 0.02, -0.27, 105, 6),
    (0.070, 0.48, 0.07, -0.20, 135, 20), (0.090, 0.47, 0.11, -0.12, 160, 32), (0.120, 0.43, 0.16, -0.03, 190, 42),
    (0.150, 0.38, 0.19, 0.05, 212, 45), (0.200, 0.31, 0.21, 0.12, 240, 36), (0.250, 0.26, 0.21, 0.16, 265, 20),
    (0.300, 0.23, 0.20, 0.18, 285, 5), (0.380, 0.21, 0.18, 0.19, 303, -10), (0.560, 0.20, 0.17, 0.19, 312, -20)]
SCALE = (0.05, 0.05, 0.05, 15.0, 10.0)
STRAIN_OK = float(sys.argv[1]) if len(sys.argv) > 1 else 0.92
if len(sys.argv) > 2:
    B.ROM["wrist_uln"] = float(sys.argv[2])


def arms(m, sho, Rt, x, prev, step=10, span=50):
    hf, hl, hu, az, el = x
    mid = E.scale(E.add(sho["L"], sho["R"]), 0.5)
    hands = E.add(mid, E.mv(Rt, (hf, hl, hu)))
    bz = (math.cos(el * D2R) * math.cos(az * D2R), math.cos(el * D2R) * math.sin(az * D2R), math.sin(el * D2R))
    lu, lf = m.len["upperarm"], m.len["forearm"]
    best = None
    rolls = range(-180, 180, step) if prev is None else range(int(prev[0]) - span, int(prev[0]) + span + 1, step)
    for roll in rolls:
        xb, qb, Rb = A.bat_frame(m, {"bat_roll": float(roll)}, hands, mid, bz)
        tot, pick, reach = 0.0, {}, 0.0
        for sd, sg in B.SIDES:
            wrist = E.add(xb, E.mv(Rb, m.wrist_b[sd]))
            reach = max(reach, E.norm(E.sub(wrist, sho[sd])) - 0.985 * (lu + lf))
            sws = range(-90, 181, step) if prev is None else range(int(prev[1 if sd == "L" else 2]) - span, int(prev[1 if sd == "L" else 2]) + span + 1, step)
            bb = None
            for sw in sws:
                elbow = A.elbow_at(sho[sd], wrist, lu, lf, sg, float(sw))
                ang = B.arm_angles(sd, E.unit(E.sub(sho[sd], elbow)), E.unit(E.sub(elbow, wrist)), Rb, Rt)
                c = B.strain(ang, oval=True)
                if prev is not None:
                    c += 0.02 * abs(sw - prev[1 if sd == "L" else 2]) / 30.0
                if bb is None or c < bb[0]:
                    bb = (c, sw, ang)
            pick[sd] = bb
        c = max(pick["L"][0], pick["R"][0]) + (0.02 * abs(roll - prev[0]) / 30.0 if prev is not None else 0.0) + 20.0 * max(0.0, reach)
        if best is None or c < best[0]:
            best = (c, roll, pick["L"][1], pick["R"][1], pick["L"][2], pick["R"][2])
    return best


def cost(m, sho, Rt, x, x0, prev):
    b = arms(m, sho, Rt, x, prev)
    j = 30.0 * max(0.0, b[0] - STRAIN_OK) ** 2 + sum(((a - c) / s) ** 2 for a, c, s in zip(x, x0, SCALE)) * 0.05
    return j, b


OUT = []


def main():
    m, ts, states, worst = A.build(close=False)
    prev = None
    rows = []
    for row in INTENT:
        t, x0 = row[0], list(row[1:])
        i = min(range(len(ts)), key=lambda k: abs(ts[k] - t))
        st = states[i]
        sho = {sd: m.joint_pos(st, "shoulder_" + sd) for sd in "LR"}
        Rt = E.qR(m.sys.pose(st, m.b["trunk"])[1])
        x = list(x0)
        j, b = cost(m, sho, Rt, x, x0, prev)
        step = [0.03, 0.03, 0.03, 8.0, 6.0]
        for it in range(14):
            moved = False
            for k in range(5):
                for sgn in (1.0, -1.0):
                    y = list(x); y[k] += sgn * step[k]
                    jy, by = cost(m, sho, Rt, y, x0, prev)
                    if jy < j - 1e-9:
                        x, j, b, moved = y, jy, by, True
                        break
            if not moved:
                step = [s * 0.5 for s in step]
                if step[0] < 0.004:
                    break
        prev = (b[1], b[2], b[3])
        rows.append((t, x, b))
        f = lambda a: " ".join("%s %+4.0f" % (k[:7], v) for k, v in a.items())
        print("# %+6.3f strain %.2f  (intent %s)\n#    L %s\n#    R %s" % (t, b[0], " ".join("%g" % v for v in x0), f(b[4]), f(b[5])))
        OUT.append("    (%+.3f, dict(hand_f=%.3f, hand_l=%.3f, hand_u=%.3f, bat_az=%.0f, bat_el=%.0f, bat_roll=%.0f, sw_L=%.0f, sw_R=%.0f)),   # strain %.2f" % (t, x[0], x[1], x[2], x[3], x[4], b[1], b[2], b[3], b[0]))
        print("    (%+.3f, dict(hand_f=%.3f, hand_l=%.3f, hand_u=%.3f, bat_az=%.0f, bat_el=%.0f, bat_roll=%.0f, sw_L=%.0f, sw_R=%.0f))," % (t, x[0], x[1], x[2], x[3], x[4], b[1], b[2], b[3]))
        sys.stdout.flush()


main()
open("sim/target/codex-probes/swing-look/arm_rows.txt", "w").write("\n".join(OUT) + "\n")
