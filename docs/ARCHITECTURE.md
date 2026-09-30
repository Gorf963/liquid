# Architecture

## Overview

This project is a Windows desktop game built in Rust with a deterministic simulation model and Macroquad for lightweight 2D rendering and input.

The architecture is intentionally split into:

- Game state and rules
- Board simulation and flow logic
- UI/menu flow
- Rendering and input handling

## High-level layers

Game State
↓
Simulation Systems
↓
Commands / Events
↓
Rendering / Input

## Core modules

### 1. App shell

Responsible for startup, menu flow, difficulty selection, board configuration, and game loop orchestration.

### 2. Game configuration

Tracks:

- menu selections
- board size
- difficulty level
- number of pumps per side
- player turn or simulation cadence

### 3. Board model

A rectangular grid of tiles. Each tile stores:

- position
- current liquid amount
- whether the tile is blocked / solid
- whether it is a pump cell
	- outgoing directed pipe connections to adjacent tiles
- owner color / side

Blocked tiles are rendered black and never accept incoming liquid. They behave as non-conductive obstacles in the pipe network.

Pipe-link ownership belongs to the board model. Each adjacent pair has at most one link, oriented from its source cell to its destination cell. The model answers directional connection queries and counts outgoing links; blocking is rejected for cells with either incoming or outgoing links. Rendering draws each directed link from the source center to the shared edge of its destination cell.

### 4. Simulation

Each simulation tick updates:

- pump output generation into the current pump cell first
- a stable post-pump snapshot
- simultaneous flow propagation along outgoing directed pipes only
- liquid movement to adjacent cells
- resource/pressure rules
- win/lose or end-of-round evaluation conditions if added later

### 4a. Opponent AI

The CPU receives a read-only board view and difficulty setting, then returns at most one pipe command per simulation cycle. It may only control links originating from blue-owned liquid cells. Difficulty scales bounded search effort; it must not change pump or flow rules. The app applies the command before the simulation tick.

### 5. Rendering

Macroquad renders the board, selected tiles, pipes, pumps, and fluid colors as a windowed UI. Its input layer translates pointer actions into explicit board commands; it does not own game rules.

Rendering should not directly contain game rules. It should display state produced by the simulation layer.

## Initial design constraints

- Keep game logic deterministic and testable
- Avoid coupling rules to Windows UI code
- Prefer explicit state transitions over hidden mutation
- Use a small board model that can grow with new rules

## Planned implementation direction

The first version uses:

- a plain Rust data model for board state
- a fixed update loop for simulation steps
- simple event-driven interactions for tile clicks
- a Macroquad tile renderer that draws the current board state

This keeps the project easy to test before adding richer visuals or advanced AI.
