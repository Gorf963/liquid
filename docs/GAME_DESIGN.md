# Game Design

## Game concept

This is a single-player strategy / simulation board game where each cell represents a liquid reservoir and pumps inject colored liquid into the board. Players manipulate pipes by clicking tiles, creating or removing connections between adjacent cells.

The goal is to manage the flow of colored liquids across the board while balancing each side's pump output and the network layout.

## Initial menu flow

### Main menu

Options:

- New Game
- Exit Game

### New Game setup

The New Game screen includes difficulty selection:

Difficulty choices:

- Easy
- Medium
- Hard

The difficulty should affect opponent strength, pump output, or board challenge level. The same screen asks for:

- Board size: Small / Medium / Large
- Number of pumps on each side
- Optional starting configuration or defaults

Initial board presets are Small 8x8, Medium 12x12, and Large 16x16. The initial pump count is two per side and can be adjusted before starting. Side One's pumps are evenly spaced along the left edge; Side Two's pumps are evenly spaced along the right edge. Pump positions exclude the corners.

## Board layout

The play screen starts as a blank board with a visible grid. The board should resemble a tiled liquid network with pump nodes drawn as circular markers inside some cells.

Some tiles are black and represent blocked cells. A blocked tile does not accept liquid and cannot be used as a pipe route. It acts like a solid obstacle inside the board.

The initial layout has a symmetric 2x2 block of blocked tiles at the board center. This is a deterministic prototype layout; later levels may vary it.

Each side has a distinct liquid color, for example:

- Red: human player
- Blue: computer opponent
- Yellow team

The exact color palette can evolve later, but the board must clearly distinguish each side by its liquid color.

## Pump behavior

Each pump cell continuously adds liquid each game cycle.

Rules:

- each pump has a side/color
- pump output is generated each tick
- pump output enters the pump's local cell and can flow through connected pipes

## Pipe behavior

Each click on a tile should perform a directed pipe-toggle interaction. The clicked tile is the source and the selected adjacent tile is the destination:

- The pointer's position within the clicked cell selects the nearest valid orthogonal neighbor; if directions are tied, use North, East, South, West order.
- If neither direction exists between the selected pair, create a pipe from the clicked tile to its neighbor
- If a pipe exists between the pair in either direction, remove it; never create a second reverse pipe
- Blocked and out-of-bounds neighbors are not valid targets

Liquid flows only from pipe source to destination. A rendered pipe starts at the source cell center and ends at the shared edge of the destination cell. Each directed pipe is drawn once.

The flow amount should be based on the number of outgoing pipes connected from a reservoir.

Example rule:

- 1 pipe => full prescribed flow
- 2 pipes => split evenly between the two outputs
- 3+ pipes => distribute proportionally or by a fixed rule, depending on design

The exact distribution formula should remain deterministic and easy to test.

## Gameplay loop

At each cycle:

1. Each pump with a non-neutral cell side adds up to 8 units to its current cell side, limited by the 160-unit cell capacity.
2. Take a stable post-pump snapshot. Each cell with liquid offers up to 8 units total through its outgoing pipes, split as evenly as possible. It must retain at least 1 unit; a neutral cell sends nothing. Resolve all transfers simultaneously: incoming liquid is added to the cell's current side and opposing liquid is subtracted. For a neutral cell, compare incoming amounts from both sides. The larger remainder determines the cell side and amount; a tie leaves it empty and neutral. Any amount a destination cannot accept remains at the source. A pipe never returns liquid against its direction.
3. Board state is rendered.

The simulation should be deterministic and consistent between frames unless explicit gameplay timing is introduced later.

Each cell has a capacity of 160 units. Pump output and pipe flow use integer units. When an even split has a remainder, distribute remaining units in a consistent direction order so the result is deterministic.

Liquid is rendered as a centered colored square whose area is proportional to the amount, using 180 units as the visual reference. For example, 16 units occupy 16/180 of the tile's total area. This visual reference does not change the simulation's 160-unit cell capacity.

The rendered prototype advances one simulation cycle per second. Keep this cadence as an application setting so it can be tuned after playtesting; rendering frame rate must not affect simulation results.

The computer makes at most one pipe decision per cycle, before pump output and pipe flow are evaluated. It may only create or remove links originating from blue-controlled cells. The human may only create or remove links originating from red-controlled cells.

Difficulty changes the CPU's bounded planning effort, not pump output: Easy uses a greedy immediate evaluation, Medium considers a short sequence of future cycles, and Hard searches a deeper bounded sequence. The CPU favors expanding blue liquid toward red territory and capturing or neutralizing red liquid. This is an initial strategy heuristic; explicit win conditions are not yet defined.

## Player interaction

Players click tiles to:

- create or remove a pipe between adjacent cells
- reshape the liquid network
- redirect flow through different routes

This is the core mechanic of the game.

## Success criteria for v1

The initial version should deliver:

- fullscreen or near-fullscreen board view
- menu with New Game, Level, and Exit Game
- board-size selection
- pump count selection
- board rendering with pump markers
- click-to-create or click-to-remove pipe flow behavior
- multi-color liquid simulation on the board

## Stretch goals

- AI opponent pressure
- scoring or win conditions
- area control mechanics
- special pipe types
- sound and polish
- save/load or replay

## Notes

The rules should remain simple and deterministic for the first playable prototype. The goal is a testable board game mechanic rather than a complex simulation system.
