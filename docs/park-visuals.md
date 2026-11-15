# Park visuals: field and stadium

**Status:** plan agreed with the owner 2026-11-15 (twelve questions, answers in §1). **Step 1 (park export and field surface) is built** (§6), **step 2 (light and materials)** (§7), **step 3 (walls and field furniture)** (§8), **step 4 (the seating bowl)** (§9), **step 5 (scoreboard, signs, backdrop)** (§10) and **step 6 (the crowd)** (§11): the plan is complete. This is the first part of the graphics stage; players and game graphics (HUD) get their own plans afterwards.

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
- **Groundskeeping** follows the Official Baseball Rules' diagram, laid out on the sim's bases and rubber: infield skin to the 95-ft arc from the rubber, grass infield with a 3-ft base path and 13-ft cuts at home and the bases, the mound's 18-ft circle, foul lines in fair ground, batter's and catcher's boxes, the runner's lane, coaches' boxes, on-deck circles, the plate, 18-in bases in their corners, the rubber. The park's own choices, as named constants: a 15-ft warning track under the fence and 10 ft along the foul-ground walls, and the mowing (owner, 2026-11-15): the infield grass one shade, and everything outside the infield square in plain 15-ft stripes running from home toward centre field, one centred on the line through the plate and second base.
- **Camera spots:** `client/run.sh 7 -RenderOffscreen -BbParkShots` saves `park_1_home`, `park_2_center`, `park_3_first_high`, `park_4_third_stands`, `park_5_mound` and `park_6_plan` to `client/Saved/Shots` and quits. The spots are placed from the park's own size.
- **Cost:** 272 frames a second in the unattended run against 277 before; 2.6 GB of memory against 2.4.

**Two things the drawing has that the physics does not.** The owner ruled on both on 2026-11-15 (Q13 and Q14, below); neither is read back by the game until that work is done.

1. **The mound's height.** The sim's ground is level. The drawn mound is the rules' mound: 10 in high, level round the rubber, falling an inch a foot toward the plate. Players are stood on the drawn ground, and a ball on the ground there is drawn on top of it rather than under it. A ball in the air is exactly where the sim has it.
2. **The shape of the infield dirt.** The sim's dirt is a disc 95 ft round home plate: it has dirt where a real park has infield grass, and grass where a real infielder stands at short or second. The drawing is the real layout, so a grounder there bounces by the sim's surface, not the one it is drawn on. The difference between the two surfaces is small (restitution 0.50 against 0.45, rolling 0.22 against 0.28).

**Rulings (owner, 2026-11-15).** Both wait for the physics work and its reviews; nothing in the sim changes in this stage (Q9).

- **Q13, the mound:** a drawing for now. The physics gets the mound's height (bounces and fielders' footing) with the fielding work, since it changes comebackers and needs a review.
- **Q14, the dirt:** the physics takes the drawn layout (skin, infield grass, warning track) when reviews return, with a ledger row and a review.

**Left for later steps:** the palette and light (step 2); real walls, padding, the screen's net, dugouts (step 3); the placeholder seating blocks, which now simply follow every wall (step 4). The grey ground beyond the stands is the edge of the ground picture.

## 7. Step 2 as built (2026-11-15)

Client only; nothing in the sim changed.

**One place for the look: `client/Source/BaseballClient/BbLook.h`.** Every colour of anything that stands still, the sun, the sky's fill, the shadows and the grade are named constants there. `BbPark.cpp` names no colour of its own any more.

- **Palette.** Grass (two mowing shades), dirt, warning track, chalk, the land outside; and, each with a colour and how matt it is: bases and rubber, wall padding, painted steel, foul poles, seats, concrete, roofs, dugout floors. **Proposed (the lead's choice, the owner has not ruled):** wall padding is dark green (it was navy) so the white ball shows against it, and the seats are a muted blue.
- **Materials.** A second material built in code at start-up beside the ground's: paint, with a colour and a roughness (the engine's shape material has only a colour). Steel and seats take a slight sheen; ground, concrete and chalk are matt.
- **Sun.** Mid-afternoon: 42° above the horizon (it was 55°), over the third-base side and a little toward the outfield, slightly warm. Shadows fall toward first base, about as long as a player is tall, and both the camera behind home and the one in centre field see lit faces. The template map's own sun and sky are now set to these values rather than left as they came.
- **Sky.** The sky's fill is about twice what it was, with green light coming back up off the grass, so faces turned from the sun are shaded, not black.
- **Shadows.** Four cascades reaching 320 m at 4096 texels (three at 2048 before). The seating blocks now cast shadows; in the afternoon the third-base stands shade their own foul ground and the left-field track.
- **Picture.** One unbound post-process component on the park, so every camera gets the same: fixed exposure (−0.75 stops; the director's camera no longer sets its own), ambient occlusion (0.6, 90 cm), saturation ×1.07, contrast ×1.05, low bloom, a slight vignette.
- **The land** now runs to the horizon (the dark band under the sky in step 1's pictures was the edge of it).

**Checked** from the six spots and six pictures of play (`client/Saved/Shots/step2_before/`, `step2_after/`, untracked). The ball against the green wall, the blue seats, grass and dirt from the centre-field camera: easy to see. **Cost:** 257 frames a second unattended against 272; memory 2.6 GB, unchanged.

**Not done here:** shadows seen through the centre-field camera's long lens have soft edges (the far cascade); grass and dirt are still flat colour with no grain. Both can be taken up when the walls and seats are real (steps 3 and 4), if the owner wants them.

## 8. Step 3 as built (2026-11-15)

Client only; nothing in the sim changed. Everything is placed by rule from the exported park, so another park would get the same furniture in its own places. The measures are named constants at the top of `BbPark.cpp`; the colours are in `BbLook.h`.

- **Walls.** Every wall's face toward the field is now exactly on the sim's boundary, with its thickness behind (before, the walls were centred on the line, so a ball at the wall sank 15 cm into the padding).
- **Outfield wall.** Padding, with a yellow line along the top at the sim's height: over it is a home run.
- **Distance figures,** in feet, white block figures painted in code: at each line (the pole's distance, 330), in each alley (the fence's distance where the figure stands, 381) and at centre (400).
- **Foul poles** carry a screen on the fair side: a ladder of rungs from above the fence to the top.
- **Batter's eye.** A dark matt wall 11 m high behind the fence, 8° either side of dead centre.
- **Bullpens.** One behind the fence in each alley, from 8° to 22° off centre, 7 m deep: grass, two mounds and two plates 60 ft 6 in apart along the fence, a bench, a padded wall round it. A park whose fence has no room there gets none.
- **Dugouts.** The sim has the opening in the stands' wall and a roof 4 ft above the field (a ball under it goes in), so the dugout is sunk: three steps down along the whole opening to a floor 0.9 m below the field, walls, a bench against the rear wall, the roof with its top where the sim has it, posts under its front edge, and a concrete lip along the field's edge. The ground mesh is open over the pit, and the drawn ground's height has the steps, so a player walking off goes down them and under the roof.
- **Backstop.** The low padded wall, posts every 7 m or so, a top rail, a cable half-way up, and the net: a see-through unlit sheet (a third material built in code) that greys a little what is behind it.
- **Placeholder stands** stay (step 4 replaces them), but leave the batter's eye and the pens clear.

**Not done, and why.** The dugout has no rail in front: the sim's opening is open, and a ball passing through a drawn rail would look wrong. The pens' mounds are flat dirt circles, not raised. The pens are decoration: nobody warms up in them yet.

**Checked** from the six spots and six pictures of play (`client/Saved/Shots/step3_before/`, `step3_after/`, untracked): the pitch camera's view is clear of the batter's eye, and the ball is as easy to see as before. **Cost:** 221 frames a second unattended against 257 (about a hundred more small shapes, each its own actor, and the net; step 4's instancing is the place to win it back); memory 2.6 GB, unchanged.

## 9. Step 4 as built (2026-11-15)

Client only; nothing in the sim changed. The placeholder blocks are gone. Everything follows the exported boundary by rule; the measures are named constants above `BuildStands` in `BbPark.cpp`, the colours in `BbLook.h`.

- **How it is made.** A cross-section (distance behind the wall, height) is carried along each piece of the boundary into one generated mesh; where two pieces meet at a corner both end on the line that halves it, so rows turn the corner without a gap. Seats, steps, posts, benches and lamps are instanced shapes. The same now holds for step 3's walls and furniture (they were an actor each).
- **The deck.** 18 rows behind every wall of foul ground, pole to pole: 0.84 m deep, 0.36 m up each, the wall's top a parapet in front of the first row. 7,038 seats (a bottom and a back each), in sections of about 9 m between aisles with a half step on every row, and an aisle at every corner. Behind the top row a concourse, a rail, and the back wall down to the land.
- **Over the dugouts** the first five rows are left out: the dugout's roof is there, and the seats start behind it, as the sim has it (a ball over the roof goes into the seats).
- **Roof.** A slab over the concourse and the last six rows, higher at the front, on posts at the aisles. It stops where the stands close in to the poles, so the seats down the lines are open.
- **Pressbox.** On the roof behind home: a room with a band of glass toward the field, and its building behind the concourse.
- **Beyond the fence.** Left field: a grass bank with a path along its top. Right field: ten rows of benches. Both leave the batter's eye and the pens clear.
- **Light towers.** Six, unlit: 32 m poles behind the stands at 32°, 78° and 125° either side of centre field, each with a rack of lamps turned to the middle of the field.
- **The third-base camera spot moved** into the seats, half-way up the deck (a light tower and the roof now stand where it was). The other five are unchanged.

**The lead's choices, for the owner to overrule if he likes:** the bank in left and the benches in right (it could be either on both sides); the roof stopping short of the poles; seats one colour.

**Not done.** No rail along the aisles or the front of the deck. The stands at each pole end square, leaving a wedge of open ground between the deck and the bank or benches. The towers are plain poles, not lattice.

**Checked** from the six spots and six pictures of play (`client/Saved/Shots/step3_after/` is the before, `step4_after/` the after; untracked): the pitch camera sees the ball against the seats behind home as easily as before, and the camera behind home is clear of the roof. **Cost:** 207 frames a second unattended against 221; memory 2.6 GB, unchanged. Drawing step 3's shapes instanced did not win the frames back as expected (207 with and without), so the cost is not the number of actors; shadows are the likely place to look if it matters.

## 10. Step 5 as built (2026-11-15)

Client only; nothing in the sim changed. Everything is placed by rule from the exported park; the measures are named constants in `BbPark.cpp`, the colours, the park's name and the sponsors' names in `BbLook.h`.

- **Letters.** `BbText.h`: a canvas of pixels painted in code, with block capitals, figures and a few marks from a five-by-seven dot alphabet. No font or picture is loaded from a file. Pictures need no longer be square, and one can be painted over again (`FillTexture`).
- **The scoreboard.** Behind the right-centre bullpen (15° off centre field), turned to the infield: a dark case 17 m by 9 m on two steel legs, its foot 7.5 m up, clear of the batter's eye. Its face is a picture of lamps that give their own light (a fourth material built in code, unlit), so it reads though the afternoon sun is behind it. It shows the park's name, **the line score from the sim's box score** (nine innings, the last nine in extra innings; runs and hits), a lamp beside the side at bat, and the count and the outs as lamps (three balls, two strikes, two outs). The director reads the box score once a pitch, through the pitches whose result is known, and gives the board the same inning, count and outs as the text line at the top of the screen; the picture is repainted only when something on it has changed.
- **The park's name: Cedar Hollow Park** (invented; a constant in `BbLook.h`).
- **Sponsors' signs.** Ten invented names (none a real firm's), each a dark ground with pale block capitals inside a line. They run along the fence wherever it carries no distance figure, clear of the batter's eye: each at least 7.5 m long over as many pieces of the fence as that takes, bending with it, a plain piece of padding between one and the next, 0.3 m of padding above and below. About ten signs in this park.
- **Flags.** Two on the scoreboard and a court of three poles at the same bearing in left-centre, behind the pen: plain fields with a pale band. They stand stiff toward one bearing; the sim has no wind.
- **Trees.** About 650 in 110 clumps, broadleaf (a ball on a trunk, three greens) and conifer (a cone), from 32 m outside the field's boundary (behind the stands and what stands with them) out to 150 m. The same wood every time (a fixed seed in `BbLook.h`; this is the client's decoration, not the sim's randomness).
- **Hills.** Two rings round the park, 650 m and 1,150 m out, the far one higher, paler and bluer; each ridge's height wanders with the bearing.
- **A seventh camera spot,** `park_7_board`: the scoreboard from the outfield grass in front of it. The other six are unchanged.

**The lead's choices, for the owner to overrule if he likes:** the park's name and the sponsors' names; the board in right-centre and the flags in left-centre; the board's layout (no errors column, no pitch speed, no batter's name); the inning now being batted shows its runs so far (0 at first) rather than a blank.

**Not done.** The flags do not move. The home side's unplayed last half shows a blank, not an "X". Trees are balls and cones on sticks. The signs' letters are the same dot alphabet as the board's, so they are blocky.

**Checked** from the seven spots and six pictures of play (`client/Saved/Shots/step4_after/` is the before, `step5_after/` the after; untracked): the board showed the count the text line showed; the pitch camera's view is unchanged, and from behind home a ball in the air is now against dark trees and hills rather than a flat horizon. **Cost:** 209 frames a second unattended against 207; memory 2.7 GB (2.6).

## 11. Step 6 as built (2026-11-15)

Client only; nothing in the sim changed. The measures are named constants above `BuildStands` in `BbPark.cpp`; the attendance, the seed and the clothes' colours are in `BbLook.h`.

- **Attendance is one number: 6,200** (`BbLook::Attendance`), in 8,114 places: the 7,038 seats, a place every 0.55 m along the benches beyond right field, and places scattered over the grass bank beyond left.
- **Who sits where.** Each place has a weight for how sought-after it is: a seat counts for more in the low rows (the top row 0.45 of the first) and near the plate (falling to 0.4 far down the lines); a bench place 0.55, a place on the bank 0.4. The places are filled by weighted draw without replacement until the attendance is reached, so exactly that many figures are drawn and the empty seats are mostly high and far. The same crowd every time (a fixed seed; this is the client's decoration, not the sim's randomness).
- **A figure** is five boxes: shins, thighs, a body, a head, and hair or a cap. Fourteen shirt colours, four skins, seven heads (four hair, three caps), three trousers; each figure is a little taller or shorter and turned a few degrees. On the bank the legs lie down the slope. All are instanced: 31,000 boxes in 28 more batches.
- **No animation, no shadows cast** (as the seats).
- The seat the third-base camera spot sits in, and those within 1.7 m, stay empty.

**The lead's choices, for the owner to overrule if he likes:** the attendance (about three quarters full); one pale shirt colour in fourteen, the rest mid or dark; no home-team colours in the crowd.

**Not done.** Nobody stands, walks the concourse or sits in the dugouts or pens. No reaction to the play. Heads are boxes; close to the camera (the third-base spot) the figures are plainly blocks.

**Checked** from the seven spots and six pictures of play (`client/Saved/Shots/step5_after/` is the before, `step6_after/` the after; untracked). From the pitch camera the ball crosses grass, dirt and the dark wall, below the crowd, and is as easy to see as before. A high fly seen against the seats is now against many colours rather than plain blue; not judged from a picture yet, to be watched in play. **Cost:** 193 frames a second unattended against 209; memory 2.6 GB (2.7).
