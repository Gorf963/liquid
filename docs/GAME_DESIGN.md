# Game Design

## Game concept

This is a single-player strategy / simulation board game where each cell represents a liquid reservoir and pumps inject colored liquid into the board. Players manipulate pipes by clicking tiles, creating or removing connections between adjacent cells.

The goal is to manage the flow of colored liquids across the board while balancing each side's pump output and the network layout.

## Initial menu flow

### Main menu

Options:

- New Game
- Level
- Exit Game

### Level menu

Difficulty choices:

- Easy
- Medium
- Hard

The difficulty should affect opponent strength, pump output, or board challenge level.

### New Game flow

When the user selects New Game, the game asks for:

- Board size: Small / Medium / Large
- Number of pumps on each side
- Optional starting configuration or defaults

## Board layout

The play screen starts as a blank board with a visible grid. The board should resemble a tiled liquid network with pump nodes drawn as circular markers inside some cells.

Some tiles are black and represent blocked cells. A blocked tile does not accept liquid and cannot be used as a pipe route. It acts like a solid obstacle inside the board.

Each side has a distinct liquid color, for example:

- Red team
- Blue team
- Yellow team

The exact color palette can evolve later, but the board must clearly distinguish each side by its liquid color.

## Pump behavior

Each pump cell continuously adds liquid each game cycle.

Rules:

- each pump has a side/color
- pump output is generated each tick
- pump output enters the pump's local cell and can flow through connected pipes

## Pipe behavior

Each click on a tile should perform a pipe-toggle interaction:

- If the clicked cell is adjacent to another cell and there is no pipe, create a pipe toward the nearest adjacent cell
- If a pipe already exists in that direction, remove it
- If multiple adjacent cells are possible, choose the nearest valid adjacent tile according to a deterministic rule

The flow amount should be based on the number of active pipes connected from a reservoir.

Example rule:

- 1 pipe => full prescribed flow
- 2 pipes => split evenly between the two outputs
- 3+ pipes => distribute proportionally or by a fixed rule, depending on design

The exact distribution formula should remain deterministic and easy to test.

## Gameplay loop

At each cycle:

1. Pump cells generate liquid.
2. Pipe connections route liquid toward neighboring cells.
3. Reservoir amounts update.
4. Board state is rendered.

The simulation should be deterministic and consistent between frames unless explicit gameplay timing is introduced later.

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
