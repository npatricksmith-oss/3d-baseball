# A4 live-ball regression fixtures

Frozen from main `9fdeec169a0ba522bf7b165da425e2d6274e1ea3`, before the fixes.
The numbers are the original pitch's executed contact and public starting rules
situation, captured via `Game::step_pitch`. No alternate outcome was generated.
Use the original seed/play for all keyed input noise. Reconstruct ordinary
average fielders with their generated roster hands, default park/atmosphere and
8.2 m/s runners. Each fixture starts before `BatContact`, independently of any
future changed game's earlier pitches.

- `1-64`: stop at third must not create a home destination.
- `13-140`: tied sixth, one out and loaded; compare the routine force home
  against carrying first, which removes the other forces.
- `70-157`: tied seventh, no outs, runners first/third; preserve a likely
  carry-second/throw-first double play against a single home tag.
- `17-203`: third baseman must not carry to unforced third outside tag reach
  instead of the available first-base force.

Expectations are the tactical cases in agent-models §5.6, not saved chosen
destinations or simulation output hashes. `audit_intended_goal` in the separate
diagnostic trace is never part of a fielder's inputs.
