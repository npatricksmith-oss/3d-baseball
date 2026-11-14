# Statcast bat-tracking analysis (2026-11-14, swing model v2)

Standard-library Python over Baseball Savant pitch-level CSVs, used by
`docs/calibration/2026-11-14-swing-model.md`. The CSVs are not committed; fetch each date with

```
curl -s -o <dir>/sc_<date>.csv "https://baseballsavant.mlb.com/statcast_search/csv?all=true&hfGT=R%7C&hfSea=2025%7C&player_type=batter&game_date_gt=<date>&game_date_lt=<date>&type=details"
```

for 2025-04-22, 2025-05-20, 2025-06-17, 2025-07-22, 2025-08-19 and 2025-09-16, then run, for example,
`python3 -I analysis/statcast/savant_bat.py <dir>`.

- `savant_bat.py`: bat speed, swing length, attack angle and direction, path tilt; by horizontal location, height, intercept depth, lateral intercept and outcome.
- `savant_sd.py`: within-batter and between-batter bat-speed SD; home runs' bat speed; bat speed by strikes.
- `savant_spray.py`: spray of balls in play by location and type (left-handed batters mirrored).
- `savant_sprayad.py`: spray against attack direction by handedness pair.
- `savant_ev.py`: exit speed and home runs by location; exit-speed shares.

Attack direction's CSV sign is taken as negative = pulled (inside pitches and out-front contact are negative).
