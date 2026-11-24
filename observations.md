# Observations

The owner's list of what he has seen and wants, with where each stands. The letter is his priority (A, B, C). Every Coordinator session updates the boxes here when it finishes something, so this file is enough to follow progress; [TASKS.md](TASKS.md) has the detail.

`[x]` done · `[~]` partly done (the note says what is left) · `[ ]` not started

**Last updated:** 2026-11-21

# Added 2026-11-16

Screen:
- [x] Reduce ball, bat and glove (owner: the ball only; leave the glove and bat as they are). *Done 2026-11-16: the ball was cut to its real size, 7.5 cm (it was 15). You still could not see it, nor a quarter larger, so it is now 11.7 cm, which you found fine. (A smallest size on screen was tried and taken out again: too big.) Still hard to see, so the grass and dirt are a little darker now, to set the white ball off.*
- [x] Ball shadow should be half as big and 30% more blurred when hit. *Done 2026-11-16: half the size, and its edge now fades out over the outer 30% of it (before, the edge was hard). Not yet built or seen.*
- [x] The score card should have the pitcher's and batter's names, the pitcher's total pitch count, and the batter's results in this game. *Done 2026-11-16: two rows under the score, the pitcher with his pitches ("42 P") and the batter with his game ("1-3, HR, 2 RBI"). The pitch's speed and type show under them once it reaches the plate.*
- [x] Speed keys: 1 to 5 for 1x to 10x, minus slows down by 0.25x. *Done 2026-11-16: 1 to 5 give 1x, 2x, 4x, 6x and 10x (the middle steps are my pick; say if you want others); minus slows by 0.25x down to 0.25x, equals speeds up by 0.25x.*
- [x] Runners in the mini base map should be according to the real runners. *Done 2026-11-21: each runner is a white dot that moves on the little diamond with his real position (the batter from the moment he leaves the box); a base lights while a runner is at it or on his lead, a runner out disappears at the call, one who scores at the plate, and the map ends on the official result with no jump.*

Fielding:
- [x] Seed 40580, play 15: "CF seriously misplayed the ball. Misjudgment? Bug?" *Found 2026-11-16: a fault in how fielders judge a fly, not a believable misjudgment. The center fielder called it and got there in time, but all the way down he thought it would land about 2 m to his right of where it did, and he still thought so half a second before it came down, with the ball 8 m from him. A real outfielder doesn't misjudge a ball's direction that late. So he stopped 1.6 m from it, realised too late to dive, and it rolled to the wall: an inside-the-park home run. The cause is in the sim's read of a fly: a small error that stays the same on every look gets trusted as if it were new information each time, so he grows sure of a wrong line and stops correcting it. With that error left out he reads it to 7 cm and makes the catch. The fix is fielding physics, which is paused, so it waits for reviews to come back (TASKS backlog).* **Fixed 2026-11-17 (`work/track-bias`):** a fielder now knows that part of his misjudgment repeats on every look, and lets the closer looks correct it. The centre fielder holds his call and catches it. Routine flies are now caught about 98% of the time by an average outfielder (93% at grade 20, 99% at grade 80), against 97% before.

Rules:
- [~] Fielders must tag a runner out if the play is not a force play. *The rules engine already requires a tag on a non-force play. Nothing on screen shows the tag being made yet (no reach toward the runner). If you saw an out with no tag at all, its seed and play number would help.*

Running:
- [ ] Runner should round 1st base if the ball is being thrown in from the outfield to the cutoff man or another base, to get a jump if the ball is misplayed.
- [ ] Need to work on rundowns. *There is a stand-in today; the real rundown is planned after the runner decision work.*

Fielding and throwing:
- [ ] After an out from deep outfield, throws should hit a cutoff person.
- [~] A. When fielding on the run, fielders should plant their foot before the throw, otherwise a throwing penalty. *The sim already makes a throw slower and wilder when his feet are not under him; the motion does not show a plant yet, and how often they skip it is your saved plays 56, 107, 142, 32 below.*
- [ ] B. Throwing across the body means a slight throwing penalty.
- [ ] C. A good throw is plant or step, then throw. *(The throwing motion.)*
- [~] D. Throws from deep in the outfield would generally not be exactly on target every time. Throws move fielders off the base, and may hit the ground or force the fielder to move. *Throws already scatter by the thrower's accuracy, receivers leave the bag or take a hop when they must, and a short throw bounces. Whether the scatter is as large as real outfield throws is unchecked.*

# General
- [x] A. Better stadium and baseball diamond visuals, more realistic looking. For now, crowds can be something low resource and reworked later. *The park is built (field, walls, seats, scoreboard, signs, trees) with a seated crowd of 6,200.*
- [ ] C. Implement better crowds. *Today's crowd sits still: no standing, no reactions.*
- [ ] B. Implement leagues/divisions/teams with rosters. *Two invented clubs exist only for the look (Cedar Hollow Larks, Stonebridge Colliers).*
- [ ] C. Implement player development.
- [ ] A. Implement wind and weather conditions: effect on ball and players (rain = more fielding/running mistakes).
- [x] A. Implement rosters (player names, player numbers, player positions). *Numbers are on the backs of the uniforms.*
- [ ] B. Team lineups (vs left vs right batting orders).
- [ ] B. Implement a game settings screen.
- [~] A. Implement baserunners sliding at appropriate times. *Since 2026-11-16 a runner slides feet-first into second, third or home when a play is being made on him there. Left: with no play on him he still stops dead standing (the sim stops him dead; slowing into the base is sim work), and a runner put out before the bag is not seen sliding into the tag. You have not seen it moving yet.* *2026-11-23 (Q39): a runner going back to his base on a close play now dives back headfirst, about once a game. This is the sim's own move: he gets there sooner than by braking, and he is down a second before he can run again. It is drawn with his hand on the bag when the sim has it touch.*
- [ ] A. Implement stolen bases.
- [~] A. Variety of body shapes, nationalities, ethnicities, skin color. *Height, build and skin tone vary by player; names do not yet follow nationality.*
- [x] A. Team uniforms.
- [x] A. Limits of play area (I saw a foul ball go back out of play and continued on). After the ball goes out of the stadium, cut back to field of play. *Since 2026-11-16, after a foul into the seats the game goes straight to the next pitch.*
- [ ] A. Runners are leading off before the pitcher gets set on the mound. In the real world the pitcher would pick them off.
- [x] A. Post game box score. *Done 2026-11-16: the box score comes up at the end of the game and on Tab during one, in the same look as the score bug, with a line score, both clubs' batting lines and the pitchers' pitch counts. A line score also shows between innings.*
- [ ] C. Post game highlights.

# Handedness
- [x] A. Implement player handedness for fielding, throwing, and batting.
- [x] A. Right-handed players throw with their right hand, and opposite with left-handed.
- [x] A. Fielders need a handedness attribute. Each player either throws with their left or right hand. Generally fielding handedness matches batting handedness but not always.
- [x] A. Some batters are switch hitters. They switch sides of the plate based on the handedness of the pitcher.

# Pitching
- [~] A. Pitchers stretch vs windup. *The pitcher is drawn delivering from the stretch on every pitch; no windup yet, and the sim has one delivery time for both.* References: [Am J Sports Med 2024](https://journals.sagepub.com/doi/abs/10.1177/03635465241247543), [Umpire Bible: pitching positions](https://umpirebible.com/pitching-positions/), and a web search for "pitching stretch vs windup visualization".

# Fielding
- [x] A. Implement baseball gloves on players along with their team uniforms.
- [x] A. Throwing motion doesn't look realistic. *Fielders now throw with a coded motion, hand on the ball's real release point. You have not seen it moving yet.*
- [x] A. A hurried throw loses accuracy and speed. If a player has time to get balanced, then the throw can gain accuracy and speed. *See below: fielders still choose to hurry too often.*
- [~] A. The outfield throw home (2026-11-19, your answer to Q32; your write-up is [docs/baseball-throw.md](docs/baseball-throw.md)). In order of what decides how quickly and accurately it gets there: 1. How the fielder catches the ball. If he catches it on the run with momentum away from the target base, he either throws off balance (less accuracy and power) or takes a beat to stop, turn, plant his back foot and throw hard and accurately, which takes time. Good fielders set up under the fly so that they catch it with a little forward momentum toward the target, for a more accurate and harder throw. 2. The cutoff: on singles and sacrifice flies the outfielder throws low *through* the cutoff man; the catcher calls "cut" or lets it go, and a cut ball goes to second or third; deep balls (about 280 ft or more) go to a relay man. *Done in part (merged 2026-11-20): a fielder moving away from his target stops and plants before a set throw; coming through the ball toward it adds a little speed (the crow hop); on singles and sacrifice flies the throw goes low through the cutoff man about 45 ft from the base and reaches the plate on one hop; from about 260 ft and deeper it goes to the relay man. Since the second merge (2026-11-20, your Q33): fielders slow into a ball they will throw and come through it toward the throw, an outfielder with time sets up behind a fly and comes through the catch, and momentum across the line is charged. Triples came down from 1.02 to 0.85 a game per team (MLB about 0.14). Left: the catcher's cut-or-let-it-go call; rounding the ball is built but almost never pays in games; first to third on singles is 36% against MLB's 32%; outfielders release 0.07 s slower than before (1.12 s).*
- [x] A. Infielder throws should behave the same (physics-wise) as outfielders regarding the mechanics of throwing as well (2026-11-20). *Done (merged 2026-11-20): infielders and outfielders use the same throw physics: the stop before a set throw, the extra speed from coming through the ball toward the target, slowing into a ball they will throw, and the charge for motion across the line (a backhand in the hole, a throw across the body).*
- [~] A. Each player has throwing ratings (accuracy, speed, throwing set-up time). *The ratings exist in the model; every player is average until players get their own ratings.*
- [~] A. Smarter player decisions based on runners on base and number of outs. E.g. with a runner on 3rd and 2 outs, fielders play to get the force out at first or second instead of throwing home. *Throw choices weigh outs and runners; more cases are in your saved plays below.*
- [~] A. Players dive for balls, increasing range. Takes effort to rise back up to throw if the catch or grounder has been gotten. *Dives and the get-up work in the sim at a realistic rate. Since 2026-11-16 the dive is its own motion: laid out with the glove on the ball, down, up through hands and knees. Left: every dive is head-first straight at the ball. You have not seen it moving yet.*

# Hitting
- [x] A. Ball shadow after hit can't be seen.
- [x] A. Once a home run goes over the wall, move camera back to center to see runners moving around bases.
- [x] A. Hitting home runs brings up display with player name, how far ball was hit and mph.
- [x] A. Hitting animation needs work. Batting stance. Initial load. Swing. Follow through. *Coded stance, stride and swing, hands on the sim's bat. Different stances per player come later. You have not seen it moving yet.*

- [ ] A. We need to fix batting stances. Each player gets a unique batting stance, swing style, and follow through. Players can have variations of core stances, but each should be slightly different. *(2026-11-22) Queued in TASKS "Priority A, not done". (2026-11-23) You chose looks only (Q37, PLAN D23): each player's style changes nothing in the game. The base swing comes first.*
- [ ] A. The current stance and swing are a decent start but incomplete and too unnatural. *(2026-11-23) The research is done and the base-swing design is written (player-visuals §13); five build steps are queued. One question for you on the follow-through (the bat after contact). (2026-11-24) You chose the full rebuild (Q44 b): the new swing is specified (docs/swing-model-v3.md), from a fixed launch with the bat up and wrapped; you answered its four questions (Q45–Q48, PLAN D26). The measurements are read in and the checking model is built: the bat now drops through the swing as the lab measured it. Next the new swing goes into the game's code, then into games.*

# Stats
- [ ] B. Clarify what "fielder's choice" means; I think the game uses it too liberally.
- [~] B. Implement statistics. *The sim works out batting lines and the box score from each game's record; no statistics screens yet.*

# Saved plays, 2026-11-15 night (seed 3647)

Watch any of them again with `client/run.sh 3647`, then R back to the play number.

Umpire and camera:
- [x] 15. Foul balls should be called at the right time, as soon as the ball is deemed foul. *Fixed 2026-11-16: a ball rolling past first or third in foul ground is now called as it passes the bag (play 15 at 1.5 s; it was 5.3 s).*
- [x] 20. It's a foul ball but the umpire calls it "In play". *Fixed 2026-11-16.*
- [x] 62. After a foul ball out of the stadium, just cut to the players already set up for the next pitch. *Done 2026-11-16.*

Runners:
- [ ] 65. Runner should be in an athletic stance when waiting for the pitch to be thrown.
- [~] 65, 67, 68. On a fly that might be caught, the runner goes part way (halfway), stops in an athletic stance and watches the ball and outfielder; if he judges it caught he goes back to tag up; even on a likely catch he comes off the base a little in case it is misplayed. *The runner starts with a moving lead. When the catch is close he checks his speed and braking distance before deciding how far off to hold. He gets back to third early when he sees a chance to tag and score. Since 2026-11-19 he keeps watching while the ball is up, looking again every half second, and goes back to tag once he is sure of the catch (from second, at even odds with none out); no runner is doubled off a deep fly any more. Since 2026-11-23 (`work/late-catch-read`, for review) a runner who has gone back to tag keeps watching too, and leaves the bag when he sees the ball will drop; his read of a high fly or pop-up no longer gives up in mid-flight while the fielder stands under it. The waiting pose and saved-play checks are open.*
- [x] (2026-11-18) The runner should have some uncertainty for close catches. Any uncertainty goes away after the umpire makes the out or not call from the fly ball. *Done (merged 2026-11-18): the runner knows his look at the ball can be off and that he can't see its spin, so close catches stay uncertain to him. He waits partway, close enough to get back, then goes back or runs a quarter second after the call. The follow-up fixes the moving hold and bases catch and throw judgments on what the runner can see.*
- [ ] 156. Runner should run through first base as if he were safe, since he can't stop that quickly.

Throws:
- [ ] 56, 107, 142, 32. Fielders should not throw on the run unless there is no time to get set; a throw on the run should be slower. 32: the third baseman with plenty of time takes a split second to step and throw strong and accurate, judging how long the runner will take. *(98 and 99, a long throw and a slow roller, you found right.)*
- [ ] 68. The outfielder judges whether to throw behind a runner who didn't tag up, or throw safely to the cutoff.
- [ ] 26. A pitcher who catches a hot liner with runners on looks to double one of them off.
- [ ] Seed 47638, play 0 (saved 11:48). "There was no attempt to throw the runner out from the 1st baseman to the pitcher. Should have made an attempt." *Not looked at yet.*

Covering and calling:
- [ ] 19. The first baseman holding a runner still goes for the grounder, unless it is near enough to second for the second baseman to take it; then he goes back to first for the throw.
- [ ] 52, 63. The pitcher covers first on a ground ball to the right side.
- [ ] 159. The first baseman who is taking the bag himself puts his hand up to tell the pitcher "I got it".
- [ ] 86. The first baseman stretches: lead foot and glove out toward the throw, to shorten it.
- [ ] 147. The right fielder should call off the center fielder, who peels off to back up. *(The caption "Fly out to center" names the fielder who caught it, as a scorer does.)*
- [x] Seed 65147, play 10 (saved 2026-11-17 20:29). "Right fielder completely misjudged a roller to the outfield." *The centre and right fielders each left it to the other. Right field now takes the chase at the first hearing and cuts it off at 5.08 s. Since 2026-11-24 he also reads the roller's distance within 5 m (it was 32 m off): a ball on the grass is judged by its angle below his eyes.*

Catcher:
- [ ] 40. On a pop foul the catcher takes his mask off and tosses it aside (for the look only).

# Saved plays, 2026-11-20 night (seed 51001)

Watch any of them again with `client/run.sh -BbSavedPlay="saved-plays/<file>.toml"` (files `seed51001-play<N>-…`). Played on main `b8898d3`, before the outfield throw work merged.

Fielding:
- [x] 66. 2nd baseman should have tried for double play. There was more than enough time. Decided not to throw to second base. Why? *Found (diagnosis note, [docs/calibration/2026-11-20-saved-plays-51001.md](docs/calibration/2026-11-20-saved-plays-51001.md)): the shortstop fielded it and misjudged when the covering second baseman would reach the bag, treating him as standing still though he was already running. So the force at second looked doubtful and he took the sure out at first. Fixed (`work/dp-cover`, [note](docs/calibration/2026-11-21-dp-cover.md)): the shortstop now judges the second baseman from how he is running and throws to second; the runner is forced out there with about 0.7 s to spare. Still no double play on this ball: the second baseman takes the bag moving away from first and the batter is too fast.*
- [x] 168. That should have been a double play attempt. The fielder understands the baseball situation (1 out, 1 runner on first, potential force out at 2nd, potential double play if a ground ball). So he should toss quickly to the shortstop covering the base so the shortstop can make an attempt at first. *Found: the shortstop covering second stops dead on the bag instead of crossing it, so he gets there about 0.4 s late and there is no time left for the throw to first. Fix queued: a cover man who is late takes the bag on the move; with it this play is a 4-6-3 double play. Double plays per chance are already a little above MLB's, so they get re-measured.* *Fixed (`work/dp-cover`, [note](docs/calibration/2026-11-21-dp-cover.md)): a man covering a base for a force crosses it when he is late for the throw and only stops on it when he is early. This ball is now a 4-6-3 double play.*
- [x] 102. 2nd baseman hardly moved when the ball was hit. Realistic or no? *Found: not chasing was right (the ball was out of his reach even with a dive, and he read it correctly). But the infield's job assignments left him with no job for 0.75 s, then swapped jobs with the shortstop. Fixed (2026-11-21): on a ground ball through the infield the middle infielder on the ball's side goes out as the cutoff man for the throw to second from the first moment, and the other covers second; a ground single the centre fielder fields in front of him no longer sends a relay man out toward third. Since 2026-11-21 the same jobs are taken while a line drive or blooper is still in the air, as soon as the infielders see it will fall in front of an outfielder, and a line-drive or fly single with nobody on gets its cutoff man for second too ([note](docs/calibration/2026-11-21-liner-single-jobs.md)).*
- [~] 71. Right fielder makes error tracking the rolling ball into the outfield. It goes by him and runner gets a triple. Why? I think the fielder's tracking is off. *Found: you are right. His read of the liner in the air was off by 4–5 m (a possible misjudgment), but at the first bounce his tracking of the ball stops and a fresh, worse guess replaces it, which put the ball 15 m closer than it was. Fix queued: he keeps tracking the ball through its bounces; with a true read he holds the batter to a single.* *Fixed 2026-11-21 (`work/bounce-track`, [note](docs/calibration/2026-11-21-bounce-track.md)): fielders now keep tracking a ball through its bounces and as it rolls, and no fielder or runner ever pictures a ball under the ground. On this ball the right fielder no longer turns for the wall: he runs at it and his read is right as it reaches him, but he is still a step short (he misjudged the liner while it was in the air) and he does not dive at it, so it is still a triple. Across 300 games triples fell by a fifth and runs by 0.1 a team-game.* *2026-11-23 (`work/of-ground-dive`, [note](docs/calibration/2026-11-23-of-ground-dive.md)): he did not dive because an outfielder only dove when he expected a clean catch; on a ground ball a knock-down stops it too, so he now dives for a grounder he can reach. On this play he dives and the ball misses his glove by under 2 cm (about a one-in-six chance), so it is still a triple. Across 300 games outfielders dive at ground balls about 0.3 more a game, and triples fall by about a seventh. After the review (2026-11-24) he weighs what a knock-down or a miss would cost him, so he no longer lays out at a ball he would have had on his feet soon after.*

Running:
- [ ] 90. Runner gives up during a rundown. Need to work on fielders performing rundowns as well. *Found: the stand-in lets a trapped runner turn round only once; he used it, so he could not go back when the ball went to third. The real rundown is planned (TASKS "Open sim work" §1, Q11).*

Umpire, camera and screen:
- [x] 89, 146. A fair ball down the line: the umpire should signal it by pointing toward the infield as soon as it is officially fair. *Done 2026-11-20: on a fair ball near a foul line, the nearer base umpire (or the plate umpire in front of the bags) points to fair ground as soon as the ball is fair under the rules; in 89 and 146 the third-base umpire points 0.15 s after the ball comes down at the bag. Routine balls up the middle get no signal. You have not seen it moving yet.*
- [x] 112. When the batter swings the camera zooms in too early (before the bat makes contact). This tells me that the hit ball is predetermined instead of realistic 3D physics. *Done 2026-11-20: the camera now cuts to the batted ball only after the bat meets it on screen (it was cutting 0.05 s early). Why it could: the hit is real physics (the bat and ball collide in the sim, nothing is rolled), but the sim works out the whole play before the screen plays it back, so the camera knew the outcome early. The other on-screen cues tied to the hit (text, pitch line, umpires) were checked and wait for contact; whether any fielder starts moving before contact has not been checked yet.*
- [x] 128. After the play, during the inning switch over, the score card flashes for 1 second or so, which is far too short. Let it stay on the screen for 10 seconds, or the player can press Enter to continue sooner. *Done 2026-11-20: it stays 10 s of real time whatever the game speed, Enter continues sooner, and the card says so. It also no longer shows an empty inning for the side about to bat.*

# Saved plays, 2026-11-22 night (seed 5593)

Watch any of them again with `client/run.sh -BbSavedPlay="saved-plays/<file>.toml"` (files `seed5593-play<N>-…`). Played on main `76d4eaa`, before the double cut merged.

- [ ] 18. No ball shadow during pitch or throw back.
- [ ] 3. Fix first baseman's catching ball. A first baseman plants one foot on the base and then steps (stretches) to catch the ball sooner to help the ball beat the runner to the base. *Same as 86 above; not built.*
- [ ] 52. Fielders appear to stop too quickly as they converge on a fly ball.
- [ ] 56. Third baseman should have either tried to tag the runner out or immediately thrown to 2nd base. Instead he ran part way to the base looking for a force out. But then threw to first base instead.
