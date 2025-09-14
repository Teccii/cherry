use arrayvec::ArrayVec;
use smallvec::SmallVec;
use crate::*;

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct ScoredMove(pub Move, pub i32);

fn select_next_64(moves: &SmallVec<[ScoredMove; 64]>) -> Option<usize> {
    if moves.is_empty() {
        return None;
    }

    moves.iter()
        .enumerate()
        .max_by_key(|(_, mv)| mv.1)
        .map(|(i, _)| i)
}

fn select_next_32(moves: &SmallVec<[ScoredMove; 32]>) -> Option<usize> {
    if moves.is_empty() {
        return None;
    }

    moves.iter()
        .enumerate()
        .max_by_key(|(_, mv)| mv.1)
        .map(|(i, _)| i)
}

#[inline]
fn mask_tactics(moves: &mut PieceMoves, their_pieces: Bitboard, ep_square: Option<Square>) {
    if moves.piece == Piece::Pawn {
        const PROMOTION_MASK: Bitboard = Bitboard(Rank::First.bitboard().0 | Rank::Eighth.bitboard().0);
        
        moves.to &= their_pieces
            | ep_square.map_or(Bitboard::EMPTY, |sq| sq.bitboard())
            | PROMOTION_MASK;
    } else {
        moves.to &= their_pieces;
    }
}

#[inline]
fn mask_quiets(moves: &mut PieceMoves, their_pieces: Bitboard, ep_square: Option<Square>) {
    if moves.piece == Piece::Pawn {
        const PROMOTION_MASK: Bitboard = Bitboard(Rank::First.bitboard().0 | Rank::Eighth.bitboard().0);

        moves.to &= !(their_pieces
            | ep_square.map_or(Bitboard::EMPTY, |sq| sq.bitboard())
            | PROMOTION_MASK);
    } else {
        moves.to &= !their_pieces;
    }
}
/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Phase {
    HashMove,
    GenPieceMoves,
    GenTactics,
    YieldGoodTactics,
    GenQuiets,
    YieldQuiets,
    YieldBadTactics,
    Finished
}

/*----------------------------------------------------------------*/

#[derive(Debug, Clone)]
pub struct MovePicker {
    phase: Phase,
    hash_move: Option<Move>,
    piece_moves: ArrayVec<PieceMoves, 20>,
    good_tactics: SmallVec<[ScoredMove; 32]>,
    bad_tactics: SmallVec<[ScoredMove; 32]>,
    quiets: SmallVec<[ScoredMove; 64]>,
}

impl MovePicker {
    #[inline]
    pub fn new(hash_move: Option<Move>) -> MovePicker {
        MovePicker {
            phase: Phase::HashMove,
            hash_move,
            piece_moves: ArrayVec::new(),
            good_tactics: SmallVec::new(),
            bad_tactics: SmallVec::new(),
            quiets: SmallVec::new(),
        }
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn phase(&self) -> Phase {
        self.phase
    }

    #[inline]
    pub fn skip_quiets(&mut self) {
        self.phase = match self.phase {
            Phase::GenQuiets | Phase::YieldQuiets => Phase::YieldBadTactics,
            _ => self.phase
        }
    }

    /*----------------------------------------------------------------*/
    
    pub fn next(&mut self, pos: &mut Position, history: &History, indices: &ContIndices) -> Option<Move> {
        if self.phase == Phase::HashMove {
            self.phase = Phase::GenPieceMoves;
            
            if self.hash_move.is_some() {
                return self.hash_move;
            }
        }

        /*----------------------------------------------------------------*/

        if self.phase == Phase::GenPieceMoves {
            self.phase = Phase::GenTactics;
            
            pos.board().gen_moves(|moves| {
                self.piece_moves.push(moves);
                false
            });
        }

        /*----------------------------------------------------------------*/

        if self.phase == Phase::GenTactics {
            self.phase = Phase::YieldGoodTactics;
            
            let board = pos.board();
            let their_pieces = board.colors(!board.stm());
            let ep_square = board.ep_square();

            for mut moves in self.piece_moves.iter().copied() {
                mask_tactics(&mut moves, their_pieces, ep_square);

                for mv in moves {
                    if self.hash_move == Some(mv) {
                        continue;
                    }

                    self.good_tactics.push(ScoredMove(mv, history.get_tactical(board, mv)));
                }
            }
        }

        /*----------------------------------------------------------------*/

        if self.phase == Phase::YieldGoodTactics {
            while let Some(index) = select_next_32(&self.good_tactics) {
                let mv = swap_pop(&mut self.good_tactics, index).unwrap();

                if pos.cmp_see(mv.0, 0) {
                    return Some(mv.0);
                } else {
                    self.bad_tactics.push(mv);
                    continue;
                }
            }
            
            self.phase = Phase::GenQuiets;
        }

        /*----------------------------------------------------------------*/

        if self.phase == Phase::GenQuiets {
            let board = pos.board();
            let their_pieces = board.colors(!board.stm());
            let ep_square = board.ep_square();

            for mut moves in self.piece_moves.iter().copied() {
                mask_quiets(&mut moves, their_pieces, ep_square);

                for mv in moves {
                    if self.hash_move == Some(mv) {
                        continue;
                    }

                    self.quiets.push(ScoredMove(mv, history.get_non_tactical(board, mv, indices)));
                }
            }

            self.phase = Phase::YieldQuiets;
        }

        /*----------------------------------------------------------------*/

        if self.phase == Phase::YieldQuiets {
            if let Some(index) = select_next_64(&self.quiets) {
                return swap_pop(&mut self.quiets, index).map(|mv| mv.0);
            }
            
            self.phase = Phase::YieldBadTactics;
        }

        /*----------------------------------------------------------------*/

        if self.phase == Phase::YieldBadTactics {
            if let Some(index) = select_next_32(&self.bad_tactics) {
                return swap_pop(&mut self.bad_tactics, index).map(|mv| mv.0);
            }
            
            self.phase = Phase::Finished;
        }

        /*----------------------------------------------------------------*/

        None
    }
}

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum QPhase {
    GenPieceMoves,
    GenEvasions,
    YieldEvasions,
    GenTactics,
    YieldTactics,
    Finished,
}

#[derive(Debug, Clone)]
pub struct QMovePicker {
    phase: QPhase,
    piece_moves: ArrayVec<PieceMoves, 20>,
    evasions: SmallVec<[ScoredMove; 32]>,
    tactics: SmallVec<[ScoredMove; 32]>,
}

impl QMovePicker {
    #[inline]
    pub fn new() -> QMovePicker {
        QMovePicker {
            phase: QPhase::GenPieceMoves,
            piece_moves: ArrayVec::new(),
            evasions: SmallVec::new(),
            tactics: SmallVec::new(),
        }
    }

    pub fn next(&mut self, pos: &mut Position, history: &History, indices: &ContIndices) -> Option<Move> {
        if self.phase == QPhase::GenPieceMoves {
            pos.board().gen_moves(|moves| {
                self.piece_moves.push(moves);
                false
            });

            if pos.in_check() {
                self.phase = QPhase::GenEvasions;
            } else {
                self.phase = QPhase::GenTactics;
            }
        }
        
        if self.phase == QPhase::GenEvasions {
            let board = pos.board();
            for moves in self.piece_moves.iter().copied() {
                for mv in moves {
                    let score = if board.is_tactical(mv) {
                        history.get_tactical(board, mv)
                    } else {
                        history.get_non_tactical(board, mv, indices)
                    };
                    
                    self.evasions.push(ScoredMove(mv, score));
                }
            }
            
            self.phase = QPhase::YieldEvasions;
        }
        
        if self.phase == QPhase::YieldEvasions {
            if let Some(index) = select_next_32(&self.evasions) {
                return swap_pop(&mut self.evasions, index).map(|mv| mv.0);
            }

            self.phase = QPhase::Finished;
        }
        
        if self.phase == QPhase::GenTactics {
            let board = pos.board();
            let their_pieces = board.colors(!board.stm());
            let ep_square = board.ep_square();

            for mut moves in self.piece_moves.iter().copied() {
                mask_tactics(&mut moves, their_pieces, ep_square);

                for mv in moves {
                    self.tactics.push(ScoredMove(mv, history.get_tactical(board, mv)));
                }
            }

            self.phase = QPhase::YieldTactics;
        }

        /*----------------------------------------------------------------*/

        if self.phase == QPhase::YieldTactics {
            if let Some(index) = select_next_32(&self.tactics) {
                return swap_pop(&mut self.tactics, index).map(|mv| mv.0);
            }
            
            self.phase = QPhase::Finished;
        }
        
        None
    }
}