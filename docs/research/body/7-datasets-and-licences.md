# Body research 7: datasets and licences (2026-11-25, worker body-research)

For [PLAN D31 / §14.1](../../../PLAN.md) and PLAN D21 (the OpenBiomechanics licence rule): which open data on hitting and pitching the game could ship numbers derived from, and which are cross-check only. **I am not a lawyer; each licence line below is what the data holder's own page said, read by me on 2026-11-25, and a commercial release needs the owner's own check.** Reading status: **FT** = the licence text or metadata record read; **2nd** = a search summary.

## 1. Datasets, with licence

| Dataset | What is in it | Licence | Can the game ship numbers derived from it? | Read |
| :-- | :-- | :-- | :-- | :-- |
| **OpenBiomechanics Project** (Driveline Baseball R&D, GitHub `drivelineresearch/openbiomechanics`): **hitting** (677 swings, 98 hitters, machine pitch ~65 mph, mocap 360 Hz + 4 force plates + instrumented bat) and **pitching** (marker mocap and force plates of pitchers) | joint angles, velocities, landmarks, force plates, C3D | **CC BY-NC-SA 4.0** for data and documentation; code MIT; the README adds that employees, contractors or significant shareholders of professional sports organisations or financial-analysis firms may not use the data "for any use whatsoever"; commercial use needs a paid written licence | **No** (non-commercial, share-alike): **cross-check only**, as PLAN D21 / Q41 already say. Derived tables are derivative works under the licence | **FT** (the repository's licence text, quoted in [../swing/6-openbiomechanics.md](../swing/6-openbiomechanics.md)) |
| **Van Trigt et al.**, "Dataset Underlying: Quantifying Within-Individual Elbow Load Variability in Youth Elite Baseball Pitchers" (TU Delft / 4TU.ResearchData, DOI 10.4121/19868722.v1, released 2023-09-05); paper: Appl Sci 2022;12:6549 | **11 male Dutch national-team (AAA) pitchers, mean age 17.4 y, 25 fastballs each (270 pitches analysed)**; Vicon at 400 Hz, modified Plug-in-Gait landmarks (static and pitch trials as Python pickle files, filtered and unfiltered position data); the **calculated external elbow valgus torque and the ball speed of each pitch** | **CC BY 4.0** (the dataset's own CFF export reads "license: CC BY 4.0") | **Yes with attribution**: commercial use and derivatives are allowed under CC BY. Limits of the *content*: **youth** pitchers (age 17), a **marker model that stops at position data** (no force plates, no ground forces), within-individual valgus torque normally distributed in 9 of 11 pitchers (abstract) | **FT** (CFF record and the paper's abstract and introduction; the data files were not downloaded) |
| **Van Trigt group, muscle activation of 11 pitchers** (4TU.ResearchData, c9b20867-…; paper Front Sports Act Living 2021, DOI 10.3389/fspor.2021.698592) | surface EMG and video, 15 fastballs each, maximal voluntary contractions | **CC BY-NC 4.0** (its CFF record) | **No** (non-commercial) | **FT** (CFF record) |
| **KinaTrax data of Attias et al.** (43 fastballs of one MLB starter) | transformation matrices of the segments at 300 Hz | "anonymized and shared by KinaTrax" to the authors; **not published** | **No** (not available) | **FT** (the paper's methods) |
| **Statcast / Baseball Savant** (pitch tracking, bat tracking: bat speed, swing length, intercept points, release points) | league tracking data | MLB's general Terms of Use (**2nd**: "personal, non-commercial home use"; whether it reaches raw Statcast downloads is unresolved; the Savant terms page was not retrieved) | **Unresolved**: the swing notes already use Statcast-derived summaries (note 9, note 10 of the swing set, Wakamiya 2024's release-point data). **Check the Savant terms of use before commercial release.** Published papers that use it (Wakamiya 2024, CC BY) can be cited | **2nd** |
| **CMU Graphics Lab Motion Capture Database** (generic human motion) | marker motion of everyday and some sports actions | CMU's own wording (**2nd**, a search summary): free for research; may be included in commercially sold products, **not resold directly even in converted form**; some third-party copies carry different terms (a 4TU FBX conversion is CC BY-NC 4.0). I found **no baseball or throwing category** | Probably for generic poses (not a source of swing or pitch numbers); not verified | **2nd** |
| **The papers' own tables** (Fleisig 2025, Escamilla 2025: CC BY-NC-ND; Anderson 2007, Kageyama 2014, Rossi 2013, Wakamiya 2024, Nakashima 2025: open access, licences vary) | summary values | per paper | Facts such as means and SDs are not what the licences protect, but **copying a table wholesale into the repository is a copy of the paper's presentation**; notes 1–6 quote single values with a citation and copy no table of more than a few rows | **FT** per paper |

## 2. The swing's own thesis data

Fortenbaugh's 2011 thesis (University of Miami) and Welch et al. 1995 (JOSPT) are the two **sources the swing synthesis rests on** ([../swing/swing-synthesis.md](../swing/swing-synthesis.md)); both are ordinary copyrighted publications. Facts (means, SDs, times) taken from them are not covered by the OpenBiomechanics licence problem; the notes cite and do not reproduce tables. Appendix E curves were digitised from the thesis's figures (note 9 of the swing set): derived from a published figure, fine for a number, a question for a reproduced curve.

## 3. What is missing

- **Any openly licensed (CC BY / CC0 / ODbL / public domain) dataset of baseball hitting** with ground forces or joint kinetics: none found. The only open hitting dataset I found is OpenBiomechanics (non-commercial).
- **Any openly licensed pitching dataset with ground forces or with professional pitchers**: the CC BY set is youth, position only.
- Driveline's other open-source initiative pages (the GitHub "sports biomechanics" topic page named "an open-source initiative for anonymised elite-level athletic motion capture data run by Driveline") are the OpenBiomechanics Project itself; I did not find a second repository.
- A **published or licensable** KinaTrax, Hawk-Eye or Statcast skeletal-tracking set (all league data is MLB-owned).

## 4. What this gives the design

1. **For shippable numbers the choice is thin:** the one open dataset that allows commercial use (van Trigt, CC BY 4.0) is youth pitchers' position data. Shipped values for adult males will have to come from **published papers' means and SDs** (notes 2–6), cited, rather than from a dataset.
2. **OpenBiomechanics stays cross-check only** (PLAN D21): its hitting and pitching files are the only adult, force-plate, open data, and they carry the non-commercial, share-alike licence, with the no-use clause for professional-sports-organisation staff.
3. The CC BY pitching set can serve to **size within-pitcher variability and to check kinematic shape**, with the youth caveat (note 6).
4. **Statcast-derived constants already in the project** (swing length, intercept, release-point spread) rest on MLB's terms; resolve that before release.

## 5. Not found

- The Savant / MLB terms page text for Statcast data; CMU's resources page (the fetch returned nothing).
- The data files of the CC BY dataset (not downloaded; size about 835 MB per a search summary).
- OpenBiomechanics' pitching dataset details (its note in the swing set covers hitting only).
- Any dataset of fielding throws.
