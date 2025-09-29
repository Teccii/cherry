# Cherry
Cherry is a UCI chess engine written in Rust.
Its internals, such as board representation and move generation are heavily based on [`Rose`](https://github.com/87flowers/Rose/) by 87flowers.

## Features
### Move Generation
- Fixed shift fancy black magic bitboards
- PEXT is used if BMI2 intrinsics are available

### Move Ordering
- Hash Move
- Phased Move Generation
- Static Exchange Evaluation (SEE)
- History Heuristic
  - Capture History
  - Continuation History

### Search
- Iterative Deepening
- Aspiration Windows
- Transposition Table
- Syzygy Endgame Tablebases (via [`pyrrhic-rs`](https://github.com/Algorhythm-sxv/pyrrhic-rs))
- Principal Variation Search
- Quiescence Search for Tactics and Evasions
- Extensions
  - Check Extensions
- Reductions
  - Fractional Reductions
  - Late Move Reductions
  - Other Reductions
- Pruning
  - Reverse Futility Pruning
  - Null Move Pruning
  - Late Move Pruning
  - Continuation History Pruning
  - Futility Pruning
  - SEE Pruning

### Evaluation
- NNUE `(768->768)x2->8`
  - Dual Perspective
  - Horizontally mirrored piece-square inputs
  - Eight output buckets
  - Self-generated training data (8 random moves, 5000 soft nodes per move)
  - Trained with [`bullet`](https://github.com/jw1912/bullet) on 930 million positions
- Static Evaluation Correction History
  - Pawn Structure
  - Minor Pieces
  - Major Pieces

### Time Management
- Best Move Stability
- Complexity of the Position
- Best Move Subtree Ratio