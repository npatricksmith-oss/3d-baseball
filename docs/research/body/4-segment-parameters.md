# Body research 4: body segment masses and inertias (2026-11-25, worker body-research)

For [PLAN D31 / §14.1](../../../PLAN.md) item 1: segment masses and inertias from `Height` and `Mass`. Reading status: **FT** = full text or the full table read; **2nd** = a transcription of a table in a notebook (checked against the primary where I could).

## 1. The recommended table: de Leva 1996, male, Table 4

P. de Leva, "Adjustments to Zatsiorsky-Seluyanov's segment inertia parameters", J Biomech 29(9):1223–1230 (1996), DOI 10.1016/0021-9290(95)00178-6. **FT** (the full text of the technical note, from a copy hosted at robotics.unibs.it, and a second transcription of the same Table 4 in a Python notebook that agrees on every male value I compared). The note adjusts Zatsiorsky & Seluyanov's gamma-ray-scan data (**100 male and 15 female college-aged Caucasians, mean ages 24 and 19 y, mostly undergraduates of a physical-education college**) so the centre-of-mass positions and radii of gyration refer to **joint centres** instead of bony landmarks. The male sample averaged **73.0 kg and 1.741 m**.

Male values. Mass is a fraction of body mass `M`. The centre-of-mass position is the fraction of segment length `L` from the **proximal (origin) endpoint**, except the head (from the vertex). Radii of gyration `r` are fractions of `L` about the segment's **centre of mass**, about the sagittal, transverse and longitudinal axes (the "sagittal" r is for rotation about the segment's front-back axis; "transverse" about its left-right axis; "longitudinal" about its long axis; this is de Leva's naming). Lengths are for the 1.741 m sample.

| Segment (origin → other end) | Mass | Length L (mm) | CM from origin | r sagittal | r transverse | r longitudinal |
| :-- | --: | --: | --: | --: | --: | --: |
| Head (vertex → mid-gonion) | 0.0694 | 203.3 | 0.5976 | 0.362 | 0.376 | 0.312 |
| Trunk (suprasternale → mid hip) | 0.4346 | 531.9 | 0.4486 | 0.372 | 0.347 | 0.191 |
| Upper trunk (suprasternale → xiphion) | 0.1596 | 170.7 | 0.2999 | 0.716 | 0.454 | 0.659 |
| Middle trunk (xiphion → omphalion) | 0.1633 | 215.5 | 0.4502 | 0.482 | 0.383 | 0.468 |
| Lower trunk (omphalion → mid hip) | 0.1117 | 145.7 | 0.6115 | 0.615 | 0.551 | 0.587 |
| Upper arm (shoulder joint centre → elbow) | 0.0271 | 281.7 | 0.5772 | 0.285 | 0.269 | 0.158 |
| Forearm (elbow → wrist joint centre) | 0.0162 | 268.9 | 0.4574 | 0.276 | 0.265 | 0.121 |
| Hand (wrist → 3rd metacarpal) | 0.0061 | 86.2 | 0.7900 | 0.628 | 0.513 | 0.401 |
| Thigh (hip joint centre → knee) | 0.1416 | 422.2 | 0.4095 | 0.329 | 0.329 | 0.149 |
| Shank (knee → lateral malleolus) | 0.0433 | 434.0 | 0.4459 | 0.255 | 0.249 | 0.103 |
| Foot (heel → toe tip) | 0.0137 | 258.1 | 0.4415 | 0.257 | 0.245 | 0.124 |

(Check: the three trunk parts sum to the trunk, 0.1596 + 0.1633 + 0.1117 = 0.4346, and head + trunk + two of each limb segment = 1.0000.) Several alternative endpoints are tabulated in the paper (for the head, trunk, hand and shank); for a game with the neck and trunk as separate bodies, use the "Head 2 / CERV" and "Trunk 2 / CERV → mid-hip" rows (head 0.5002 of 242.9 mm from the vertex, r 0.303/0.315/0.261; trunk 0.5138 of 603.3 mm from the cervicale, r 0.328/0.306/0.169), which I read from the notebook transcription only.

**How to use it.** `m = fraction × M`; `L = L_table × (Height / 1.741 m)` (the paper gives no scaling rule: scaling length by stature is the assumption, and the paper's own caveat is that mean values "generalised to an individual" carry errors); `I_axis = m (r·L)²` about the centre of mass (de Leva's own formula, `I = (M·m)(l·r)²`). Example, 1.88 m and 95 kg: thigh `m` = 13.45 kg, `L` = 0.456 m, `I` about the left-right axis = 0.303 kg·m², about its long axis 0.062 kg·m²; upper arm 2.57 kg, 0.304 m, 0.0193 and 0.0059 kg·m².

### What the table cannot support

- **Sample.** Soviet-era college men of about 73 kg, not athletes and not heavy men. A 95 kg pitcher is 30 % heavier at 8 % taller: the table assumes the same mass distribution at any size.
- The radii of gyration are scaled from Zatsiorsky's by length ratios (the paper's own "adjustments"), so they inherit Zatsiorsky's uncertainty; the longitudinal radii are the least precise (the paper says so).
- Segment lengths are estimated from the radii (the original data were not readable: the paper's own note); the differences between its two estimates range 0.0–2.1 mm.
- The table gives **no data for the shoulder girdle, the pelvis separately from the trunk, the neck, or the bat**.

## 2. The older table: Dempster (as adapted by Winter)

Dempster 1955 (cadavers) as adapted in D. A. Winter, *Biomechanics and Motor Control of Human Movement* (4th ed. 2009), Table 4.1. **2nd** (a transcription in M. Duarte's open "BMC" Python notebook; I did not open the book). It rests on **8 male cadavers, mostly older** (two of unknown age), measured in the United States.

| Segment | Mass fraction | CM from proximal | r about CM | r about proximal end |
| :-- | --: | --: | --: | --: |
| Hand | 0.0060 | 0.506 | 0.297 | 0.587 |
| Forearm | 0.0160 | 0.430 | 0.303 | 0.526 |
| Upper arm | 0.0280 | 0.436 | 0.322 | 0.542 |
| Foot | 0.0145 | 0.500 | 0.475 | 0.690 |
| Leg (shank) | 0.0465 | 0.433 | 0.302 | 0.528 |
| Thigh | 0.1000 | 0.433 | 0.323 | 0.540 |
| Head + neck | 0.0810 | 1.000 (at C7–T1) | 0.495 | 0.116 |
| Thorax | 0.2160 | 0.820 | — | — |
| Abdomen | 0.1390 | 0.440 | — | — |
| Pelvis | 0.1420 | 0.105 | — | — |
| Trunk (whole) | 0.4970 | 0.500 | — | — |

The same notebook compares the two models: **de Leva's thigh mass fraction is 42 % larger than Dempster's** (male; 0.1416 against 0.1000 — but Dempster's thigh is hip-trochanter to knee and de Leva's is hip joint centre to knee, so the segments are defined differently), and the whole trunk 13 % smaller than Dempster's (the segments again are defined differently).

## 3. Which is better for adult male athletes

- **Neither was measured on athletes.** De Leva's basis is **living, young, male, fit students** (100 men); Dempster's is **eight older cadavers**. On the sample alone de Leva is the closer match to a pro baseball player and is the standard choice for young males.
- **Athletes differ from both.** Rossi et al. 2013 (J Sports Sci Med 12:761–775, **FT**): DXA scans of **10 elite male swimmers, 8 elite female swimmers and 10 healthy young Caucasian men**; five indirect methods (including Dempster, Hanavan, Yeadon and Zatsiorsky variants) were compared with DXA as the criterion. **All five gave errors above 10 % for at least one segment parameter in every one of the three groups**, including the young men resembling Zatsiorsky's sample; the elite male swimmers had the **largest trunk and upper-arm** inertial parameters. So a sport-specific body (a thrower's trunk and arms) will be missed by any standard table by more than 10 % on some segment, and I found no baseball-player scan.
- **Geometric models** (Hanavan; Yeadon 1990, built from about 95 body measurements, which the cricket models of [note 1](1-forward-dynamics-precedents.md) used) fit an individual far better but need measurements the game does not have; with only `Height` and `Mass`, a regression-scaled table is the available form.
- Attias's baseball model (one MLB starter, 1.905 m, 97.5 kg) used subject-specific inertial properties from a Visual3D model scaled to him, not a published table; it adds a 0.145 kg, radius 0.0364 m, 7.68×10⁻⁵ kg·m² ball (solid sphere).

**Recommendation.** De Leva male, scaled by `Height` for lengths and by `Mass` for masses, with the head as one body and the trunk in its three parts if a lumbar and a thoracic joint are modelled. If a player's trunk and arms are meant to be heavier than the table's (an athlete), that is a **tunable ratio, not a sourced one**.

## 4. What this gives the design

1. A **complete male table** (11 segments, mass, centre of mass, three radii of gyration) is in hand, from a published and widely used source. Its values are fractions, so any `Height` and `Mass` give the inertias.
2. **Lengths from stature is an assumption** (scaling the 1.741 m sample); stature-based segment lengths (e.g. thigh/stature) are not given by de Leva and should be a named, configurable parameter.
3. The table treats the **trunk as three parts with different mass shares** (upper 0.160, middle 0.163, lower 0.112), which fits a twisting torso with a hip and a shoulder girdle.
4. **Expect ≥ 10 % error** on some segment for an athlete (Rossi 2013); a sensitivity check (±10 % on trunk and arm inertia) belongs in the design.
5. **Not provided:** shoulder girdle, bat, glove, spike weights, wobbling masses (the cricket models used them for the shank, thigh and trunk; [note 1](1-forward-dynamics-precedents.md)).

## 5. Not found

- A **segment table measured on baseball players** (or any throwing athletes); a **DXA study on pitchers or hitters**.
- De Leva's guidance on **scaling lengths from stature**, or Zatsiorsky's own stature ratios.
- Winter's book itself (the Dempster values are a transcription; its radii of gyration for the trunk segments are absent from the notebook).
- A source for **pelvis and shoulder-girdle** inertia separate from the trunk.
- Licence: de Leva's table is facts reproduced from a journal note for citation; no more than that table is copied.
