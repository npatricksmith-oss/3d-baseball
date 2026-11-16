# Observations

The owner's list of what he has seen and wants, with where each stands. The letter is his priority (A, B, C). Every lead session updates the boxes here when it finishes something, so this file is enough to follow progress; [TASKS.md](TASKS.md) has the detail.

`[x]` done · `[~]` partly done (the note says what is left) · `[ ]` not started

**Last updated:** 2026-11-16

# Added 2026-11-16

Screen:
- [x] Reduce ball, bat and glove (owner: the ball only; leave the glove and bat as they are). *Done 2026-11-16: the ball was cut to its real size, 7.5 cm (it was 15). You still could not see it, nor a quarter larger, so it is now 11.7 cm, which you found fine. (A smallest size on screen was tried and taken out again: too big.) Still hard to see, so the grass and dirt are a little darker now, to set the white ball off.*
- [x] Ball shadow should be half as big and 30% more blurred when hit. *Done 2026-11-16: half the size, and its edge now fades out over the outer 30% of it (before, the edge was hard). Not yet built or seen.*
- [ ] Runners in the mini base map should be according to the real runners. *Today the map shows the bases as they were before the pitch until the play ends, then jumps to the result.*

Fielding:
- [ ] Seed 40580, play 15: "CF seriously misplayed the ball. Misjudgment? Bug?" *Found 2026-11-16: a fault in how fielders judge a fly, not a believable misjudgment. The center fielder called it and got there in time, but all the way down he thought it would land about 2 m to his right of where it did, and he still thought so half a second before it came down, with the ball 8 m from him. A real outfielder doesn't misjudge a ball's direction that late. So he stopped 1.6 m from it, realised too late to dive, and it rolled to the wall: an inside-the-park home run. The cause is in the sim's read of a fly: a small error that stays the same on every look gets trusted as if it were new information each time, so he grows sure of a wrong line and stops correcting it. With that error left out he reads it to 7 cm and makes the catch. The fix is fielding physics, which is paused, so it waits for reviews to come back (TASKS backlog).*

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
- [~] A. Implement baserunners sliding at appropriate times. *Since 2026-11-16 a runner slides feet-first into second, third or home when a play is being made on him there. Left: with no play on him he still stops dead standing (the sim stops him dead; slowing into the base is sim work), and a runner put out before the bag is not seen sliding into the tag. You have not seen it moving yet.*
- [ ] A. Implement stolen bases.
- [~] A. Variety of body shapes, nationalities, ethnicities, skin color. *Height, build and skin tone vary by player; names do not yet follow nationality.*
- [x] A. Team uniforms.
- [x] A. Limits of play area (I saw a foul ball go back out of play and continued on). After the ball goes out of the stadium, cut back to field of play. *Since 2026-11-16, after a foul into the seats the game goes straight to the next pitch.*
- [ ] A. Runners are leading off before the pitcher gets set on the mound. In the real world the pitcher would pick them off.
- [~] A. Post game box score. *There is a box score page: it comes up at the end of the game, and Tab shows it during one. It gets the new look in the last HUD step. The broadcast-style score bug and play caption went in on 2026-11-16.*
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
- [~] A. Each player has throwing ratings (accuracy, speed, throwing set-up time). *The ratings exist in the model; every player is average until players get their own ratings.*
- [~] A. Smarter player decisions based on runners on base and number of outs. E.g. with a runner on 3rd and 2 outs, fielders play to get the force out at first or second instead of throwing home. *Throw choices weigh outs and runners; more cases are in your saved plays below.*
- [~] A. Players dive for balls, increasing range. Takes effort to rise back up to throw if the catch or grounder has been gotten. *Dives and the get-up work in the sim at a realistic rate. Since 2026-11-16 the dive is its own motion: laid out with the glove on the ball, down, up through hands and knees. Left: every dive is head-first straight at the ball. You have not seen it moving yet.*

# Hitting
- [x] A. Ball shadow after hit can't be seen.
- [x] A. Once a home run goes over the wall, move camera back to center to see runners moving around bases.
- [x] A. Hitting home runs brings up display with player name, how far ball was hit and mph.
- [x] A. Hitting animation needs work. Batting stance. Initial load. Swing. Follow through. *Coded stance, stride and swing, hands on the sim's bat. Different stances per player come later. You have not seen it moving yet.*

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
- [ ] 65, 67, 68. On a fly that might be caught, the runner goes part way (halfway), stops in an athletic stance and watches the ball and outfielder; if he judges it caught he goes back to tag up; even on a likely catch he comes off the base a little in case it is misplayed.
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

Catcher:
- [ ] 40. On a pop foul the catcher takes his mask off and tosses it aside (for the look only).
