# HUD: game graphics

**Status:** plan agreed with the owner 2026-11-15 (three questions, Q22–Q24, all as recommended; §1). Nothing built yet. Third part of the graphics stage, after [the park](park-visuals.md) and [the players](player-visuals.md).

**Where it starts.** `ABbHUD::DrawHUD` (`BbGameMode.cpp`) draws a dark strip with one line of score text, one line about the last pitch, a three-square runners diamond, a footer of keys, the between-innings card and the home-run card. All of it is the engine's large font on the canvas.

## 1. Decisions (owner, 2026-11-15)

| # | Question | Ruling |
|---|---|---|
| Q22 | Style | **A broadcast-style score bug** in a corner: clubs, score, inning and half, the diamond, count, outs. Drawn in code. |
| Q23 | What else shows | **The full set, built in this order:** the bug and the play caption; a batter card when he steps in (name, number, position, his day so far); the pitch's speed and type after each pitch; the pitcher's pitch count; a strike-zone box with the pitch's location, on a key; a line-score card between innings; a full box score at the end. |
| Q24 | Lettering | **The engine's built-in font** for the HUD. The dot alphabet stays on the scoreboard. |

Carried over from the park plan: the lead builds alone in short sessions, shows screenshots after each merge, no Codex reviews, nothing binary committed.

## 2. Approach

- **Everything shown is read from the sim,** never counted in the client: the score, the line score and the batters' lines from `bb_game_box_score` (statistics are functions of the event log, invariant 6); names, numbers and positions from `bb_game_roster`; the pitch's speed, type and plate location from the timeline. If a figure the HUD needs is not exported (the pitcher's pitch count, the pitch's type), it is added as a read-only export under the players plan's rule (Q21): digests unchanged, one ledger line.
- **A HUD apart from the game mode.** New `BbHud.{h,cpp}` holds the panels; each panel is a small function over one plain struct of what to show, so the director fills structs and never draws.
- **One place for the look.** Panel colours, sizes and the clubs' colours come from `BbLook.h`. The clubs' names and short forms come from the players plan (§5 there).
- **Canvas drawing,** scaled from a 720-line layout as now. No widget assets.
- **The scoreboard in the park keeps working** and must agree with the bug: both read the same box score.

## 3. Steps

1. **The bug and the caption.** Replaces the strip. The home-run card and the between-innings card are restyled to match.
2. **The batter card and the pitch readout.** Card on stepping in, gone at the first pitch; speed and type after each pitch; the pitch count beside the pitcher's name.
3. **The strike-zone box.** The batter's own zone from the sim, each pitch of the at-bat marked where it crossed, called strikes and balls told apart; a key turns it on and off.
4. **Line score between innings; box score at the end.**

## 4. Checking the look

- `client/run.sh 7 -RenderOffscreen -BbShots=6`, read at 720 lines and at 1080.
- A flag that holds each panel on screen for a still, so a panel that shows for two seconds can be photographed without hunting for the moment.
- The bug must not cover the ball's usual paths from the broadcast camera.
