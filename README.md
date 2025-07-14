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
- Captures and Evasions in QSearch

### Search
- Iterative Deepening
- Aspiration Windows
- Transposition Table
  - Depth-preferred replacement strategy
  - Lockless
- Syzygy Endgame Tablebases
- Principal Variation Search
- Quiescence Search for Captures and Evasions
- Extensions
  - Singular Extensions
  - Check Extensions
- Reductions
  - Fractional Reductions
  - Internal Iterative Reductions
  - Late Move Reductions
  - History Reductions
  - Other Reductions
- Pruning
  - Reverse Futility Pruning
  - Razoring
  - Null Move Pruning
  - Late Move Pruning
  - History Pruning
  - Futility Pruning
  - SEE Pruning
  - Delta Pruning in QSearch

### Evaluation
- Piece-Square Tables
- Bishop Pair
- Rook/Queen on Open/Semiopen File
- Threats to Minor and Major Pieces
- Mobility and Center Control
- Pawn Structure Evaluation
  - Passed Pawns
  - Isolated Pawns
  - Backwards Pawns
  - Doubled Pawns
- King Safety
  - Attack Units
  - 
- Tapered Evaluation

### Time Management
- Optimal and Maximum Time Limits
  - Aborts after a depth is complete if the optimal time limit is exceeded
  - Aborts during the search if the maximum time limit is exceeded
- Dynamically adjusts optimal time depending on
  - Best Move Stability
  - Ratio of the Best Move's Subtree to the Full Search Tree