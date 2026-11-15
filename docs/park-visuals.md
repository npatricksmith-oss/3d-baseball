# Park visuals: field and stadium

**Status:** plan agreed with the owner 2026-11-15 (twelve questions, answers in §1). Nothing built yet. This is the first part of the graphics stage; players and game graphics (HUD) get their own plans afterwards.

**Why now:** Codex's weekly limit is nearly used, so the physics and behaviour work, which needs outside reviews, pauses. The owner moved the looks forward from the polish milestone (PLAN D13, revised the same day). This stage runs without Codex reviews.

## 1. Decisions (owner, 2026-11-15)

| # | Question | Ruling |
|---|---|---|
| Q1 | Look | **Clean stylized.** Crisp shapes, good lighting, simple readable materials; a tidy architectural model of a ballpark. Not broadcast realism, not toy-like. |
| Q2 | How the art is made | **All in code.** Meshes generated in C++; textures and materials built at startup or by a script. No binary assets committed, no Git LFS. |
| Q3 | Where the shape comes from | **The sim describes the park.** One park description from the sim; the client builds every visual from it. One park for now. |
| Q4 | Size | **Small park, one deck.** Minor-league size: one seating deck, a pressbox behind home, berm or bleachers in the outfield, light towers, a scoreboard, trees and sky beyond. |
| Q5 | Time of day | **Day now, night later.** One good afternoon look. Light towers are built as geometry now and switched on in a later stage. |
| Q6 | Crowd | **Simple seated figures.** Thousands of instanced low-detail figures in varied colours, no animation yet. |
| Q7 | Playing surface | **Full groundskeeping.** One generated field mesh: true mound height, mown stripes, infield dirt with cut-outs, warning track, chalk lines and boxes, on-deck circles, raised bases and rubber. No 3D grass blades. |
| Q8 | Identity | **Fictional park, live board.** Distance markers, invented sponsor signs, a batter's eye, an outfield scoreboard showing the real line score from the sim, a backdrop of trees and hills. No real brands or team names. |
| Q9 | Sim changes | **One read-only export allowed.** A `bb_ffi` call returning the park from `bb_physics::ground::Field`, tested against what the physics uses, digests unchanged, one line in the metrics ledger under "No effect on games". No Codex review. Nothing else in the sim changes in this stage. |
| Q10 | Order | **Field, light, walls, stands** (§3). |
| Q11 | Who builds | **The lead, step by step,** in short sessions. No workers. |
| Q12 | Sign-off | **Show each step.** Before/after screenshots from the fixed camera spots (§4) after each merge. The owner comments when he likes; changes fold into the next step. Work does not wait. |

## 2. Approach

- **One source of shape.** The client's own copies of the dimensions (`FenceAt`, `FoulWallH`, the dugout and screen constants in `BbDirector.cpp`) go away. The park description carries: the fence as a polyline with wall heights, the foul-territory walls, dugout and backstop openings, foul poles, the mound (centre, radius, height profile), the surface regions (infield dirt, grass, warning track), bases, rubber and plate. Anything purely decorative (seat rows, tower positions, sign text) is derived in the client from that description by rule, so a future park with other dimensions draws itself.
- **A park builder apart from the director.** New `BbPark.{h,cpp}` owns the field, walls, stands, scoreboard, crowd and lights. `BbDirector` keeps the play. `BuildField`, `BuildStands` and `BuildLights` move out of it.
- **Generated meshes.** The field is one triangulated surface whose height follows the sim's ground function, so the ball and the players' feet meet the drawn ground. Walls, seating risers, the pressbox and towers are extruded from the park polylines. Repeated things (seats, crowd figures, railing posts) are instanced.
- **Materials.** The engine's shape material takes only a flat colour. Stripes, chalk, signs and per-instance crowd colours need a few base materials (vertex colour, a texture parameter, per-instance colour). **Open technical point, settled in step 1:** either an engine-supplied material that already offers these, or an editor Python script that generates the few base materials into `client/Content/Generated/` as untracked, reproducible build output. Either way the repository stays text-only. Textures (stripes, chalk, numbers, signs) are drawn in code at startup.
- **Budget.** 14 GiB RAM, a Radeon RX 6650-class GPU. No starter content, no photo-scanned assets. The check after every step includes frame time and memory in the headless run.

## 3. Steps

Each step is merged on its own, with screenshots from the spots in §4.

1. **Park export and field surface.** The `bb_ffi` park call and its test; `BbPark`; the generated field mesh with the true mound, dirt cut-outs, warning track, chalk, boxes, on-deck circles, raised bases, rubber and plate; mown stripes. The base-material question is answered here.
2. **Light and materials.** The afternoon sun angle, sky, shadow quality and reach, exposure, a little ambient occlusion, colour grading. A palette for the whole park (grass, dirt, wall, seats, steel, concrete) held in one place.
3. **Walls and field furniture.** The outfield wall with padding and a top rail, distance markers, foul poles with screens, the batter's eye, foul walls, dugouts with benches and steps, the backstop and its net, bullpens if the park description has room for them.
4. **Seating bowl.** One deck of risers and seats following the foul walls, aisles and railings, a concourse roof line, the pressbox behind home, outfield berm or bleachers, light towers (unlit).
5. **Scoreboard, signs, backdrop.** The outfield board with the live line score, count and outs from the sim; invented sponsor signs on the wall; trees and hills beyond the park; flags.
6. **Crowd.** Instanced seated figures over a share of the seats, varied colours, attendance as one number.

Later, outside this plan: night games (towers lit), crowd reaction, per-team colours, more parks.

## 4. Checking the look

- `client/run.sh 7 -RenderOffscreen -BbShots=6` already saves PNGs to `client/Saved/Shots`. Step 1 adds fixed **park camera spots** that do not depend on the play: behind home (broadcast), centre field looking in, high first-base side, upper stands behind third, a low mound close-up, and an overhead plan view. The same spots are used before and after every step.
- The overhead view is also the correctness check: the drawn fence, lines and bases must sit on the sim's coordinates.
- The ball must stay easy to see against the grass, the dirt, the wall and the crowd from the broadcast camera. A step that makes it harder to follow is not done.

## 5. Records

- PLAN D13 is revised by the owner's ruling of 2026-11-15.
- Sim rule for this stage: only the read-only park export (Q9). Its ledger line goes under "No effect on games".
- No Codex reviews in this stage (owner). The rest of the workflow holds: small commits, TASKS.md at the end of each session, a `/clear` point after each merged step.
