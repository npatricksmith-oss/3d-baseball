# A4 live-ball regression fixtures

Frozen from main `9fdeec169a0ba522bf7b165da425e2d6274e1ea3` before the fixes (`1-64`, `13-140`,
`17-203`) and under Savant 2024 infield alignment (`23-109`, `3-140`).
The numbers are the original pitch's executed contact and public starting rules
situation, captured via `Game::step_pitch`. No alternate outcome was generated.
Use the original seed/play for all keyed input noise. Reconstruct ordinary
average fielders with their generated roster hands, default park/atmosphere and
8.2 m/s runners. Each fixture starts before `BatContact`, independently of any
future changed game's earlier pitches.

- `1-64`: stop at third must not create a home destination.
- `23-109`: fourth inning, one out and loaded; a carry to first is an option,
  which removes the other forces, and the holder must weigh the sure lead force
  home instead (`[4, 1]`; the feed home goes at full effort, a force at first being
  behind it).
- `13-140`: tied sixth, one out and loaded; the original case of the same choice.
  It was dropped with the Savant alignment, when the +35.4° chopper skipped past a
  first baseman holding near his bag, and restored on 2026-11-14 once he played at
  Standard depth with second occupied. Only the choice is asserted.
- `3-140`: tied sixth, one out, runners first/third; preserve a likely
  throw-second/throw-first double play against a single home tag (replaces
  frozen `9fdeec1` `70-157`, whose 102 mph +11.0° LHB grounder shoots cleanly
  between Savant LHB DP SS −5.7° and 2B +19.0° into CF).
- `17-203`: third baseman must not head for third for a runner already back
  on second; he takes the first-base force.
- `65147-10` (frozen 2026-11-20 from `4f18c47`, the owner's saved play): a chopper
  through the right side that the centre and right fielders each left to the other;
  the right fielder must cut it off. It was a whole-game replay in `live.rs` until the
  catch-throw review fixes changed the game's earlier plays.
- `155-23` and `264-109` (frozen 2026-11-20 from `8621f54`, `work/approach`, where both
  were lost): flies with a runner on second and none out that main catches and the set-up
  behind a fly dropped (a deep fly to left-centre, the centre fielder; a sinking liner, the
  left fielder). The outfielder must catch them in the air (agent-models §5.5, the set-up's
  timing; approach note §6).
- `1048-193` and `136-13` (frozen 2026-11-20 from `af5825d`, `work/approach`, where the ball
  got by the fielder; Opus approach review P2-3): a comebacker the pitcher backed off to take
  deeper, and a grounder the second baseman was paced to meet exactly at a later hop. Each must
  be fielded cleanly by that fielder (agent-models §5.2, the braked take timed as the movement
  runs it, and the rule for a ball coming at him).
- `207-66` and `1042-28` (frozen 2026-11-20 from `af5825d`; review P2-2): a pop-up to shallow
  left and a fly to right-centre that main catches and the set-up, chosen on a first read, lost.
  The outfielder must catch them in the air (agent-models §5.5, the set-up only on a settled
  read).

Expectations are the tactical cases in agent-models §5.6, not saved chosen
destinations or simulation output hashes. `audit_intended_goal` in the separate
diagnostic trace is never part of a fielder's inputs.
- `5-60`, `5-149`, `0-17` (frozen 2026-11-21 from `445ef2c`, TASKS §3 cases (a) and (d);
  [liner-single note](../../../../../../docs/calibration/2026-11-21-liner-single-jobs.md)): a
  liner into the dirt behind second with a runner on third, a fly falling in front of the right
  fielder with runners on first and second, and a fly single to right with nobody on. While the
  ball is up the infielders take the single's jobs (agent-models §5.7, "A ball in the air that
  will come down in front of an outfielder"); with nobody on the second baseman is the cutoff
  man for second.
- `2-117` and `11-203` (frozen 2026-11-21 from `445ef2c`): a pop-up the third baseman catches
  51 m out with a runner on second, and a ball over the centre fielder with a runner on third.
  What must not change: no cutoff on the first, a middle infielder out as the relay man on the
  second while it is up.
  The guard test also reuses `155-23` and `264-109` (flies caught with a runner on second).
- `61-187`, `21-86`, `77-114` (frozen 2026-11-21 from `445ef2c`; Opus liner-single review P1-1):
  balls in the air with a runner in scoring position on which one infielder judges a single and
  the man he counts out on the cutoff home stays on his bag (a liner to left first down at
  82 m, a fly to right first down at 77 m, a fly to right-centre first down at 90 m). The bag is
  not held by two men for longer than it takes to see the cutoff man not go, and someone is out
  on a cutoff or relay once the outfielder's chase is known.
