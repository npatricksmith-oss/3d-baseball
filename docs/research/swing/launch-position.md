# Where the swing starts: the sim's bat against the measured launch (diagnosis, 2026-11-24)

Worker launch-diag, branch `work/base-swing-step1`. Diagnosis and a proposal only: no source, test or spec changed. Written for the owner's complaint that the stance and swing look "incomplete and too unnatural" after build step 1 (player-visuals §13.10: at the swing's start the bat lies nearly level, pointing back at the catcher, the hands in front of the chest; pictures `client/Saved/Shots/bat1_swing_3_loaded.png`, `bat1_swing_7_swing.png`). PLAN D21 holds: the measured figures are Fortenbaugh 2011 [4] and Welch 1995 [3]; OpenBiomechanics [6] is not used.

**Summary.** The sim's bat agrees with the measured bat at contact (elevation −31° against −30 ± 5°, azimuth +5° against −8 ± 12°) and parts from it before about −40 ms. Earlier than that it is a different swing: flat instead of 40° up, pointing at the catcher instead of wrapped toward the pitcher's side, in line with the arms instead of cocked 120–130° behind them, and turning at a third of the measured rate at −93 ms. The cause is structural: the sim's whole swing is one rigid plane tilted about 32°, laid backward from the contact pose by two bell-shaped turning rates, and the same handful of parameters shape both its first 100 ms and its last 50 ms. Nothing that decides a game reads the bat before 25 ms ahead of the planned contact **except a checked swing**: the brake and the ruling on whether he went. Recommendation: (a), a launch term in the sim's swing that brings the bat from the measured launch pose onto today's path by 40 ms before contact; every swing that is not checked stays bit for bit; checked swings change, and the check-swing ruling must be rewritten anyway for a wrapped bat.

## 1. Where the swing-start pose and the path come from

The sim's swing (agent-models §3.6, `bb_agents/src/swing.rs`) is a kinematic double pendulum in **one plane for the whole swing**:

| What | Where | Effect on the path |
| :-- | :-- | :-- |
| Pose at time t: knob = C + ρ·ê(φ(t)), bat axis = â_c turned about the plane normal n by θ(t) | `swing.rs:331-341` | The bat never leaves the plane Π, so its elevation can never exceed the plane's tilt β (about 32°; the measured bat is 39–44° up in the load). |
| φ(t), θ(t): integrals of Gaussian rate bumps, both zero at t_c (so the pose is laid **backward** from contact) | `swing.rs:220-247` (`Bump::angle`), `swing.rs:279-288` | The start pose is whatever the contact pose turned back by the bumps' areas is: hands about 76°, bat about 123° (agent-models §3.6 "Derived"). |
| Plane and contact pose: tilt β = 32° − 9.9°·(h − ½) + 11°·x_in, attack angle α, direction ψ, bat ⊥ path at t_c, C at x = −1.10 m, ρ from the aim | `swing.rs:377-405` (`contact_pose`), template `swing.rs:105-139` | Fixes where the bat points at contact and therefore, through the fixed plane, at the start. |
| Timing: `hand_width` 0.055 s, `bat_width` 0.055 s, `bat_peak` +0.010 s, hands' peak τ_h = 2.146·w_h − T_swing = −0.032 s, `hand_rate_ratio` q = 0.32, `lag_c` = 0; all scaled by T_swing/0.150 | `swing.rs:119-121`, `swing.rs:479-482` | Fitted to Statcast's swing length (7.2 ft over the last 150 ms) and sweet-spot speed at contact (agent-models §3.6). |
| t_s ("swing start") = hands at 10% of their peak rate = t_c − T_swing | `swing.rs:312-315` | A definition on the hand bump, not a pose. |
| Before t_s: stance pose and a 50 ms straight-line load, presentation only | `bb_game/src/timeline.rs:31` (`LOAD_S`), `:40-43` (`stance_pose`), `:281-290` | The client draws its own stance and load since step 1 (player-visuals §13.4, §13.10) and ends it at the sim's t_s pose. |

**Which parameters shape which part.** There is no parameter for the early part alone. The first 100 ms are set by β (the plane), the bumps' widths and peak times, q and ρ; the last 50 ms by the same parameters plus α, ψ, the aim, `BatSpeed` (k) and `lag_c`. Changing any of the shared ones to move the start moves contact too: `bat_width` and `bat_peak` were fitted to Statcast's swing length, q to the hands' share of the sweet-spot speed, β to Statcast's swing-path tilt. That is why option (a) below adds a term rather than retuning these.

## 2. What of the pre-contact path can change a game

| Reader | Reads | File and line | Before t_c − 25 ms? |
| :-- | :-- | :-- | :-- |
| Contact detection | bat pose and velocity on [t_c − 0.025, t_c + 0.025] (an overlap at the window's first instant counts as contact) | `bb_physics/src/contact.rs:78` (`WINDOW_S`), `:88-93`, `:120-122` | No |
| Collision | bat kinematics at the contact instant | `contact.rs:129-131` | No |
| **Check swing: brake** | the batter brakes at t_now + latency (0.05 s, `bb_agents/src/batter.rs:54`) if t_now < t_abort = t_s + 0.6·(t_c − t_s) = t_c − 0.060 | `bb_game/src/pa.rs:532`, `:547-551` | **Yes**: brakes start from about t_s + 0.01 to t_c − 0.01 (§3) |
| **Check swing: the braked path** | angle and rate of both bumps **at the brake time**, then a linear stop over 40 ms | `swing.rs:274-288` | **Yes** |
| **Check swing: the ruling** | bat axis at the end of the brake: "swung" if `a.y ≥ 0` (the barrel no longer points toward the catcher) | `swing.rs:304-311`, `pa.rs:580` | **Yes** |
| Contact on a checked swing | the braked bat, within the same window | `contact.rs:88-93` with `t_brake` set | Indirectly (where the bat stopped) |
| Fielders' read of the swing | bat yaw **at t_c**; the swing's lead t_plate − t_s (a time, not a pose) | `bb_game/src/game.rs:409-415` | No |
| When the swing may start | `start_no_earlier_than(t_commit)` on t_s; commit time from T_swing | `swing.rs:319-328`, `pa.rs:473`, `:528`, `:569` | No (times only) |
| Hit by pitch | a fixed body capsule, never the bat | `pa.rs:368-386` | No |
| The batter's perception, the catcher, the pitcher | nothing reads the bat (searched `rec.swing`, `swing_done`, `BatPose` users) | | No |
| Event log | `check_swing` and `judged_swing` flags only | `bb_game/src/eventlog.rs:147-148` | (outcome) |
| Timeline, FFI (presentation) | the executed swing every 5 ms from t_s to t_c + 0.12 s | `timeline.rs:271-300`; `bb_ffi/src/lib.rs` | presentation |

So on a swing that is not checked, **nothing that decides a game reads the bat earlier than 25 ms before the planned contact**. On a checked swing, where the bat is when he brakes and how far it turns decide strike or ball, and the ruling's test `a.y ≥ 0` assumes the bat starts pointing toward the catcher: **a wrapped bat (tip toward the pitcher's side, a.y > 0) would be ruled a swing before it moves.** rules-spec §4.5 already words the rule as yaw *measured from the bat's pose at the start of the swing*; the code's shortcut is only right for today's start pose.

## 3. The sim's bat against the measured bat

PROBE-PLACEHOLDER

## 4. Options

OPTIONS-PLACEHOLDER

## 5. Recommendation and the owner's question

REC-PLACEHOLDER

## 6. Not established

NOT-PLACEHOLDER
