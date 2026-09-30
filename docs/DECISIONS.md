# Decisions

# Decision: Initial game direction

## Context

The project needs a clear playable concept before code implementation begins. The current design calls for a board-based liquid simulation with a menu, difficulty settings, selectable board sizes, and pump-based flow generation.

## Decision

The project will begin as a deterministic Rust game with a simple menu flow and a grid-based simulation model. The first version will focus on the core interaction loop: selecting board size, defining pump counts, creating pipe connections by clicking tiles, and updating liquid flow based on connection count.

## Alternatives

Other approaches considered included:

- a more advanced RTS-style game loop
- a full real-time fluid simulation
- a card- or token-based strategy game
- a more elaborate AI-driven opponent model

These were rejected for the first build because they add complexity before the core loop is proven.

## Consequences

Benefits:

- clear technical scope
- deterministic simulation and testing
- simple and testable rules
- ability to iterate quickly

Costs:

- limited gameplay depth at the start
- simple visuals initially
- minimal AI complexity in version 1

## Status

Accepted

## Review

This is the initial accepted decision for the project. It will be revisited once the board mechanic is playable and additional requirements are clarified.

# Decision: Directed pipe links

## Context

Pipe interaction needs unambiguous source and destination semantics. The clicked cell is intended to send liquid toward the adjacent cell selected by the pointer, and visual pipes should make that direction apparent.

## Decision

Represent each adjacent pipe pair as at most one directed link from the clicked/source cell to the selected/destination cell. Clicking either endpoint toggles off an existing link regardless of its orientation; it does not create a reverse link. Queries report only the requested direction, and per-cell connection counts include outgoing links only. Blocking a cell is prohibited when it has any incoming or outgoing link. Each tick adds pump output to pump cells before taking the simulation snapshot and resolving simultaneous flow along directed links. Render each link once from the source center to the shared edge of the destination cell.

## Alternatives

- Keep links bidirectional, which permits unintended pump backflow and does not express pointer direction.
- Permit a separate reverse link between the same pair, which violates the one-link-per-pair interaction rule.
- Draw only a short connector across the shared boundary, which does not clearly identify the source cell.

## Consequences

- Flow direction matches the player's click and is visible in the pipe geometry.
- Pump output reaches an outgoing neighbor during the same tick without using that pipe to flow backward.
- Outgoing-link counts determine splitting; incoming links do not reduce a cell's available output share.
- The board model must inspect neighboring outgoing bits when checking whether a cell can be blocked.

## Status

Accepted

## Review

Review the direction and visual readability after testing the first playable board with users.

# Decision: Deterministic liquid cycle

## Context

The board model needs precise liquid-flow rules before implementing simulation ticks. The game uses discrete reservoirs and pipe links, so flow and side contests must be deterministic and testable.

## Decision

Each cell has a capacity of 160 units. At the start of a tick, each pump with a non-neutral cell side adds up to 8 units using that current side, limited by remaining capacity. The board then takes a stable post-pump snapshot and resolves all pipe transfers simultaneously. A cell distributes at most 8 units across its pipes, retaining at least 1 unit and splitting integer amounts evenly. Incoming amounts are resolved by adding the cell's current-side liquid and subtracting opposing liquid; the remaining side controls the cell. A tie leaves the cell empty and neutral. Pumps in neutral cells emit nothing. Incoming liquid that cannot fit remains at its source. Rendering follows the completed flow resolution; the simulation tick interval is one second.

## Alternatives

- Continuous or fractional fluid simulation
- Pump output fixed to its originally assigned side
- Letting cells empty completely during pipe flow
- Discarding liquid that reaches a full cell

## Consequences

- Integer arithmetic keeps simulation deterministic and easy to test.
- Cells retain a color-bearing unit unless opposing liquid cancels it.
- Pumps can change output side when their cell changes color.
- Pipe destinations at capacity do not destroy liquid, though flow allocation must be deterministic.

## Status

Accepted

## Review

Revisit flow allocation and contention rules after the first playable simulation is available.

# Decision: Macroquad for the 2D application shell

## Context

The project needs a Windows window, menus, tile rendering, and mouse input for a 2D grid game. The domain model and simulation are intentionally independent of the rendering framework.

## Decision

Use Macroquad for the first 2D application shell and board renderer. Keep game state and rules in ordinary Rust modules; Macroquad handles the window, drawing, and input translation only.

## Alternatives

- Bevy, which provides an ECS and broader engine systems beyond the current game's needs
- Continue without a framework, which would require more platform and rendering infrastructure

## Consequences

- Fast path to a visible, interactive Windows prototype
- Small rendering API and no ECS requirement
- Rendering/input code must remain a thin adapter over the board model
- Introduces a third-party runtime dependency

## Status

Accepted

## Review

Revisit if the game grows beyond a focused 2D board experience or needs engine-level systems that Macroquad does not provide.

# Decision: Initial board setup and pipe targeting

## Context

The first rendered prototype needs concrete board presets and a way for pointer input to choose among up to four adjacent pipe targets.

## Decision

Use square presets of 8x8, 12x12, and 16x16. Start with two pumps per side, evenly spaced along opposite left/right edges and excluding corners. Place a deterministic symmetric 2x2 obstacle block at the board center. A click selects the nearest valid orthogonal neighbor by pointer position within the clicked cell; ties resolve North, East, South, West. Blocked and out-of-bounds cells cannot be targets. The rendered prototype advances one simulation tick per second, independent of render frame rate.

## Alternatives

- Random obstacle placement, which would complicate reproducibility and pump placement
- A fixed pipe-direction cycle, which is less spatially direct than clicking toward a neighbor
- Selecting a neighbor from cell occupancy, which would make pointer intent harder to predict

## Consequences

- The first board is reproducible and easy to test.
- Board dimensions and starting layout are implementation defaults that can be adjusted after playtesting.
- Pointer position selects pipe direction without adding a separate direction control.
- A one-second initial tick cadence keeps gameplay moving and remains easy to tune.

## Status

Accepted

## Review

Revisit the central obstacle layout and input targeting after trying the first rendered board.

# Decision: Difficulty selection in New Game setup

## Context

The separate Level menu added a navigation step before starting a game. Difficulty belongs with the other new-game configuration choices.

## Decision

Remove Level from the main menu and present Easy, Medium, and Hard on the New Game setup screen, alongside board size and pump count. Preserve the selection when navigating away and back.

## Alternatives

- Keep a dedicated Level screen reachable from the main menu
- Put difficulty selection in the active game HUD

## Consequences

- New Game setup contains the choices needed to configure a session.
- Difficulty can no longer be changed from the main menu or during a game.
- The selected difficulty remains stored; its gameplay effect is not implemented yet.

## Status

Accepted

## Review

Revisit if difficulty needs to change during active play.

# Decision: Red player and difficulty-scaled blue CPU

## Context

The game has red and blue liquid sides and an Easy/Medium/Hard selection, but no computer-controlled opponent. Pump strength should remain part of the shared rules; difficulty should represent how much the CPU plans.

## Decision

Red is controlled by the human and blue by the CPU. Each simulation cycle, before pump output and pipe flow, the CPU may issue one directed pipe command from a blue-controlled liquid cell. The player may issue commands only from red-controlled cells. Easy uses greedy immediate evaluation, Medium uses short bounded lookahead, and Hard uses deeper bounded lookahead or a larger candidate budget. The CPU seeks to expand blue territory and contest red territory. All difficulty settings preserve identical pump and flow rules.

## Alternatives

- Change pump output by difficulty, which would confound AI skill with game rules
- Let the CPU alter the board every render frame, which would be frame-rate-dependent and unfair
- Couple opponent logic to rendering and mouse input

## Consequences

- AI can be tested deterministically against board states.
- Difficulty is expressed as planning effort and remains bounded on large boards.
- A territory-pressure heuristic is provisional until win conditions are specified.

## Status

Accepted

## Review

Revisit the heuristic and decision cadence when explicit win conditions and playtest feedback are available.
