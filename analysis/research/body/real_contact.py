"""The real hitter's recorded bat path against a game fastball arriving earlier or later, by the game's collision
(docs/body-motion.md 4.16, Q73; the design's review, P2-4). A what-if by replay: his swing is not changed, only when the
ball arrives against it. No body runs.

    uv run --with numpy --with scipy --with c3d python3 analysis/research/body/real_contact.py [<swing id>] [--cut <Hz>|--raw]
        > sim/target/codex-probes/body3d/s14/real_contact.out

THE DATA ARE NOT IN THIS REPOSITORY (CC BY-NC-SA 4.0, PLAN D46); see real_swing.py. Its output is derived from them:
keep it under sim/target.

How:
- The bat: his raw C3D bat markers (Marker1 at the knob end; the far end the middle of Marker2 and Marker3), at his own
  size, sim frame; the samples up to his contact only (after it his bat carries his own ball's impulse), low-pass
  filtered there (--cut, default 30 Hz [E]) or not (--raw); velocities by differences. Between samples (2.8 ms) the
  knob's and tip's places go evenly. The bat's spin about its own axis is not seen by the markers and taken as none.
  The collision's bat is the game's 34-in bat (contact3d: bbphys.bat_mass_properties) moving as his bat moved.
- First, as a check: HIS pitch (HitTrax: 61.2 mph, 5.4 deg down) arriving touching the barrel's face at his sweet spot at
  his contact instant: where the game's collision sends it, against HitTrax's measured 101.5 mph, 18 deg up, 17 deg to
  the pull side.
- Where on the barrel's face, up and down, he met his ball is below what the markers can place (a centimetre moves the
  launch by several degrees, and with his barrel tilted down to a low pitch, the side too): two stated heights are run,
  the ball centred on the sweet spot's path and 2 cm above it [E].
- Then the game's average four-seamer (practice3d: 37.98 m/s at the plate, 5.27 deg down, 1.62 deg across) on a line
  through the same point, arriving `shift` ms earlier or later than his contact: + the swing is EARLY against it (the ball
  arrives later), - the swing is LATE (the ball arrives sooner and is met deeper, earlier in his swing's arc). Only
  late swings can be read: an early one meets the ball after his recorded contact, where his bat had already hit his ball.
"""
import math
import os
import sys

import numpy as np
from scipy.signal import butter, filtfilt

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import real_swing as RS                       # noqa: E402
import contact3d as C3                        # noqa: E402
import practice3d as PR                       # noqa: E402
from contact3d import BP                      # noqa: E402

MPH = 0.44704


def main(argv):
    sid = next((a for a in argv if not a.startswith("--") and "_" in a), RS.DEFAULT_SWING)
    raw = "--raw" in argv
    cut = float(argv[argv.index("--cut") + 1]) if "--cut" in argv else 30.0
    sw = RS.Swing(sid)
    dt = 1.0 / sw.rate
    ic = sw.ic
    n = ic + 1
    rot = lambda v: v @ RS.LAB_TO_SIM.T
    knob = rot(sw.lab["Marker1"][:n])
    far = rot(sw.lab["bat_far"][:n])
    if not raw:
        b, a = butter(4, cut / (sw.rate / 2.0))
        knob, far = filtfilt(b, a, knob, axis=0), filtfilt(b, a, far, axis=0)
    ax = far - knob
    ax /= np.linalg.norm(ax, axis=1, keepdims=True)
    tipL = C3.BAT_L
    tip = knob + ax * tipL
    ucm = BP.bat_mass_properties()["u_cm"]
    cm = knob + ax * ucm
    vcm = np.gradient(cm, dt, axis=0)
    axd = np.gradient(ax, dt, axis=0)
    om = np.cross(ax, axd)                      # the turn across the bat's axis; none about it
    t = sw.t[:n]
    uss = C3.BAT_L - C3.SS_FROM_TIP
    sweet = knob + ax * uss
    vss = np.gradient(sweet, dt, axis=0)

    def bat_at(tt):
        q = (tt - t[0]) / dt
        i = min(n - 2, max(0, int(math.floor(q))))
        f = q - i
        mix = lambda X: tuple((1.0 - f) * X[i] + f * X[i + 1])
        k_, tp = np.array(mix(knob)), np.array(mix(tip))
        a_ = tp - k_
        return tuple(k_), tuple(a_ / np.linalg.norm(a_)), mix(vcm), mix(om)

    say = print
    say("# OpenBiomechanics swing %s: his recorded bat path (raw markers, %s) through the game's collision. DERIVED FROM CC BY-NC-SA DATA: keep out of git."
        % (sid, "unfiltered" if raw else "%.0f Hz to contact" % cut))
    say("his sweet spot (the game's, %.3f m from the knob) at the sample before contact: %.1f m/s (%.1f mph); fastest before contact %.1f m/s at %.0f ms"
        % (uss, np.linalg.norm(vss[ic - 1]), np.linalg.norm(vss[ic - 1]) / MPH, np.linalg.norm(vss, axis=1)[: ic].max(),
           1e3 * t[int(np.argmax(np.linalg.norm(vss, axis=1)[: ic]))]))
    # the face at his contact: the sweet spot's velocity square to the bat
    k_ = ic - 1
    v = vss[k_]
    face = v - ax[k_] * np.dot(v, ax[k_])
    face /= np.linalg.norm(face)
    # the ball's centre 1.5 cm inside touching at his last sample before contact, so that a ball arriving on time is met
    # (at its first touch, a fraction of a millisecond sooner) at any of the heights run
    pc0 = sweet[k_] + face * (BP.bat_radius(uss) + BP.RADIUS - 0.015)
    tc = t[k_]

    def strike(nb, speed, sh, off):
        bv = tuple(np.array(nb) * speed)
        up = np.array([0.0, 0.0, 1.0]) - np.array(nb) * nb[2]
        up /= np.linalg.norm(up)
        pc = pc0 + up * off
        tb = tc + sh * 1e-3
        ball_at = lambda tt: tuple(pc + np.array(bv) * (tt - tb))
        return C3.strike(ball_at, bv, C3.pitch_spin(nb), bat_at, t[0] + 0.01, t[-2]), pc

    def run(label, nb, speed, shifts, off):
        say("\n## %s; the ball's centre %+.1f cm above the sweet spot's path" % (label, off * 100))
        for sh in shifts:
            r, pc = strike(nb, speed, sh, off)
            what = "his timing" if sh == 0 else "%.0f ms %s" % (abs(sh), "EARLY" if sh > 0 else "LATE")
            if r["hit"]:
                out_cm = -np.dot(np.array(r["where"]) - pc, nb) * 100
                say("  the swing %-11s met %+6.1f cm along the ball's line (%s), at %+6.1f ms against his contact, %+5.1f cm from the sweet spot; bat %.1f m/s, %+5.1f deg from square;"
                    " BALL %+6.1f deg (%s), %+5.1f deg up, %5.1f mph"
                    % (what, out_cm, "further out" if out_cm >= 0 else "deeper", (r["t"] - tc) * 1e3, (r["u"] - uss) * 100, r["bat_mps"], r["square_deg"],
                       r["spray_deg"], C3.side(r["spray_deg"]), r["launch_deg"], r["exit_mps"] / MPH))
            else:
                say("  the swing %-11s NO CONTACT (%s; nearest %.1f cm)" % (what, r["why"], r["gap"] * 100))

    # his own pitch, as a check
    sp = float(sw.hit["pitch"]) * MPH
    dd = math.radians(float(sw.hit["pitch_angle"]))
    nb_his = (0.0, -math.cos(dd), math.sin(dd))
    # where on the barrel's face he met it, up and down, is below what the markers can place (a centimetre moves the launch
    # by several degrees), and with his barrel tilted down to a low pitch a height on the face moves the ball's side too, so
    # no one height matches both HitTrax's launch and its side: two stated heights are run, centred and 2 cm up [E]
    for off in (0.0, 0.01, 0.02):
        run("Check: HIS pitch (%.1f mph, %.1f deg down) at his contact. HitTrax measured: %s mph, %s deg up, %s deg (pull side -)"
            % (sp / MPH, -math.degrees(dd), sw.meta["exit_velo_mph_x"], sw.hit["la"], sw.hit["bearing"]), nb_his, sp, (0.0, -2.0, -4.0), off)
    d, ac = math.radians(PR.BALL_DESCENT), math.radians(PR.BALL_ACROSS)
    nb = (math.cos(d) * math.sin(ac), -math.cos(d) * math.cos(ac), -math.sin(d))
    run("The game's average fastball (%.2f m/s, %.2f deg down, %.2f across) on a line through his contact point, arriving earlier or later"
        % (PR.BALL_SPEED, PR.BALL_DESCENT, PR.BALL_ACROSS), nb, PR.BALL_SPEED, (0.0, -3.0, -6.0, -9.0, -12.0, -15.0, -20.0, -25.0, -30.0), 0.0)
    run("The same, the ball 2 cm higher on the face", nb, PR.BALL_SPEED, (0.0, -3.0, -6.0, -9.0, -12.0, -15.0, -20.0, -25.0, -30.0), 0.02)


if __name__ == "__main__":
    main(sys.argv[1:])
