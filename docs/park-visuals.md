# Park visuals: field and stadium

**Status:** plan agreed with the owner 2026-11-15 (twelve questions, answers in §1). **Step 1 (park export and field surface) is built** (§6), **step 2 (light and materials)** (§7), **step 3 (walls and field furniture)** (§8), **step 4 (the seating bowl)** (§9), **step 5 (scoreboard, signs, backdrop)** (§10) and **step 6 (the crowd)** (§11): the plan is complete. This is the first part of the graphics stage; the players ([player-visuals.md](player-visuals.md)) and the game graphics ([hud.md](hud.md)) have their own plans.

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

- **Park geometry.** `bb_game_park` exports the boundary, wall heights, openings, poles, bases, rubber and dirt radius. `ABbPark::Build` uses it for the field and stands. The drawn mound and detailed surface layout are still client geometry (Q13 and Q14 in §6).
- **Generated meshes.** `ABbPark::BuildSurface` triangulates the drawing's ground. Seats and crowd figures use instanced meshes.

- **Materials.** The engine's shape material takes only a flat colour. Stripes, chalk, signs and per-instance crowd colours need a few base materials (vertex colour, a texture parameter, per-instance colour). **Settled in step 1:** the base materials are put together in C++ at start-up (a `UMaterial` with its expressions, compiled by the editor build the client runs in), and nothing is saved. No engine material offers a lit texture parameter, and a script that writes assets would add a build step for no gain. The cost: this needs the editor's material compiler, so a packaged build will need the same code run once at cook time to save the materials. Until then a build without it falls back to flat grass. The repository stays text-only. Textures (stripes, chalk, numbers, signs) are drawn in code at startup.
- **Budget.** 14 GiB RAM, a Radeon RX 6650-class GPU. No starter content, no photo-scanned assets. The check after every step includes frame time and memory in the headless run.

## 3. Steps

The six built steps are field and park export (§6), lighting (§7), walls and field furniture (§8), seating (§9), scoreboard and backdrop (§10), and crowd (§11). Presentation follow-ups are in [TASKS.md](../TASKS.md).

## 4. Checking the look

- `client/run.sh 7 -RenderOffscreen -BbShots=6` already saves PNGs to the generated Shots folder. `-BbParkShots` uses fixed **park camera spots** that do not depend on the play: behind home (broadcast), centre field looking in, high first-base side, upper stands behind third, a low mound close-up, an overhead plan view, and a scoreboard close-up. The same spots are used before and after every step.
- The overhead view is also the correctness check: the drawn fence, lines and bases must sit on the sim's coordinates.
- The ball must stay easy to see against the grass, the dirt, the wall and the crowd from the broadcast camera. A step that makes it harder to follow is not done.

## 5. Records

- PLAN D13 is revised by the owner's ruling of 2026-11-15.
- Sim rule for this stage: only the read-only park export (Q9). Its ledger line goes under "No effect on games".
- No Codex reviews in this stage (owner). The rest of the workflow holds: small commits, TASKS.md at the end of each session, a `/clear` point after each merged step.

## 6. Step 1 as built (2026-11-15)

**Sim.** `bb_game_park(game, &BbPark)` in `bb_ffi` (`sim/crates/bb_ffi/include/bb_ffi.h`): the boundary of the playing field as a closed loop of 75 straight pieces, each with its kind (fence, backstop, side wall, stands closing to a pole, dugout opening) and the height of its top; the foul poles, the three bases, the rubber, the dirt radius, and the four heights. It is filled from the game's own `Field` (`Game::field()`, a new read-only accessor). The test `the_exported_park_is_the_one_the_physics_uses` asks the physics about every piece: both ends lie on the boundary, a step to the field side is inside by that step and a step the other way is outside, the nearest wall there is the kind and height exported, the fence's chords stay within 3 cm of the curve, the loop turns once clockwise, and reading the park leaves the game's log unchanged. No game logic changed.

**Client.** `client/Source/BaseballClient/BbPark.cpp` holds everything that stands still; `BbDirector` keeps the play. Walls, poles, dugouts and stands are built from the exported pieces.

- **Ground.** One generated mesh (`UProceduralMeshComponent`): the infield in 0.25 m cells, the rest in 2 m cells. Its colours are two pictures painted in code at start-up (4096² each, with their smaller copies): the whole park at 3.7 cm a texel and the infield at 1.4 cm. Painting and meshing take about 1 s.
- **Groundskeeping** follows the Official Baseball Rules' diagram, laid out on the sim's bases and rubber: infield skin to the 95-ft arc from the rubber, grass infield with a 3-ft base path and 13-ft cuts at home and the bases, the mound's 18-ft circle, foul lines in fair ground, batter's and catcher's boxes, the runner's lane, coaches' boxes, on-deck circles, the plate, 18-in bases in their corners, the rubber. The park's own choices, as named constants: a 15-ft warning track under the fence and 10 ft along the foul-ground walls, and the mowing (owner, 2026-11-15): the infield grass one shade, and everything outside the infield square in plain 15-ft stripes running from home toward centre field, one centred on the line through the plate and second base.
- **Camera spots:** `client/run.sh 7 -RenderOffscreen -BbParkShots` saves `park_1_home`, `park_2_center`, `park_3_first_high`, `park_4_third_stands`, `park_5_mound` `park_6_plan` and `park_7_board` to the generated Shots folder and quits. The spots are placed from the park's own size.
- **Cost:** 272 frames a second in the unattended run against 277 before; 2.6 GB of memory against 2.4.

**Two things the drawing has that the physics does not.** The owner ruled on both on 2026-11-15 (Q13 and Q14, below); neither is read back by the game until that work is done.

1. **The mound's height.** The sim's ground is level. The drawn mound is the rules' mound: 10 in high, level round the rubber, falling an inch a foot toward the plate. Players are stood on the drawn ground, and a ball on the ground there is drawn on top of it rather than under it. A ball in the air is exactly where the sim has it.
2. **The shape of the infield dirt.** The sim's dirt is a disc 95 ft round home plate: it has dirt where a real park has infield grass, and grass where a real infielder stands at short or second. The drawing is the real layout, so a grounder there bounces by the sim's surface, not the one it is drawn on. The difference between the two surfaces is small (restitution 0.50 against 0.45, rolling 0.22 against 0.28).

**Rulings (owner, 2026-11-15).** Both wait for the physics work and its reviews; nothing in the sim changes in this stage (Q9).

- **Q13, the mound:** a drawing for now. The physics gets the mound's height (bounces and fielders' footing) with the fielding work, since it changes comebackers and needs a review.
- **Q14, the dirt:** the physics takes the drawn layout (skin, infield grass, warning track) when reviews return, with a ledger row and a review.

## 7. Step 2 as built (2026-11-15)

Client only; nothing in the sim changed.

**One place for the look: `client/Source/BaseballClient/BbLook.h`.** Every colour of anything that stands still, the sun, the sky's fill, the shadows and the grade are named constants there. `client/Source/BaseballClient/BbPark.cpp` names no colour of its own any more.

- **Palette.** Grass (two mowing shades), dirt, warning track, chalk, the land outside; and, each with a colour and how matt it is: bases and rubber, wall padding, painted steel, foul poles, seats, concrete, roofs, dugout floors. **Proposed (the lead's choice, the owner has not ruled):** wall padding is dark green (it was navy) so the white ball shows against it, and the seats are a muted blue.
- **Materials.** A second material built in code at start-up beside the ground's: paint, with a colour and a roughness (the engine's shape material has only a colour). Steel and seats take a slight sheen; ground, concrete and chalk are matt.
- **Sun.** Mid-afternoon: 42° above the horizon (it was 55°), over the third-base side and a little toward the outfield, slightly warm. Shadows fall toward first base, about as long as a player is tall, and both the camera behind home and the one in centre field see lit faces. The template map's own sun and sky are now set to these values rather than left as they came.
- **Sky.** The sky's fill is about twice what it was, with green light coming back up off the grass, so faces turned from the sun are shaded, not black.
- **Shadows.** Four cascades reaching 320 m at 4096 texels (three at 2048 before). The seating blocks now cast shadows; in the afternoon the third-base stands shade their own foul ground and the left-field track.
- **Picture.** One unbound post-process component on the park, so every camera gets the same: fixed exposure (−0.75 stops; the director's camera no longer sets its own), ambient occlusion (0.6, 90 cm), saturation ×1.07, contrast ×1.05, low bloom, a slight vignette.
- **The land** now runs to the horizon (the dark band under the sky in step 1's pictures was the edge of it).


## 8. Step 3 as built (2026-11-15)

Client only; nothing in the sim changed. Everything is placed by rule from the exported park, so another park would get the same furniture in its own places. The measures are named constants at the top of `client/Source/BaseballClient/BbPark.cpp`; the colours are in `client/Source/BaseballClient/BbLook.h`.

- **Walls.** Every wall's face toward the field is now exactly on the sim's boundary, with its thickness behind (before, the walls were centred on the line, so a ball at the wall sank 15 cm into the padding).
- **Outfield wall.** Padding, with a yellow line along the top at the sim's height: over it is a home run.
- **Distance figures,** in feet, white block figures painted in code: at each line (the pole's distance, 330), in each alley (the fence's distance where the figure stands, 381) and at centre (400).
- **Foul poles** carry a screen on the fair side: a ladder of rungs from above the fence to the top.
- **Batter's eye.** A dark matt wall 11 m high behind the fence, 8° either side of dead centre.
- **Bullpens.** One behind the fence in each alley, from 8° to 22° off centre, 7 m deep: grass, two mounds and two plates 60 ft 6 in apart along the fence, a bench, a padded wall round it. A park whose fence has no room there gets none.
- **Dugouts.** The sim has the opening in the stands' wall and a roof 4 ft above the field (a ball under it goes in), so the dugout is sunk: three steps down along the whole opening to a floor 0.9 m below the field, walls, a bench against the rear wall, the roof with its top where the sim has it, posts under its front edge, and a concrete lip along the field's edge. The ground mesh is open over the pit, and the drawn ground's height has the steps, so a player walking off goes down them and under the roof.
- **Backstop.** The low padded wall, posts every 7 m or so, a top rail, a cable half-way up, and the net: a see-through unlit sheet (a third material built in code) that greys a little what is behind it.
- **Placeholder stands** stay (step 4 replaces them), but leave the batter's eye and the pens clear.


## 9. Step 4 as built (2026-11-15)

Client only; nothing in the sim changed. The placeholder blocks are gone. Everything follows the exported boundary by rule; the measures are named constants above `BuildStands` in `client/Source/BaseballClient/BbPark.cpp`, the colours in `client/Source/BaseballClient/BbLook.h`.

- **How it is made.** A cross-section (distance behind the wall, height) is carried along each piece of the boundary into one generated mesh; where two pieces meet at a corner both end on the line that halves it, so rows turn the corner without a gap. Seats, steps, posts, benches and lamps are instanced shapes. The same now holds for step 3's walls and furniture (they were an actor each).
- **The deck.** 18 rows behind every wall of foul ground, pole to pole: 0.84 m deep, 0.36 m up each, the wall's top a parapet in front of the first row. 7,038 seats (a bottom and a back each), in sections of about 9 m between aisles with a half step on every row, and an aisle at every corner. Behind the top row a concourse, a rail, and the back wall down to the land.
- **Over the dugouts** the first five rows are left out: the dugout's roof is there, and the seats start behind it, as the sim has it (a ball over the roof goes into the seats).
- **Roof.** A slab over the concourse and the last six rows, higher at the front, on posts at the aisles. It stops where the stands close in to the poles, so the seats down the lines are open.
- **Pressbox.** On the roof behind home: a room with a band of glass toward the field, and its building behind the concourse.
- **Beyond the fence.** Left field: a grass bank with a path along its top. Right field: ten rows of benches. Both leave the batter's eye and the pens clear.
- **Light towers.** Six, unlit: 32 m poles behind the stands at 32°, 78° and 125° either side of centre field, each with a rack of lamps turned to the middle of the field.
- **The third-base camera spot moved** into the seats, half-way up the deck (a light tower and the roof now stand where it was). The other five are unchanged.

**The lead's choices, for the owner to overrule if he likes:** the bank in left and the benches in right (it could be either on both sides); the roof stopping short of the poles; seats one colour.


## 10. Step 5 as built (2026-11-15)

Client only; nothing in the sim changed. Everything is placed by rule from the exported park; the measures are named constants in `client/Source/BaseballClient/BbPark.cpp`, the colours, the park's name and the sponsors' names in `client/Source/BaseballClient/BbLook.h`.

- **Letters.** `client/Source/BaseballClient/BbText.h`: a canvas of pixels painted in code, with block capitals, figures and a few marks from a five-by-seven dot alphabet. No font or picture is loaded from a file. Pictures need no longer be square, and one can be painted over again (`FillTexture`).
- **The scoreboard.** Behind the right-centre bullpen (15° off centre field), turned to the infield: a dark case 17 m by 9 m on two steel legs, its foot 7.5 m up, clear of the batter's eye. Its face is a picture of lamps that give their own light (a fourth material built in code, unlit), so it reads though the afternoon sun is behind it. It shows the park's name, **the line score from the sim's box score** (nine innings, the last nine in extra innings; runs and hits), a lamp beside the side at bat, and the count and the outs as lamps (three balls, two strikes, two outs). The director reads the box score once a pitch, through the pitches whose result is known, and gives the board the same inning, count and outs as the text line at the top of the screen; the picture is repainted only when something on it has changed.
- **The park's name: Cedar Hollow Park** (invented; a constant in `client/Source/BaseballClient/BbLook.h`).
- **Sponsors' signs.** Ten invented names (none a real firm's), each a dark ground with pale block capitals inside a line. They run along the fence wherever it carries no distance figure, clear of the batter's eye: each at least 7.5 m long over as many pieces of the fence as that takes, bending with it, a plain piece of padding between one and the next, 0.3 m of padding above and below. About ten signs in this park.
- **Flags.** Two on the scoreboard and a court of three poles at the same bearing in left-centre, behind the pen: plain fields with a pale band. They stand stiff toward one bearing; the sim has no wind.
- **Trees.** About 650 in 110 clumps, broadleaf (a ball on a trunk, three greens) and conifer (a cone), from 32 m outside the field's boundary (behind the stands and what stands with them) out to 150 m. The same wood every time (a fixed seed in `client/Source/BaseballClient/BbLook.h`; this is the client's decoration, not the sim's randomness).
- **Hills.** Two rings round the park, 650 m and 1,150 m out, the far one higher, paler and bluer; each ridge's height wanders with the bearing.
- **A seventh camera spot,** `park_7_board`: the scoreboard from the outfield grass in front of it. The other six are unchanged.

**The lead's choices, for the owner to overrule if he likes:** the park's name and the sponsors' names; the board in right-centre and the flags in left-centre; the board's layout (no errors column, no pitch speed, no batter's name); the inning now being batted shows its runs so far (0 at first) rather than a blank.


## 11. Step 6 as built (2026-11-15)

Client only; nothing in the sim changed. The measures are named constants above `BuildStands` in `client/Source/BaseballClient/BbPark.cpp`; the attendance, the seed and the clothes' colours are in `client/Source/BaseballClient/BbLook.h`.

- **Attendance is one number: 6,200** (`BbLook::Attendance`), in 8,114 places: the 7,038 seats, a place every 0.55 m along the benches beyond right field, and places scattered over the grass bank beyond left.
- **Who sits where.** Each place has a weight for how sought-after it is: a seat counts for more in the low rows (the top row 0.45 of the first) and near the plate (falling to 0.4 far down the lines); a bench place 0.55, a place on the bank 0.4. The places are filled by weighted draw without replacement until the attendance is reached, so exactly that many figures are drawn and the empty seats are mostly high and far. The same crowd every time (a fixed seed; this is the client's decoration, not the sim's randomness).
- **A figure** is five boxes: shins, thighs, a body, a head, and hair or a cap. Fourteen shirt colours, four skins, seven heads (four hair, three caps), three trousers; each figure is a little taller or shorter and turned a few degrees. On the bank the legs lie down the slope. All are instanced: 31,000 boxes in 28 more batches.
- **No animation, no shadows cast** (as the seats).
- The seat the third-base camera spot sits in, and those within 1.7 m, stay empty.

**The lead's choices, for the owner to overrule if he likes:** the attendance (about three quarters full); one pale shirt colour in fourteen, the rest mid or dark; no home-team colours in the crowd.
