# AI Design

## Opponent

The computer controls the blue side; the human controls the red side. The AI is a deterministic board-state planner, independent of rendering and input.

## Turn contract

- The CPU receives an immutable board snapshot and selected difficulty.
- It returns at most one pipe command per simulation cycle, before pump output and flow.
- The CPU may only add or remove directed links whose source currently contains blue liquid.
- The human may only add or remove links whose source currently contains red liquid.
- No legal CPU action is a normal no-op.

## Strategy

The CPU should expand blue liquid through open cells toward red territory, preferring actions that capture or neutralize red cells while retaining blue-controlled cells. Since explicit win conditions are not defined yet, this is a territory-pressure heuristic, not a claim that the AI can secure a win.

## Difficulty

Difficulty scales bounded planning effort while leaving game rules unchanged:

- Easy: greedy immediate evaluation with minimal lookahead.
- Medium: short multi-cycle lookahead.
- Hard: deeper bounded lookahead and/or a larger candidate budget.

The implementation must cap work per decision for all board sizes. Choices and tie-breaking remain deterministic so tests can reproduce decisions. Difficulty should change the search budget meaningfully; it must not alter pump output, tick cadence, or pipe flow.

## Testing

Test the planner with board states only, without opening the game window. Cover legal blue-only actions, no-action states, deterministic tie-breaking, decisions that expand toward/capture red territory, and monotonically increasing planning budgets across difficulty levels.
