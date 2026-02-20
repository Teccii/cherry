<div align="center">

# Cherry

<img src="assets/cherry_logo.png" width=250>

[![License][license-badge]][license-link]
[![Release][release-badge]][release-link]
[![Commits][commits-badge]][commits-link]

![Rust][rust-badge]
![LGBTQ+ Friendly][lgbtqp-badge]
![Trans Rights][trans-rights-badge]
</div>

Cherry is a superhuman UCI chess engine.
It supports both Standard and [(Double) Fischer Random][dfrc] chess,
and is exclusively trained on self-generated training data.

### Board Representation and Move Generation
Internally, Cherry uses a mailbox representation.
It uses incrementally updated attack tables for move generation.
Vector math is used to achieve similar speeds to traditional bitboard engines,
which is also the reason why Cherry is much faster on AVX-512 machines than on AVX-2 machines.

These attack tables are useful for certain heuristics, such as threat-bucketed history tables,
Static Exchange Evaluation, and in the future, NNUE threat inputs.
All credit goes to [87flowers] for coming up with the techniques to vectorize these operations
and for writing [an amazing blog series][attack-table-blog] about it.

### Search
- Iterative Deepening
- Aspiration Windows
- Principal Variation Search (PVS)
- Quiescence Search (QS)
- Transposition Table
- Reverse Futility Pruning (RFP)
- Null Move Pruning (NMP)
  - Verification Search
- Move Loop Pruning
  - SEE Pruning
  - Late Move Pruning (LMP)
  - Futility Pruning (FP)
  - History Pruning
- Singular Extensions (SE)
  - Double Extensions
  - Negative Extensions
  - Multi-Cut
- Late Move Reductions (LMR)
- Syzygy Endgame Tablebases

### Move Ordering
- Hash Move
- Quiet History
- Tactic History
- Pawn History
- Continuation History (1 and 2 plies)
- Static Exchange Evaluation (SEE)

### Static Evaluation
- NNUE `(768hm->1024)x2->1x8`
  - Dual Perspective
  - Horizontal Mirroring
  - Eight Output Buckets
  - Self-generated training data (8 random moves, 5000 soft nodes per move)
  - Trained with [`bullet`][bullet] on 930 million positions
- Correction History
  - Pawn Structure
  - Minor Pieces
  - Major Pieces
  - White Non-Pawns
  - Black Non-Pawns
  - Continuation Correction (1 and 2 plies)
- Material Scaling

### Time Management
- Move Stability
- Score Stability
- Subtree Ratio

### UCI Options

| Name         | Type    | Default   | Valid Values      | Description                                                                                            |
|--------------|---------|-----------|-------------------|--------------------------------------------------------------------------------------------------------|
| Threads      | Integer | 1         | `1..=2048`        | Number of Search Threads                                                                               |
| Hash         | Integer | 16        | `1..=67108864`    | Memory Allocated to the Transposition Table (in MiB)                                                   |
| MultiPV      | Integer | 1         | `1..=218`         | Number of Variations to Display                                                                        |
| Minimal      | Boolean | `false`   | `true` or `false` | When enabled, Cherry outputs only the final info line and best move                                    |
| EvalScaling  | Boolean | `true`    | `true` or `false` | When enabled, Cherry's evaluation function is scaled according to internal heuristics                  |
| SyzygyPath   | String  | `<empty>` | Any Path          | File path of Syzygy Tablebases (Can only be configured once during the runtime of the program)         |
| MoveOverhead | Integer | 100       | `0..=5000`        | Time in milliseconds used to compensate for the delay between engine and interface communication       |
| SoftTarget   | Boolean | `false`   | `true` or `false` | When enabled, `go nodes <n>` and `go movetime <ms>` will only stop after a completed depth             |
| Ponder       | Boolean | `false`   | `true` or `false` | When enabled, Cherry will think on the opponent's time                                                 |
| UCI_Chess960 | Boolean | `false`   | `true` or `false` | Whether to output UCI moves using standard notation (e1g1/e1c1) or Chess960 notation (e.g. e1h1, e1a1) |

### Building
Cherry requires Make and any version of Rust.
The required toolchain and version will be automatically installed.

```bash
> make EXE=<NAME>
```
- Replace `<NAME>` with the desired name for the binary. The default name is `Cherry`.
- `EVALFILE=<FILE>` can also be passed in to build a binary with a specific neural network embedded, though the code must be changed to reflect this network's architecture.

Since neural networks are extremely large files,
Cherry's neural networks are stored in [a separate repository][cherry-nets] to avoid bloating this repository's size.
Because of this, you need to run `make` **at least once** before you can use `cargo` to build and run Cherry.

### Credit
These engines have been notable sources of ideas or inspiration:
- [Stormphrax][stormphrax]
- [Viridithas][viridithas]
- [Pawnocchio][pawnocchio]
- [Clockwork][clockwork]
- [Reckless][reckless]
- [Hobbes][hobbes]
- [Icarus][icarus]
- [Rose][rose]

Cherry is tested on [MattBench][mattbench],
which is an [OpenBench][openbench] instance maintained by [Nocturn9x][nocturn9x].

Additionally, these individuals have made developing Cherry easier and a more enjoyable experience:
- [Ciekce][ciekce]: Author of Stormphrax, smart catboy :3
- [Cosmo][cosmo]: Author of Viridithas, certified neural network enjoyer
- [Jonathan Hallström][swedishchef]: Author of Pawnocchio and Co-Author of [Vine][vine]
- [Dan Kelsey][kelseyde]: Author of [Calvin][calvin] and Hobbes (not the cartoon)
- [A_randomnoob][arandomnoob]: Author of [Sirius][sirius], true shashin gigachad
- [Sp00ph][sp00ph]: Author of Icarus, Rust and SIMD wizard
- [87flowers]: Author of Rose, SIMD wizard

[license-badge]: https://img.shields.io/github/license/Teccii/Cherry?style=for-the-badge
[release-badge]: https://img.shields.io/github/v/release/Teccii/Cherry?style=for-the-badge
[commits-badge]: https://img.shields.io/github/commits-since/Teccii/Cherry/latest?style=for-the-badge

[license-link]: https://github.com/Teccii/Cherry/blob/main/LICENSE
[release-link]: https://github.com/Teccii/Cherry/releases/latest
[commits-link]: https://github.com/Teccii/Cherry/commits/main

[rust-badge]: https://img.shields.io/badge/Rust-%23000000.svg?e&logo=rust&logoColor=white&color=red
[lgbtqp-badge]: https://pride-badges.pony.workers.dev/static/v1?label=lgbtq%2B%20friendly&stripeWidth=6&stripeColors=E40303,FF8C00,FFED00,008026,24408E,732982
[trans-rights-badge]: https://pride-badges.pony.workers.dev/static/v1?label=trans%20rights&stripeWidth=6&stripeColors=5BCEFA,F5A9B8,FFFFFF,F5A9B8,5BCEFA

[dfrc]: https://en.wikipedia.org/wiki/Chess960
[bullet]: https://github.com/jw1912/bullet
[openbench]: https://github.com/AndyGrant/OpenBench/
[mattbench]: https://chess.n9x.co/
[attack-table-blog]: https://87flowers.com/byteboard-attack-tables-1/
[cherry-nets]: https://github.com/Teccii/cherry-networks/

[stormphrax]: https://github.com/Ciekce/Stormphrax
[viridithas]: https://github.com/cosmobobak/viridithas
[pawnocchio]: https://github.com/JonathanHallstrom/pawnocchio
[clockwork]: https://github.com/official-clockwork/Clockwork
[reckless]: https://github.com/codedeliveryservice/Reckless
[hobbes]: https://github.com/kelseyde/hobbes-chess-engine
[calvin]: https://github.com/kelseyde/calvin-chess-engine
[sirius]: https://github.com/mcthouacbb/Sirius
[icarus]: https://github.com/Sp00ph/icarus
[rose]: https://github.com/87flowers/Rose
[vine]: https://github.com/vine-chess/vine

[arandomnoob]: https://github.com/mcthouacbb/
[swedishchef]: https://github.com/JonathanHallstrom
[87flowers]: https://github.com/87flowers
[nocturn9x]: https://github.com/nocturn9x
[kelseyde]: https://github.com/kelseyde
[ciekce]: https://github.com/Ciekce
[sp00ph]: https://github.com/Sp00ph
[cosmo]: https://github.com/cosmobobak