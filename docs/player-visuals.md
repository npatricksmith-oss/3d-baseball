# Player visuals: figures, motions, kit

**Status:** plan agreed with the owner 2026-11-15 (seven questions, Q15–Q21, all as recommended; §1). Nothing built yet. This is the second part of the graphics stage, after [the park](park-visuals.md) and before [the HUD](hud.md).

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

- The clubs' names and colours, to be put here in step 1.

## 6. Records

- PLAN D13 (revised 2026-11-15) covers this stage.
- TASKS.md "Resume here" carries the step reached; a `/clear` point after each merged step.
