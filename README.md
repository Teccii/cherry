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
- Material Scaling

### Time Management
- Move Stability
- Score Stability
- Subtree Ratio

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