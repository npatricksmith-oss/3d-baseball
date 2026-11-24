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
- Relay hand-overs (frozen 2026-11-21 and 2026-11-22 from main `7622093`;
  [relay hand-overs note](../../../../../../docs/calibration/2026-11-22-relay-handovers.md);
  agent-models §5.7). Each test fails on `7622093` or without its change:
  - `6-6`, `3-63` (Opus liner-single review P2-2): a fly single to right and a ground single
    up the middle with a runner on first who rounds second. The first baseman stays on first
    and the shortstop is the cutoff to third. `13-227`: runners on first and third, a fly
    single to centre; the first baseman out on the cutoff home keeps it when the run scores.
  - `23-94`, `115-32`, `63-211` (Opus liner-single re-review P2-1): balls in the air with a
    runner in scoring position that a middle infielder goes out on as the relay man while up,
    first down short of 80 m. He keeps the job through the outfielder's pickup to the throw.
  - `13-224`, `93-113` (re-review P3-1): deep balls with the first baseman on the cutoff home
    and the shortstop out. Second is covered once both are known out.
  - `107-40`, `109-209` (item 5): a runner on third, a ball to left with the third baseman and
    shortstop both out. The pitcher keeps third and does not flip with each re-call.
  - `12-9` (Opus cutoff-home review P3-3): a chopper the shortstop fields 40 m out with the
    right fielder the known lead. The first baseman stays on first.
- Frozen 2026-11-22 from main `ba40063` for the double cut (owner Q35 (c);
  [double-cut note](../../../../../../docs/calibration/2026-11-22-double-cut.md); agent-models
  §5.7). Balls that get past the outfielders with a runner who may score; each fails on
  `ba40063` (`a_ball_past_the_outfielders_adds_the_relay_man_and_keeps_the_first_baseman`):
  - `23-83`: runners on first and third, a ball into left-centre that falls in and rolls past
    the left fielder. The first baseman, out on the single's cutoff home, stays out behind the
    shortstop; second is covered at once.
  - `97-119`: runners on first and second, the same to left. The third baseman comes back from
    the single's cutoff to third at once, the first baseman goes out.
  - `118-162` (Opus relay hand-overs review P3-3): a runner on first, a grounder past the right
    fielder that the centre fielder takes over. The deep call does not flip back. *Unused since
    2026-11-23:* the right fielder now lays out for that grounder and knocks it down
    (agent-models §5.3, the outfielder's dive at a ground ball), so it no longer gets past him.
  - `17-103` (frozen the same way after the double-cut reviews, P2-1; fails without that fix,
    and on `ba40063` at its premise, there being no double cut): a runner on first, none out, a ball past the
    centre fielder; he throws to second, past the first baseman out on the double cut, who goes
    back to first at once (`a_throw_that_goes_by_the_double_cut_sends_the_first_baseman_back_to_first`).
