# Fielder communication: what a fielder knows about his teammates (2026-11-14)

**Finding:** [Codex live-play review](../reviews/2026-11-14-codex-live-play-review.md) P1-1, with [Opus infield-fixes review](../reviews/2026-11-14-opus-infield-fixes-review.md) P2-3 and the `Cover(1)` check added on 2026-11-14. Defensive coordination in `live.rs` read a teammate's private plan: his intended intercept point (`target`), his believed intercept time (`t_int`), his call value, and a chase he had chosen in the same tick. agent-models §5.7 already said that a call reaches teammates after `T_call` = 0.3 s; the code did not implement this. This note gives the information model, the mechanism, and the replacement for each read.

## 1. What a fielder may use

| Source | What | How it reaches him |
|---|---|---|
| **His own** | his read of the ball (`belief`), his own intercept (`t_int`, `int_pt`), his own call, his own job and body | at once |
| **What he sees now** | every player's position and velocity, the ball's position and velocity, who holds the ball, which bases are occupied | at once, exact. This is the same interim simplification that runners use when they read fielders (review "Input audit": "exact public motion, explicitly noiseless"). There is no observation noise on another body yet. |
| **What he knows before the pitch** | teammates' attributes (speed, arm, reaction), the pre-pitch alignment (`home`), the staff's coverage, backup and relay tables | at once (interim spec: teammates' attributes are known) |
| **What he is told or sees a teammate start** | a teammate's chase and the strength of his call | `CALL_HEARD_S` = 0.3 s [E] after the teammate began it |

He may not use a teammate's `target`, `belief`, `t_int`, `int_pt` or call value, or a chase that a teammate started less than 0.3 s ago.

**Jobs from the tables.** A cover, backup, relay or idle job comes from the staff's tables. The tables' inputs are public (what everyone sees and knows) once this change is in, so every fielder can work out the same assignment for himself, and nobody needs to be told it. Reading such a job directly therefore leaks nothing. A **chase** is different: each man decides it privately, from his own read. Teammates know of it only when they hear the call or see him go, which is 0.3 s later. This rule is not symmetric: a man who *stops* chasing and takes a table job is treated as visible at once. That error is small, because the job he takes is one the tables gave him, and it is listed in §5.

## 2. The mechanism

- **What each fielder has shown** (`Fielder::shown`): a short log of `(time, chasing?, call tier)` entries. One entry is appended at the end of any tick in which that state changed (`Sim::publish`). Teammates read it through `Sim::known(j)`, which returns the newest entry at least `CALL_HEARD_S` old. Before a fielder's first entry he is known to be at his pre-pitch spot and not chasing.
- **A new ball** (off a glove, a carom, a throw that gets away; `set_free`) begins a new set of calls. A call made on an earlier ball no longer ranks against calls on this one (`Sim::ball_since`). A man still seen chasing counts as chasing with no call.
- **The call tiers** are those of §5.7, from each man's own read: 3, a sure call by an outfielder; 2, a sure call by an infielder; 1, going for a ball in the air without calling it; 0, no catch in the air. Only tiers 2 and 3 are calls, and only calls are heard.
- **The priority table** (`CALL_PRIORITY` [E], the spec's "[data file]" item): when two sure calls of the same tier meet, the higher priority keeps the ball. Outfield: CF over LF and RF. Infield: SS, 2B, 3B, 1B, C, P, in that order. This replaces the comparison of the two men's time to spare, which neither could hear.
- **When knowledge changes** (`Sim::hear`, run every tick): if any fielder's known state changed this tick, each chaser re-checks whether he should yield, each man with a recent read re-checks whether he should go, and the tables are filled again with what is now known.

## 3. Who goes for the ball (`assign_among`, `Sim::decide`)

Each candidate reads the ball and makes his own call. Then he decides alone:

- **A ball he believes he catches in the air** (tier ≥ 1). He goes, unless he has *heard* a sure call that outranks his: a higher tier, or the same tier with higher priority. When he yields and he is an infielder, he peels off to back up the catch from his own read, as before.
- **A ball he cannot catch in the air** (tier 0). He yields to any heard sure call. Otherwise an infielder breaks under the ground-ball rule (his own read: a grounder, an intercept inside the infield; the first baseman only to his side of the mound), as before. Any other fielder goes if, by his own estimate, he gets to his intercept point before every teammate who could. For each teammate he works out the run time to that same point (`Sim::eta`): from the teammate's visible position and velocity, at his known speed, after what remains of his known reaction. The outfielders' 0.15 s edge on balls in between is kept. A teammate already known to be on another job does not count.
- **Mistakes this allows.** Two men can both go until they hear each other (the spec's "both commit before a call arrives"). Two men can also each defer to the other. That is safe because, 0.3 s later, each knows the other is not going, and the next `hear` sends one of them.

**The lead chaser** (`Sim::lead`) replaces `primary` wherever the rest of the defense needs "the man fielding the ball". He is the man everyone knows is chasing, ranked by known call tier, then priority (sure calls only), then nearness to the ball now. He is unknown for the first 0.3 s after the first read. Every chaser replans on his own clock. The once-a-second re-call is each nearby man re-reading and re-deciding.

## 4. Each read, and what replaces it

| Read (before) | Replacement |
|---|---|
| `cover_roles`, `cover_roles_after_chasers`: second base's cover chosen from the sign of the chaser's `target.x` (**the probe**) | The side the ball is on, as everyone sees it (`Sim::ball_side_x`): the holder's position, else the fielding point below once a lead is known, else where the ball is heading (its ground position plus 1 s of its ground velocity). |
| `cover_roles`: the chaser cannot be chosen to cover | A man the others *know* is chasing is skipped. A man who has just chosen to chase is still given the cover by the tables, but he keeps chasing, so the base stays open until his chase is known. Then `hear` fills it from the next man in the table. |
| `fielding_point()` while the ball is loose: the chaser's `target` (cutoff spot, relay depth, backups, re-call candidates) | `Sim::fielding_point() -> Option<Vec3>`: the holder's position; while loose, the point on the ball's ground track nearest the known lead (the ball's ground position, then along its ground velocity). A fly he is camped under puts that point at his feet, and a ball he is cutting off puts it where his path meets its line. With no known lead it is `None`. Backups and the relay wait for it (0.3 s at most). |
| `relay_job`: the relay man compares his `t_int` with the chaser's `t_int` | He compares his own `t_int` with his estimate of the lead's run to the relay man's own intercept point (`Sim::eta`). |
| `recall`: candidates nearest the chaser's `target` | Nearest the fielding point (or the ball's ground position while no lead is known), plus every man chasing. Each re-decides for himself. |
| `assign_among`: every candidate's private call and intercept ranked in one place, at once | §3: each decides alone from his own read, heard calls and his own estimates of teammates. `RECALL_MARGIN_S` goes, because it compared two men's private time to spare. |
| Opus P2-3: the `Cover(2)` middle infielder breaks only if his partner's role (set this tick) is not `Chase` | He breaks only if, by his own estimate, he gets to his intercept point before his partner can (`eta`), so the ball is his. When a middle infielder chasing a grounder learns his partner is chasing too, and his own estimate says the partner is there first, he yields and covers second. |
| `Cover(1)` check: the first baseman stays on the bag if nobody else's role (set this tick) is `Cover(1)` | He stays with the bag if, by his own estimate, the pitcher gets to the first baseman's intercept point first. A pitcher who fields the ball himself cannot cover first. |

## 5. What is left, and what is outside this finding

- **Reads of present motion are exact** (no observation noise on another body). This is interim and the same for runners. It was reviewed and accepted as an explicit simplification.
- **A man who stops chasing is visible at once** (§1). Only the start of a chase and a call wait 0.3 s.
- **The holder chooses a receiver from teammates' current roles.** Those roles come from the tables, so they are public. In the 0.3 s before a chaser is known, the tables can name him for a base he is not going to. The holder normally has the ball later than that.
- **The world's capture timing** reads the true future (review "Input audit", `try_captures`). That is a different finding and is not addressed here.

## 6. Tests

- `second_base_cover_does_not_read_a_private_target`: the review's probe. Moving only the centre fielder's target does not change who covers second.
- `a_call_is_not_heard_before_its_delay`: a teammate who outranks a sure call does not yield to it until `CALL_HEARD_S` after it was made, and does yield after that.
- Existing tests keep their intent; where one set `primary` or depended on instant arbitration, it now sets up what the fielders know.

## 7. Effect on the game

See the [metrics ledger](../metrics-ledger.md) row "Fielder communication (live-play review P1-1)".
