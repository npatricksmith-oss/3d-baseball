# A called-off infielder backs up the catch (2026-11-13)

Owner item 1 of 2026-11-13 (TASKS "Resume here"), from
docs/calibration/2026-11-13-popup-read-and-calls.md, "Open": agent-models §5.7 says "a
fielder who hears a higher-priority call peels off to back up", and that was not built. An
infielder called off a ball in the air either got a cover job or pulled up where braking
stopped him and stood there for the rest of the play.

## The play that showed it

Seed 29 play 135: a pop-up to short center that comes down at (−7, 62) m, 5.9 s after
contact (`cargo run --release -p bb_game --example trace_play -- 29 135`).

At the first call (0.64 s) the centre fielder, 98 m out, reads it as coming down at (3, 105)
m (his depth draw is −1.47 σ of a ~25 m first-look read) and is sure of it by the call rule,
so his tier-3 call outranks the second baseman and shortstop, who were both sure of it
too (tier 2). The shortstop is given second base, then loses it to the second baseman at
the next re-call (the ball is now read to the left side), goes Idle at (−6, 42) and stands
there, 20 m from where the ball lands. By 2.7 s the centre fielder is no longer sure, but
nobody near the ball believes he can still catch it with time to spare, so he keeps the
ball, comes in, misreads it the other way, and it lands behind him.

## The change (`PEEL_BACKUP_M` = 5 m [E], `Fielder::peel`)

On a sure call (tier 2 or 3), every *infielder* in that call who believed he could catch
the ball in the air (tier ≥ 1) is called off. Unless the cover step gives him a base, he
backs up the catch: he goes to a spot 5 m short of where **his own read** has the ball
coming down, on the side he is coming from (`assign_backups`, before any other backup job).
He keeps that job through re-calls he is not part of. It ends when the ball comes down,
is caught, or is thrown (visible facts), checked every tick (`expire_peels`); a loose ball
is then re-called at once so he can field it. A glove deflection is a new ball and a new
call. While the call stands he makes no glove attempt at all (`try_captures`), however close
the ball comes to him. Outfielders called off keep the existing job behind
the fielder, deeper on the ball's line from home.

Why his own read and not the caller's target: he backs up the ball, not the man, and on a
shallow pop-up the infielder's read is the better one (1 m against 25 m at first look). Why
5 m: out of the caller's way and well beyond the 1 m standing reach, but close enough to
field a ball that drops or pops out, or to call it on a re-call. The distance alone does
not keep him off the caller's ball (his read can be off, and a man already under the ball
needs time to back away: Codex peel-off review #1), so the yield is explicit.

That last part matters: he is now near the ball, so when the caller's read falls apart the
once-a-second re-call can hand it to him by the existing rule (a higher tier wins). In seed
29 play 135 the shortstop drifts out to (−7, 61); at 2.7 s the centre fielder drops to tier
1, the shortstop is sure (tier 2), calls it and catches it at 5.8 s.

Test: `live::tests::a_called_off_infielder_backs_up_the_catch`. A pop-up at (20, 36) m that
both the first and second basemen are sure of; the first baseman calls it; the second
baseman, with no base to cover, backs up from 5 m short of the landing point on his side; a
re-call he is not in leaves him the job; with the ball in his reach while the call stands he
makes no glove attempt; once it is down the tick's expiry step ends the job. On the old code
he is Idle; without the yield he catches the first baseman's ball.

## Numbers (`sim-cli games --n 1024`, before → after)

| seed | runs/team-game | PU hit (MLB ≈ .02) | FB hit (≈ .13) | LD hit | GB hit |
|---|---|---|---|---|---|
| 5 | 5.47 → 5.44 | .018 → .008 | .122 → .120 | .723 → .724 | .411 → .411 |
| 2 | 5.27 → 5.26 | .018 → .008 | .120 → .119 | .712 → .715 | .407 → .406 |

(After the Codex review fixes; before them seed 5 gave 5.43 runs and LD .723, seed 2 the
same.) Uncaught pop-ups (launch > 50°) in 128 games (`--example pu_hits -- 128`): 12 → 7. All
seven left are 50–53° balls hit 81–100 mph that land deep, first touched by an outfielder
6–9 s after contact; no infielder can be near them.

Pop-up hits are now under the MLB reference (.008 against ≈ .02). Not tuned: the mechanism
is a real one (infielders who drift out under a pop-up catch the ones an outfielder misjudges),
and the remaining excess catching comes from the read model, whose depth error at the first
look is owner item 2 (§5.1 launch-state perception), which will move these numbers again.
Time per game was not measured under comparable load (the review notes the runs overlapped).
