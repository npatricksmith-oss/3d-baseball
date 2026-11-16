# Player visuals: figures, motions, kit

**Status:** plan agreed with the owner 2026-11-15 (seven questions, Q15–Q21, all as recommended; §1). Steps 1 (the figure), 2 (the kit), 3 (the delivery and the catcher), 4 (the swing) and 5 (throws and catches in the field) are built; steps 6 and 7 are not. This is the second part of the graphics stage, after [the park](park-visuals.md) and before [the HUD](hud.md).

**Where it starts.** Every player is the engine's grey mannequin (`SKM_Manny_Simple`) with a team tint, moved by stock clips standing in for baseball: a charged punch for the delivery, an attack for the throw and the stance, a landing for the catcher's crouch, a wall jump for the dive (`BbDirector.cpp`, `LoadAssets`). The bat is a block. There are no gloves, caps, numbers, umpires or coaches.

## 1. Decisions (owner, 2026-11-15)

| # | Question | Ruling |
|---|---|---|
| Q15 | What the figures are | **Built in code on the mannequin's skeleton.** Head, cap, torso, limbs as simple shaped pieces riding on the bones, so the existing motion still drives them. The grey mannequin itself is hidden. Same family as the park and the crowd. No imported character model. |
| Q16 | Motions | **Baseball motions written in code as poses,** timed by the sim so the hand is at the release point and the bat at contact (invariant 4: the animation conforms to the physics). Order: pitching delivery, swing, throw, catcher's crouch and receive, fielder's ready and catch, slide. No motion-capture assets. |
| Q17 | Uniforms and teams | **Two invented clubs.** Home in white, visitors in grey, each with a trim colour and a cap; the roster's number on the back. No names on the back. The club names are the lead's to propose (§5). |
| Q18 | Body variety | **Client-side for now.** Height, build and skin tone derived from the player's id, the same every time, no sim change. Replaced by the roster's own figures when ratings arrive. |
| Q19 | Equipment | **Full simple kit.** Glove on the correct hand, a turned bat, batting helmets, the catcher's mask and chest protector. |
| Q20 | Other people | **Umpires, coaches, benches.** A plate umpire and three base umpires who signal the sim's calls; two base coaches standing in their boxes; seated figures on the dugout benches. All decoration: the coach's read (Q10) stays with the paused physics work. |
| Q21 | Sim changes | **Read-only exports only,** and only if a motion needs something the timeline does not carry (for example the bat's path). Game digests unchanged, one line in the metrics ledger under "No effect on games". |

Carried over from the park plan without asking again: the lead builds alone, one step per short session; before/after screenshots after each merge, work does not wait for an OK; no Codex reviews in this stage; nothing binary is committed.

## 2. Approach

- **A figure builder apart from the director.** New `BbFigure.{h,cpp}` owns what a player is made of: the pieces, their sizes, the uniform, the kit. `BbDirector` keeps who stands where and when. The look (club colours, skin tones, sizes of pieces) goes in `BbLook.h` with the park's.
- **Pieces on bones.** Each figure keeps its skeletal mesh as an invisible driver; shaped pieces are attached to its bones (head, cap, chest, pelvis, upper and lower arms, hands, thighs, shins, feet). The pieces are generated meshes, as the park's are, so a forearm tapers and a cap has a bill. This is the open technical point of step 1: if attached pieces cost too much at 24 figures plus the extras, the fallback is one generated skinned mesh on the same skeleton.
- **Poses in code.** A motion is a short table of key poses (joint angles at named moments) blended over the sim's own times: the delivery ends with the hand at the sim's release point at release time; the swing puts the barrel at the sim's contact point at contact time; a throw releases when the sim's throw leaves. Hands and bat are placed by two-bone IK onto the physical point, so the motion can never disagree with the ball. Left-handers are the same tables mirrored (the director already mirrors figures).
- **Stock clips stay where they are honest:** standing, walking, jogging, running.
- **Numbers and lettering** are painted in code with `BbText.h`, as the wall's distance figures are.
- **Budget.** 193 fps unattended after the crowd. Every step reports the frame rate from the same headless run.

## 3. Steps

1. **The figure.** `BbFigure`; pieces on the skeleton; home whites and road greys with trim, cap, the number on the back; height, build and skin tone from the player's id. Motion is whatever it was.
2. **Kit.** The glove on the glove hand, a turned bat, batting helmets for the batter and runners, the catcher's mask and chest protector.
3. **The delivery and the catcher.** Set, leg lift, stride, release at the sim's point and time, follow-through; the catcher's crouch, target and receive.
4. **The swing.** Stance, load, stride, the barrel to the sim's contact point (or through the zone on a miss), follow-through; the take. A read-only export of the bat's path goes here if the timeline lacks it (Q21).
5. **Throws and catches in the field.** The fielder's ready position, the glove to the ball, the transfer, the throw with release at the sim's time; the first baseman's stretch as far as the sim has one.
6. **The slide,** and the dive redone in the same way as the other motions.
7. **Other people.** Four umpires with out, safe, strike and foul signals from the sim's calls; two base coaches; the benches.

## 4. Checking the look

- `client/run.sh 7 -RenderOffscreen -BbShots=6` for play; `-BbParkShots` for the seven fixed spots. Steps 1 and 2 add **figure spots** that do not depend on the play: a line-up of both nines front and back at close range, and one figure turned through eight angles.
- Motions are checked as strips: the same motion photographed at its named moments (as `-BbDiveShots` does for the dive), with the ball in the frame at release and at contact.
- The ball must stay easy to see against the uniforms, the white ones above all.

## 5. Proposed by the lead, for the owner to overrule

- **The clubs** (`BbLook::Clubs`). Home: the **Cedar Hollow Larks**, white with navy lettering, number, socks and cap, "LARKS" across the chest. Visitors: the **Stonebridge Colliers**, grey with dark red lettering, number, socks and cap bill under a black cap, "STONEBRIDGE" across the chest. Both invented.
- **The cut of the uniform:** short sleeves over bare arms, a dark belt, trousers to the knee with the sock showing below (the old high-sock look, so the two clubs differ at a distance by their legs as well as their shirts), black shoes.
- **Bodies:** 177–197 cm standing and builds from 0.92 to 1.14 of the middle one, bunched toward the middle; four skin tones and three hair colours, all by the player's id.
- **No faces.** A head is a plain shape under a cap; at the distances the game is watched from a face would not show. Say if one is wanted.

## 6. Step 1 as built (2026-11-15): the figure

`BbFigure.{h,cpp}`. A figure is 23 pieces, each a shape turned about its own axis (a tapered limb with rounded joints, an egg for the head, flattened rings for the trunk) and generated once at start-up from the mannequin's own skeleton, so each runs from joint to joint wherever the joints are: hips, belt, the shirt's middle, the chest, neck, head, hair, cap and bill, and on each side a sleeve, upper arm, forearm, hand, thigh, sock and shoe. Each piece is a static mesh fixed to one bone; the mannequin still plays every clip and is not drawn. The chest carries a picture painted in code (`BbText.h`): collar, placket, the club's lettering on the front, the number on the back.

- **The open technical point of §2 is settled: pieces on bones are cheap enough.** No skinned mesh was needed.
- **A figure shown in a mirror** (the director mirrors a left-hander's delivery and throw) has its lettering painted backwards so it still reads, and its pieces' drawing orders are remade when the mirror changes; without that the engine drew the figure inside out. The check picture `figure_6_turn_mirrored` found it.
- **Checks added:** `-BbFigureShots` (no game: each nine in a row from the front and from behind, then one player turned through eight angles, plain and in a mirror, as `figure_<n>_<what>.png`); `-BbMannequin` (the old grey mannequin, for comparison). For pictures large enough to read the lettering, add `-ResX=1920 -ResY=1080 -ForceRes`.
- **Pictures:** `client/Saved/Shots/players1_before/` and `players1_after/` (untracked): the six figure pictures and six of play each.
- **Cost: not yet measured cleanly.** The owner's own game was running on the laptop throughout, so the 193 fps figure could not be repeated. Under that shared load, same run (`-BbShots=6`, seed 7): 73 fps with the mannequin, 65 and 52 with the figures (two runs). Memory 2.6 GB, unchanged. Measure again on a free machine at the start of step 2; if the cost is real, the first thing to try is not moving the pieces of figures that are hidden (40 figures are kept, at most 16 shown).
- **Left for later:** the shirt's hem shows a slightly ragged edge where the chest overlaps the middle piece; hands are plain rounded shapes (step 2 gives one of them a glove).

## 7. Step 2 as built (2026-11-15): the kit

Nine more pieces a figure (32 in all), each worn or not by what he is doing (`ABbFigure::SetRole`), and a turned bat. Sizes and colours are in `BbLook.h`.

- **Glove:** a leather shape over the hand he does not throw with (the roster's `throws`), about 26 cm long, larger than life as the ball and bat are. A figure shown in a mirror moves it to the other bone, so it stays on the same hand as the viewer sees it (checked by `figure_8`). Every fielder wears one; the catcher's is 18% larger.
- **Batting helmet:** a shiny shell in the cap's colour with flaps over both ears and a short bill, in place of the cap, for every batter and runner.
- **The catcher:** the same shell without a bill, a dark mask over the face, a chest protector in the club's trim, dark shin guards (dark so that they show against the sock).
- **The bat:** turned on the figures' lathe (knob, handle, barrel), 35% thicker than real so the swing reads, stretched between the sim's knob and tip as the block was. With `-BbMannequin` the block stays.
- **Checks added:** `-BbFigureShots` now takes nine pictures: `figure_7_kit_field` (the home nine as they take the field), `figure_8_kit_field_mirrored_back` (the same in a mirror, from behind), `figure_9_kit_bat` (eight visitors in helmets through eight angles, a bat stood among them).
- **Pictures:** `client/Saved/Shots/players2_before/` (six of play) and `players2_after/` (untracked).
- **Cost, measured this time** (seed 7, `-BbShots=6 -BbUncapped`, a browser busy on the machine): 174 fps with figures and kit, 190 with the bare mannequin in the same build, against 193 before the players. So the figures cost about a tenth. Not acted on; the first thing to try is still not moving the pieces of hidden figures.
- **Left for later:** the mask is a solid shape, not a cage; the first baseman has an ordinary glove; a glove is one shape without a thumb or web; (the hands met the bat with the swing, step 4: §9); batting gloves and the batter's shin guard are not made.

## 8. Step 3 as built (2026-11-15): the delivery and the catcher

The first motions written as poses in code. `BbPose.{h,cpp}` holds all of it.

- **A pose is a stick figure** (`FBbStick`): where the hips are and which way they face, which way the chest and head face, each wrist from its own shoulder (in the chest's frame, so an arm goes round with the trunk), each ankle, and which way each elbow and knee points. `FBbSkeleton::Solve` turns one into the mannequin's bone transforms: the spine and neck share out the turn between hips, chest and head; each limb is two-bone IK. A hand can be told a point it must reach (`FBbReach`): the body moves up to 24 cm for the arm to get there.
- **Poses over clips.** `UBbPoseAnim` is the engine's single-clip player with a pose laid over the clip by a weight, so a figure goes from standing into a motion and back in 0.3 s. Every figure has one; the stock clips play through it as before. `-BbStockMotions` turns the coded motions off.
- **The delivery** (`BbMotion::Delivery`): eight key poses on the sim's clock, from the stretch: set (−1.4 s), leg lift (−0.84), stride (−0.48), plant (−0.16), the arm laid back (−0.045), release (0), follow-through (+0.2), finish (+0.6), joined by a curve that keeps its speed through each key. Round the release the throwing hand is put on the sim's release point (the ball's place at time 0), so **the ball leaves the hand where and when the sim says**. Through the wind-up the ball is drawn in his hand. The wind-up shown before each pitch is 1.4 s now (was 1.0). A left-hander is the same motion in a mirror.
- **Where he finishes.** The delivery carries him 1.3 m down the mound and he stands there, as a pitcher does; the sim still has him on the rubber. The figure is drawn off the sim's position by that much (`FFigureState::Off`): gathered in at 3 m/s once a play has him running, backed up the mound at 1.3 m/s between pitches. He comes set in the last 0.8 s of the reset.
- **The catcher** (`BbMotion::Crouch`): a squat on the balls of his feet, knees wide, bare hand behind his right leg, sitting 66 cm behind the sim's spot for him so that the mitt is out in front. The mitt is held for a target, goes to where the sim has the pitch caught over the last quarter second, and holds the ball there for 0.7 s; its fingers point away from his middle (up for a high pitch, down for a low one), and he bends to a low one. On a hit ball the mitt goes to where the pitch was heading and he is up at once. He squats in the last 0.8 s of the reset.
- **Measured** (`-BbStagingLog`, 66 pitches of seed 7, and seed 6 for a left-hander): at release the ball in the pitcher's hand is 2.3–2.4 cm from the sim's release point, right- or left-handed, at two heights of pitcher; the mitt's wrist is where it was asked to be on the median pitch and 11 cm short at worst (pitches in the dirt).
- **Checks added:** `-BbMotionShots` photographs the second pitch from beside the pitcher, the third from the plate and the fourth beside the catcher, each held on its named moments, as `motion_<pitch>_<n>_<what>.png` (20 pictures); `-BbStagingLog` adds `BB_RELEASE` and `BB_RECEIVE` lines.
- **Pictures:** `client/Saved/Shots/players3_after/` (untracked): the strips and six of play. Before: `players2_after/`.
- **Cost:** 187 fps (seed 7, `-BbShots=6 -BbUncapped`), against 174 at step 2 on a busier machine: nothing measurable.
- **Left for later:** a throw that begins while a motion is still going out cuts to the throw clip at once (step 5 redoes throws); a left-hander's figure flips out of its mirror as the motion ends (both done in step 5: §10); the feet of a figure coming out of a motion slide a short way instead of stepping; no wind-up, only the stretch; the catcher gives the same target every pitch (the sim does not export where the pitcher is aiming); no sign-giving, no throw back from the crouch.

## 9. Step 4 as built (2026-11-16): the swing

The batter holds his bat and swings it. `BbMotion::Swing` in `BbPose.{h,cpp}`; the bat's handling is in `BbDirector`.

- **The body follows the bat.** The swing is not a table of key poses on a clock: every frame the pose is worked out from where the bat is. Both hands are put on the handle (his lower hand 6 cm up from the knob, the upper 15 cm); the hips and shoulders turn by how far round him his hands have come since the swing began (the hips first, nearly open at contact; the shoulders turned in at the start and coming round with the hands); the back foot turns on its toe with the hips; he bends toward a ball away from him or low; his eyes are on the ball. So a checked swing stops the body when the bat stops, a late swing is late all over, and **the barrel is at the sim's contact point because the bat is the sim's** (invariant 4).
- **The timeline's bat was enough (Q21): no new export.** It already has the knob and the direction every 5 ms through the swing. One presentation value in the sim moved: the made-up stance (`bb_game::timeline::stance_pose`) had the knob over the batter's own spot, inside his chest; it is now 0.40 m toward the catcher, 0.10 m toward the plate and 1.30 m up, in front of his back shoulder. Game digests are unchanged (ledger, "No effect on games"). What the swings ask of the hands was measured first with a new example, `cargo run --release -p bb_game --example swing_pose_probe` (1,042 swings: the knob 0.57 m behind his spot as the swing starts, 0.34 m toward the plate at contact, the hands' arc 0.56 m in radius).
- **The stride** is on the clock, and the same at every pitch whether he swings or not, so it gives nothing away: the front foot comes up as the ball is let go and is down 16 cm nearer the pitcher 0.17 s later.
- **The bat between swings is his to carry.** The sim's bat track ends 0.12 s after contact. After a swing and a miss or a foul he brings the bat back to his shoulder over 0.8 s (unwound in front of him, not drawn back through his chest), stands at ease with it between pitches, and lifts it into the stance as the pitcher comes set. A batter who runs has dropped it: it is no longer drawn (it used to hang in the air for a quarter of a second, and through the whole of a walk).
- **A left-handed batter** is the same in a mirror, by which box he stands in (so a switch hitter is right).
- **Measured** (`-BbStagingLog`, `BB_SWING`, six balls hit in seed 7, right- and left-handed): both hands on the handle at contact, 0.0 cm off.
- **Checks added:** `-BbMotionShots` now goes on after its first three pitches to the first ball hit (`motion_4_*`: stance, stride, launch, turn, contact, through, finish, from across the plate), the first swing and miss (`motion_5_*`: the same and the bat coming back, from the mound's side) and the first pitch taken (`motion_6_*`): 38 pictures in all.
- **Pictures:** `client/Saved/Shots/players4_after/` (untracked): the strips and six of play. Before: `players3_after/`.
- **Cost:** 166 fps against 173 for step 3 run on the same machine minutes apart (seed 7, `-BbShots=6 -BbUncapped`, other work going on): nothing measurable.
- **Left for later:** a new batter walks up without a bat and has one when he stands in (carrying it needs a pose over the walk); no dropped bat lying on the ground; the hands are plain shapes that do not close round the handle; the arms are straight at the start of the swing because the sim's hands start 0.57 m back; every batter has the same stance and stride; no check-swing or bunt look of its own; after a swing the feet stay wide until the next pitch is near.

## 10. Step 5 as built (2026-11-16): throws and catches in the field

Every fielder gets ready, puts his glove on the ball, brings it in to his hand and throws it. `BbMotion::Field` in `BbPose.{h,cpp}`; who catches and throws when is read from the timeline in `BbDirector` (`NoteCatches`, `AddThrow`).

- **One pose from several parts,** as the swing is, not a clip on a clock: the ready position, the glove's reach, the ball carried in both hands, and the throw are each a pose, laid one over another by how much of each is on. So a quick throw runs straight on from the catch, and nothing cuts.
- **Ready.** Infielders and outfielders are down and set as the pitcher comes set (feet wide, knees bent, hands in front of the knees) and stay so until the ball is past the batter or 0.3 s after it is hit.
- **The glove to the ball.** The whole timeline is read as the pitch comes up, so each catch is known ahead: the glove sets out 0.36 s before and is on the ball where the sim has it taken (invariant 4). He bends and gets down for a ball on the ground, goes up for one over his head (the sim has catches to 2.5 m), lunges and steps for one away from him. The sim has most balls taken over the fielder's own feet; he gives ground so that it is caught a forearm in front of him. **A man going slower than 3 m/s turns to face the ball** and his feet are the pose's; faster, he takes it on the run facing where he runs.
- **On the run only the trunk and arms are posed.** `UBbPoseAnim::LowerWeight`: the hips and legs stay the running clip's and the trunk is built on the clip's own hips, so a man running under a fly keeps running while his glove goes up. A ball on the ground or overhead takes the legs too.
- **The ball in his hands.** It sits in the glove's pocket as he takes it and passes to the throwing hand over the next third of a second. The timeline's ball jumps to the sim's holding point over his feet in the last sixtieth of a second before a catch; the drawn ball is carried on along its own line into the glove instead.
- **The throw** is seven key poses on the sim's clock from 0.45 s before the release to 0.5 s after: gather, side on with the hands breaking, front foot down and the arm cocked, the arm laid back, release, follow-through, finish. **The hand is on the sim's release point as the ball goes** (1.8 m up and 0.35 m to his throwing side; a man on the mound stands above the sim's level field and sinks to it). The ball leaves his hand on the sim's path with nothing to ease. The stock throw clip and its left-arm mirror are gone (they remain under `-BbStockMotions`).
- **The first baseman's stretch** is as far as the sim has one, which is not far: the sim's receiver has no stretch (TASKS, fielding left open), so he takes throws near his feet. A man taking a ball away from him steps to it with the foot on that side, and that is all the stretch there is.
- **Between pitches** the ball's returner makes the same throw (his hand at 1.8 m, where the lob now starts) and the pitcher puts his glove up for it.
- **A left-handed fielder is in a mirror the whole time he is in the field,** not only while a motion shows, so he no longer flips as a motion ends (left over from step 3).
- **Reaches are exact now.** What a hand holds sits on along the forearm, which is not the line from the shoulder when the elbow is bent; the solver moves the wrist until the held thing is on the point. The pitcher's release, 2.3–2.4 cm off in step 3, is 0.1 cm.
- **Measured** (`-BbStagingLog`, `BB_CATCH` and `BB_LETGO`, the play held on each catch and release; 200 s each of seeds 7 and 6 at six times speed, 232 pitches): 111 catches, the glove's pocket on the ball to within 3 cm in 108 and 10 cm off at worst (ground balls taken more than a metre from his feet); 52 throws, the hand within 0.5 cm of the sim's release point in every one, left-handers included.
- **Checks added:** `-BbMotionShots` goes on to the first ball fielded and thrown by one man (`motion_7_*`: ready, coming, reach, catch, in, step, cocked, laid back, release, follow) and the next throw taken (`motion_8_*`): 53 pictures in all. The camera goes with the man, in front of him until he has the ball and beside him for the throw.
- **Pictures:** `client/Saved/Shots/players5_after/` (untracked): the strips and six of play. Before: `players4_after/`.
- **Cost:** 150 fps against 153 with the stock clips, the same build minutes apart (seed 7, `-BbShots=6 -BbUncapped`; the machine gave lower figures all round than on the day of step 4).
- **Left for later:** fly balls are taken at the belt because that is where the sim has them caught; a throw made on the run turns the whole figure to the target, legs and all; a pitcher or catcher who goes from his own motion into a catch within a third of a second passes through standing; a fielder who gives ground for a ball slides his feet rather than stepping; the hands do not close on the ball; no tag is shown, and no foot on the bag; relay men and cut-offs stand as anyone does.

## 11. Step 6 as built (2026-11-16): the slide and the dive

The last two stand-in clips are gone: a diving fielder and a sliding runner are poses in code. `BbMotion::Dive` and `BbMotion::Slide` in `BbPose.{h,cpp}`; when each happens is read from the timeline in `BbDirector` (`NoteSlides`, and the sim's own dive record).

- **The dive** is the sim's (it says who dives, which way, and when he is up; 1.5 s in all). He drives off his back foot, is laid out in the air with the glove leading, lands on his front, lies there, and gets up through his hands and knees and one knee, the ball coming in to both hands. The tipped-over leap clip and the landing clip played backwards are no longer used (they remain under `-BbStockMotions`).
- **The drawn dive starts 0.17 s before the sim's** (`DiveLeadS`). The sim has a diving fielder take the ball at the instant he leaves his feet, his body travelling after it for 0.3 s. Drawn on the sim's clock he would catch it standing, 1.4 m away. Drawn a sixth of a second early he is in the air with **the glove on the ball where and when the sim has it caught** (invariant 4), and the glove stays on that spot while the rest of him comes down behind it. A ball farther off than his arm reaches has him drawn up to 60 cm ahead of the sim's place for him until the sim's own travel catches up.
- **The slide is the client's reading of what the sim did.** The sim has no slide: a runner going on to a base he stops at runs into it at full speed and is still from the next moment. `NoteSlides` finds each such stop at second, third or home as the pitch comes up and, **if a play is being made on him there, draws it as a feet-first slide**: the ball taken within 5 m of the bag within 1.5 s of his getting there, or already held there, or he is called out there; and he came in at 5 m/s or more. Nothing feeds back to the sim.
- **The slide itself** is seven key poses on the clock from 0.42 s before he is at the bag to 1.2 s after: a last stride, sitting back with the left leg folding under, down on his left hip with the right foot out and his hands off the dirt, the foot on the bag, sitting up, up over the bag foot, standing on it. The sim's place for him is his leading foot once he is down, so **the foot reaches the bag when the sim has him there** and his body trails 60 cm behind it. Feet-first and right leg leading every time.
- **Measured** (`-BbStagingLog`, seed 7, 274 pitches at six times speed): 16 arrivals at a base to stop; 3 were walked back to; of the 13 at a run, 2 had a play on them and slid; the one diving catch had the glove 0.0 cm off the ball (96 cm before the lead).
- **Checks added:** `-BbSlideShots=<n>` and `-BbDiveShots=<n>` hold the play on five moments of the next n slides and dives and photograph each from beside (`slide_<k>_<1..5>.png`: going down, sliding, at the bag, sitting up, getting up; `dive_<k>_<1..5>.png`: driving off, in the air at the sim's catch, down, hands and knees, getting up). They work at any playback speed, so `-BbSpeed=6` finds them quickly. `-BbStagingLog` logs each arrival as `BB_SLIDE` (speed in, whether a play was on him, whether he slid) and a diving catch as `BB_CATCH ... dive=1`.
- **Pictures:** `client/Saved/Shots/players6_after/` (untracked): two slides and two dives, five moments each.
- **Cost:** 161 fps (seed 7, `-BbShots=6 -BbUncapped`) against 150 at step 5: nothing measurable.
- **For the owner to overrule if he likes:** a runner slides only when a play is being made on him; every slide is feet-first with the right leg; he sits a third of a second and stands up on the bag.
- **Left for later:** 11 of the 13 running arrivals had no play near and still stop dead standing, because the sim stops them dead: a runner who slows into a base he will hold is sim work (with the paused runner work). A runner put out before he reaches the bag leaves the picture where he is, so no slide into a tag is seen, and no tag (step 5's list). No head-first slide, no dive back to a base, no slide at first. A play that ends within a second of the slide cuts the getting-up short as the walk-off begins. Every dive is head-first along the sim's direction; there is no backhand sprawl or leap. The man on hands and knees looks back over his shoulder at the ball too far.

## 12. Records

- PLAN D13 (revised 2026-11-15) covers this stage.
- TASKS.md "Resume here" carries the step reached; a `/clear` point after each merged step.
