# 10. Smoothed sources: what in the project copies a motion that a source smoothed (PLAN D35)

Written 2026-11-25 by the worker `smooth-audit` on `work/smoothing-audit` (from main `65be0c2`). Docs only: no code, spec, fit or calibration changed. The owner's instruction after D35: "Make sure we aren't trying to mimic any other motion that is based on smoothed data."

Evidence levels as in the other notes: **FT** full text read, **TAB** a table of it, **FIG** read off a figure, **AB** abstract only, **EXC** an excerpt or vendor page, **derived** computed here from measured numbers by the stated method, **[E]** assumed. Every size below that says "derived" is an estimate from a stated shape passed through a stated filter, not a measurement.

**The probe.** `sim/target/codex-probes/smoothing-audit/probe.py` (scipy through `uv run --with scipy --with numpy python probe.py`; 20 s; output `probe.out` beside it; gitignored, like every probe there). It filters *positions* (an angle, or a marker's coordinates) and differentiates them with a 3-point central difference, as [4] did ([4] §3.4: "A three-point central difference method was used to calculate global velocities", FT). [4] states only "a 12 Hz Butterworth filter", so four variants are run [E]: **A** 2nd order forward and back (a 4th-order zero-phase filter, −6 dB at 12 Hz); **B** 4th order forward and back (8th-order zero-phase); **C** as A with the two-pass correction (−3 dB at 12 Hz); **D** 4th order in one pass (not zero-phase). `batspeed.py` (body-motion §4.8) was rerun beside it with `BB_FORT_CSV` pointing at the digitised curves; its output is `batspeed.out` there.

## 1. The answer, in one table

Sources: **[3]** Welch et al. 1995 (7 professionals, tee); **[4]** Fortenbaugh 2011 (33 AA hitters, live batting practice); **[OB]** OpenBiomechanics (98 mostly college hitters, a machine); **Statcast** (MLB bat tracking); **Blast** (a sensor on the knob); **ASMI** (pitching kinetics).

| What is matched | To what | Where | Processing that reaches it | Verdict where we read it |
| :-- | :-- | :-- | :-- | :-- |
| **Force-driven body (phase 1 lab fit)** | | `analysis/research/body/fit3d.py` `residuals()`: 15 curves at 5 ms steps from −250 ms to contact, 3 values at contact, the contact instant, the bat's first motion | | |
| bat's azimuth rate curve | [4] App. E | `BAND["bat_az_vel"]` | 12 Hz through the impact | **Affected from about −20 ms to contact**: 2,217°/s read at contact against 2,649–2,841 true (straight-piece form, §4.8): 1.7–2.5 band units at contact; 0.1–0.7 at −10 ms. Also a 3–7% overshoot of the loss at −40 to −30 ms (the filter's ringing, 30–70°/s) |
| bat's elevation curve and at contact (−30 ± 5°) | [4] App. E, Table 1 | `bat_elev`, `AT_CONTACT` | same | **Affected from about −20 ms**: −29.7° read, −33.4 to −35.6° true (§4.8): 0.5–0.7 band units in the curve, 0.7–1.2 SD at contact |
| lag (the cock) curve and at contact (65 ± 10°) | [4] | `bat_lag`, `AT_CONTACT` | same | **Affected from about −20 ms**: 64.5° read, 57.9–61.2° true: 0.3–0.7 units |
| the contact instant (bat at −8° azimuth) | [4] Table 1 | `CONTACT_AZ` | same | **Affected, small**: the true bat passes −8° 1.1–2.3 ms before the smoothed one (derived from §4.8's −4.9 to −1.1° at contact over 2,775–3,164°/s): 0.2–0.5 of the ±5 ms band |
| pelvis rate curve | [4] | `pelvis_rot_vel` | 12 Hz; mean curve | **Not affected by the filter** (peak 157 ms wide: < 0.5%; no sudden event for the pelvis). **Averaging**: peak 3.3% under the swings' own (0.25 band units) |
| trunk rate curve | [4] | `trunk_rot_vel` | same | **Filter 0.4–1.6%** at the peak (124 ms wide). **Averaging**: 5.8% under (0.47 band units at the peak) |
| pelvis travel | [4] | `pelvis_y` | 12 Hz on a slow position | **Not affected** |
| six ground-force curves | [4] | `*_grf_*` | **force plates unfiltered** (FT); mean curve | **No filter. Averaging is large at the peaks**: the lead foot's vertical peak 130% BW on the mean curve against 149 ± 12 for the swings' own (1.3 band units); toward the pitcher 50 against 56 (0.8); lateral 28 against 36 (1.0); the rear foot's lateral 24 against 28 (0.5); the rear foot's vertical 1% |
| lead knee; both elbows (angles) | [4] | `lead_knee_flex`, `*_elbow_flex` | 12 Hz; mean | **Not affected where read** (angles; their rate peaks 109–135 ms wide lose ≤ 2%; the impact does not slow the hands, §3.4). A small kick at the grip after impact, spread back about 20 ms, is not sized |
| pelvis at contact (64.5°) | [4]'s rate curve integrated | `AT_CONTACT` | | **Not affected** |
| the bat's first motion (236°/s at −170 ms) | [4] §4.1.4 | `first_motion_time` | | **Not affected** (170 ms from the impact) |
| the placed starting state at −250 ms (slopes over ±10 ms) | [4] | `body.place` | | **Not affected** (no sudden event within 90 ms; foot-down at −340 ms is soft) |
| the inverse dynamics' torques | [4] twice differenced | `inverse3d.py` | 12 Hz; mean | **Affected in the last 20–25 ms** (the bat is made to brake before contact, so the wrists and forearms are charged a torque no hitter applies). **Averaging makes every peak torque a lower bound** for a single batter: the swings' own rate peaks are 3–16% over the mean curve's, and accelerations more |
| held-out: shoulders' elevations | [4] | `held_out` 1 | | **Not affected** |
| held-out: lead arm's turning rate 1,160 ± 96°/s at −65 ms | [3] Table 2 | `held_out` 2 | 13.3 Hz | **Not affected** (65 ms from the tee impact; a mean of the hitters' own peaks) |
| held-out: muscle activity (EMG) | Shaffer 1993 | `held_out` 3 | phase averages of rectified EMG | Not a motion; outside D35's question |
| **held-out 4: the bat at 98% of its peak at contact, peak 2 ± 5 ms before** | [OB] | `held_out` 4; body-motion §4.8 | **20 Hz on every marker-derived signal, through contact** (FT); contact "an exploratory estimate", its method not stated | **Cannot be told.** A 20 Hz zero-phase filter through a real impact would show the peak 11–14 ms before the impact and about 90% of it at the impact (derived, §3.3). [OB]'s 98% at its contact therefore says its "contact" is not the impact's instant, or its impacts were filtered apart, neither stated. It is not evidence of the true shape until [OB]'s contact event is known |
| the gate's bat speed, 29–34 m/s | [3]; Statcast | body-motion §13 | [3]: 13.3 Hz through the impact, a marker on the bat's end; Statcast: not published | [3]: **affected twice** (the impact, §4.8; and, new here, the filter shrinking a fast-turning marker's speed by 9–15% even with no impact, §3.3). Statcast: **unknown** |
| the hands: 6.5 m/s at contact, 11.0 at their fastest | [4]'s curves through the placement; Blast 10.3–13.0 | body-motion §4.8, §4.9 | 12 Hz; mean | **6.5 at contact: not a smoothing effect** (6.7 with the impact's smoothing undone). **11.0: 1–3% low from the filter, and probably more from averaging** [E] (§5) |
| **Pitcher (§11; design only, nothing fitted)** | ASMI torques; [N3] timings | body-motion §11, note 3 | 13.4 Hz 4th-order Butterworth through the stride foot's landing and ball release | **Affected near release** for the arm's rates (internal rotation peaks narrower than 40 ms lose a third or more, §3.2); the torque peaks near maximum external rotation (30–50 ms before release) less; sizes unknown without a source compared at two cut-offs. Checks, not targets, today |
| **The game's swing today (v2)** | Statcast bat tracking; Blast | `swing.rs`, agent-models §3.6 | Statcast and Blast: not published | **No shape copied from a smoothed curve** (the bumps and τ_b = +10 ms are [E]). Its Statcast numbers' processing is **unknown** |
| **Swing v3 (specified; oracle and goldens built, not in the game)** | [4]'s azimuth rate *as a ratio to its contact value*, −93 ms to contact; [4]'s elevation to −30 ms; [3]'s rates and tip speeds as checks | `analysis/oracle/swing_v3.py` `RATE4_RATIO`; swing-model-v3 §5, §6.3, §6.7.3 | 12 Hz through the impact | **Affected everywhere, not only near contact**: the ratio divides by [4]'s smoothed contact rate (2,214), so every ratio before −25 ms is 1.20–1.28 times the true one; the bat slows into contact (peak at −9 ms, 1.02–1.06 × contact). Elevation to −30 ms: not affected |
| running: top speed | Statcast sprint speed (best one-second window) | agent-models §5.3, §10 | a 1 s moving average | **Affected by under 1%** (already noted in the runner-acceleration note: 8.23 against about 8.25 m/s) |
| throws, pitches in flight, batted balls | Statcast ball tracking | physics-spec, throwing notes | fits to a ball's flight | **Not affected**: a ball in flight has no sudden event; out of D35's reach |
| fielders' braking, jump, dives, slides | Harper 2020 (decelerations); [E] values | agent-models §5.3 | not a recorded curve | **Nothing copied** |
| the client's figures | poses in code, by eye (player-visuals §2–§12); §13's timing table from [4] (design; step 1 built) | player-visuals §13 | 12 Hz | **Only a picture.** §13 keys the body to [4]'s events; the ones inside 20 ms of contact ([4]'s bat peak 10 ms before contact) are smoothing |

## 2. How each source processed its data, in its own words

| Source | Markers | Forces | Through which sudden events | How the curves were averaged | Read |
| :-- | :-- | :-- | :-- | :-- | :-- |
| [4] Fortenbaugh 2011 | "the raw XYZ positions of the markers were passed through a 12 Hz Butterworth filter within the motion analysis software to smooth out any artifacts created from digitizing errors or skin movement" (§3.4). Order and zero-phase: not stated. Cropped "from a few frames before the hitter's initial movement until a few frames past follow-through", so one pass over the whole swing. 300 Hz. Velocities by 3-point central difference | "The force plate data were left unfiltered to avoid oversmoothing and subsequent underestimation of peak GRF" (§3.4); 300 Hz | The bat meeting the ball (inside); the lead foot's landing at −340 ms (inside the file, outside the fit's window); no catch or landing. Contact itself, "the instant when the incoming pitch deviated from its initial trajectory", is a frame typed into a spreadsheet by hand (Appendix D `complete_baseball.m` reads `Ball tracking.xlsx`); whether it was found on filtered ball paths: not stated | "All kinematic and GRF variables were separately plotted against time (relative to BC) with each participant's averaged trials. Those plots were then averaged together" (§3.5): aligned at contact in real time, not time-normalised; each batter's trials averaged, then the batters. The swings' own peaks are given in Tables 2–5 (computed per trial by `complete_baseball.m`, over the whole trial) | FT, TAB |
| [3] Welch et al. 1995 | "low-pass filtered with a 4th-order zero-phase Butterworth at 13.3 Hz"; 200 Hz; markers on the bat's handle above the hands and on its barrel end | 1,000 Hz; whether filtered: not stated in the note | The tee impact (inside) | Three best line drives per batter, then means ± SD of the seven batters' own values | FT (as the note records it) |
| [OB] OpenBiomechanics | "Marker-derived signals were filtered with a 4th-order Butterworth low-pass filter at 20 Hz" (datasheet); "Kinematics/kinetics are computed … then filtered" — velocities are filtered as signals. 360 Hz | "4th order Butterworth low pass filter with a cutoff frequency of 40.0 Hz"; 1,080 Hz | The impact (inside: the whole signal is filtered). "Contact and marker-derived bat metrics are exploratory estimates; the illustrated ball flight is not measured" (hitting README): no ball is tracked, and how contact is found is not stated | Per swing (one row per swing); also Blast's sensor at contact beside the markers (30.5 m/s against 29.6) | FT (README, datasheet) |
| ASMI (Fleisig; Escamilla 2025; [N3]) | "4th-order Butterworth filter at 13.4 Hz", 240 Hz ([N3]'s summary); later ASMI papers: "fourth-order zero-lag Butterworth filter; 13.4-Hz cutoff" | not stated in [N3] | The stride foot's landing and ball release, both inside | Means of the pitchers' own peaks | EXC (note 3; search results) |
| Statcast bat tracking | Hawk-Eye, 12 cameras a park, "five of them running at 300 frames per second"; "the number used is from the video frame when the bat is closest to the ball" (FanGraphs, quoted in collision-targets §4). How a velocity is made from positions (a frame difference or a fitted model across frames, and across the impact or not): **not stated**; "MLB has spent more than two years refining the bat-tracking model" | — | The impact, if the fit spans it: not stated | A player's season figure is "the average of the top 90% of a player's swings" | EXC |
| Blast | A sensor on the knob (gyroscope and accelerometer); speeds from its own model. Filtering: **not stated** | — | The impact: not stated | Boards of best or average swings | EXC (vendor) |
| Statcast sprint speed | "feet per second in a player's fastest one-second window" | — | — | over a player's competitive runs | EXC |

## 3. What the filters do where we read them (derived)

### 3.1 A sudden loss at contact

The bat loses about a third of its turning in a millisecond when it meets the ball (body-motion §4.8: a free bat would lose 1,519°/s; [4]'s own curve, refitted, 794–1,073). Through a zero-phase filter the loss arrives early. Share of the loss already showing, by the filter (probe §1):

| | −30 ms | −20 | −15 | −10 | −5 | 0 | 5% of the loss first shows | angle's error at contact |
| :-- | --: | --: | --: | --: | --: | --: | --: | --: |
| [4], 12 Hz at 300 Hz (A, B, C) | −2 to +1% | 5–11% | 14–20% | 25–30% | 39–42% | 54–55% | −20 to −23 ms | −3.0 to −3.9 ms × the loss |
| [3], 13.3 Hz at 200 Hz | −3 to 0% | 4–10% | 13–19% | 26–30% | 41–43% | 57–59% | −15 to −20 ms | −2.3 to −3.1 ms × the loss |
| [OB], 20 Hz at 360 Hz | | −3 to +1% | 1–6% | 12–18% | 32–36% | 56–57% | −11 to −14 ms | −1.6 to −2.2 ms × the loss |
| one pass (D), any | 0 | 0 | 0 | 0 | 0 | 0 | after contact, 8–17 ms late | −19 to −33 ms × the loss |

- **The window D35 asks us to leave out is the last 20–25 ms before contact for [4]** (the "about 40 ms" of body-motion §4.8 is where the filter's response is not yet zero; where it is larger than 5% of the step it is 20–23 ms). A zero-phase filter also rings: at −40 to −30 ms the rate reads 2–7% of the loss *above* the true one.
- **[4]'s filter was zero-phase, or acts as one.** A one-pass filter would leave the rate untouched before contact and delay it after; [4]'s curve falls before contact (2,419°/s at −10 ms, 2,217 at contact) and is about half-way at contact, as the zero-phase variants are. (Derived; the order stays [E].)
- **An angle at contact is off by the loss times 2–4 ms**: 1,000°/s of loss puts the bat's azimuth 3–4° behind at contact, which is §4.8's lag and elevation result from another direction.

### 3.2 A peak of a given width

Kept share of a Gaussian rate peak through [4]'s filter, by its true full width at half height (probe §2): 20 ms 48–58%; 30 ms 66–74%; 40 ms 77–84%; 60 ms 90–94%; 80 ms 95–98%; 100 ms 98–99%; 150 ms over 99%.

[4]'s own peaks, already filtered, are broad: the pelvis's 157 ms at half height, the trunk's 124, the lead knee's extension 135, the rear elbow's 109 (from the angle curves' slopes). **So the 12 Hz filter takes under 2% off any body segment's rate peak in [4]**; the trunk 0.4–1.6%. What blunts the body's peaks is the averaging (§4). The one narrow peak, the bat's, is made by the impact.

For the pitcher (§11): a shoulder's internal rotation rate rising to its peak near release and falling in a few tens of ms is in the 20–40 ms class, where 13.4 Hz keeps half to four fifths. How narrow ASMI's peaks are was not read [E], so for the delivery this is a warning, not a size.

### 3.3 A marker on a fast-turning bat, and the bat's speed

A filter run on a marker's coordinates shrinks a curved path: a point going round at 2,841°/s (7.9 Hz) keeps 1/(1 + (7.9/12)⁴) = 84% of its circle's radius through filter A at 12 Hz. Speeds taken from filtered marker paths are low even with no impact; **angles between two markers are not** (both shrink together). Probe §4: a marker 0.75 m from fixed hands [E], the bat's true rate from §4.8's straight-piece fit (1,830°/s at −40 ms, 2,841 just before contact, a loss of 1,073 at contact) joined to [4]'s curve at ±70 ms. (This reconstruction reads 190°/s under [4]'s curve at −10 ms when filtered; the ratios below are the result, not the absolute speeds.)

| The bat's end, speed from the filtered marker | [4] 12 Hz | [3] 13.3 Hz | [OB] 20 Hz |
| :-- | --: | --: | --: |
| no impact, at contact, of the true speed (A; B and C) | 81%; 89% | 85%; 91% | 94%; 96% |
| with the impact, at contact | 67–71% | 68–71% | 74–75% |
| with the impact, the filtered speed's peak | 13–17 ms before contact | 15 ms before | 11–14 ms before |
| the azimuth rate from the same filtered markers against the angle filtered directly | within 40°/s | within 40°/s | within 20°/s |

- **[4] is not touched by this**: it reports angles and their rates, and the placed motion's speeds are made from angles.
- **[3]'s bat-end speeds are.** Its 31 m/s (fastest, −15 ms) and 29 (at contact) are from its end marker's filtered path: low by 9–15% from the path's curvature alone [derived; the order of "4th-order zero-phase" decides between 85 and 91%], so at least 34–36 m/s at the end before any impact correction, and the impact's spread takes more at contact. Its peak at −15 ms is the filter's, as [4]'s is. The gate's "29" (body-motion §13) and swing-model-v3's tip-speed checks inherit both.
- **[OB]'s bat speeds are low by 4–6% from the curvature at 20 Hz** if they come from filtered marker paths or filtered velocity components (both are filtered, per its datasheet), before any impact effect. Blast's sensor on the same swings reads 30.5 m/s against the markers' 29.6 at contact, 3% higher (Blast's own processing not stated).
- **[OB]'s 98% at contact.** Through its own 20 Hz filter a real impact would leave the bat's speed at contact near 90% of the filtered peak and the peak 11–14 ms early. [OB] reports 98% and 2 ± 5 ms. So its "contact", an "exploratory estimate" without a tracked ball, is probably not the impact's instant (it may sit at or before the speed's top). The shape body-motion §4.8 took from it ("a real bat does not slow before it meets the ball") is not shown by these data; the physics says the same (nothing touches the bat before the ball does), and that is what the conclusion rests on.

### 3.4 The hands (the owner's named case; body-3d-7fix has the detail)

My own estimate. **6.5 m/s at contact is not a smoothing effect**: the placed motion on the curves with the impact's smoothing undone has the hands at 6.6–6.7 m/s at −10 ms and contact (`batspeed.py` §4, 2nd order each way, corrected), because what the filter spreads is the bat's turning, and the hands do not jump at impact. On the game's bat (physics-spec §6.2: 0.965 kg, centre of mass 0.597 m from the knob, 0.0382 kg·m² about it) the centre of percussion about the lead hand is 0.674 m from the knob and the sweet spot 0.714 m, 4 cm beyond it, so a *free* bat's handle would be pushed **forward** by the ball (about 0.5 m/s per N·s; 8.9 N·s at [4]'s mean exit speed), never slowed; held, far less, and [4] reports no hand path, so none of that reached the placement. **11.0 m/s at the hands' fastest (−50 ms) is 30 ms outside the impact's reach.** The filter takes 1–3% off a hand-speed peak 80–100 ms wide (probe §5; the placed curve falls from 11.0 at −50 to 9.5 at −20 and 6.5 at contact, so it is at least that wide). The bigger and unknown part is averaging: the arm's own rate peaks are 8% (lead shoulder's azimuth) and 11% (rear elbow's extension) over [4]'s mean curves (§4), and the hands are their sum, so a single batter's hand peak is plausibly 5–10% over the mean motion's [E]: 11.5–12 m/s, inside Blast's 10.3–13.0 (whose own processing is not stated). The fitted body's 3.3 m/s at contact against 6.5 is therefore not explained by smoothing.

## 4. Averaging is smoothing too

[4]'s mean curves are aligned at contact, in real time, each batter's trials averaged first (§2). Near contact they are aligned well; elsewhere every difference in a batter's timing blurs a peak. Tables 2–5 give the swings' own peaks and their times for middle fastballs (probe §6; TAB):

| | Mean curve's peak | Swings' own peaks, mean ± SD | Their times, mean ± SD | Curve under the swings' | In the fit's band units |
| :-- | --: | --: | --: | --: | --: |
| pelvis rate | 581°/s at −70 ms | 601 ± 38 | −72 ± 15 ms | 3.3% | 0.25 |
| trunk rate | 766 at −57 | 813 ± 71 | −59 ± 15 | 5.8% | 0.47 |
| bat azimuth rate | 2,435 at −10 | 2,486 ± 152 | −10 ± 4 | 2.1% | 0.20 |
| lead knee extension rate | 263 at −40 | 313 ± 41 | −30 ± 25 | 16% | (an angle is fitted) |
| lead shoulder azimuth rate | 342 at −43 | 373 ± 92 | −44 ± 13 | 8.3% | (not fitted) |
| rear elbow extension rate | 868 at +17 | 979 ± 169 | +16 ± 18 | 11% | (an angle is fitted) |
| lead foot, vertical force | 130% BW at −93 | 149 ± 12 | −93 ± 19 | 13% | **1.3** |
| lead foot, toward the pitcher | 50 at −83 | 56 ± 5 | −90 ± 17 | 11% | 0.8 |
| lead foot, lateral | 28 at −123 | 36 ± 3 | −135 ± 16 | 22% | **1.0** |
| rear foot, lateral | 24 at −127 | 28 ± 2 | −132 ± 15 | 14% | 0.5 |
| rear foot, vertical | 92 at about −500 | 93 ± 2 | −491 ± 55 | 1% | 0.1 |

- **The forces are unfiltered, but the mean curve still has no batter's peak in it**: every batter's front foot peaks at 149% of body weight on average, the mean curve at 130. [OB]'s college hitters on a machine: 2.0 BW (SD 0.4, filtered at 40 Hz). The fit's average body is asked for 130 and gives 127.
- The kinematic peaks in Tables 2–3 are each filtered at 12 Hz too (computed from the same filtered paths); the bat's "peak at −10 ± 4 ms" is in every swing, so it is the filter's, not the averaging's.
- The spread of the swings' peak times (15–25 ms SD) would blur a Gaussian peak of 20–57 ms SD by the amounts shown, which fits the widths seen. A timing-aligned mean (each swing's curve shifted to its own peak, or registered by events) would keep a batter's peak; [4] did not make one.
- [3] gives means of the hitters' own values (FT, as noted), so its peaks are not blunted by averaging.

## 5. Today's game

- **The swing (v2, `sim/crates/bb_agents/src/swing.rs`; agent-models §3.6)** is fitted to Statcast (the sweet spot's speed at contact, swing length, attack angle and direction at the frame nearest the ball, swing path tilt over the 40 ms before contact, contact depth, the within-batter SD of bat speed) and Blast (time to contact 0.149 s, the vertical bat angle). None of its shapes copy [3], [4] or [OB]: the two Gaussian bumps, τ_b = +10 ms (the bat still speeding up at contact) and q are [E]. **Statcast's processing is the open item.** If its velocity at the frame nearest the ball comes from a model fitted across the impact, its bat speeds carry some of the same loss; if from frames before the impact, none. Not published (collision-targets §4 searched; this search found only "five [cameras] running at 300 frames per second" and the 90% rule). At 300 frames a second a difference over one frame either side of contact would straddle the impact.
- **Running**: Statcast's sprint speed is the best one-second window of a run; for v = v_max(1 − e^(−t/τ)) at τ = 0.76 s the window reads under 1% low (runner-acceleration note §2 already put the asymptote at about 8.25 against the sim's 8.23 m/s). No sudden event inside.
- **Throws, pitches and batted balls** are matched to tracked flights, which have no sudden event; not D35's concern.
- **Nothing else in the sim crates is fitted to a recorded body curve** (`grep` of `sim/crates` for Fortenbaugh, Welch, Appendix E: only `bb_game/examples/swing_contact_baseline.rs`, a measuring example that uses [4]'s azimuth convention).

So **nothing in the game as it plays today is fitted to a smoothed stretch that is known**; the one unknown is how Statcast makes a bat speed.

## 6. What to change, ordered by what is at stake

1. **Swing v3, before its build step 4 (the game as it will play).** `analysis/oracle/swing_v3.py` divides [4]'s rate curve by its smoothed contact value (`RATE4_RATIO`, line 197, over `RATE4_DPS[-1]` = 2,214°/s); `I_HAT`, `I_93` and `RATE_93` (0.558) follow. With the true contact rate 2,649–2,841°/s, every ratio outside the last 25 ms is 1.20–1.28 times the true one: v3's bat, scaled to Statcast's contact speed, turns 20–28% faster at −93 ms than [4]'s hitters did against their own contact speed, sweeps more of its arc early, and then slows for its last 9 ms (swing-model-v3 §5 row "Peak bat azimuth rate", §6.7.3). A fix touches the rate shape (`RATE4_DPS` or its denominator, with the last 20–25 ms replaced by a stated unsmoothed form [E]), the sweep integrals, the goldens `golden/swing_v3.json`, and swing-model-v3 §5, §6.3, §6.7.3, §11 (the 1.0–1.15 tip-speed ratio check and the uncocking peak at −20 ms from [3]). Its elevation is used only to −30 ms and is clean.
2. **The force-driven body's fit (phase 1; body-motion §13, `fit3d.py`).** Leave out, or replace with a stated unsmoothed form, the last 20–25 ms of the bat's rate, elevation and lag curves, and read the lag and elevation at contact and the contact instant (−8°) as the unsmoothed values (§4.8's table gives them, [E] form). Everything else the fit reads is clean of the filter. This is what Q59 and D35 already ask; this note sizes the window (20–25 ms, not 40).
3. **The fit's force targets (averaging).** The lead foot's three peak forces on the mean curve are 11–22% under the swings' own (0.8–1.3 band units). Either fit the peaks to Tables 4–5 (the swings' own values and times) beside the curves, or state that the average body is fitted to a blurred force pattern and read its peak forces against Table 5 as a held-out check. The same holds, smaller, for the trunk's rate (0.47 units).
4. **The gate's bat speed.** Read Statcast's figure as the target and [3]'s only as a lower bound (two effects push it down, §3.3). Ask MLB, or compare Statcast with a sensor on the same swings, to settle Statcast's own processing.
5. **Held-out check 4 ([OB]'s 98% at contact).** Drop it, or keep it only as a check that the bat is not slowing before contact, without [OB]'s numbers, until [OB]'s contact definition is known.
6. **The inverse dynamics (`inverse3d.py`).** Do not read any torque in the last 20–25 ms; treat peak torques from the mean motion as lower bounds for a single batter.
7. **The pitcher (phase 5).** Choose the delivery's dataset with its filter stated; ASMI's 13.4 Hz cannot give the arm's rates near release, and its torques near maximum external rotation should be read as lower bounds. Look for a source run at two cut-offs or filtered separately around release.
8. **The client (player-visuals §13; only a picture).** The body's timings 10 ms or less before contact taken from [4] (its bat peaking 10 ms early) are the filter's; the sim's own bat (peak +10 ms) is already what the figure should follow.

## 7. For the owner

**What we were copying that was not real.** Two studies we use filtered their camera data over the whole swing, straight through the moment the bat meets the ball. That makes the bat look as if it starts slowing about 20 ms before contact; in fact it is still at full speed when the ball arrives. The force-driven body was being fitted to that last 20 ms (its bat's turning, how far the barrel has dropped, how much the wrists are still cocked, and exactly when it meets the ball), and the new swing written for the game (v3, not yet in the game) copies it more widely: it measures the whole swing's speed against the study's slowed-down contact speed, so its bat turns about a quarter too fast early in the swing and then eases off into the ball.

**How big.** At contact the study reads the bat's turning about 20% slow, the barrel 4–6° too high, the wrists 3–7° too cocked, and the contact 1–2 ms late. The older tee study's bat-end speeds (where the "29 m/s" came from) are low twice over: by the same impact effect, and by another 9–15% that its filter takes off any marker going round fast. The college lab we used as a check filters too (less), and how it times contact is unpublished, so its "the bat is at 98% at contact" is not proof of anything.

**Averaging.** The study's curves are averages of 33 hitters who did not peak at the same moment, so no hitter's peak is in them: the front foot's peak push is 130% of body weight on the average curve where the hitters themselves averaged 149%; the trunk's turning 6% lower; the knee's straightening 16% lower. The body is being fitted to a blurred average batter. This is not a filter, but it is the same kind of thing.

**What is clean.** Everything more than about 25 ms before contact: the hips' and trunk's turning (the filter takes under 2% off them), the stride, the knees and elbows, the shoulders, the bat's first move, how the body starts. The hands' speed at contact (6.5 m/s) is not a smoothing effect; their top speed (11 m/s) is a little low, mostly from averaging. The game as it plays today copies no smoothed motion: its swing is fitted to Statcast and its running to Statcast sprint speed (under 1% off). The one thing nobody has published is how Statcast itself turns its camera frames into a bat speed.

**Waiting on you:** nothing new. Q59 and D35 already cover the body's bat targets; items 1 and 3 of §6 are choices for the next swing and body sessions.

## 8. Not established, and what would settle it

- **[4]'s filter order and the two-pass correction**: the window (20–25 ms) and the sizes move by about a fifth between variants. [4]'s unfiltered marker paths, or Motion Analysis's EVaRT/Cortex documentation for 2010, would settle it.
- **Statcast's bat speed**: MLB's methodology (a frame difference or a model fit, through the impact or not).
- **[OB]'s contact event**: its code or authors; or recomputing from its raw C3D files (non-commercial use only, D21), which would also give single-batter peaks unfiltered by averaging.
- **The hands' single-batter peak**: any set with hand markers and per-swing peaks (Blast's is a sensor; its processing is not stated).
- **The pitcher's arm rates near release** at 13.4 Hz against a higher cut-off: no study comparing cut-offs on the same deliveries was found in one search.
- **Not reached**: the fielders' Statcast "jump" and burst (no motion model in the sim copies them), the catcher's pop time (derived in the sim, not fitted), and Kinoshita's finger forces (sensor processing not read).
