# Research for a force-driven body (PLAN D31, §14.1 phase 0): index

Written 2026-11-25 by the worker `body-research`. Docs only; nothing here changes code, a spec or a calibration. Each note says, for every number, how I read its source (full text, a table of an open paper, an abstract only, a secondary description) and lists what stayed unfound.

| n | Note | What it answers | Strongest source | Stayed unfound |
| :-- | :-- | :-- | :-- | :-- |
| 1 | [Forward-dynamics precedents](1-forward-dynamics-precedents.md) | How a body driven by a few torque generators has been built for golf, cricket bowling, batting and pitching; how few generators were enough; how they were kept on a path | MacKenzie & Sprigings 2009 (4 generators, table of parameters, one golfer fitted to 1° and 1.3 m/s); Felton et al. 2023 (16-segment cricket model, 88 timings, ten bowlers) | A torque-generator simulation of a baseball swing or pitch with no tracked motion; the nine-parameter strength function's coefficients |
| 2 | [Hitting kinetics](2-hitting-kinetics.md) | Ground forces, hand force and couple on the bat, trunk and hip torque and power in the swing | Koike et al. 2003 (instrumented bat, 5 players: ~750 N along the bat, ~190 N·m shoulder torque, read from figures); Fortenbaugh 2011 and Welch 1995 (in hand) for ground forces | Peak hip, knee, trunk torques in N·m (paywalled or plotted only); pitched-ball hand forces |
| 3 | [Pitching kinetics](3-pitching-kinetics.md) | Shoulder and elbow torques, ground forces, sequence timing, release timing and finger forces | Fleisig 2025 (20 pitchers: 100 N·m internal rotation and varus, 119 N·m horizontal adduction); Kinoshita 2017 (finger forces 50–100 N, release timing) | Drive-leg and stride-leg force time courses as numbers; absolute phase durations; trunk torque |
| 4 | [Segment masses and inertias](4-segment-parameters.md) | The mass, centre of mass and radii of gyration of each segment from height and mass | de Leva 1996 (male table, all segments, full text) with Dempster/Winter alongside | A table measured on baseball players; how to scale lengths from stature |
| 5 | [Strength limits](5-strength-limits.md) | Maximum joint torque and its fall with speed; how fast drive rises | Anderson et al. 2007 (leg torque–angle–velocity parameters); Trunt et al. (33 collegiate pitchers' isokinetic shoulder rotation, N·m/kg) | Strength at the speeds of a swing or a pitch; trunk, elbow, forearm numbers for athletes; measured rise times |
| 6 | [Movement variability](6-movement-variability.md) | How variable skilled timing, release and force are; sizes for a timing or strength error | Jegede 2005, Hore & Watts 2011, Chowdhary & Challis 1999 (release windows 1–10 ms); Wakamiya 2024 (344 MLB starters' release points); Nakashima 2025 (batter's tolerance 9 ms) | Trial-to-trial SD of any muscle group's onset or strength in hitting or pitching |
| 7 | [Datasets and licences](7-datasets-and-licences.md) | Which open data could be shipped, which are cross-check only | OpenBiomechanics licence (cross-check only); Van Trigt dataset CC BY 4.0 (youth pitchers, positions only) | Any open adult-pitcher or hitting data with ground forces and a commercial-friendly licence; Statcast terms |

## The three findings that most change what the design can rest on

1. **No published model produces a motion without fitting it to a measured one.** Every torque-generator model (golf, cricket, batting, OpenSim pitching) finds its onset times, durations and torque scalings by a global search against a measured performance, then optimises. D31's "no motion fitted to its planned end" has no precedent in the sports literature; what a few lumped generators can do is **reproduce one measured motion to 1–4 % in angle and about 1 m/s at the end** (note 1).
2. **Torque generators cannot drive the last links.** The pitcher's shoulder internal-rotation torque in the delivery (90–120 N·m at ~115 rad/s) is larger than the strongest voluntary isokinetic value measured for such pitchers (52–80 N·m at ≤ 4.7 rad/s), and a Hill-type force–velocity curve falls to zero well below 115 rad/s. The late speed of elbow, wrist, the bat's barrel and the ball is made by **inertial (velocity-dependent) torques and elastic storage**, not by muscle drive at those joints (notes 1, 3, 5; Hirashima 2008, Koike's contribution analyses, Roach 2013/14). The design must model segment dynamics and a shoulder elastic element, with generators driving the proximal links.
3. **Error sizes cannot be read from a published onset-time or strength SD; there is none.** They can be sized from outcome spreads that are published: release windows of 7 ms (skilled; 1 ms at the fingers), release-point σ of 3–6 cm (MLB, from 344 starters), a batter's tolerance of about 9 ms and contact-depth σ of 0.27 m (≈ 7 ms) (note 6). Timing is also the lever that matters: re-timing alone moved simulated bat speed 10–12 % and lower-trunk speed 25 % (notes 1, 2).

## What I could not find (summary)

- Peak joint torques of the swing and the delivery's ground forces as **tabulated numbers** (they are in paywalled full texts or in plots: Ae/Koike 2017–2020, Horiuchi 2021–24, Yanaka 2026, MacWilliams 1998, Fleisig 1999 tables).
- Any measurement of **strength at swing or pitch speeds**, and any measured **muscle rise time** relevant to the models.
- **Within-athlete SDs of muscle onset times or strength**; a **body table measured on baseball players**.
- Any **openly, commercially licensed adult hitting or pitching dataset with ground forces** (OpenBiomechanics is non-commercial; the CC BY set is youth, positions only).
- Cross 2005/2009 (bat as a double pendulum), Alexander 1991, Herring & Chapman 1992, Hirashima 2002/2003, Feltner & Dapena 1986 and Werner et al. were not opened.

## Tried and ruled out

Phase 1's dead ends (worker body-3d, 2026-11-25), one line each, so nobody retries them. Detail: [body-motion §4.4](../../body-motion.md). The commit is where the attempt or its replacement is.

| Tried | Why it failed | Commit |
| :-- | :-- | :-- |
| Placing the 3D body with both feet flat and the pelvis level | [4]'s two knees then ask for pelvis heights 9.5 cm apart at −340 ms and 12.9 cm apart the other way at contact; the rear knee came out at 62–73° against 44–50°, and undriven the lead foot never took more than 18% of body weight | `a3389b8`, replaced in `6df9026` |
| Moving the pelvis forward until both knees fit with flat feet | needs 8–12 cm, puts the centre of mass where [4]'s ground forces say it is not, and makes the pelvis travel backward after −170 ms | `6df9026` |
| Every drive on its bound, set by hand | a leg on its bound is a jump: both feet off the ground by −110 ms | `a3389b8` |
| Starting the placed body at rest, or with four rates set by hand | the pelvis stays back; with every velocity taken as a slope of [4]'s curves it follows [4]'s travel for 100 ms undriven | `6df9026` |
| Starting velocities as slopes over ±3 ms | the digitised pelvis curve's steps give 0.41 m/s instead of 0.62; ±5 and ±10 ms agree | `6df9026` |
| The whole-body fit under the rules, 16 channels, no range stop at the shoulders | bat round 36–44 ms late at 18–19 m/s | `a60e856` |
| The same with the shoulders' range stops (135°) | 41–52 ms late at 17–18 m/s; the stop carries only 63 N·m | `987e92d` |
| One rule relaxed at a time on that fit (arms' onset free; shoulders and elbows × 2; no rise floor; legs and trunk × 1.5; wrists × 3) | contact +15.7, +28.1, +30.2, +29.4, +32.0 ms; never inside ±5 ms, never over 21 m/s; the pelvis peaks after contact in every one | `987e92d` |
| A stop on the wrists' cock (5° or 20° past the start) | moves contact 3–5 ms, the bat's speed not at all | `987e92d` |
| "The legs cannot turn the pelvis in time" | they can: the lower body alone, inside the bounds, has the pelvis at 497–588°/s at −93 ms under three weightings; the pelvis was late in the whole-body fit because of what the upper body asks of it | `876b3aa` |
| "The fit's ground-force terms straighten the legs and delay the pelvis" | with those terms the lower-body fit has the pelvis at 569°/s at −93 ms and the lead foot at 128% of body weight; the knees still reach their stops | `876b3aa` |
| Hip abduction and adduction as the missing part | adding them (about the pelvis's forward axis) was not what the pelvis needed; the inverse dynamics has them inside their bounds except the lead hip's adduction, which is at it | `876b3aa` |
| Solving the hands' point exactly from both elbows and the lag at every instant | no exact solution from −85 to −30 ms (up to 7° of misfit); the nearest point is taken and the path smoothed | `504b263` |
| Reading peak torques off the inverse dynamics at a 5 ms differencing | noise: peaks double against 10 and 15 ms; the impulses and the 10–15 ms peaks are what hold | `504b263` |
| Splitting a hip's torque about the pelvis's own axes, or about the clinical flexion, floating and thigh axes (not square to each other) | both put 200–280 N·m on "the rotators"; along the thigh's own three axes, how strength is tested, the rotators need 49–68 N·m | `504b263` |
