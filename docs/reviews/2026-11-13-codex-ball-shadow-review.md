# Ball-shadow review — 2026-11-13

Reviewed `6116939..e436d83` (A15), with HEAD at `e436d83` and `CLAUDE.md` read. No P1 finding. The yaw, elevation calculation, units, and placement order are correct for the supported camera paths. One P2 sizing/owner-acceptance risk and one P3 documentation finding follow.

## P2 — The new cue can exceed the rejected 50 cm shadow's apparent size

**Location:** `client/Source/BaseballClient/BbDirector.cpp:1328–1336`; constants at `BbDirector.h:297–306`.

The distance floor and ground-plane stretch compound. From the settled follow camera, a ground ball at sim `(0,20)` gets a **0.439 × 0.763 m** footprint, projecting to **16.00 × 9.22 px** at 1600×900. A fixed 50 cm circular disc in that same view projects to **18.20 × 6.04 px**. The new cue is narrower but has approximately **1.34× the projected area** of the size the owner found too big on 2026-11-12. At `(0,120)` it becomes **1.191 × 3.405 m**, or **16.00 × 5.60 px**, about **16.2×** the projected area of a 50 cm disc there. At ball height 30 m both dimensions shrink by 0.55; the comparative enlargement remains.

The pitch view also changes, despite A15 originating with visibility after a hit. Near home, a ball 1 m high gets a **0.201 × 0.589 m** shadow: **15.76 × 5.38 px**, versus **11.59 × 1.35 px** before this commit. A 50 cm disc with the same climb shrink would be **38.64 × 4.50 px** there. Thus the new pitch shadow has substantially less area than the rejected disc, but is taller on screen and longer on the ground. Its approximately 20 cm across dimension alone understates the change.

This is a measured size-calibration risk; owner rejection of the new appearance has not been established. The enlarged deep-field cue achieves A15's visibility aim, but the current constants do not establish that the earlier size complaint has been avoided, especially in the near infield. The supplied headless fly shots do not settle that preference.

**Suggested action:** compare the old 15 cm, rejected 50 cm, and proposed cue at the same near-infield, deep-field, and pitch-camera moments before treating the size as accepted. Tune the transverse size and stretch together; if necessary, restrict or ease the enlargement for camera situations where the prior cue was already visible. Keep this entirely in presentation.

## P3 — The documented minimum size and screen share are applied before height shrink

**Location:** `docs/unreal-spike.md:54`, `TASKS.md:344`, and `client/Source/BaseballClient/BbDirector.h:298–300`; implementation at `BbDirector.cpp:1327–1328`.

The descriptions say “at least 15 cm” and “at least 1%” (about 16 px). The implementation multiplies `max(0.15, ViewM * 0.01)` by `Climb` afterward. At 30 m height the nominal screen share is **0.55%**, about **8.8 px**, and the nominal physical floor is **8.25 cm**. For example, the requested pitch-camera calculation at 30 m yields an across dimension of **11.22 cm**, below 15 cm. That elevated pitch-camera example is a mathematical probe, not the normal broadcast camera selection after contact.

The header's `BallShadowM` comment correctly qualifies its minimum with “with the ball on the ground”; the screen-share comment and the two prose documents need the same qualification. Mentioning shrink separately does not make the unconditional minimum claims true. The existing offscreen claim of approximately 5 px is consistent with a high fly: `888 * 0.01 * 0.55 = 4.884` px across.

**Suggested action:** describe the ground-height target and the subsequent 1.0-to-0.55 multiplier explicitly. Preserve the intended height cue unless the owner chooses a different sizing policy.

## Geometry and ordering checks

`ToUE(x,y,z) = 100 * (y,x,z)`. Unreal's yaw-only rotation maps local X to `(cos(yaw), sin(yaw), 0)`. Consequently `atan2(Look.Y, Look.X)` aligns the cylinder's elongated local X with the horizontal camera-to-shadow ray. Reversing that ray would rotate the ellipse by 180° and produce the same footprint. Examples: the follow camera looking toward `(30,100)` gives UE yaw **12.6193°**; the pitch camera looking toward home gives **176.2139°**. No extra axis swap or sign reversal is needed after `ToUE`.

`-Look.Z / |Look|` is the sine of the ray's elevation above the ground, with the camera above the shadow. The follow-camera values at the three requested ground positions are **0.33172, 0.13698, 0.12237**. The pitch-camera value near home is **0.11650**. The stretch changes the central-ray projected aspect ratio from `sin(E)` to approximately `sqrt(sin(E))`, until the 3× cap takes over at `sin(E) <= 1/9`. It deliberately leaves the shadow flatter than a screen-facing circle.

Mode 2 fixes the camera at sim `(0,55,230)` and points its optical axis straight down. Only the point directly underneath has `SinE = 1`; an off-center ground point has a slanted viewing ray even with a vertical optical axis. The code correctly uses that ray. Directly below the camera, the footprint is circular, stretch is 1, and yaw is immaterial. At `(0,120)`, `SinE = 0.96231` and stretch is only **1.0194**. There is no overhead singularity.

For finite frame coordinates and the configured FOVs, no new division-by-zero or NaN path was found. The `DistM > 0` conditional prevents division by zero for coincident camera/shadow positions; the other branch uses `SinE = 1`. The square-root input is at least `1e-4`, so its reciprocal is finite and capped at 3. A horizontal or clamped-below-ground ray gives the capped stretch. Unreal's generic `Atan2` implementations explicitly return zero for `(0,0)`; at the vertical ray the choice of yaw cannot affect the circle. The helper assumes successful actor spawning and finite inputs, as the surrounding camera/ball code already does.

In `Tick`, both timeline sampling and `ResetFrame` enter the same `bHave` block. Ball placement/visibility precedes actor posing, then `UpdateCamera` runs, then the optional dive-shot location/rotation override, and finally `PlaceBallShadow(Frame, bBallShown)`. Mode 2's early return is only from `UpdateCamera`, so it still reaches shadow placement. FOV changes are also complete before sizing. Screenshot requests are queued for rendering after these updates.

The camera and initially hidden shadow are created in `BeginPlay` before simulation playback can enter this block. Preview modes return before starting the sim and leave the ball/shadow hidden. `BeginPlayback` and reset transitions near the end of `Tick` change playback state but do not move the camera or displayed ball; the next successful frame updates them together. If sampling fails, the previous ball, camera, and shadow remain displayed together, as before. Thus a retained old frame is possible, but no new mismatch between the shadow and the displayed ball/camera was introduced. There is no new shown-shadow-before-camera path under normal actor initialization.

Everything the old inline block did is retained: XY directly below the sampled/staged ball, center at 0.012 m, Z scale 0.002, negative-height clamp, height shrink, and the exact same `bBallShown` visibility flag. With the 1 m cylinder primitive, the center is **1.2 cm** above ground and its thickness is **2 mm**, putting its bottom at **1.1 cm**, above the mound layer's 5 mm top. Collision remains disabled and the cue still casts no shadow. No sim file, trajectory, or physics outcome changed.

## Computed sizes

These are ideal projected planar footprints, **width × height in pixels**, before occlusion, antialiasing, and the cylinder's 2 mm thickness. The view is 1600×900. The follow camera is at sim `(0,-34,19)`, settled at FOV 42°, with its look target at the ball's XY and height `0.5 * min(ball_z,15)`, as in `UpdateCamera`. Actual transition frames vary with the interpolated aim and FOV (initially 48°). “Before” uses this commit's parent: a circular 15 cm disc times the same climb factor.

| Ball XY (m) | Ball height (m) | Before (px) | After (px) | After footprint, across × along (m) |
|---|---:|---:|---:|---:|
| (0,20) | 0 | 5.46 × 1.81 | 16.00 × 9.22 | 0.439 × 0.763 |
| (0,20) | 30 | 3.03 × 1.01 | 8.87 × 5.15 | 0.242 × 0.420 |
| (30,100) | 0 | 2.26 × 0.31 | 16.00 × 5.92 | 1.064 × 2.876 |
| (30,100) | 30 | 1.24 × 0.17 | 8.81 × 3.27 | 0.585 × 1.582 |
| (0,120) | 0 | 2.01 × 0.25 | 16.00 × 5.60 | 1.191 × 3.405 |
| (0,120) | 30 | 1.11 × 0.14 | 8.81 × 3.09 | 0.655 × 1.873 |

The pitch camera is at sim `(-9,136,16)`, aimed at `(0,6,1)`, FOV 8.5°. Shadow XY is home `(0,0)`:

| Ball height (m) | Before (px) | After (px) | After footprint, across × along (m) |
|---|---:|---:|---:|
| 0 | 11.77 × 1.37 | 16.00 × 5.46 | 0.204 × 0.598 |
| 1 | 11.59 × 1.35 | 15.76 × 5.38 | 0.201 × 0.589 |
| 30 (probe) | 6.47 × 0.75 | 8.80 × 3.00 | 0.112 × 0.329 |

The apparent-size formula uses slant distance, whereas perspective magnification uses camera-space depth. Thus the “fixed share” comment at `BbDirector.cpp:1321–1322` is an approximation, especially during camera lag or the dive override. The settled follow view gives only the small differences from 8.8 px shown above. In overhead mode, the ground shadow at `(0,55)` is **16×16 px**, while `(0,120)` is approximately **16.63×16.95 px**, because its slant range is greater than its depth. This produces no singularity or loss of visibility in the checked cases.

## Verification

- Read the full four-file diff, `CLAUDE.md`, `client/run.sh`, coordinate conversion, actor construction, playback/reset transitions, `Tick`, camera updates, and `FBbSim::Sample`. Confirmed the commit changes only the two client files and two documentation files.
- Read the installed Unreal 5.8.4 `RotationTranslationMatrix.h`, `GenericPlatformMath.cpp`, `CameraComponent.cpp`, `CameraStackTypes.cpp`, and `BaseEngine.ini`. The camera's reference aspect ratio is 1.777778. Although the engine defaults to MaintainYFOV, its nonlegacy projection converts the horizontal FOV using that reference aspect ratio; at 1600×900 the horizontal FOV is therefore the requested 42°/8.5°. The legacy projection switch defaults to false; no project override was found.
- Ran independent Python 3 calculations through shell heredocs, without creating probe files. Used `f = 800 / tan(FOV/2)` (**2084.0713 px** follow, **10765.3003 px** pitch), the code's size/stretch formulas, and perspective division in a camera basis. Projected 16,384 points around each ellipse to obtain the tabulated bounding dimensions, including off-axis aim at 30 m height. Checked yaw alignment, zero distance, zero/tiny elevation, vertical viewing, and overhead off-center viewing; all checked size values were finite. The thickness can add up to approximately 0.16 px vertically in the pitch view; tabulated footprints omit it.
- Inspected the pre-existing seed-7 PNGs `shot_13.png`, `shot_16.png`, and `shot_19.png` and their PNG dimensions (**888×500**). The visible cue and inherited log command match the documented screenshot scenario; the log reports a seven-play, 84-second run. These are existing artifacts, not a fresh reproduction or owner acceptance test. Approximately 5 px across at that resolution is consistent with the high-ball calculation; the cue grows again as the ball descends.
- Did not build or launch Unreal, or run sim tests: this review verifies the rendering formulas and control flow, with existing images as supporting evidence. No fresh runtime assertion of every camera/reset path is claimed. Only this review file was written; source, tests, specs, and `TASKS.md` were left unchanged.

## Resolution

Both findings fixed on 2026-11-13.

- **P2 (fixed).** The view share went from 1% to 0.7% (`BallShadowViewShare`) and the stretch cap from 3× to 2.5× (`BallShadowMaxStretch`). The rejected 50 cm disc is now the ceiling, and the deep field still has to show. These are the review's settled follow-camera views on a 1600 × 900 window, ball on the ground, by the code's formulas along the central ray:

  | View | Before A15 | e436d83 (1%, 3×) | Now (0.7%, 2.5×) | Rejected 50 cm |
  |---|---|---|---|---|
  | Near infield (0,20) | 5.5 × 1.8 | 16.0 × 9.2 | 11.2 × 6.5 | 18.2 × 6.0 |
  | Deep (0,120) | 2.0 × 0.2 | 16.0 × 5.6 | 11.2 × 3.4 | 6.7 × 0.8 |
  | Pitch camera, at home | 11.8 × 1.4 | 16.0 × 5.5 | 11.8 × 3.4 | 39.2 × 4.6 |

  In the near infield the shadow now has about two-thirds of the rejected disc's area and is narrower. At the pitch camera the 15 cm floor governs, so the width is unchanged and only the height grows. Deep flies stay far larger than the rejected disc, which A15 needs. Re-checked in headless seed-7 shots after a rebuild: the shadow still shows under the fly to the right-field corner (shots 14 and 17). The owner has not yet judged the size in a window, and a single constant changes it.
- **P3 (fixed).** The header, `docs/unreal-spike.md` and `TASKS.md` now say that the 15 cm floor and the view share hold with the ball on the ground, and that the climb's shrink applies after both. The code comment on the view share also says that slant distance stands in for camera-space depth.
