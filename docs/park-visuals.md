# Park visuals: field and stadium

**Status:** plan agreed with the owner 2026-11-15 (twelve questions, answers in §1). **Step 1 (park export and field surface) is built** (§6); steps 2 to 6 are not. This is the first part of the graphics stage; players and game graphics (HUD) get their own plans afterwards.

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
- **Materials.** The engine's shape material takes only a flat colour. Stripes, chalk, signs and per-instance crowd colours need a few base materials (vertex colour, a texture parameter, per-instance colour). **Settled in step 1:** the base materials are put together in C++ at start-up (a `UMaterial` with its expressions, compiled by the editor build the client runs in), and nothing is saved. No engine material offers a lit texture parameter, and a script that writes assets would add a build step for no gain. The cost: this needs the editor's material compiler, so a packaged build will need the same code run once at cook time to save the materials. Until then a build without it falls back to flat grass. The repository stays text-only. Textures (stripes, chalk, numbers, signs) are drawn in code at startup.
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

## 6. Step 1 as built (2026-11-15)

**Sim.** `bb_game_park(game, &BbPark)` in `bb_ffi` (`include/bb_ffi.h`): the boundary of the playing field as a closed loop of 75 straight pieces, each with its kind (fence, backstop, side wall, stands closing to a pole, dugout opening) and the height of its top; the foul poles, the three bases, the rubber, the dirt radius, and the four heights. It is filled from the game's own `Field` (`Game::field()`, a new read-only accessor). The test `the_exported_park_is_the_one_the_physics_uses` asks the physics about every piece: both ends lie on the boundary, a step to the field side is inside by that step and a step the other way is outside, the nearest wall there is the kind and height exported, the fence's chords stay within 3 cm of the curve, the loop turns once clockwise, and reading the park leaves the game's log unchanged. No game logic changed.

**Client.** `BbPark.{h,cpp}` holds everything that stands still; `BbDirector` keeps the play. The client's copies of the park's dimensions are gone: walls, poles, dugouts and the placeholder stands are built from the exported pieces, so another park would draw itself.

- **Ground.** One generated mesh (`UProceduralMeshComponent`): the infield in 0.25 m cells, the rest in 2 m cells. Its colours are two pictures painted in code at start-up (4096² each, with their smaller copies): the whole park at 3.7 cm a texel and the infield at 1.4 cm. Painting and meshing take about 1 s.
- **Groundskeeping** follows the Official Baseball Rules' diagram, laid out on the sim's bases and rubber: infield skin to the 95-ft arc from the rubber, grass infield with a 3-ft base path and 13-ft cuts at home and the bases, the mound's 18-ft circle, foul lines in fair ground, batter's and catcher's boxes, the runner's lane, coaches' boxes, on-deck circles, the plate, 18-in bases in their corners, the rubber. The park's own choices, as named constants: a 15-ft warning track under the fence and 10 ft along the foul-ground walls, and mowing in 15-ft bands along both base lines (six to a base path).
- **Camera spots:** `client/run.sh 7 -RenderOffscreen -BbParkShots` saves `park_1_home`, `park_2_center`, `park_3_first_high`, `park_4_third_stands`, `park_5_mound` and `park_6_plan` to `client/Saved/Shots` and quits. The spots are placed from the park's own size.
- **Cost:** 272 frames a second in the unattended run against 277 before; 2.6 GB of memory against 2.4.

**Two things the drawing has that the physics does not.** Both are for the owner to rule on when the physics work resumes (TASKS, Q13 and Q14); neither is read back by the game.

1. **The mound's height.** The sim's ground is level. The drawn mound is the rules' mound: 10 in high, level round the rubber, falling an inch a foot toward the plate. Players are stood on the drawn ground, and a ball on the ground there is drawn on top of it rather than under it. A ball in the air is exactly where the sim has it.
2. **The shape of the infield dirt.** The sim's dirt is a disc 95 ft round home plate: it has dirt where a real park has infield grass, and grass where a real infielder stands at short or second. The drawing is the real layout, so a grounder there bounces by the sim's surface, not the one it is drawn on. The difference between the two surfaces is small (restitution 0.50 against 0.45, rolling 0.22 against 0.28).

**Left for later steps:** the palette and light (step 2); real walls, padding, the screen's net, dugouts (step 3); the placeholder seating blocks, which now simply follow every wall (step 4). The grey ground beyond the stands is the edge of the ground picture.
