# AI Design

## Current status

This project does not require a complex AI system for the first playable build.

The current design is primarily a player-driven board simulation game with difficulty affecting challenge level rather than a true autonomous opponent.

## Planned AI usage

The Level option will adjust difficulty by modifying one or more of:

- pump output rate
- board challenge / obstacles
- number of adversarial network pressures
- AI-assisted challenge conditions

## Design principle

If AI is introduced, it should operate on the same game-state model as the player and should not directly manipulate rendering code.

For the first pass, the difficulty setting should remain simple and deterministic.
