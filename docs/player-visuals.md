# Player visuals: figures, motions, kit

**Status:** plan agreed with the owner 2026-11-15 (seven questions, Q15–Q21, all as recommended; §1). All seven steps are built (§6–§12): the figure, the kit, the delivery and the catcher, the swing, throws and catches in the field, the slide and the dive, and the other people. The plan is complete. A redesign of the batter's stance and swing (§13, 2026-11-23) is specified and not yet built. This is the second part of the graphics stage, after [the park](park-visuals.md) and before [the HUD](hud.md).

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

- **A figure builder apart from the director.** New `client/Source/BaseballClient/BbFigure.cpp` owns what a player is made of: the pieces, their sizes, the uniform, the kit. `BbDirector` keeps who stands where and when. The look (club colours, skin tones, sizes of pieces) goes in `client/Source/BaseballClient/BbLook.h` with the park's.
- **Pieces on bones.** Each figure keeps its skeletal mesh as an invisible driver; shaped pieces are attached to its bones (head, cap, chest, pelvis, upper and lower arms, hands, thighs, shins, feet). The pieces are generated meshes, as the park's are, so a forearm tapers and a cap has a bill.
- **Poses in code.** A motion is a short table of key poses (joint angles at named moments) blended over the sim's own times: the delivery ends with the hand at the sim's release point at release time; the swing puts the barrel at the sim's contact point at contact time; a throw releases when the sim's throw leaves. Hands and bat are placed by two-bone IK onto the physical point, so the motion can never disagree with the ball. Left-handers are the same tables mirrored (the director already mirrors figures).
- **Stock clips stay where they are honest:** standing, walking, jogging, running.
- **Numbers and lettering** are painted in code with `client/Source/BaseballClient/BbText.h`, as the wall's distance figures are.
- **Budget.** 193 fps unattended after the crowd. Every step reports the frame rate from the same headless run.

## 3. Steps

The seven built steps are the figure (§6), kit (§7), delivery and catcher (§8), swing (§9), throws and catches (§10), slide and dive (§11), and crew (§12). The client implementation lives in `client/Source/BaseballClient/BbFigure.cpp`, `client/Source/BaseballClient/BbPose.cpp` and `client/Source/BaseballClient/BbCrew.cpp`.

## 4. Checking the look

- `client/run.sh 7 -RenderOffscreen -BbShots=6` for play; `-BbParkShots` for the seven fixed spots. `-BbFigureShots` uses **figure spots** that do not depend on the play: a line-up of both nines front and back at close range, and one figure turned through eight angles.
- Motions are checked as strips: the same motion photographed at its named moments (as `-BbDiveShots` does for the dive), with the ball in the frame at release and at contact.
- The ball must stay easy to see against the uniforms, the white ones above all.

## 5. Proposed by the lead, for the owner to overrule

- **The clubs** (`BbLook::Clubs`). Home: the **Cedar Hollow Larks**, white with navy lettering, number, socks and cap, "LARKS" across the chest. Visitors: the **Stonebridge Colliers**, grey with dark red lettering, number, socks and cap bill under a black cap, "STONEBRIDGE" across the chest. Both invented.
- **The cut of the uniform:** short sleeves over bare arms, a dark belt, trousers to the knee with the sock showing below (the old high-sock look, so the two clubs differ at a distance by their legs as well as their shirts), black shoes.
- **Bodies:** 177–197 cm standing and builds from 0.92 to 1.14 of the middle one, bunched toward the middle; four skin tones and three hair colours, all by the player's id.
- **No faces.** A head is a plain shape under a cap; at the distances the game is watched from a face would not show. Say if one is wanted.

## 6. Step 1 as built (2026-11-15): the figure

`client/Source/BaseballClient/BbFigure.cpp`. A figure is 23 pieces, each a shape turned about its own axis (a tapered limb with rounded joints, an egg for the head, flattened rings for the trunk) and generated once at start-up from the mannequin's own skeleton, so each runs from joint to joint wherever the joints are: hips, belt, the shirt's middle, the chest, neck, head, hair, cap and bill, and on each side a sleeve, upper arm, forearm, hand, thigh, sock and shoe. Each piece is a static mesh fixed to one bone; the mannequin still plays every clip and is not drawn. The chest carries a picture painted in code (`client/Source/BaseballClient/BbText.h`): collar, placket, the club's lettering on the front, the number on the back.

- **A figure shown in a mirror** (the director mirrors a left-hander's delivery and throw) has its lettering painted backwards so it still reads, and its pieces' drawing orders are remade when the mirror changes; without that the engine drew the figure inside out. The check picture `figure_6_turn_mirrored` found it.
- **Checks added:** `-BbFigureShots` (no game: each nine in a row from the front and from behind, then one player turned through eight angles, plain and in a mirror, as `figure_<n>_<what>.png`); `-BbMannequin` (the old grey mannequin, for comparison). For pictures large enough to read the lettering, add `-ResX=1920 -ResY=1080 -ForceRes`.

## 7. Step 2 as built (2026-11-15): the kit

Nine more pieces a figure (32 in all), each worn or not by what he is doing (`ABbFigure::SetRole`), and a turned bat. Sizes and colours are in `client/Source/BaseballClient/BbLook.h`.

- **Glove:** a leather shape over the hand he does not throw with (the roster's `throws`), about 26 cm long, larger than life as the ball and bat are. A figure shown in a mirror moves it to the other bone, so it stays on the same hand as the viewer sees it (checked by `figure_8`). Every fielder wears one; the catcher's is 18% larger.
- **Batting helmet:** a shiny shell in the cap's colour with flaps over both ears and a short bill, in place of the cap, for every batter and runner.
- **The catcher:** the same shell without a bill, a dark mask over the face, a chest protector in the club's trim, dark shin guards (dark so that they show against the sock).
- **The bat:** turned on the figures' lathe (knob, handle, barrel), 35% thicker than real so the swing reads, stretched between the sim's knob and tip as the block was. With `-BbMannequin` the block stays.
- **Checks added:** `-BbFigureShots` now takes nine pictures: `figure_7_kit_field` (the home nine as they take the field), `figure_8_kit_field_mirrored_back` (the same in a mirror, from behind), `figure_9_kit_bat` (eight visitors in helmets through eight angles, a bat stood among them).
- **Cost, measured this time** (seed 7, `-BbShots=6 -BbUncapped`, a browser busy on the machine): 174 fps with figures and kit, 190 with the bare mannequin in the same build, against 193 before the players. So the figures cost about a tenth. Not acted on; the first thing to try is still not moving the pieces of hidden figures.

## 8. Step 3 as built (2026-11-15): the delivery and the catcher

The first motions written as poses in code. `BbPose.{h,cpp}` holds all of it.

- **A pose is a stick figure** (`FBbStick`): where the hips are and which way they face, which way the chest and head face, each wrist from its own shoulder (in the chest's frame, so an arm goes round with the trunk), each ankle, and which way each elbow and knee points. `FBbSkeleton::Solve` turns one into the mannequin's bone transforms: the spine and neck share out the turn between hips, chest and head; each limb is two-bone IK. A hand can be told a point it must reach (`FBbReach`): the body moves up to 24 cm for the arm to get there.
- **Poses over clips.** `UBbPoseAnim` is the engine's single-clip player with a pose laid over the clip by a weight, so a figure goes from standing into a motion and back in 0.3 s. Every figure has one; the stock clips play through it as before. `-BbStockMotions` turns the coded motions off.
- **The delivery** (`BbMotion::Delivery`): eight key poses on the sim's clock, from the stretch: set (−1.4 s), leg lift (−0.84), stride (−0.48), plant (−0.16), the arm laid back (−0.045), release (0), follow-through (+0.2), finish (+0.6), joined by a curve that keeps its speed through each key. Round the release the throwing hand is put on the sim's release point (the ball's place at time 0), so **the ball leaves the hand where and when the sim says**. Through the wind-up the ball is drawn in his hand. The wind-up shown before each pitch is 1.4 s now (was 1.0). A left-hander is the same motion in a mirror.
- **Where he finishes.** The delivery carries him 1.3 m down the mound and he stands there, as a pitcher does; the sim still has him on the rubber. The figure is drawn off the sim's position by that much (`FFigureState::Off`): gathered in at 3 m/s once a play has him running, backed up the mound at 1.3 m/s between pitches. He comes set in the last 0.8 s of the reset.
- **The catcher** (`BbMotion::Crouch`): a squat on the balls of his feet, knees wide, bare hand behind his right leg, sitting 66 cm behind the sim's spot for him so that the mitt is out in front. The mitt is held for a target, goes to where the sim has the pitch caught over the last quarter second, and holds the ball there for 0.7 s; its fingers point away from his middle (up for a high pitch, down for a low one), and he bends to a low one. On a hit ball the mitt goes to where the pitch was heading and he is up at once. He squats in the last 0.8 s of the reset.
- **Measured** (`-BbStagingLog`, 66 pitches of seed 7, and seed 6 for a left-hander): at release the ball in the pitcher's hand is 2.3–2.4 cm from the sim's release point, right- or left-handed, at two heights of pitcher; the mitt's wrist is where it was asked to be on the median pitch and 11 cm short at worst (pitches in the dirt).
- **Checks added:** `-BbMotionShots` photographs the second pitch from beside the pitcher, the third from the plate and the fourth beside the catcher, each held on its named moments, as `motion_<pitch>_<n>_<what>.png` (20 pictures); `-BbStagingLog` adds `BB_RELEASE` and `BB_RECEIVE` lines.
- **Cost:** 187 fps (seed 7, `-BbShots=6 -BbUncapped`), against 174 at step 2 on a busier machine: nothing measurable.

## 9. Step 4 as built (2026-11-16): the swing

*To be replaced in parts by [§13](#13-the-base-stance-and-swing-design-2026-11-23-not-yet-built) as its build steps land; until then this is what the game does.*

The batter holds his bat and swings it. `BbMotion::Swing` in `BbPose.{h,cpp}`; the bat's handling is in `BbDirector`.

- **The body follows the bat.** The swing is not a table of key poses on a clock: every frame the pose is worked out from where the bat is. Both hands are put on the handle (his lower hand 6 cm up from the knob, the upper 15 cm); the hips and shoulders turn by how far round him his hands have come since the swing began (the hips first, nearly open at contact; the shoulders turned in at the start and coming round with the hands); the back foot turns on its toe with the hips; he bends toward a ball away from him or low; his eyes are on the ball. So a checked swing stops the body when the bat stops, a late swing is late all over, and **the barrel is at the sim's contact point because the bat is the sim's** (invariant 4).
- **The timeline's bat was enough (Q21): no new export.** It already has the knob and the direction every 5 ms through the swing. One presentation value in the sim moved: the made-up stance (`bb_game::timeline::stance_pose`) had the knob over the batter's own spot, inside his chest; it is now 0.40 m toward the catcher, 0.10 m toward the plate and 1.30 m up, in front of his back shoulder. Game digests are unchanged (ledger, "No effect on games"). What the swings ask of the hands was measured first with a new example, `cargo run --release -p bb_game --example swing_pose_probe` (1,042 swings: the knob 0.57 m behind his spot as the swing starts, 0.34 m toward the plate at contact, the hands' arc 0.56 m in radius).
- **The stride** is on the clock, and the same at every pitch whether he swings or not, so it gives nothing away: the front foot comes up as the ball is let go and is down 16 cm nearer the pitcher 0.17 s later.
- **The bat between swings is his to carry.** The sim's bat track ends 0.12 s after contact. After a swing and a miss or a foul he brings the bat back to his shoulder over 0.8 s (unwound in front of him, not drawn back through his chest), stands at ease with it between pitches, and lifts it into the stance as the pitcher comes set. A batter who runs has dropped it: it is no longer drawn (it used to hang in the air for a quarter of a second, and through the whole of a walk).
- **A left-handed batter** is the same in a mirror, by which box he stands in (so a switch hitter is right).
- **Measured** (`-BbStagingLog`, `BB_SWING`, six balls hit in seed 7, right- and left-handed): both hands on the handle at contact, 0.0 cm off.
- **Checks added:** `-BbMotionShots` now goes on after its first three pitches to the first ball hit (`motion_4_*`: stance, stride, launch, turn, contact, through, finish, from across the plate), the first swing and miss (`motion_5_*`: the same and the bat coming back, from the mound's side) and the first pitch taken (`motion_6_*`): 38 pictures in all.
- **Cost:** 166 fps against 173 for step 3 run on the same machine minutes apart (seed 7, `-BbShots=6 -BbUncapped`, other work going on): nothing measurable.

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
- **Cost:** 150 fps against 153 with the stock clips, the same build minutes apart (seed 7, `-BbShots=6 -BbUncapped`; the machine gave lower figures all round than on the day of step 4).

## 11. Step 6 as built (2026-11-16): the slide and the dive

The last two stand-in clips are gone: a diving fielder and a sliding runner are poses in code. `BbMotion::Dive` and `BbMotion::Slide` in `BbPose.{h,cpp}`; when each happens is read from the timeline in `BbDirector` (`NoteSlides`, and the sim's own dive record).

- **The dive** is the sim's (it says who dives, which way, and when he is up; 1.5 s in all). He drives off his back foot, is laid out in the air with the glove leading, lands on his front, lies there, and gets up through his hands and knees and one knee, the ball coming in to both hands. The tipped-over leap clip and the landing clip played backwards are no longer used (they remain under `-BbStockMotions`).
- **The drawn dive starts 0.17 s before the sim's** (`DiveLeadS`). The sim has a diving fielder take the ball at the instant he leaves his feet, his body travelling after it for 0.3 s. Drawn on the sim's clock he would catch it standing, 1.4 m away. Drawn a sixth of a second early he is in the air with **the glove on the ball where and when the sim has it caught** (invariant 4), and the glove stays on that spot while the rest of him comes down behind it. A ball farther off than his arm reaches has him drawn up to 60 cm ahead of the sim's place for him until the sim's own travel catches up.
- **The slide is the client's reading of what the sim did.** The sim has no slide: a runner going on to a base he stops at runs into it at full speed and is still from the next moment. `NoteSlides` finds each such stop at second, third or home as the pitch comes up and, **if a play is being made on him there, draws it as a feet-first slide**: the ball taken within 5 m of the bag within 1.5 s of his getting there, or already held there, or he is called out there; and he came in at 5 m/s or more. Nothing feeds back to the sim.
- **The slide itself** is seven key poses on the clock from 0.42 s before he is at the bag to 1.2 s after: a last stride, sitting back with the left leg folding under, down on his left hip with the right foot out and his hands off the dirt, the foot on the bag, sitting up, up over the bag foot, standing on it. The sim's place for him is his leading foot once he is down, so **the foot reaches the bag when the sim has him there** and his body trails 60 cm behind it. Feet-first and right leg leading every time.
- **Measured** (`-BbStagingLog`, seed 7, 274 pitches at six times speed): 16 arrivals at a base to stop; 3 were walked back to; of the 13 at a run, 2 had a play on them and slid; the one diving catch had the glove 0.0 cm off the ball (96 cm before the lead).
- **Checks added:** `-BbSlideShots=<n>` and `-BbDiveShots=<n>` hold the play on five moments of the next n slides and dives and photograph each from beside (`slide_<k>_<1..5>.png`: going down, sliding, at the bag, sitting up, getting up; `dive_<k>_<1..5>.png`: driving off, in the air at the sim's catch, down, hands and knees, getting up). They work at any playback speed, so `-BbSpeed=6` finds them quickly. `-BbStagingLog` logs each arrival as `BB_SLIDE` (speed in, whether a play was on him, whether he slid) and a diving catch as `BB_CATCH ... dive=1`.
- **Cost:** 161 fps (seed 7, `-BbShots=6 -BbUncapped`) against 150 at step 5: nothing measurable.
- **For the owner to overrule if he likes:** a runner slides only when a play is being made on him; every slide is feet-first with the right leg; he sits a third of a second and stands up on the bag.
- **The dive back (2026-11-23, owner's Q39 (a)) is the sim's, not the client's reading.** The sim says who dives back to which bag, when he leaves his feet, when his hand touches and when he is up (agent-models §6.4, "Diving back"; the `Dive`, `BaseTouch` and `DiveUp` markers with the bag in `base`, and `BbDive.base` and `touch_in_s` in the FFI, data-formats §5.4). `BbMotion::DiveBack` draws it headfirst:
  - **The pose.** Off the left foot, low, with both hands reaching. Laid out and coming down chest first. On his front, with his right hand on the bag and his body behind it. Through his hands and knees, the right foot up to the bag, and standing on it over the last 0.7 s of the sim's 1.0 s down.
  - **The hand.** The figure is drawn from his touching hand, not his path point. The sim's glide is at a constant speed, so the director puts that hand on the straight line from where it was at take-off to where it was when the glide ended (`hand_from`, `hand_to`), at the glide's fraction, and on `hand_to` from then on. `FBbReach` holds the right wrist there. **The hand is on the bag when the sim has him touch it** (invariant 4). At take-off the origin is still his path point, 0.4 m behind the hand; it moves onto the hand over the first half of the glide, so the figure does not step forward.
  - **Leaving his feet on time.** The pose blends in over the 0.12 s before the sim's take-off (`DiveBackLeadS`), read from the frame that far ahead, so he is leaving his feet when the sim has him leave them. Before this he was still upright in his running pose at take-off (review P3-6).
  - **Put out in the air** (no `BaseTouch`; review P2-2): he glides as before and lies where the glide ended, his hand on the dirt short of the bag, and gets up there when the sim has him up. Before this such a dive had no record, and he was drawn upright, gliding into the tag.
  - **Not fixed:** the fielder covering the bag can stand on or in the lying runner; nothing keeps two bodies apart (TASKS §1).
  - **No slide on top.** `NoteSlides` no longer draws a feet-first slide for a runner whose stop at a bag is a dive back; the umpire's safe signal stays.
  - **The check.** `-BbDiveBackShots=<n>` photographs the next n dives back from beside, at five moments each (`diveback_<k>_<1..5>.png`: leaving his feet, in the air, the hand on the bag, down, getting up). With `-BbStagingLog`, each is logged as `BB_DIVEBACK` (play, runner, bag, and the times he leaves his feet, touches and is up).

## 12. Step 7 as built (2026-11-16): other people

Four umpires, the batting side's two base coaches and the men in both dugouts (owner's Q20). They are the same figures as the players, posed in code; `client/Source/BaseballClient/BbCrew.cpp` owns them and `BbDirector` tells it each tick where the play has got to. All decoration: the sim knows none of them, nothing feeds back, and a ball goes through an umpire.

- **Umpires** wear a pale blue shirt, charcoal trousers and a black cap (`BbLook::Umpires`, dressed as a third side); the plate umpire also wears a mask. The plate umpire stands 1.45 m behind the catcher's place, a step toward the batter's side; the first- and third-base umpires 3.5 m past their bags, just foul; the second-base umpire on the grass 7 m behind the bag toward short. They go down with their hands on their knees as the pitch is thrown (the plate umpire squats lower), watch the ball once it is hit, and turn to the play they call.
- **Signals come from the sim's own record of the pitch** (`NoteSignals`): a **strike** called or swung at (the plate umpire's arm out to his right; "ball" has no signal, as in life), a **foul** when the ball comes down (both arms up, by the plate umpire or the umpire on that line), a **fair** ball that had a call to make (below), each **out** from the sim's umpire calls (the fist; the umpire at that base, or the one nearest a caught fly). **Safe** (arms swept out flat) is the client's reading, as the slide is: a runner who stops at a base with a play made on him there and no out, or the batter reaching first as a throw is taken there. A signal lasts 1.5 s and runs on the play's clock, so a replay shows it again.
- **Fair (owner, seed 51001 plays 89 and 146).** The umpire points the arm on the infield side straight out toward fair ground, 0.15 s (`UmpireCallLagS`, the foul call's lag too) after the ball is fair under OBR "Fair Ball", read by the client from the ball's drawn path: a ball first touched by a fielder off the ground (not one taken in the air: the out is that call); a ball that has touched the ground and passes first or third (on the roll or after a bound); a ball that settles (under 0.5 m/s for 0.1 s) in front of the bag; a fly that comes down at or beyond the bag; a fly that clears the fence (`dead_reason` 2). The third-base umpire has a ball on the left-field side, the first-base umpire one on the right-field side, the plate umpire one that is fair more than 3 m short of the bag. It is signalled only when the ball is decided within `ABbDirector::FairNearLineDeg` (4°, seen from home plate) of the foul line, at the place it is decided (1.9 m inside the line at the bags, 7 m at the fence): the balls that "could have been fair or foul"; routine ground balls and flies get no signal. (First built with a 4 m distance taken over the whole roll; a bounce near the plate is always within 4 m of the lines, so routine grounders got it, review 2026-11-20.) Check: `client/check-fair-signals.sh` runs saved plays 89 and 146 (one fair signal each) and 66 and 102 (none). Pose `ESignal::Fair`: the arm raises and extends along the direction to fair ground, worked out in the umpire's own frame (`FCrewIn::Point`); he faces the ball's place as for an out. Check: `-BbCrewShots` picture 11, `crew_11_fair.png` (first-base umpire pointing in from the line); `BB_SIGNAL ... fair` logs each one.
- **Base coaches** stand in the chalked boxes in the uniform of the side at bat, in helmets, hands on hips; they bend to their knees for the pitch and watch the ball. They change sides with the half inning (at once, not walked).
- **The benches.** Each dugout seats that side's nine, each as himself (his number, his build), whenever he is not out on the field or walking off it, plus four reserves (`BbLook::BenchReserves`) in numbers the nine do not wear, and a manager standing at the end nearer the plate. Three ways of sitting, by the player's id; heads follow the ball. So the side at bat has a full bench and the side in the field a nearly empty one.
- **Measured** (`-BbStagingLog`, seed 7, 94 pitches): 60 signals: 33 strikes, 17 outs, 7 fouls, 2 fairs, 1 safe (re-measured 2026-11-20 with the fair signal; it was 59 with 2 safes before it).
- **Check added:** `-BbCrewShots` (no game played): eleven pictures `crew_<n>_<what>.png`: the plate umpire set and calling a strike; the first-base umpire calling out, safe and foul, and set; the coach at first; each bench; everyone from behind home; the first-base umpire pointing fair. `-BbStagingLog` logs each signal as `BB_SIGNAL`. `-BbNoCrew` leaves them all out.
- **Cost: about a sixth of the frame rate.** 141 fps with the crew against 170 without, the same build minutes apart (seed 7, `-BbShots=6 -BbUncapped`). It is 34 more figures of 32 pieces. Two savings are in: a man on the bench has his bones worked out only when his head turns (about once a second), and nobody under a dugout roof throws a shadow. Still above the 120 the game is watched at.
- **For the owner to overrule if he likes:** the umpires' pale blue; the plate umpire signals swinging strikes too; the nine sit in batting order with the reserves beyond them.

## 13. The base stance and swing (design, 2026-11-23; not yet built)

**Status: design, nothing here is built.** It replaces the body part of §9 (the stance, the stride, the turn of hips and shoulders after the bat, the bat between the end of the sim's track and his shoulder) step by step as §13.8 is built; until then §9 describes the game. Written from [the swing research](research/swing/swing-synthesis.md) under PLAN D21 (numbers from Fortenbaugh [4] and Welch [3]; OpenBiomechanics [6] only as a check), D22 (coil and hip turn on every pitch, held up on a take) and D23 (per-player styles are looks only). Source numbers in brackets are the synthesis's: [4] Fortenbaugh 2011, [3] Welch 1995, [2] OnBase manual, [5] Driveline clip, [7] Ochart webinar, [8] HDR video, [6] OBP (check only).

One generalized, idealized major-league right-handed hitter; a left-hander is the same in a mirror, as now. Angles: a knee or elbow is **flexion, 0° straight** (the scale [3] and [4] share). The hips' and the chest's **turn** is 0° when their line runs along the pitcher line (square to the plate, as he stands) and +90° when they face the pitcher; negative is coiled back toward the catcher ([4]'s convention, and the one `BbMotion::Swing` already uses).

### 13.1 What the batter is seen doing

**On a swing**, at a 93 mph fastball (0.42 s from release to the bat; times after the event name are from the pitcher's release):

| Event | When | What you see |
| :-- | :-- | :-- |
| **Stance** | from the pitcher coming set | Feet a little wider than his shoulders, both flat, toes at the plate; knees soft (front 36°, back 51°); a slight bend from the hips over the plate; hands just in front of the back shoulder, the bat tipped back; head level, both eyes on the pitcher. Still. |
| **Settle and load** | −0.34 s to foot off (the pitcher in his stride) | The weight drifts onto the back leg: the hips slide 6 cm toward the catcher, the knees sink a little (front to 48°, back to 56°). The back knee stays over the back foot, never outside it. The hands start to drift back. |
| **Lift** | foot off −0.07 s, top at +0.03 s (as the ball is let go) | The front foot comes up a few centimetres, the knee to 52°, nearly all his weight on the back leg. Hips and chest turn back together, about 8–10°: the start of the coil. |
| **Stride** | −0.07 s to +0.13 s | A controlled drift toward the pitcher, not a push: the front foot travels 22 cm forward and 11 cm toward the plate, toe pointing at the plate. The hips move 7 cm forward, the head does not. The hands keep going back and up, away from the chest, and the bat tips toward the pitcher behind his head. |
| **Foot down** | +0.13 s | The front toe touches first, softly ("on thin ice"), the foot closed, the front knee at 48°. Hips coiled to −18°, chest −16°: coiled but together (about 2° apart). The trunk is bent forward about 21° over the plate. Weight still mostly back. |
| **Coil held, then the hips go** | +0.13 s to the commitment | The hips reach their deepest coil (−21°) and the chest a moment later (−22°); the bat is wrapped furthest. On a slow pitch he waits here longer, weight back, foot down. |
| **Weight commitment** | 0.17 s before contact (+0.25 s on this pitch) | The front heel goes down and the weight goes onto the front foot. **The hips start to open while the chest stays closed**: the first move of the swing. The back elbow drops toward the hip. This happens on every pitch (D22); on a take it is held up (below). |
| **Hip turn and separation** | −0.10 s | Hips at +20°, chest still at +2°: the belt is open about 18° more than the chest, the most it gets. The front leg firms and starts to straighten; the weight is driven onto it (130% of body weight at −0.09 s). Hands in front of the back hip; the bat still laid back. |
| **Launch** | −0.15 s (the sim's swing start) on | The bat is the sim's from here: the hands come round on their arc and the bat releases. Hips peak at about 600°/s, the chest about 13 ms later, then the arms and the bat (the sim's). |
| **Contact** | 0 | Front leg nearly straight (18°), the back knee bent (50°) with the heel up and the back foot pivoted on its toe; hips nearly facing the pitcher (77°) with the chest square to them; **both elbows still bent** (back 81°, front 56°); the barrel below the hands (the sim's bat); the spine tilted away from the pitcher, about 20° toward the back leg; head down and back, eyes where the ball was met. |
| **Extension** | +0.02 to +0.08 s | Both arms straighten after the ball (front elbow straightest at +0.04 s, back at +0.08 s); the front knee locks out (11° at +0.07 s); the front toe opens. |
| **Through** | to +0.2 s | The bat keeps going round and rises; the elbows begin to fold; the chest turns on past the hips (chest 135°, hips 95° at +0.2 s); the head comes up to follow the ball. |
| **Finish** | +0.45 s | Bat wrapped high over the front shoulder and behind the head, two hands; chest past facing the pitcher; standing tall on a firm front leg, the back foot up on its toe. Balanced: he could hold it. |
| **Recovery** | from +0.6 s | A ball in play: he drops the bat and runs (as now). A foul or a miss: the bat comes back to his shoulder (as now, but from the finish) and the front foot steps back to its place. |

**On a take**, everything is the same up to the weight commitment, timed from the same pitch. Then the hips start to open as on a swing, and after a few hundredths of a second **he holds up**: the hips stop about square to the plate, some 25° round from their deepest coil, the chest never starts, the hands stay back with the bat laid back, the front heel is down and the weight about half on it. He watches the ball into the mitt and then relaxes back to his stance, the front foot staying where it landed until he resets.

**On a checked swing**, everything is the swing until the sim brakes the bat (it stops the hands within 0.04 s; agent-models §3.6). The body brakes with it, the same way: the hips are already open (they went first), the chest partly round, the bat stopped in front of him with the barrel still behind the plate. He holds that a moment and recovers as after a take. **A late swing is late all over and an early one early all over**: once the sim has him swing, his body is timed from the sim's bat, not from the pitch.

### 13.2 The timing model

**Three clocks, all read from the timeline when the pitch comes up**, so the pose at any moment is a function of play time and the timeline only (a held or replayed moment shows the same pose, as now):

1. **The pitcher's clock**, from the sim's release (play time 0). Everything up to foot down runs on it and is the same on every pitch: the batter cannot read the pitch before it is thrown, and [4] found stance, stride and coil the same against fastballs and changeups.
2. **The pitch's clock**, from **T_a**, the moment this pitch reaches the sim's contact plane (y = 0.35 m, the swing model's `contact_y0`), found from the ball track as the mitt's catch time is now (extrapolated through contact on a ball hit first). Every event between foot down and contact is placed by time before T_a. It depends only on the pitch, so it is the same whether he swings or not (D22).
3. **The swing's clock**, from **t_c**, the sim's own nominal contact time for this swing (the moment its hands and bat line up). It takes over from the pitch's clock once the sim's bat first moves (**t_L**: the start of the sim's load, never before his decision, `timeline.rs`), so nothing on a swing differs from a take before the decision.

**From the pitch's clock to the swing's.** Δ = t_c − T_a is the swing's timing error (late positive). The body's time is moved by Δ smoothly over the window W = max(0.10 s, |Δ|) from t_L, so the body clock runs at most twice as fast or half as slow, and only during the coil held (an early swing hurries through it, a late one waits longer). From t_L + W on, the body is on the swing's clock exactly.

**Scaling the sources' slow pitches.** [4]'s pitches took 0.52 s; [6]'s about 0.45 s; a 93 mph fastball takes about 0.42 s. The last 0.17 s, the swing itself, matches across sources and is the sim's (`T_swing` 0.150 s plus [4]'s commitment 0.02 s ahead of it), so it is **not scaled**. Everything earlier is squeezed toward the commitment by **s = (T_ref − 0.17) / (0.52 − 0.17)**, where T_ref is a reference fastball's flight (0.42 s ⇒ s = 0.714): an event [4] puts d seconds before its commitment is put s·d before the commitment of the reference pitch. That fixes the pitcher's-clock events once (foot off −0.07 s, top of the lift +0.03 s, foot down +0.13 s from release), and the coil events relative to the commitment (deepest hip coil 0.057 s before it, chest 0.011 s before it). Check: foot down is then 0.29 s before contact on the reference pitch; [6] and [2] put it 0.17–0.20 s before on slower pitches, so it is not too late. On a slower pitch the foot is down at the same moment and he waits longer coiled; on a faster one the coil events are clamped to come no earlier than 0.01 s after foot down.

**Swing timing between batters.** The sim's `T_swing` runs 0.130–0.174 s by `QuickHands`. The base keeps the body's swing times from [4] unscaled (at most ±13% off, invisible). Scaling them by `T_swing`/0.150 as the sim scales its own is a later refinement.

**Invariant 4.** The body conforms to the sim's bat and ball, never the reverse. The hands are on the drawn bat on every frame (two-bone IK, as now), and the drawn bat is the sim's from the handover (§13.4) to the end of the sim's swing track. Before the sim's bat moves, and after its track ends, the bat is presentation: the sim's stance and load samples are already made up (`stance_pose`, `LOAD_S`, "presentation only"), and after contact the bat's path is cosmetic (physics-spec §6.3).

### 13.3 Parameters

Every value is named and configurable ([E]) in the client (`BbMotion`), not a literal. "By eye" marks a value no base source gives; it is set from the picture checks. **Per player** (D23): every knob marked "yes" may later vary by player as **looks only**: none changes an outcome.

**Times** (s; release-relative "R", before the commitment "C−", or from nominal contact "τ").

| Name | Base | Sources' range | Source | Per player |
| :-- | :-- | :-- | :-- | :-- |
| `RefFlightS` T_ref, a reference fastball's release to the contact plane | 0.42 (set in step 1 to the sim's median four-seam) | 0.38–0.46 (100–85 mph); [4] 0.52 | sim | no |
| `CommitLeadS`, weight commitment before nominal contact | 0.170 | 0.15–0.17 ([4] 0.170; sim's start 0.150) | [4] | yes (timing style) |
| Pre-commitment scale s | 0.714 (derived) | — | [4], §13.2 | no |
| `SettleS`, stance drift begins | R −0.34 | [4] −1.0 s, scaled | [4] | yes |
| `FootOffS` | R −0.07 | [4] −0.621, scaled | [4] | yes |
| `LiftTopS`, front knee at its most bent | R +0.03 | [4] −0.483, scaled | [4] | yes |
| `FootDownS`, front toe down | R +0.13 | [4] −0.340, scaled (0.29 s before contact); check ≤ −0.20 [6,2] | [4] | yes |
| Deepest hip coil | C −0.057 | [4] −0.250, scaled; tee −0.350 [3] | [4] | yes |
| Hands furthest back (load pose L) | C −0.052 | [4] −0.243 (bat's least azimuth), scaled | [4] | yes |
| Deepest chest coil | C −0.011 | [4] −0.186, scaled; tee −0.265 [3] | [4] | yes |
| Back knee straightest | C −0.036 | [4] −0.220, scaled | [4] | no |
| Peak separation | τ −0.103 | −0.103 [4]; check −0.123 [6] | [4] | no |
| Front leg firm (most force) | τ −0.093 | [4] | [4] | no |
| Hips fastest (check) | τ −0.070 | −0.070 [4], −0.075 [3] | [4,3] | no |
| Chest fastest (check) | τ −0.057 | −0.057 [4], −0.065 [3] | [4,3] | no |
| Front elbow straightest | τ +0.043 | [4] | [4] | no |
| Front knee straightest | τ +0.073 | [4] | [4] | no |
| Back elbow straightest | τ +0.083 | [4] | [4] | no |
| `ThroughS` | τ +0.20 | [4] (ends there) | [4] | no |
| `FinishS` | τ +0.45 | none measured ([6] to +0.43, OBP) | by eye | yes |
| `RecoverS`, recovery begins | τ +0.60 | none | by eye | yes |
| `HandoverS`, the bat from the client's to the sim's (§13.4) | 0.05 from t_L, at least 0.03 into the swing | sim's `LOAD_S` 0.05 | [E] | no |
| `AnchorWindowS` W | max(0.10, \|Δ\|) | — | [E] | no |
| `TakeShowS`, a take's hips follow the swing's for | 0.03 | none | by eye | yes |
| `HoldBrakeS`, a take's or checked swing's body brakes over | 0.040 | sim's `BRAKE_S` 0.040 | sim | no |
| `TakeRelaxS`, back to the stance after the catch | +0.2 then over 0.6 | none | by eye | yes |
| `BatHandoffS`, the follow-through takes the bat (§13.5) | contact +0.03 (Proposal 1); +0.12 as now | sim track +0.12 | [E] | no |

**Angles** (degrees; radians in code).

| Name | Base | Sources' range | Source | Per player |
| :-- | :-- | :-- | :-- | :-- |
| Hips' turn: stance / foot down / deepest coil / commitment | −10 / −18 / −21 / −13 | coil −21 [4], −28 tee [3] | [4] | yes (coil size) |
| Hips' turn: τ −0.093 / contact / +0.2 / finish | +20 / +77 / +95 / +100 | contact 61–79 by location [4] | [4]; finish by eye | finish yes |
| Chest's turn: stance / foot down / deepest coil / τ −0.093 | −9 / −16 / −22 / +2 | coil −22 [4], −52 tee [3], ~8 beyond the stance [7] | [4] | yes (coil size) |
| Chest's turn: contact / +0.2 / finish | +75 / +135 / +150 | [4]; finish none | [4]; finish by eye | finish yes |
| Separation (chest − hips), check | ≈ 0 at foot down; −18 at τ −0.103; −2 at contact | 18 [4]; check 21 [6], 22 [7] | [4] | — |
| Hips' tilt: front hip rising, τ −0.093 → contact | 14 → 27 | [4] (obliquity; the side it rises on is inferred) | [4] | no |
| Front knee: stance / foot off / lift top / foot down / commitment | 36 / 48 / 52 / 48 / 48 | lift 52 [4] (event-averaged, a floor); check 83 [6] | [4] | yes (lift) |
| Front knee: τ −0.093 / contact / +0.073 / finish | 40 / 18 / 11 / 15 | contact 15 [3]–18 [4]; check 14 [6] | [4,3]; finish by eye | no |
| Back knee: stance / foot off / foot down / straightest / commitment / contact | 51 / 56 / 50 / 44 / 47 / 50 | contact 45 [3]–54 [4]; check 71 [6] | [4,3] | yes (crouch) |
| Front toe, closed from the pitcher line: stance / foot down / commitment / contact / +0.06 | 69 / 63 / 56 / 49 / 36 | 61 at foot down [3] | [4] | yes |
| Stride direction, closed | from 0.11 m (below); 8° toe to toe | 12° [3] | [4] | yes |
| Trunk bent forward over the plate: stance / foot off / foot down / contact | 15 / 21 / 21 / 10 | 21 at foot off [3]; ~25 at landing [7]; contact −9 [3] (global frame) to +16 at +0.03 [4] | [3]; stance and contact by eye | yes (posture) |
| Trunk tilted toward the back leg: foot off / contact | −6 (toward the pitcher) / 20 | contact 20 [3]; check 28–32 [6,7] | [3] | no |
| Head tilt toward the back shoulder: commitment / contact | 15 / 31 | [4] | [4] | no |
| Head turn at contact, check (eyes on the ball give it) | 47 from the pitcher | 38–55 by location [4] | [4] | — |
| Elbows at contact, check: back / front | 81 / 56 | back 66–89 by location [4]; check 79 / 53 [6] | [4] | — |
| Elbows after: front at +0.043 / back at +0.083 | 42 / 37 | [4] | [4] | no |
| Bat in the load: elevation / wrap | 44 above level / tip toward the pitcher behind the head | [4] (elevation, least azimuth −232°) | [4]; checked against L | yes (hand load) |
| Follow-through bat: elevation at +0.15 / turned by +0.2 | +28 / 246 from contact | [4] | [4] | yes |
| Follow-through bat: turning rate at contact / +0.2 | 2233 / 827°/s (39 / 14 rad/s) | [4] | [4] | no |

**Distances** (m; H = the figure's height).

| Name | Base | Sources' range | Source | Per player |
| :-- | :-- | :-- | :-- | :-- |
| `StanceWidth`, ankle to ankle along the pitcher line | 0.30 H (0.55 m at 1.84 m; now 0.54) | [6] 0.68 (OBP only); "bat length" 0.86 [8] is wider than the stride at foot down, so not used | by eye | yes |
| Stride at foot down, toe to toe | 0.42 H (0.77 m) | [4] 0.42 H; [3] 0.85 m (~0.46 H) | [4] | yes |
| Front foot's travel | 0.12 H (0.22 m) forward, 0.11 m toward the plate | check 0.23 m [6] | [4] | yes |
| `LiftHeight`, front ankle | 0.08 | [6] 0.22 (OBP only); now 0.09 | by eye | yes (leg kick or toe tap) |
| Hips' travel: back in the settle / stride / to commitment / to τ −0.093 | −0.06 / +0.07 / +0.11 / +0.05 (+0.17 from the stance) | [4] | [4] | yes (load size) |
| Hips' rise, τ −0.093 → contact | +0.05 | [4]; [6]'s drop is OBP only | [4] | no |
| Head travel toward the pitcher after foot down, check | ≤ 0.08 | "quiet head" [2,7] | by eye | — |
| Stance hands: knob from his spot | 0.40 toward the catcher, 0.10 toward the plate, 1.30 up | (today's `stance_pose`) | by eye | yes (hand set) |
| Load pose L: knob and bat direction | the mean of the sim's swing-start pose (now ~0.57 m toward the catcher), measured in step 1 | — | sim | yes (hand load) |

**Speeds** (checks, not inputs: the keys are angles at times, joined by a curve that keeps its speed through each key, as the delivery is).

| Check | Range | Source |
| :-- | :-- | :-- |
| Hips' peak turning rate | 580–710°/s (10.1–12.4 rad/s), at τ −0.08 to −0.06 | [4,3] |
| Chest's peak | 770–940°/s (13.4–16.4 rad/s), 4–13 ms after the hips | [4,3] |
| Order of peaks | hips, chest, hands, bat | all |
| Hands before the sim's swing | ≤ 1.5 m/s, no step over 2 cm between frames | [E] |
| Body clock rate in the anchor window | 0.5–2 | [E] |

**Weight** is not a pose field. It shows through the hips' travel and the knees, which come from [4]'s measured positions; the force figures (rear leg 85–90% at foot off, the front foot 50% at the commitment, 130% at τ −0.093, 84% at contact [3]) are what a reviewer checks the pictures against, not something solved.

### 13.4 How the pose is built

Two layers, as now (a stick figure solved by `FBbSkeleton` with hands placed by `FBbReach`), with the job split differently:

- **The body is on the clock.** Legs, feet, hips (place, turn, tilt), the trunk's turn and bend, the head's tilt and the weight are key poses over the body's time (§13.2), joined by a speed-keeping curve. The knees are set by solving the hips' height and tilt so that both legs show their key flexion for the figure's own thigh and shin lengths. The eyes stay on the ball, as now.
- **The hands are on the bat.** Both hands are on the handle every frame (grip as now). The arms run from the shoulders the body put there to the hands the bat put there. The elbows' bend is therefore an outcome, and a check: bent at contact (81° and 56°), straight just after.
- **Where the two meet.** The chest's turn is the clock's plus a correction toward where the hands are, of at most ±15° [E], so an arm is never stretched past its length or folded flat; `FBbReach` still moves the body up to 24 cm as a last resort. The correction is logged: a large one means the keys and the sim's bat disagree, and it is a defect to fix in the keys, not to hide.
- **What replaces today's drive.** The hips and shoulders no longer turn by how far the hands have come round (`TurnDeg`); the stride is no longer a smoothstep from −0.2 s to +0.17 s. The swing weight that blends the swing in is no longer needed: the stance is the clock's first key.

**The bat, by stretch of time.**

1. **Before the sim's bat moves** (all of a take; a swing until t_L): **the client's bat.** The stance hold, then the load: from foot off the hands drift back and up and the bat tips over toward the pitcher, reaching the load pose L at C −0.052 s, and stay there. Identical on swings and takes. The sim's own stance and load samples (`stance_pose`, `LOAD_S`) are no longer drawn; the sim is unchanged (§13.6).
2. **The handover** (t_L to t_L + 0.05 s): the drawn bat is a blend of the client's and a target, the weight rising smoothly 0 → 1. The target is the sim's swing-start pose until the sim's swing starts (t_s), and the sim's bat from then; so the blend has no backward wiggle through the sim's own straight-line load, and nothing jumps. The client finds t_s as the end of the sim's load, whose samples lie on a straight line at one speed; the swing's first sample leaves that line. When the sim cut the load short (the decision came less than 0.05 s before t_s), the blend ends 0.03 s into the swing at the latest, still 0.12 s or more before contact: for those few hundredths the drawn bat converges on the sim's instead of being on it. Because L is the sim's mean start pose, the distance to close is only the spread of start poses (measured in step 1).
3. **The swing** (to the end of the sim's track): the sim's bat exactly. t_c, for the swing's clock, is the time of the bat's fastest turning less 0.010 s (the template's bat peak, `bat_peak`); a checked swing never reaches its peak, and uses t_s + 0.150 s.
4. **After the sim's track**: the client's follow-through (§13.5), then the bat back to his shoulder or dropped, as now.

**A late or early swing.** The body changes from the pitch's clock to the swing's over the anchor window from t_L (§13.2). An early swing (fooled, out in front) fires the hips early; a late one holds the coil longer and is late all over. The hands are on the bat throughout.

**A take.** From the commitment the body follows the swing's keys for `TakeShowS` (0.03 s), then its clock brakes to a stop over `HoldBrakeS` (0.04 s, the same law as the sim's check-swing brake: speed falling evenly to zero). It holds there, the bat at L, until `TakeRelaxS` after the catch, and then eases back to the stance pose over 0.6 s.

**A checked swing.** The sim's bat brakes at t_brake (its last movement less 0.04 s, read from the track). The body's clock brakes over the same 0.04 s from the same moment, holds, and recovers as after a take. The hips, already open, stay open; nothing is drawn later than the bat.

**No jump anywhere.** Every input above is continuous in play time: clocks are warped, not reset; the bat is blended, not switched; a hold brakes over 0.04 s. The check in §13.7 measures it.

### 13.5 The bat after the sim's track ends

**What happens now.** The sim draws the bat until 0.12 s after its nominal contact, then the client brings it back to his shoulder. The sim's template slows the bat after contact as a bell curve about the bat's peak (width 0.055 s; agent-models §3.6), so it turns at 95% of its contact rate at +0.03 s, 67% at +0.06 s, 27% at +0.10 s and **14% at +0.12 s**: by the end of the track it has turned about 160° past contact and nearly stopped. [4]'s hitters were still turning the bat at 37% of the contact rate at +0.2 s, 246° past contact. So the sim's last 0.09 s is a bat braking far harder than a hitter's, and any follow-through started at +0.12 s starts from a bat that has nearly stopped. The template's shape after contact never mattered to the game (contact is decided within ±0.025 s of the planned time, and after contact the bat is cosmetic, physics-spec §6.3).

**The two options asked about, and a third.**

| | (A) Sim track to about +0.4 s | (B) Client from +0.12 s (as asked) | (C) Client from contact + 0.03 s |
| :-- | :-- | :-- | :-- |
| What it takes | A follow-through model written in the sim (the template stops by +0.2 s, so extending it gives a bat frozen in its plane); `timeline.rs` presentation code | Client only | The sim's bat track ends 0.03 s after contact (after a miss, after its nominal contact); one constant in `timeline.rs`. The client draws the rest |
| Look | Good if the sim's follow-through is good; the body still has to follow it | **Poor**: the bat brakes to 14% before the client takes over, then wraps from almost a standstill | Good: the client takes a bat still at 95% of its speed and slows it as [4] measured |
| Schema, FFI | None (more samples: about 60 more per swing, ~2 KB) | None | None (fewer samples) |
| Game digests | Unchanged (the timeline is not in the event log, as with `stance_pose`, ledger 11-16) | Unchanged | Unchanged |
| Ledger | One "No effect on games" line | None | One "No effect on games" line |
| Replay contract | Same build + seed ⇒ same timeline still | Unchanged | Same |
| Cost elsewhere | Puts presentation motion design in the sim, which is for physics; every future change to a finish is a sim change | None | The client must hand over with the bat's speed and direction matched (it already reads the track ahead) |

**Recommendation: (C).** The client carries the bat after the sim's physical part, from contact + 0.03 s, continuing the sim's bat with its position, direction and turning rate matched and slowing as [4] measured (to 827°/s at +0.2 s, elevation rising to +28° by +0.15 s), wrapping over the front shoulder to the finish (by eye). Per-player finishes (D23) then stay client work. If the owner prefers not to touch the sim at all, (B) can be built first and switched to (C) with one constant, but it will look worse.

> **Proposal 1 (for the Coordinator to number and raise).** *What the game does now:* after the ball is hit or missed, the sim keeps drawing the bat for 0.12 s more, and in that time it slows to a seventh of its speed, far quicker than a real hitter's bat, so the follow-through would start from a bat that has nearly stopped. *What is proposed:* the sim's bat path ends 0.03 s after contact, just after the moment that decides the hit, and the client draws the rest of the swing and the finish itself from the measured swings. Nothing in any game changes (same digests, one "no effect" ledger line). *Alternatives:* keep the sim's path to +0.12 s (looks worse); or write a follow-through into the sim (puts presentation in the physics code). *Recommended:* end the sim's path at +0.03 s.

### 13.6 What the sim must provide

**Nothing new for §13.1–§13.4.** The client finds everything from the timeline it already samples: the pitch's arrival T_a from the ball track, the load start t_L from the bat's first movement, the swing start t_s from where the load's straight line ends, the swing's nominal contact t_c from the bat's fastest turning, a checked swing's brake from where the track stops. Step 1 measures that these inferences are exact enough (t_s within 5 ms, t_c within 2 ms on the sim's own records) before anything relies on them; if one is not, the fallback is a read-only export under Q21 (the swing start and nominal contact in `BbPlayInfo`), digests unchanged.

**Only Proposal 1 touches the sim**: the bat track's end, `t1 = sw.t_c + 0.12` in `bb_game::timeline::encode`, becomes contact (or nominal contact) + 0.03 s, presentation only.

Two things found in the timeline that this design does not use, for the record: the `PlateCross` marker is written at a fixed 0.42 s whatever the pitch, and `CheckSwingBrake` exists in the schema but is never written. Neither is read by the client.

### 13.7 Checks

All headless, as in §8–§12.

**Pictures.** `-BbMotionShots`, strips 4 (a ball hit), 5 (a swing and miss) and 6 (a take), re-keyed to the events of §13.1 and shot from across the plate: stance, foot off, top of the lift, foot down, deepest coil, commitment, peak separation, contact, extension (+0.08), through (+0.2), finish (+0.45), recovery (+1.0). In each, a reviewer should see:

- **Foot down:** front toe touching first, foot closed (toes toward the plate), front knee bent, weight back, hips and shoulders coiled together, hands back near the back shoulder away from the chest, bat tipped toward the pitcher.
- **Commitment and separation:** the belt buckle turning toward the pitcher while the letters still face the plate; the back elbow dropping; the front heel down.
- **Contact:** front leg nearly straight, back knee bent with the heel up, hips almost facing the pitcher with the chest square to them, both elbows bent, barrel below the hands, spine tilted away from the pitcher, head down and back.
- **Extension and through:** both arms straight just past contact, then folding; the bat rising round.
- **Finish:** bat high over the front shoulder, two hands, chest past the pitcher, tall on the front leg, back toe down.
- **Take (strip 6):** the same as strip 4 to the commitment; then hips a little open and stopped, chest closed, bat back, eyes on the ball into the mitt.

New: **strip 9, a checked swing** (the first one the run finds), at the same moments: hips open, bat stopped in front of him with the barrel behind the plate, no part of him still moving after the bat stops. **`-BbSwingPairShots`**: a swing and a take of nearly the same flight time (within 0.01 s), shot side by side at the same moments up to the swing's t_L; the two pictures of each pair must look the same (D22).

**Numbers** (`-BbStagingLog`, a new `BB_SWINGPOSE` line at each named moment with the joint angles, the hips' and chest's turn and rate, the chest's correction, the hands' distance from the handle and the drawn knob's speed; and a script beside `client/check-fair-signals.sh` that runs seeds 7 and 6 and fails on any of these):

- At contact: front knee 10–25°, back knee 40–60°, back elbow 66–95°, front elbow 45–70°, hips 60–90°, chest minus hips within ±8°.
- Peaks: hips 500–800°/s, chest 650–1000°/s, the chest's peak 0–25 ms after the hips'.
- Foot down at least 0.20 s before contact on every swing.
- Hands on the handle (0.0 cm) from the handover's end to the end of the sim's track; the drawn knob never faster than 1.5 m/s before t_L and never stepping more than 2 cm between frames outside the sim's swing.
- **D22**: for every pitch of the run, the pose worked out with the swing withheld (as a take) and with it equal to 0.1 cm at every joint for all times before t_L.
- A take's hips never more than 35° round from their deepest coil; a checked swing's body still within 0.02 s after the bat stops.
- At the follow-through handover the bat's position and direction continuous (step under 2 cm and 2° a frame) and its turning rate never rising afterwards.
- The chest's correction under 15° on 95% of frames of a swing (a larger one is a key to fix).
- Inferred times against the sim's records (step 1's probe): t_s within 5 ms, t_c within 2 ms.

**Sim side**, only if Proposal 1 is taken: a `bb_game` test that the bat track ends at contact + 0.03 s; `cargo run --release -p bb_game --example digest` identical before and after; one ledger line.

**Cost:** frame rate from `-BbShots=6 -BbUncapped` against the build before, as every step reports.

### 13.8 Build steps

Each a worker-sized client job with its own picture strip and numbers from §13.7, most visible first. Step 1 starts with the measurements the rest rely on.

1. **The body clock, the legs and the stride.** Measure first: T_ref (the sim's median four-seam flight to the contact plane), the swing-start pose's mean and spread, how often and by how much the sim's load is cut, and the accuracy of the inferred t_s and t_c (extend `swing_pose_probe`). Then the three clocks and the anchor window; the stance, settle, lift, stride and foot down on the pitcher's clock; the knees, hips' travel and rise, front toe and back heel on the body clock. *The owner sees:* a real stride that lands soft and closed, the weight going back then forward, a front leg that straightens into contact and a back heel that comes up.
2. **The coil and the hip turn, and the take.** Hips' and chest's turn and the trunk's bend on the clock (D22), the head's tilt, the chest's correction toward the hands; the take's hold-up and relax. *Sees:* the body coils, the hips go before the hands and the chest catches up at contact; on a take the hips start and stop.
3. **The follow-through and finish.** Per Proposal 1 (or from +0.12 s if the owner keeps the sim's path): the bat continued and slowed as [4] measured, the elbows straightening then folding, the chest turning on, the finish held, then the bat back to the shoulder or dropped. *Sees:* the swing no longer stops dead after the ball; a full two-handed finish.
4. **The hands' load and the handover.** The client's bat before the sim's swing: stance, load to L, the handover blend. *Sees:* the hands drift back and the bat tips over through the stride, the same on every pitch; no jump from the stance into the swing.
5. **Checked, late and early swings, and the checks still missing.** The checked swing's brake and hold; strip 9; `-BbSwingPairShots`; the D22 comparison over whole runs. *Sees:* a checked swing that stops with the hips open; a fooled hitter out in front with his whole body.

Each step updates §9 and this section's status to say what is built, ticks nothing in observations.md until the owner has seen it move, and adds no ledger row (client only; Proposal 1 is one "No effect on games" line).

### 13.9 Not in the base, and open points

**Q37 decided (a), PLAN D23 (owner, 2026-11-23):** each player's own stance, swing style and follow-through (the owner's examples: Pete Rose, Darryl Strawberry, Gary Carter, all variations of core stances, none quite alike) are **looks only**: client-side variety that changes no outcome. So every "per player" knob in §13.3 is looks only, and the sim's swing (timing, extension, plane) stays the sim's from the batter's attributes alone. Tying stances to attributes can be reopened when batter ratings exist.

**Left to per-player stances (looks only, after the base):**
- The stance: width, open or closed, crouch, knee bend, how upright, hand height and bat angle, a waggle.
- The load: how far the hips go back, how high the leg lifts (a kick against a toe tap), how far the hands load and the bat wraps, when it all starts (`FootOffS`, `SettleS`).
- The coil's size (chest 8–26° in [6]'s spread; tee hitters more [3]).
- The finish: one or two hands, high or level, how far round; a bat flip.
- The body's swing times scaled by the batter's `T_swing` (§13.2).
- The hips and chest at contact by pitch location ([4]: hips 79° on a high inside pitch, 61° low and away). The base uses the middle pitch's values everywhere and lets the chest's correction follow the hands.

**Open points the sources do not settle, and what the base chooses:**
- **The finish** (none measured; [4] stops at +0.2 s): bat wrapped high over the front shoulder and behind the head, two hands, held 0.15 s, chosen from the Driveline clip [5] and common pictures. Set by eye in step 3.
- **One-hand release:** not in the base. Two hands throughout, as [4] analysed and [5] shows. A later per-player look.
- **Leaving the box:** on a ball in play, the bat drops from both hands when the sim's batter-runner first moves, or at the finish if that is later, and the pose gives way to the running clip over 0.3 s as now; the bat is not drawn after (as now). No measured source; the base keeps today's behaviour.
- **Balance after the finish:** standing on the front leg with the back toe down; no source.
- **Stance width:** no base source ([6] is a check only; HDR's "bat length" is wider than [4]'s stride at foot down); 0.30 H by eye, close to today's.
- **Leg-lift height:** OBP only; 0.08 m by eye, close to today's.
- **The trunk's bend at contact:** [3] and [4] decompose it in different frames (−9° in [3]'s global frame, +16° at +0.03 s in [4]'s); 10° forward plus 20° toward the back leg is chosen to sit between them, and checked by eye.
- **[4]'s own angles and speeds disagree a little:** its hips turn 58° in the last 93 ms (624°/s on average) against a stated peak of 581°/s, and its chest 73° (785°/s) against 766°/s; the two come from different decompositions. The keys are [4]'s angles; the speed checks are the wider [4]–[3] ranges.
- **Bunts and hit-by-pitch evasion** (agent-models §3.7) are separate motions, not this one.

## 14. Records

- PLAN D13 (revised 2026-11-15) covers this stage.
