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
