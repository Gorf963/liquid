# Architecture

## Overview

This project is a Windows desktop game built in Rust with a simple deterministic simulation loop and a lightweight rendering layer.

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
- whether it has a pipe connection
- connections to adjacent tiles
- owner color / side

Blocked tiles are rendered black and never accept incoming liquid. They behave as non-conductive obstacles in the pipe network.

### 4. Simulation

Each simulation tick updates:

- pump output generation
- pipe flow propagation
- liquid movement to adjacent cells
- resource/pressure rules
- win/lose or end-of-round evaluation conditions if added later

### 5. Rendering

Renders the board, selected tiles, pipes, pumps, and fluid colors as a windowed UI.

Rendering should not directly contain game rules. It should display state produced by the simulation layer.

## Initial design constraints

- Keep game logic deterministic and testable
- Avoid coupling rules to Windows UI code
- Prefer explicit state transitions over hidden mutation
- Use a small board model that can grow with new rules

## Planned implementation direction

The first version will likely use:

- a plain Rust data model for board state
- a fixed update loop for simulation steps
- simple event-driven interactions for tile clicks
- a tile renderer that draws the current board state

This keeps the project easy to test before adding richer visuals or advanced AI.
