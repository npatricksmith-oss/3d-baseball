# Player visuals: figures, motions, kit

**Status:** plan agreed with the owner 2026-11-15 (seven questions, Q15–Q21, all as recommended; §1). Step 1 (the figure) is built; steps 2–7 are not. This is the second part of the graphics stage, after [the park](park-visuals.md) and before [the HUD](hud.md).

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

- `client/run.sh 7 -RenderOffscreen -BbShots=6` for play; `-BbParkShots` for the seven fixed spots. Step 1 adds **figure spots** that do not depend on the play: a line-up of both nines front and back at close range, and one figure turned through eight angles.
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

## 7. Records

- PLAN D13 (revised 2026-11-15) covers this stage.
- TASKS.md "Resume here" carries the step reached; a `/clear` point after each merged step.
