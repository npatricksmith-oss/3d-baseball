# A4 live-ball regression fixtures

Frozen from main `9fdeec169a0ba522bf7b165da425e2d6274e1ea3` before the fixes (`1-64`, `17-203`)
and under Savant 2024 infield alignment (`23-109`, `3-140`).
The numbers are the original pitch's executed contact and public starting rules
situation, captured via `Game::step_pitch`. No alternate outcome was generated.
Use the original seed/play for all keyed input noise. Reconstruct ordinary
average fielders with their generated roster hands, default park/atmosphere and
8.2 m/s runners. Each fixture starts before `BatContact`, independently of any
future changed game's earlier pitches.

- `1-64`: stop at third must not create a home destination.
- `23-109`: fourth inning, one out and loaded; a carry to first is an option,
  which removes the other forces, and the holder must weigh the sure lead force
  home instead (`[4, 1]`). Replaces frozen `9fdeec1` `13-140`, whose +35.4° chopper
  skips past 1B holding at Savant LHB DP depth (88.9 ft / +41.0°) into shallow RF.
- `3-140`: tied sixth, one out, runners first/third; preserve a likely
  throw-second/throw-first double play against a single home tag (replaces
  frozen `9fdeec1` `70-157`, whose 102 mph +11.0° LHB grounder shoots cleanly
  between Savant LHB DP SS −5.7° and 2B +19.0° into CF).
- `17-203`: third baseman must not head for third for a runner already back
  on second; he takes the first-base force.

Expectations are the tactical cases in agent-models §5.6, not saved chosen
destinations or simulation output hashes. `audit_intended_goal` in the separate
diagnostic trace is never part of a fielder's inputs.
