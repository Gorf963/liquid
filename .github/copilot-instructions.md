# Game Project Copilot Instructions

## Project

This is a Windows PC game written in Rust.

The project must use:

- Stable Rust
- Rust 2024 edition
- Cargo for dependency management, building, testing, and running
- rustfmt for formatting
- clippy for static analysis
- cargo test for automated tests

Do not introduce nightly-only Rust features unless the Architect explicitly approves
the decision and documents the reason in docs/DECISIONS.md.

## General Engineering Principles

Prioritize:

1. Correctness
2. Maintainability
3. Testability
4. Performance
5. Simplicity

Do not introduce abstractions without a concrete reason.

Prefer small, cohesive modules with clear ownership of responsibilities.

Keep game rules and simulation logic independent from rendering and platform-specific
code whenever practical.

Avoid putting game logic directly into rendering code.

Avoid global mutable state unless there is a documented architectural reason.

Prefer explicit data flow over hidden state.

Use Rust ownership and borrowing idiomatically. Do not fight the borrow checker with
unnecessary cloning, reference counting, or interior mutability.

## Cargo

Use Cargo commands rather than manually modifying generated Cargo metadata.

Prefer:

cargo add
cargo remove
cargo update
cargo check
cargo test
cargo clippy
cargo fmt

Do not manually pin dependency versions without a reason.

When adding a dependency, explain briefly why it is needed if the reason is not
obvious.

Keep Cargo.lock committed because this is an application/game rather than a library.

## Quality Gate

Before considering a coding task complete, run as appropriate:

cargo fmt --check
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets

A task is not complete merely because the code compiles.

## Architecture

The Architect owns architectural decisions.

Other agents may propose architectural changes but should document the proposal
and ask the Architect to approve significant changes.

Significant changes include:

- changing the game engine
- changing the ECS architecture
- changing major dependencies
- changing the module boundaries
- introducing a new persistence mechanism
- introducing networking
- changing the AI architecture
- changing the rendering architecture

## Documentation

Before making a significant architectural change:

1. Review docs/ARCHITECTURE.md
2. Review docs/GAME_DESIGN.md
3. Review docs/AI_DESIGN.md when AI is involved
4. Review docs/DECISIONS.md
5. Update the appropriate document if the decision changes the project

Do not silently contradict existing architectural decisions.

## Testing

Game rules should be deterministic and testable wherever practical.

Prefer testing:

- game state transitions
- entity behavior
- combat calculations
- movement rules
- AI decisions
- win/loss conditions
- resource calculations
- serialization/deserialization

Avoid tests that depend unnecessarily on rendering, timing, random values, or the
Windows UI.

When randomness is required for game logic, make the random source injectable or
seedable so tests can be deterministic.

## AI

Opponent AI must not be tightly coupled to rendering.

AI should operate primarily on a representation of game state and produce decisions
or actions.

AI behavior should be testable without starting the complete game.

AI implementations should avoid frame-rate-dependent behavior unless explicitly
required by the design.

## Agent Collaboration

Agents must respect the ownership boundaries defined by the Architect.

Do not rewrite unrelated code.

Do not undo another agent's changes simply to impose a preferred coding style.

If a task exposes an architectural problem, document it rather than silently
creating a second architecture.

When uncertain, inspect existing code and documentation before making assumptions.