# Cherry
Cherry is a WIP UCI compliant chess engine written in Rust.
Its internals are a modified version of the [`cozy-chess`](https://github.com/analog-hors/cozy-chess) library written by analog-hors.

## Features
### Move Generation
- Fixed shift fancy black magic bitboards
- PEXT and PDEP are used if BMI2 intrinsics are available

### Move Ordering
- Hash Move
- Phased Move Generation
- Static Exchange Evaluation (SEE)
- History Heuristic
- Capture History Heuristic
- Continuation History Heuristic

### Search
- Iterative Deepening
- Aspiration Windows
- Transposition Table
  - Always replace replacement strategy
  - Lockless
- Syzygy Endgame Tablebases (via [`pyrrhic-rs`](https://github.com/Algorhythm-sxv/pyrrhic-rs))
- Principal Variation Search
- Quiescence Search for Captures, Promotions, and Evasions
- Extensions
  - Check Extensions
- Reductions
  - Fractional Reductions
  - Late Move Reductions
  - History Reductions
  - Other Reductions
- Pruning
  - Reverse Futility Pruning
  - Null Move Pruning
  - Late Move Pruning
  - Continuation History Pruning
  - Futility Pruning
  - SEE Pruning

### Evaluation
- NNUE `(736->512)x2->1`
  - Dual Perspective
  - Horizontally mirrored piece-square inputs
  - Self-generated training data (8 random moves, 5000 nodes per move)
  - Trained with [`bullet`](https://github.com/jw1912/bullet) on 414 million positions
- Static Evaluation Correction History
  - Pawn Structure
  - Minor Pieces
  - Major Pieces

### Time Management
- Optimal and Maximum Time Limits
  - Aborts after a depth is complete if the optimal time limit is exceeded
  - Aborts during the search if the maximum time limit is exceeded
- Dynamically adjusts optimal time depending on
  - Best Move Stability
  - Ratio of the Best Move's Subtree to the Full Search Tree