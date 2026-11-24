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

*To be replaced in parts by [§13](#13-the-base-stance-and-swing-design-2026-11-23-not-yet-built) as its build steps land; until then this is what the game does.* **Step 1 of §13 is built (2026-11-24): the bat before the swing is no longer the sim's stance and load.** Until the sim's bat first moves the client draws its own: still in the stance, then from 0.325 s before release the hands drift back to the load pose L (the sim's mean swing-start pose) and wait there, the same on every pitch whether he swings or not; from the moment the sim's bat first moves it blends into the sim's over 0.05 s, and the sim's bat is drawn from there to the end of its track. (`BbMotion::LoadBat`, `HandoverWeight`; `ABbDirector::DrawnBat`; a pitch taken eases back to the stance 0.2 s after the catch.) The text below still describes the body, which step 1 does not touch. `-BbBatLegacy` draws the old bat for comparison.

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

**Status (2026-11-24): design; build step 1 is built** (export E1, the bat before the swing and the measurements, §13.8, results §13.10); steps 2–5 are not. It replaces the body part of §9 (the stance, the stride, the turn of hips and shoulders after the bat, the bat between the end of the sim's track and his shoulder) step by step as §13.8 is built; until then §9 describes the game. Written from [the swing research](research/swing/swing-synthesis.md) under PLAN D21 (numbers from Fortenbaugh [4] and Welch [3]; OpenBiomechanics [6] only as a check), D22 (coil and hip turn on every pitch, held up on a take) and D23 (per-player styles are looks only). Source numbers in brackets are the synthesis's: [4] Fortenbaugh 2011, [3] Welch 1995, [2] OnBase manual, [5] Driveline clip, [7] Ochart webinar, [8] HDR video, [6] OBP (check only). Revised after [the review](reviews/2026-11-23-sonnet-base-swing-spec-review.md) (2026-11-23): fooled and late swings follow [4] §4.3, the warp is bounded, the contact checks depend on location, Q43 and export E1 are stated in full.

One generalized, idealized major-league right-handed hitter; a left-hander is the same in a mirror, as now. Angles: a knee or elbow is **flexion, 0° straight** (the scale [3] and [4] share). The hips' and the chest's **turn** is 0° when their line runs along the pitcher line (square to the plate, as he stands) and +90° when they face the pitcher; negative is coiled back toward the catcher ([4]'s convention, and the one `BbMotion::Swing` already uses).

### 13.1 What the batter is seen doing

**On a swing**, at a 93 mph fastball met on time in the middle of the zone (0.407 s from release to the bat, the sim's median four-seam, measured in step 1; times after the event name are from the pitcher's release):

| Event | When | What you see |
| :-- | :-- | :-- |
| **Stance** | from the pitcher coming set | Feet a little wider than his shoulders, both flat, toes at the plate; knees soft (front 36°, back 51°); a slight bend from the hips over the plate; hands just in front of the back shoulder, the bat tipped back; head level, both eyes on the pitcher. Still. |
| **Settle and load** | −0.33 s to foot off (the pitcher in his stride) | The weight drifts onto the back leg: the hips slide 6 cm toward the catcher, the knees sink a little (front to 48°, back to 56°). The back knee stays over the back foot, never outside it. The hands start to drift back. |
| **Lift** | foot off −0.07 s, top at +0.025 s (as the ball is let go) | The front foot comes up about 8 cm, the thigh raised a little and the knee bent, nearly all his weight on the back leg. Hips and chest turn back together, about 8–10°: the start of the coil. |
| **Stride** | −0.07 s to +0.12 s | A controlled drift toward the pitcher, not a push: the front foot travels about 22 cm forward and 11 cm toward the plate, toe pointing at the plate. The hips move 7 cm forward, the head does not. The hands keep going back and up, away from the chest, and the bat tips toward the pitcher behind his head. |
| **Foot down** | +0.12 s | The front toe touches first, softly ("on thin ice"), the foot closed, the front knee at 48°. Hips coiled to −18°, chest −16°: coiled but together (about 2° apart). The trunk is bent forward over the plate. Weight still mostly back. |
| **Loaded and coiled** | to about +0.23 s | The hips reach their deepest coil (−21°), the hands their furthest back with the bat wrapped, the chest its deepest (−22°). He holds this until the pitch's timing calls for the swing: on a slow pitch longer, foot down, weight back. |
| **Weight commitment** | 0.17 s before contact (+0.237 s on this pitch) | The front heel goes down and the weight goes onto the front foot. **The hips start to open while the chest stays closed**: the first move of the swing. The back elbow drops toward the hip. This happens on every pitch (D22); on a take it is held up (below). |
| **Hip turn and separation** | −0.10 s | Hips at +20°, chest still at +2°: the belt is open about 18° more than the chest, the most it gets. The front leg firms and starts to straighten; the weight is driven onto it (130% of body weight at −0.09 s). Hands in front of the back hip; the bat still laid back. |
| **Launch** | −0.15 s (the sim's swing start) on | The bat is the sim's from here: the hands come round on their arc and the bat releases. Hips peak at about 600°/s, the chest about 13 ms later, then the arms and the bat (the sim's). |
| **Contact** | 0 | Front leg nearly straight (18°), the back knee bent (about 50°) with the heel up and the back foot pivoted on its toe; hips nearly facing the pitcher (77°) with the chest square to them; **both elbows still bent** (back 81°, front 56°); the barrel below the hands (the sim's bat); the spine tilted away from the pitcher, about 20° toward the back leg; head down and back, eyes where the ball was met. Away from the middle of the zone the hips and the back elbow differ by location (§13.7). |
| **Extension** | +0.02 to +0.08 s | Both arms straighten after the ball (front elbow straightest at +0.04 s, back at +0.08 s); the front knee locks out (11° at +0.07 s); the front toe opens. |
| **Through** | to +0.2 s | The bat keeps going round and rises; the elbows begin to fold; the chest turns on past the hips (chest 135°, hips 95° at +0.2 s); the head comes up to follow the ball. |
| **Finish** | +0.45 s | Bat wrapped high over the front shoulder and behind the head, two hands; chest past facing the pitcher; standing tall on a firm front leg, the back foot up on its toe. Balanced: he could hold it. |
| **Recovery** | from +0.6 s | A ball in play: he drops the bat and runs (as now). A foul or a miss: the bat comes back to his shoulder (as now, but from the finish) and the front foot steps back to its place. |

**On a take**, everything is the same up to the weight commitment, timed from the same pitch. Then the hips start to open as on a swing, and after a few hundredths of a second **he holds up**: the hips stop about square to the plate, some 25° round from their deepest coil, the chest never starts, the hands stay back with the bat laid back, the front heel is down and the weight about half on it. He watches the ball into the mitt and then relaxes his body back toward the stance (hips, trunk, knees, hands); **his feet do not move** until he resets between pitches, when the front foot steps back to its place.

**Out in front of a changeup** (fooled: he read fastball). His stride and coil are the same as on every pitch. His legs and hips then fire on the fastball's time, early for this pitch; his hands try to wait for it. So against the ball the hips are early by nearly three times what the bat is: the hips open well before the hands come, the chest follows the hips, and at contact the hips are round, the chest past square, **the back elbow more bent** (67° on a changeup hit, 51° on a miss, against 81°) and the barrel out in front. This is [4] §4.3 (Tables 40–41): changeup swings had the pelvis peak 17 ms (hit) and 38 ms (miss) earlier against contact than fastball swings, the bat only 6 and 14 ms earlier.

**Late on a fastball.** His stride and coil are on time, as on every pitch: he had not decided yet. A little late (up to about 0.03 s), he simply waits a moment longer coiled and then everything goes late together with the bat. Later than that, his hips have already made their first move on time, exactly as a take's do, and past about 0.06 s they have begun to hold up as on a take; then the decision comes, the hips go again and everything after goes late with the bat: a small hitch, then a swing behind the ball. Very late (more than about 0.075 s), his hips cannot wait any longer: they are round before his hands come, the hands drag, the elbows are more bent and the barrel is behind at contact. No source measures a late swing; this is reasoned from D22 and the sim (§13.2).

**On a checked swing**, everything is the swing until the sim brakes the bat (it stops the hands within 0.04 s; agent-models §3.6). The body brakes with it, the same way: the hips are already open (they went first), the chest partly round, the bat stopped in front of him with the barrel still behind the plate. He holds that a moment and recovers as after a take.

### 13.2 The timing model

**Three clocks, all read from the timeline when the pitch comes up**, so the pose at any moment is a function of play time and the timeline only (a held or replayed moment shows the same pose, as now):

1. **The pitcher's clock**, from the sim's release (play time 0). The stance, settle, lift, stride, foot down, the deepest coil and the hands' load run on it and are the same on every pitch: the batter cannot read the pitch before it is thrown, and [4] found stance, stride and coil the same against fastballs and changeups (synthesis §4: the same load for a high fastball and a slow curve).
2. **The pitch's clock**, from **T_a**, the moment this pitch reaches the sim's contact plane (y = 0.35 m, the swing model's `contact_y0`), found from the ball track as the mitt's catch time is now (extrapolated from the last 30 ms before contact on a ball hit first; the error is under 0.1 ms, measured in step 1, §13.10). The weight commitment and everything after it are placed by time before T_a. It depends only on the pitch, so it is the same whether he swings or not (D22).
3. **The swing's clock**, from the sim's swing: its start **t_s** and its nominal contact **t_c** (the moment the template's hands and bat line up), and from them `T_swing` = t_c − t_s. It can act only from **t_L**, the moment the sim's bat first moves (the start of the sim's load, never before his decision, `timeline.rs`), so nothing on a swing differs from a take before the decision.

**Keys on the pitcher's clock with a hold.** The deepest hip coil, the hands' load pose L and the deepest chest coil are reached at their reference times on the pitcher's clock (R +0.183, +0.188, +0.226 s) or earlier if the pitch's commitment comes sooner (no later than C −0.054, −0.049, −0.011 s), and held until the commitment's own keys take over. So on a slow curve he loads, coils and waits; he does not drift into the load.

**What the body times its swing from.** The legs, the hips, the weight and the chest are keyed to a time **A**, the moment the body treats as contact: their keys after the commitment are at fixed times before A. Before t_L, and on every take, A = T_a. On a swing the body moves to a target **A\*** chosen from the swing's timing error **Δ = t_c − T_a** (positive late):

- **Early (Δ < 0): the hips go earlier than the bat** ([4] §4.3). A\* = T_a + κ·Δ with **κ = 2.7** [E, from [4] Table 41: pelvis peak 17 and 38 ms early against bat peaks 6 and 14 ms early, ratios 2.8 and 2.7; the lead knee's extension gives 2.8 and 2.6]. So the body's lead over the bat is (κ − 1)·|Δ|. It is capped at **e_max = 0.025 s** ([4]'s largest group, changeup misses: 24 ms), reached at |Δ| = 0.015 s; beyond it A\* = t_c − 0.025: [4] has no data past that, so the body keeps that lead and goes early with the bat.
- **Late (Δ ≥ 0): the body goes late with the bat**, A\* = t_c (κ = 1 for late swings, by reasoning: in the sim a swing is late because the decision or the start came late, and everything after the decision follows it; no source measures a late swing).
- **The arms follow the sim's bat.** They must: the hands are on it. The difference between the body's timing and the bat's is taken up by the elbows (an outcome of where the shoulders and the hands are) and by the chest's correction toward the hands (§13.4).
- **The swing's own length.** Between the commitment and contact the body's keys are spread over `T_swing` + 0.020 s instead of 0.170 s, so the hips' first move stays 0.02 s ahead of the hands' start for every batter (the sim's `T_swing` runs 0.130–0.174 s by `QuickHands`; with an unscaled 0.17 s the slowest batters' hands would start before their hips). On a take `T_swing` is unknown and 0.150 s is used.

**Moving the body from A = T_a to A\*** (the warp). The body's own time τ_b(t) (its time to its contact) runs at a rate r(t). Before t_L, r = 1 (or slowing to 0 in a take's hold-up, below). From t_L:

- r ramps over R = 0.02 s from its value at t_L, r₀ (1, or less if a hold-up had begun), to a constant r\*, holds r\*, and ramps back to 1 over the last R of a window of length W.
- The window must bring the body to its target: τ_b(t_L + W) = t_L + W − A\*. The time the body covers is ∫r = r\*·(W − R) + (r₀ + 1)·R/2, so
  **r\* = [t_L + W − A\* − τ_b(t_L) − (r₀ + 1)·R/2] / (W − R).**
- W is the shortest window of at least 0.06 s that gives **0.5 ≤ r\* ≤ 2**, and never longer than **W_max = t_c − 0.03 − t_L**, so the warp always ends at least 0.03 s before contact. W_max ≥ 0.10 s always, because t_L ≤ t_s ≤ t_c − 0.130.
- **The bound.** r(t) only ever moves linearly between r₀, r\* and 1, so it stays between the smallest and largest of them: **0 ≤ r ≤ 2 always, 0.5 ≤ r ≤ 2 whenever no hold-up had begun (r₀ = 1)**. It is never negative, so no part of him ever moves backward. It starts at t_L, never before the decision (D22), and ends by t_c − 0.03.
- **Where it is capped.** If no W ≤ W_max gives r\* in bounds, W = W_max and r\* takes its bound; the body then misses A\* by the rest. In practice (t_L ≈ t_c − 0.20, W_max ≈ 0.17 s): a late swing reaches its target for Δ up to about 0.075 s, an early one for A\* up to about 0.15 s ahead of T_a. **What is seen beyond:** on a very late swing the body stays ahead of the bat by the rest (hips round before the hands, hands dragging, elbows more bent: the "very late" picture of §13.1); on a very early swing the hips' lead over the hands shrinks, in the worst case to the hips starting with them. The chest's correction and the elbows take what is left; `FBbReach` takes anything beyond.
- **Late swings and the hold-up.** If t_L comes after a take's hold-up would have begun (C + `TakeShowS`), the pose before t_L is the take's (D22), so r₀ < 1: he holds up, then goes. That is the hitch of a late swing.

**Scaling the sources' slow pitches.** [4]'s pitches took 0.52 s; [6]'s about 0.45 s; a 93 mph fastball takes about 0.41 s (the sim's median four-seam: 0.407 s, measured in step 1). The last 0.17 s, the swing itself, matches across sources and is the sim's, so it is scaled only by the batter's `T_swing` (above). Everything earlier is squeezed toward the commitment by **s = (T_ref − 0.17) / (0.52 − 0.17)**, where T_ref is a reference fastball's flight (0.407 s ⇒ s = 0.677): an event [4] puts d seconds before its commitment is put s·d before the commitment of the reference pitch. That fixes the pitcher's-clock events once (foot off −0.07 s, top of the lift +0.025 s, foot down +0.12 s from release, the deepest coils and L as above). Check: foot down is then 0.285 s before contact on the reference pitch; [6] and [2] put it 0.17–0.20 s before on slower machine pitches, so it is not too late. On a faster pitch the keys tied to the commitment are clamped to come no earlier than 0.01 s after foot down.

**The body against the bat.** The sim's bat turns fastest 0.010 s after its nominal contact (`bat_peak`); [4]'s bat 0.010 s before contact. With the body keyed to contact as [4]'s is, the sim's hips lead its bat's peak by about 80 ms against [4]'s 61 ms. This is not visible and is accepted; the order check in §13.7 cannot see it, so it is stated here.

**Invariant 4.** The body conforms to the sim's bat and ball, never the reverse. The hands are on the drawn bat on every frame (two-bone IK, as now), and the drawn bat is the sim's from the handover (§13.4) to the end of the sim's swing track. Before the sim's bat moves, and after its track ends, the bat is presentation: the sim's stance and load samples are already made up (`stance_pose`, `LOAD_S`, "presentation only"), and after contact the bat's path is cosmetic (physics-spec §6.3).

**A swing under way at the release** (`t0 <= 0` in `timeline.rs`) has no bat track at all. It should not happen (a swing waits for the decision, after the release); if it does, the body stays on the pitch's clock as on a take and the bat stays the client's, and `-BbStagingLog` logs it.

### 13.3 Parameters

Every value is named and configurable ([E]) in the client (`BbMotion`), not a literal. "By eye" marks a value no base source gives; it is set from the picture checks. "Derived" marks a value computed from others. **Per player** (D23): every knob marked "yes" may later vary by player as **looks only**: none changes an outcome.

**Times** (s; release-relative "R", before the commitment "C−", or "τ" from the body's contact A).

| Name | Base | Sources' range | Source | Per player |
| :-- | :-- | :-- | :-- | :-- |
| `RefFlightS` T_ref, a reference fastball's release to the contact plane | 0.407 (the sim's median four-seam, 4,816 pitches in 40 games: p5 0.400, p95 0.414; step 1; was 0.42 by estimate) | sim by type: slider 0.453, changeup 0.457, curveball 0.492; [4] 0.52 | sim | no |
| `CommitLeadS`, weight commitment before contact | `T_swing` + 0.020 (0.170 at 0.150) | [4] 0.170, 0.020 ahead of the hands | [4], sim | yes (timing style) |
| Pre-commitment scale s | 0.677 | — | derived, §13.2 | no |
| `SettleS`, stance drift begins | R −0.325 | [4] −1.0 s, scaled | [4] | yes |
| `FootOffS` | R −0.07 | [4] −0.621, scaled | [4] | yes |
| `LiftTopS`, front foot highest | R +0.025 | [4] −0.483 (knee most bent), scaled | [4] | yes |
| `FootDownS`, front toe down | R +0.12 | [4] −0.340, scaled (0.285 s before contact); check ≤ −0.20 [6,2] | [4] | yes |
| Deepest hip coil | R +0.183, no later than C −0.054, then held | [4] −0.250, scaled; tee −0.350 [3] | [4] | yes |
| Hands furthest back (load pose L) | R +0.188, no later than C −0.049, then held | [4] −0.243 (bat's least azimuth), scaled | [4] | yes |
| Deepest chest coil | R +0.226, no later than C −0.011, then held | [4] −0.186, scaled; tee −0.265 [3] | [4] | yes |
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
| `EarlyGain` κ, the body's timing against the bat's on an early swing | 2.7 | 2.6–2.8 (pelvis, lead knee) | [4] Table 41 | no |
| `EarlyLeadMaxS` e_max, cap on the body's lead over the bat | 0.025 | [4] 0.024 (changeup misses) | [4] Table 41 | no |
| `LateGain`, the same on a late swing | 1 | none | by reasoning | no |
| `WarpRampS` R / `WarpMinS` / warp end before contact | 0.02 / 0.06 / 0.03 | — | [E] | no |
| `WarpRate` bounds r\* | 0.5–2 | — | [E] | no |
| `HandoverS`, the bat from the client's to the sim's (§13.4) | ends at t_s, but at least **0.05** after t_L (step 1: 0.03 gave jumps, §13.10) | sim's `LOAD_S` 0.05 | [E] | no |
| `TakeShowS`, a take's hips follow the swing's for | 0.03 | none | by eye | yes |
| `HoldBrakeS`, a take's or checked swing's body brakes over | 0.040 | sim's `BRAKE_S` 0.040 | sim | no |
| `TakeRelaxS`, the body back toward the stance after the catch (feet stay) | +0.2, then over 0.6 | none | by eye | yes |
| Follow-through takes the bat (§13.5) | the end of the sim's track **as read from the track** (+0.12 today; +0.03 under Q43) | — | sim | no |

**Angles** (degrees; radians in code).

| Name | Base | Sources' range | Source | Per player |
| :-- | :-- | :-- | :-- | :-- |
| Hips' turn: stance / foot down / deepest coil / commitment | −10 / −18 / −21 / −13 | coil −21 [4], −28 tee [3] | [4] | yes (coil size) |
| Hips' turn: τ −0.093 / contact (middle) / +0.2 / finish | +20 / +77 / +95 / +100 | contact 61–79 by location [4] (§13.7) | [4]; finish by eye | finish yes |
| Chest's turn: stance / foot down / deepest coil / τ −0.093 | −9 / −16 / −22 / +2 | coil −22 [4], −52 tee [3], ~8 beyond the stance [7] | [4] | yes (coil size) |
| Chest's turn: contact / +0.2 / finish | +75 / +135 / +150 | [4]; finish none | [4]; finish by eye | finish yes |
| Separation (chest − hips), check | ≈ 0 at foot down; −18 at τ −0.103; −2 at contact | 18 [4]; check 21 [6], 22 [7] | [4] | — |
| Hips' tilt: front hip rising, τ −0.093 → contact | 14 → 27 | [4] (obliquity; the side it rises on is inferred) | [4] | no |
| Front knee: stance / foot off / foot down / commitment | 36 / 48 / 48 / 48 | [4] | [4] | yes (crouch) |
| Front knee at the top of the lift | an outcome of the lift height (about 60–75°), not a key | 52 [4] (event-averaged, a floor); check 83 [6] | derived | — |
| Front hip at the top of the lift: thigh raised forward | 15–25 | none | by eye | yes (lift) |
| Front knee: τ −0.093 / contact / +0.073 / finish | 40 / 18 / 11 / 15 | contact 15 [3]–18 [4] (19–20 on changeups [4]); check 14 [6] | [4,3]; finish by eye | no |
| Back knee: stance / foot off / foot down / straightest / commitment | 51 / 56 / 50 / 44 / 47 | [4] | [4] | yes (crouch) |
| Back knee at contact | 50 | 45 [3]–54 [4]; check 71 [6] | midpoint, by eye | no |
| Front toe, closed from the pitcher line: stance / foot down / commitment / contact / +0.06 | 69 / 63 / 56 / 49 / 36 | 61 at foot down [3] | [4] | yes |
| Stride direction, closed | 8 (toe to toe, from the 0.11 m below) | 12 [3] | derived | yes |
| Trunk bent forward over the plate: stance / foot off / foot down / contact | 15 / 21 / 23 / 10 | 21 at foot off [3]; ~25 at landing [7]; contact −9 [3] (global frame) to +16 at +0.03 [4] | stance, foot down and contact by eye; foot off [3] | yes (posture) |
| Trunk tilted toward the back leg: foot off / contact | −6 (toward the pitcher) / 20 | contact 20 [3]; [4] 9 → 4 in the acceleration (trunk-relative); check 28–32 [6,7] | [3] | no |
| Head tilt toward the back shoulder: commitment / τ −0.093 / contact | 9 / 15 / 31 | [4] §4.1.4 (9→15 in initiation, 15→31 in acceleration) | [4] | no |
| Head turn at contact, check (eyes on the ball give it) | 47 from the pitcher | 38–55 by location [4] | [4] | — |
| Elbows at contact, check: back / front | 81 / 56 on middle fastballs | back 66–89 by location, 51–81 by pitch type [4]; check 79 / 53 [6] | [4] | — |
| Elbows after: front at +0.043 / back at +0.083 | 42 / 37 | [4] | [4] | no |
| Bat in the load: elevation / wrap | **not used in step 1**: the bat's direction at L is the sim's, 9° above level with the tip toward the catcher (§13.10); [4]'s load has the bat 44° above level wrapped behind the head | [4] (elevation, least azimuth −232°) | sim (L); [4] only to compare | yes (hand load) |
| Follow-through bat: elevation at +0.15 / turned by +0.2 | +28 / 246 from contact | [4] | [4] | yes |
| Follow-through bat: turning rate at contact / +0.2 | 2233 / 827°/s (39 / 14 rad/s) | [4] | [4] | no |
| Chest's correction toward the hands, cap | ±15 on middle aims, ±25 elsewhere | none | [E] | no |

**Distances** (m; H = the figure's height).

| Name | Base | Sources' range | Source | Per player |
| :-- | :-- | :-- | :-- | :-- |
| `StanceWidth`, ankle to ankle along the pitcher line | 0.30 H (0.55 m at 1.84 m; now 0.54) | [6] 0.68 (OBP only); "bat length" 0.86 [8] is wider than the stride at foot down, so not used | by eye | yes |
| Stride at foot down, toe to toe | 0.42 H (0.77 m) | [4] 0.42 H; [3] 0.85 m (~0.46 H) | [4] | yes |
| Front foot's travel forward | 0.42 H − `StanceWidth` (0.12 H, 0.22 m) | check 0.23 m [6] | derived (changes with the width) | yes |
| Front foot's travel toward the plate | 0.11 | [4] | [4] | yes |
| `LiftHeight`, front ankle | 0.08 | [6] 0.22 (OBP only); now 0.09 | by eye | yes (leg kick or toe tap) |
| Hips' place along the pitcher line, from the stance: settled / foot down / commitment / τ −0.093 | −0.06 / +0.01 / +0.12 / +0.17 | [4]: back 0.06, then forward 0.07 in the stride, 0.11 to the commitment, 0.05 more (0.23 in all from the start of the stride) | [4] | yes (load size) |
| Hips' rise, τ −0.093 → contact | +0.05 | [4]; [6]'s drop is OBP only | [4] | no |
| Head travel toward the pitcher after foot down, check | ≤ 0.08 | "quiet head" [2,7] | by eye | — |
| Stance hands: knob from his spot | 0.40 toward the catcher, 0.10 toward the plate, 1.30 up | (today's `stance_pose`) | by eye | yes (hand set) |
| Load pose L: knob and bat direction | the mean of the sim's swing-start pose, measured in step 1 (§13.10): knob 0.57 m toward the catcher, 0.03 m away from the plate side of his spot, 1.25 m up; bat nearly level (9° above), tip toward the catcher and out from the plate | its spread: a start is a median 13.6 cm and 6° from L (95%: 29.5 cm, 15.7°) | sim | yes (hand load) |

**Speeds** (checks, not inputs: the keys are angles at times, joined by a curve that keeps its speed through each key, as the delivery is).

| Check | Range | Source |
| :-- | :-- | :-- |
| Hips' peak turning rate | 580–710°/s (10.1–12.4 rad/s), at τ −0.08 to −0.06 | [4,3] |
| Chest's peak | 770–940°/s (13.4–16.4 rad/s), 4–13 ms after the hips | [4,3] |
| Order of peaks | hips, chest, hands, bat | all |
| Hands before the sim's swing | ≤ 1.5 m/s | [E] |
| Body clock rate | 0–2 always; 0.5–2 unless restarting from a hold-up | §13.2 |

**Weight** is not a pose field. It shows through the hips' place and the knees, which come from [4]'s measured positions; the force figures (rear leg 85–90% at foot off, the front foot 50% at the commitment, 130% at τ −0.093, 84% at contact [3]) are what a reviewer checks the pictures against, not something solved.

### 13.4 How the pose is built

Two layers, as now (a stick figure solved by `FBbSkeleton` with hands placed by `FBbReach`), with the job split differently:

- **The body is on the clock.** Legs, feet, hips (place, turn, tilt), the trunk's turn and bend, the head's tilt and the weight are key poses over the body's time (§13.2), joined by a speed-keeping curve. The knees are set by solving the hips' height and tilt so that both legs show their key flexion for the figure's own thigh and shin lengths, except in the lift (below). The eyes stay on the ball, as now.
- **The leg lift: the foot's height wins.** The front ankle is put `LiftHeight` up, under the front hip and a little ahead; the hips' height is set by the back leg's knee; the front thigh is raised forward (15–25°, by eye) and the front knee bends as much as that leaves (about 60–75°). [4]'s 52° is an event-averaged mean, a floor for a single swing, and [6]'s per-swing 83° is consistent with the outcome. With level hips and both knees at their [4] values the foot would sit 1.4 cm below the ground, so the knee cannot be the key. **This is the first thing to look at in step 2's pictures.**
- **The hands are on the bat.** Both hands are on the handle every frame (grip as now). The arms run from the shoulders the body put there to the hands the bat put there. The elbows' bend is therefore an outcome, and a check: bent at contact, straight just after.
- **Where the two meet.** The chest's turn is the clock's plus a correction toward where the hands are, capped at ±15° on middle aims and ±25° elsewhere [E], so an arm is never stretched past its length or folded flat; `FBbReach` still moves the body up to 24 cm as a last resort. On a fooled swing the correction is how the chest, which went with the hips, leans back toward the late hands. The correction is logged: a large one on a middle fastball means the keys and the sim's bat disagree, a defect to fix in the keys, not to hide.
- **What replaces today's drive.** The hips and shoulders no longer turn by how far the hands have come round (`TurnDeg`); the stride is no longer a smoothstep from −0.2 s to +0.17 s. The swing weight that blends the swing in is no longer needed: the stance is the clock's first key.

**The bat, by stretch of time.** The client draws the sim's bat **interpolated between the 5 ms samples** (knob by straight line, direction by normalized blend), not held on the last sample as `bb_timeline_sample` returns it today (the bat turns up to 10° between samples): either the client samples the two neighbouring sample times itself, or the FFI interpolates the bat as it already does the players (export E1, §13.6).

1. **Before the sim's bat moves** (all of a take; a swing until t_L): **the client's bat.** The stance hold, then the load: from the settle (R −0.325 s) the hands drift back to L and the bat tips over to L's direction (the sim's: laid back toward the catcher, §13.10), reaching L on the pitcher's clock (§13.2), and stay there. Identical on swings and takes. The sim's own stance and load samples (`stance_pose`, `LOAD_S`) are no longer drawn; the sim is unchanged.
2. **The handover** (t_L until t_s, or t_L + 0.05 s if that is later): the drawn bat is a blend of the client's and a target, the weight rising smoothly 0 → 1. The target is the sim's swing-start pose (its bat at t_s) until t_s, and the sim's bat from then; so the blend has no backward wiggle through the sim's straight-line load, and nothing jumps. When the sim cut its load short (the decision came less than 0.05 s before t_s), the blend ends 0.05 s after t_L, at most 0.05 s into the swing and still 0.10 s before contact: for those few hundredths the drawn bat converges on the sim's instead of being on it. Because L is the sim's mean start pose, the distance to close is only the spread of start poses: measured in step 1 (§13.10), a median 13.6 cm and 6° (95%: 29.5 cm and 15.7°; at most 48 cm and 31°).
3. **The swing** (to the end of the sim's track): the sim's bat exactly (interpolated).
4. **After the sim's track**: the client's follow-through (§13.5), then the bat back to his shoulder or dropped, as now.

**A take.** From the commitment the body follows the swing's keys for `TakeShowS` (0.03 s), then its clock brakes to a stop over `HoldBrakeS` (0.04 s, the same law as the sim's check-swing brake: speed falling evenly to zero). It holds there, the bat at L, until `TakeRelaxS` after the catch; then the hips, trunk, knees and hands ease back toward the stance over 0.6 s **with the feet where they are**: the front foot stays planted (the knees and hips are solved over the feet as they stand) until the reset between pitches, when it steps back to its place.

**A checked swing.** The sim's bat brakes at t_brake (with export E1, t_s and t_c are exact; otherwise t_brake is the track's last movement less 0.04 s). The body's clock brakes over the same 0.04 s from the same moment, holds, and recovers as after a take. The hips, already open, stay open; nothing is drawn later than the bat.

**No jump anywhere.** Every input above is continuous in play time: clocks are warped, not reset; the bat is blended, not switched; a hold brakes over 0.04 s. The checks in §13.7 measure it against the expected motion.

### 13.5 The bat after the sim's track ends

**What happens now.** The sim draws the bat until 0.12 s after its nominal contact (`timeline.rs`, `t1 = sw.t_c + 0.12`); the FFI then holds the last sample for 0.25 s more and reports no bat after that (`bb_ffi`, `has_bat` = 0 from +0.37 s); the client brings the bat back to his shoulder from the moment the bat stopped moving. The sim's template slows the bat after contact as a bell curve about the bat's peak (width 0.055 s; agent-models §3.6), so at `T_swing` 0.150 it turns at 95% of its contact rate at +0.03 s, 67% at +0.06 s, 27% at +0.10 s and **14% at +0.12 s**; by then it has turned about 170–185° past contact and nearly stopped. The figures scale with the batter's `T_swing` (a 0.130 s batter's bat is down to about 7% at +0.12 s). [4]'s hitters were still turning the bat at 37% of the contact rate at +0.2 s, 246° past contact. So any follow-through started at +0.12 s starts from a bat that has nearly stopped. The template's shape after contact never mattered to the game (contact is decided within ±0.025 s of the planned time, and after contact the bat is cosmetic, physics-spec §6.3).

**Options.**

| | (A) Sim track to about +0.4 s | (B) Client from the end of today's track (+0.12 s) | (C) Sim track ends 0.03 s after contact; client from there |
| :-- | :-- | :-- | :-- |
| What it takes | A follow-through model written in the sim (the template stops by +0.2 s, so extending it gives a bat frozen in its plane) | Client only | The client part of (B), and in the sim the end of the track moved (below) |
| Look | Good if the sim's follow-through is good; the body still has to follow it | **Poor**: the bat brakes to 14% before the client takes over, then wraps from almost a standstill | Good: the client takes a bat still at 95% of its speed and slows it as [4] measured |
| Schema, FFI header | None | None | None |
| Game digests | Unchanged (the timeline is not in the event log, as with `stance_pose`, ledger 11-16) | Unchanged | Unchanged |
| Ledger | One "No effect on games" line | None | One "No effect on games" line |
| Cost elsewhere | Presentation motion design in the sim, which is for physics; every future change to a finish is a sim change | None | See below |

**What (C) touches besides the constant:**

1. **The FFI hold.** `bb_timeline_sample` holds the last sample until 0.25 s after it and then sets `has_bat` = 0; under (C) the bat disappears at contact + 0.28 s, before the finish (+0.45 s). The client's follow-through therefore draws the bat on its own from the track's end, **whatever `has_bat` says**, until the bat is back on his shoulder or dropped. Built that way in step 4, it works under (B) and (C) alike.
2. **The users of the track's end (`EndT`, the bat's last movement, in `NoteBat`).** It decides when the bat is the sim's (`bBatSim`), when the swing is on (`bSwinging`), when the bat starts back to the shoulder (`BatReturnS`), when he eases (`EndT` + 0.3 s) and the motion strips' "finish", "back" and "rest" marks. Step 4 replaces the return-to-shoulder start with the follow-through and re-keys the others to the follow-through's own times. Under (C) all of them move 0.09 s earlier.
3. **Order.** The sim change lands **with build step 4 or after it, never before**: today's client would start bringing the bat back to his shoulder at +0.03 s from a bat at 95% of its speed, and snap it round.
4. **Saved plays keep their old tracks.** A saved play stores its timeline (`.timeline` with its md5, `BbSavePlay.cpp`), so the owner's saved plays, `client/check-fair-signals.sh` and `-BbSavedPlayCheck` will carry the +0.12 s track for good. **The client reads the end of the track it has, never a constant**, and step 4 is tested on an old saved play (+0.12 s) and on a new game (+0.03 s once (C) is in).
5. **Actual or nominal contact.** The track ends at **0.03 s after the later of the actual contact and the nominal one** (actual contact can come up to 0.025 s after the nominal), and on a miss or a foul tip 0.03 s after the nominal contact. So the end is always at least 0.03 s after any contact.
6. **Stale text to fix in the same commit:** the comment in `timeline.rs` ("follows the executed swing every 5 ms through the follow-through"), `swing_pose_probe.rs` (its doc comment and its `sw.t_c + 0.12` sample) and data-formats §5.3 where it describes the bat track.

**Recommendation: (C), built in that order.** The client carries the bat from the end of the sim's track, continuing it with its position, direction and turning rate matched and slowing as [4] measured (to 827°/s at +0.2 s, elevation rising to +28° by +0.15 s), wrapping over the front shoulder to the finish (by eye). Per-player finishes (D23) then stay client work. Step 4 is built and checked first against today's +0.12 s track (B); the sim's change (C) follows it.

> **Q43 (owner, not yet decided).** *Now:* after the ball is hit or missed, the sim keeps drawing the bat for 0.12 s, and in that time it slows to a seventh of its speed, far quicker than a real hitter's bat, so the follow-through starts from a bat that has nearly stopped. *Proposed:* the sim's bat path ends 0.03 s after contact, just after the moment that decides the hit, and the game draws the rest of the swing and the finish from the measured swings. No game changes. It goes in together with the new follow-through, never before it. *Alternatives:* keep the 0.12 s (looks worse), or write a follow-through into the sim (puts drawing into the physics code). *Recommended:* end it at 0.03 s.

### 13.6 What the sim must provide

**Export E1 (first choice; for the Coordinator under Q21, no owner question).** Three small read-only changes, all presentation, none in the event log:

- `BbPlayInfo.swing_start_s`: t_s, from the timeline's existing `SwingStart` marker (written at exactly `sw.t_start()` since M4, so old saved plays have it too).
- `BbPlayInfo.swing_contact_s`: t_c, the executed swing's nominal contact. The timeline does not carry it: one appended field in its telemetry table (default −1, so old saved plays read −1 and the client falls back to inference).
- `bb_timeline_sample` interpolates the bat between its 5 ms samples, as it already does the players.

*Why it is needed (Q21: an export only when a motion needs something the timeline does not carry):* the swing's clock needs t_c, which the timeline does not carry, and t_s, which it carries but the FFI does not pass on. Inferring them works but is not exact enough everywhere: t_s from the end of the load's straight line is good to 5 ms; t_c from the bat's fastest turning needs a fit over the samples to reach 2 ms (the peak is flat) and an offset that depends on `T_swing` (8.7–11.6 ms); on a checked swing, which never reaches its peak, t_s + 0.150 s is off by up to 24 ms, enough to shift the body visibly. With E1 all of these, and `T_swing` = t_c − t_s, are exact, and the client code is smaller. *Cost:* `bb_ffi.h` (two fields, `BB_ABI_VERSION` 2 → 3), one schema field and `./schemas/gen.sh`, an FFI test; game digests unchanged (the timeline is not in the event log); one "No effect on games" ledger line. It lands in build step 1.

**Fallback without E1**, and for saved plays older than it: t_s where the sim's load samples (a straight line at one speed) end; t_c by fitting the bat's turning rate over the samples round its peak and subtracting `bat_peak` × `T_swing`/0.150, with `T_swing` from t_c − t_s iterated once; a checked swing's t_c from its brake time and the bat's rate there through the template's rate law. Step 1 measured each against the sim's own records (§13.10): t_s is exact, t_c is within 0.011 ms by the fit on unchecked swings, and on a checked swing t_s + 0.150 s is exact while every batter's `T_swing` is 0.150 (it will be off by up to 24 ms once it varies).

**Q43 (§13.5)** moves the end of the bat track: `t1` in `bb_game::timeline::encode`, presentation only, with the points listed there.

Two things found in the timeline that this design does not use, for the record: the `PlateCross` marker is written at a fixed 0.42 s whatever the pitch, and `CheckSwingBrake` exists in the schema but is never written. Neither is read by the client.

### 13.7 Checks

All headless, as in §8–§12.

**Pictures.** `-BbMotionShots`, strips 4 (a ball hit), 5 (a swing and miss) and 6 (a take), re-keyed to the events of §13.1 and shot from across the plate: stance, foot off, top of the lift, foot down, deepest coil, commitment, peak separation, contact, extension (+0.08), through (+0.2), finish (+0.45), recovery (+1.0). In each, a reviewer should see:

- **Top of the lift:** the front foot clearly off the ground (about 8 cm), thigh raised a little, the weight on the back leg; not a foot dragging or a knee folded flat.
- **Foot down:** front toe touching first, foot closed (toes toward the plate), front knee bent, weight back, hips and shoulders coiled together, hands back near the back shoulder away from the chest, bat tipped toward the pitcher.
- **Commitment and separation:** the belt buckle turning toward the pitcher while the letters still face the plate; the back elbow dropping; the front heel down.
- **Contact:** front leg nearly straight, back knee bent with the heel up, hips almost facing the pitcher with the chest square to them, both elbows bent, barrel below the hands, spine tilted away from the pitcher, head down and back.
- **Extension and through:** both arms straight just past contact, then folding; the bat rising round.
- **Finish:** bat high over the front shoulder, two hands, chest past the pitcher, tall on the front leg, back toe down.
- **Take (strip 6):** the same as strip 4 to the commitment; then hips a little open and stopped, chest closed, bat back, eyes on the ball into the mitt, feet still.

New: **strip 9, a checked swing** (the first the run finds): hips open, bat stopped in front of him with the barrel behind the plate, no part of him still moving after the bat stops. **Strip 10, an early swing on an off-speed pitch** (the first with Δ < −0.01 s): at contact the hips and chest open, the back elbow more bent than strip 4's. **Strip 11, a late swing** (the first with Δ > 0.03 s; the sim has none beyond +0.051 s, §13.10, so the hitch itself, which needs more than 0.06 s, is shown by a probe that forces t_L late): the hips started and holding up, the hitch at t_L, then the swing. **`-BbSwingPairShots`**: a swing and a take of nearly the same flight time (within 0.01 s), shot side by side at the same moments up to the swing's t_L; they must look the same (D22; the numeric test below is the exact one).

**Numbers** (`-BbStagingLog`, a new `BB_SWINGPOSE` line at each named moment with the joint angles, the hips' and chest's turn and rate, the chest's correction, Δ, the warp's W and r\*, the hands' distance from the handle, the drawn knob's speed and the pitch's location and type; and a script beside `client/check-fair-signals.sh` that runs seeds 7 and 6 and fails on any of these):

*Contact checks, by pitch.* The ranges come from [4] for its own swings; they apply where the sim's swing is like those, and elsewhere what [4] gives by location is expected. "Middle" = the pitch within the middle third of the zone across and up (`BbPlayInfo` `plate_x_m`, `plate_z_m` and the zone); "fastball" = four-seam, sinker or cutter.
- On every swing: front knee 8–30° (18–20° on every pitch type and location in [4]); a take's hips never more than 35° round from their deepest coil; a checked swing's body still within 0.02 s after the bat stops.
- **Middle fastballs met on time (|Δ| ≤ 0.01 s):** back knee 40–60°, back elbow 66–95°, front elbow 45–70°, hips 65–90°, chest minus hips within ±8°; the chest's correction within 15° on 95% of the swing's frames.
- **Other locations, fastballs:** hips within ±12° and the back elbow within ±15° of [4]'s location means (hips: high inside 79°, high outside 69°, low inside 76°, low outside 61°; back elbow 89°, 77°, 77°, 66°); the chest's correction within 25°. Reported, and failed only beyond these.
- **Off-speed pitches and early swings:** back elbow 40–90° ([4] Table 40: 67° on a changeup hit, 51° on a miss); hips 60–95°. Reported, not failed in between.
- Checks against [6] (allowed as checks, D21; wide tolerance): front knee at contact 14° ± 12°, peak separation 21° ± 10°, back elbow 79° ± 15° on middle fastballs.

*Timing and continuity.*
- Peaks: hips 500–800°/s, chest 650–1000°/s, the chest's peak 0–25 ms after the hips' (on early swings up to 40 ms).
- Foot down at least 0.20 s before contact on every swing.
- The body clock's rate within §13.2's bounds on every frame, never negative, and the warp over by t_c − 0.03.
- Hands on the handle (0.0 cm) from the handover's end to the end of the sim's track; the drawn knob never faster than 1.5 m/s before t_L.
- **Continuity against the expected motion,** at the handover into the sim's swing and at the follow-through's start: the drawn bat within 2 cm and 2° of the motion extrapolated from the last 10 ms before the change; at the follow-through, the turning rate within 5% of the sim's last rate and never rising afterwards. (A frame-to-frame step is not a test: at contact the bat turns about 35° a frame at 60 fps.)
- **D22**: for every pitch of the run, the pose worked out with the swing withheld (as a take) and with it, at identical play times, equal to 0.1 cm at every joint for all times before t_L.
- Inferred times against the sim's records (step 1's probe, and whenever E1 is absent): t_s within 5 ms, t_c within 2 ms; T_a from a track cut at contact against T_a from the same pitch's full track on swings that miss: within 1 ms. **Measured in step 1: 0.001 ms, 0.011 ms and 0.1 ms** (§13.10).

*What these checks test.* The elbows at contact, the speed peaks and their order, the inferred times, the D22 comparison, the hands on the handle and the continuity are outcomes the keys do not set: they test the design. The contact ranges for the knees, the hips and the separation, and "foot down 0.20 s before contact", sit round the keys themselves: they are kept as integration tests of the clocks and the solver, not as tests of the research.

**Sim side**, with export E1: an FFI test that `swing_start_s` equals the `SwingStart` marker and `swing_contact_s` the swing's `t_c`, and that the bat is interpolated; with Q43: a `bb_game` test that the bat track ends 0.03 s after the later contact. For either, `cargo run --release -p bb_game --example digest` identical before and after, and one ledger line.

**Step 1's checks, built:** `-BbBatCheck` (with `-BbStagingLog`) logs `BB_BATCHECK` per pitch (the drawn bat's speed before the sim's bat moves, the continuity at both ends of the handover, the swing-withheld comparison), `BB_BATNOTE` (the clocks, with t_c inferred when E1 is absent) and `BB_BATHANDS`; `client/check-bat-step1.sh` runs seeds 7 and 6 with it and against `-BbBatLegacy` (the bat as before) and fails on any of them; `-BbBatShots=2` takes the pictures (`bat1_swing_*`, `bat1_take_*`). The D22 comparison is of the drawn bat alone, which is all the pose reads before the sim's bat moves (stride and eyes are on the pitcher's clock and the pitch's ball, the swing weight 0 until then); it is not yet of every joint.

**Cost:** frame rate from `-BbShots=6 -BbUncapped` against the build before, as every step reports.

### 13.8 Build steps

Each a worker-sized job with its own pictures and numbers from §13.7, checkable on its own, in this order:

1. **The bat before the swing, and the measurements.** Export E1 (if the Coordinator approves it) and the bat interpolated. The client's bat: stance, the hands' load to L on the pitcher's clock, the handover into the sim's swing. Measure first (extend `swing_pose_probe`): T_ref (the sim's median four-seam flight to the contact plane), the swing-start pose's mean and spread (for L), the distribution of Δ and of t_s − t_decide (how often the load is cut), T_a from a cut track against the full track, and, without E1, the accuracy of the inferred t_s and t_c. Checks: hand speed, handover continuity, hands on the handle, the D22 comparison (it needs t_L, and before t_L everything is now the client's). *The owner sees:* the hands drift back and the bat tips over through the stride, the same on every pitch; no jump from the stance into the swing.
2. **The legs and the stride, on the pitcher's and the pitch's clocks.** Stance, settle, lift (the foot's height first, §13.4), stride, foot down, the knees, the hips' place and rise, front toe and back heel. Every swing is timed from T_a for now. *Sees:* a real stride that lands soft and closed, the weight going back then forward, a front leg that straightens into contact and a back heel that comes up.
3. **The coil and the hip turn, and the take.** Hips' and chest's turn and the trunk's bend on the clock (D22), the head's tilt, the chest's correction toward the hands; the take's hold-up and relax with the feet still. *Sees:* the body coils, the hips go before the hands and the chest catches up at contact; on a take the hips start and stop.
4. **The follow-through and finish.** Built against today's track (B): the bat continued from the track's end as read, slowed as [4] measured, drawn on regardless of `has_bat`; the elbows straightening then folding, the chest turning on, the finish held, then the bat back to the shoulder or dropped; `EndT`'s users re-keyed. Then, if the owner answers Q43 yes, the sim's change (C) in the same step or after it. Tested on an old saved play and a new game. *Sees:* the swing no longer stops dead after the ball; a full two-handed finish.
5. **The swing's clock: early, late and checked swings.** A\* from Δ (κ, e_max, late), the warp with its bounds, the body's swing times over `T_swing` + 0.02 s, the checked swing's brake and hold; strips 9–11 and `-BbSwingPairShots`. *Sees:* a fooled hitter's hips flying open ahead of his hands; a late hitter's hitch; a checked swing that stops with the hips open.

Each step updates §9 and this section's status to say what is built, ticks nothing in observations.md until the owner has seen it move, and adds no ledger row except for E1 and Q43 (one "No effect on games" line each).

### 13.9 Not in the base, and open points

**Q37 decided (a), PLAN D23 (owner, 2026-11-23):** each player's own stance, swing style and follow-through (the owner's examples: Pete Rose, Darryl Strawberry, Gary Carter, all variations of core stances, none quite alike) are **looks only**: client-side variety that changes no outcome. So every "per player" knob in §13.3 is looks only, and the sim's swing (timing, extension, plane) stays the sim's from the batter's attributes alone. Tying stances to attributes can be reopened when batter ratings exist.

**Left to per-player stances (looks only, after the base):**
- The stance: width, open or closed, crouch, knee bend, how upright, hand height and bat angle, a waggle.
- The load: how far the hips go back, how high the leg lifts (a kick against a toe tap), how far the hands load and the bat wraps, when it all starts (`FootOffS`, `SettleS`).
- The coil's size (chest 8–26° in [6]'s spread; tee hitters more [3]).
- The finish: one or two hands, high or level, how far round; a bat flip.
- The hips and chest at contact by pitch location ([4]: hips 79° on a high inside pitch, 61° low and away). The base keys the middle pitch's values everywhere and lets the chest's correction follow the hands; the checks expect [4]'s location values only as reported ranges (§13.7).

**Open points the sources do not settle, and what the base chooses:**
- **The finish** (none measured; [4] stops at +0.2 s): bat wrapped high over the front shoulder and behind the head, two hands, held 0.15 s, chosen from the Driveline clip [5] and common pictures. Set by eye in step 4. The least certain part of the design.
- **A late swing** is not measured by any source; the base has the body go late with the bat (§13.2), and a hitch where a take's hold-up had begun.
- **An early swing beyond [4]'s range** (body lead capped at 0.025 s): [4]'s changeup groups are the only data; beyond them the body goes with the bat.
- **One-hand release:** not in the base. Two hands throughout, as [4] analysed and [5] shows. A later per-player look.
- **Leaving the box:** on a ball in play, the bat drops from both hands when the sim's batter-runner first moves, or at the finish if that is later, and the pose gives way to the running clip over 0.3 s as now; the bat is not drawn after (as now). No measured source; the base keeps today's behaviour.
- **Balance after the finish:** standing on the front leg with the back toe down; no source.
- **Stance width:** no base source ([6] is a check only; HDR's "bat length" is wider than [4]'s stride at foot down); 0.30 H by eye, close to today's.
- **Leg-lift height:** OBP only; 0.08 m by eye, close to today's; the foot's height wins over the knee (§13.4).
- **The trunk's bend at contact:** [3] and [4] decompose it in different frames (−9° in [3]'s global frame, +16° at +0.03 s in [4]'s); 10° forward plus 20° toward the back leg is chosen to sit between them, and checked by eye.
- **A take does not depend on where the pitch is:** without a decision input before t_L it cannot (D22 gives that up on purpose).
- **[4]'s own angles and speeds disagree a little:** its hips turn 58° in the last 93 ms (624°/s on average) against a stated peak of 581°/s, and its chest 73° (785°/s) against 766°/s; the two come from different decompositions. The keys are [4]'s angles; the speed checks are the wider [4]–[3] ranges.
- **Bunts and hit-by-pitch evasion** (agent-models §3.7) are separate motions, not this one.

### 13.10 Measured in step 1 (2026-11-23)

`cargo run --release -p bb_game --example swing_pose_probe -- 40`: 40 games (seeds 0–39), 5,019 swings (643 checked) and 9,318 pitches with a flight to the plane (4,816 four-seams, 2,087 sliders, 1,230 curveballs, 1,185 changeups).

- **T_ref**, release to the plane y = 0.35 m: four-seam median **0.407 s** (min 0.393, p5 0.400, p95 0.414, max 0.424); slider 0.453, changeup 0.457, curveball 0.492. §13 had assumed 0.42; s, the pitcher's-clock keys and the foot-down check are recomputed from 0.407 above (every key moved by 0.02 s or less).
- **Load pose L**, the mean of the sim's swing-start pose in his frame (a right-hander; mirrored for a left-hander): knob 0.03 m away from the plate side of his spot, **0.569 m toward the catcher, 1.250 m up** (sd 4 cm, 8 cm, 14 cm); bat direction (toward the plate, toward the pitcher, up) = (−0.465, −0.871, 0.160): nearly level (9° above), the tip toward the catcher and a little away from the plate. **A start is a median 13.6 cm and 6.0° from L** (p5 4.6 cm, p95 29.5 cm and 15.7°, max 48 cm and 31°): that is what the handover closes. **§13.1's picture of the load (bat wrapped behind the head, tip toward the pitcher, [4]'s 44° elevation) is not what the sim's bat does at its start**; the bat is the sim's (invariant 4), so L stands and §13.1 is a look to revisit with the owner's eyes (the later steps do not depend on it).
- **The sim's load**: t_s − t_decide has a median 46 ms (p5 21, p95 67, max 88); the 50 ms load is cut short on **61.4%** of swings, by a median 11 ms (p95 32 ms), and absent on 9 (0.2%). So t_L is 0 to 50 ms before t_s (median 46), and with the handover at least `BatHandoverS` long it is **always 0.05 s**: it ends at t_L + 0.05 s, which is t_s or up to 0.05 s into the swing. **0.03 s, the spec's first value, was too short:** the start poses are a median 13.6 cm from L and up to 48 cm, so closing them in 0.03 s takes 7–14 m/s and the bat decelerates into the sim's swing hard enough that 4 of 35 swings in the first check (seeds 7 and 6) were 2.0–2.65 cm from the motion carried on from the last 10 ms at the handover's end (the limit is 2 cm); at 0.05 s none were (worst 1.2 cm, handover speeds up to 5 m/s).
- **Δ = t_c − T_a**: median −9.4 ms, p5 −52 ms, p95 +13 ms, min −144 ms, **max +51 ms**. |Δ| ≤ 10 ms on 43.5% of swings, Δ < −10 ms on 48.3%, Δ < −25 ms on 22.8%, Δ > +30 ms on 0.3% (15), Δ > +60 ms on none. Balls hit: median −3.9 ms; swings that miss: −24.6 ms. By pitch: four-seam median −5.3 ms, slider −12, curveball −17, changeup −25. So the sim's batters are early far more often than late, and a **late swing past the hold-up (Δ > 0.06 s) does not occur**; the early-swing design (κ, `EarlyLeadMaxS`) is the common case, in about half of all swings.
- **T_a from the track cut at contact**: against the same pitch's full track on the 2,169 swings that miss, 0.0 to +0.1 ms; on balls hit the 30 ms extrapolation moves by under 0.001 ms against one from 30 ms earlier. §13.2's "about 1 ms" holds with room.
- **Inferred times** (a timeline without E1, from the encoded bat samples): t_s from where the load's straight line ends is exact (under 0.001 ms) on the 5,010 swings with a load; t_c from the fastest turning of the bat less 0.010 s × `T_swing`/0.150 is within **0.011 ms** on the 4,375 unchecked swings; a checked swing (638) has a bat that has stopped at the track's end, which tells it from an unchecked one on all 5,013 swings, and t_s + 0.150 s is exact for it because every batter's `T_swing` is 0.150 today (the parameters vary it 0.130–0.174 s later: then up to 24 ms off). With E1 none of this is needed; the client keeps it for saved plays recorded before.
- **E1**: game digests identical before (`82751d0`) and after for seeds 1, 2, 3, 42 (`cargo run --release -p bb_game --example digest`).

## 14. Records

- PLAN D13 (revised 2026-11-15) covers this stage.
