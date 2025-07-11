use arrayvec::ArrayVec;
use cherry_core::*;
use crate::*;

/*----------------------------------------------------------------*/

pub const MAX_MOVES: usize = 218;

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone)]
pub struct ScoredMove(pub Move, pub i16);

/*----------------------------------------------------------------*/

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Phase {
    HashMove,
    GenPieceMoves,
    GenCaptures,
    YieldCaptures,
    GenQuiets,
    YieldQuiets,
    YieldBadCaptures,
    Finished
}

/*----------------------------------------------------------------*/

#[derive(Debug, Clone)]
pub struct MovePicker {
    phase: Phase,
    hash_move: Option<Move>,
    piece_moves: ArrayVec<PieceMoves, 20>,
    captures: ArrayVec<ScoredMove, MAX_MOVES>,
    bad_captures: ArrayVec<ScoredMove, MAX_MOVES>,
    quiets: ArrayVec<ScoredMove, MAX_MOVES>,
}

impl MovePicker {
    #[inline(always)]
    pub fn new(hash_move: Option<Move>) -> MovePicker {
        MovePicker {
            phase: Phase::HashMove,
            hash_move,
            piece_moves: ArrayVec::new(),
            captures: ArrayVec::new(),
            bad_captures: ArrayVec::new(),
            quiets: ArrayVec::new(),
        }
    }

    /*----------------------------------------------------------------*/

    #[inline(always)]
    pub fn skip_quiets(&mut self) {
        self.phase = match self.phase {
            Phase::GenQuiets | Phase::YieldQuiets => Phase::YieldBadCaptures,
            _ => self.phase
        }
    }
    
    pub fn next(&mut self, pos: &mut Position, history: &History, ) -> Option<Move> {
        if self.phase == Phase::HashMove {
            self.phase = Phase::GenPieceMoves;
            
            if self.hash_move.is_some() {
                return self.hash_move;
            }
        }

        /*----------------------------------------------------------------*/

        if self.phase == Phase::GenPieceMoves {
            self.phase = Phase::GenCaptures;
            
            pos.board().gen_moves(|moves| {
                self.piece_moves.push(moves);
                false
            });
        }

        /*----------------------------------------------------------------*/

        if self.phase == Phase::GenCaptures {
            self.phase = Phase::YieldCaptures;
            
            let board = pos.board();
            
            for moves in self.piece_moves.iter().copied() {
                for mv in moves {
                    if self.hash_move == Some(mv) || !board.is_capture(mv) {
                        continue;
                    }
                    
                    let see = see(board, mv);
                    let score = history.get_capture(board, mv);
                    
                    if see >= 0  {
                        self.captures.push(ScoredMove(mv, score));
                    } else {
                        self.bad_captures.push(ScoredMove(mv, score));
                    }
                }
            }

            self.captures.sort_by_key(|mv| mv.1);
            self.bad_captures.sort_by_key(|mv| mv.1);
        }

        /*----------------------------------------------------------------*/

        if self.phase == Phase::YieldCaptures {
            if let Some(mv) = self.captures.pop() {
                return Some(mv.0);
            }
            
            self.phase = Phase::GenQuiets;
        }

        /*----------------------------------------------------------------*/

        if self.phase == Phase::GenQuiets {
            let board = pos.board();

            for moves in self.piece_moves.iter().copied() {
                for mv in moves {
                    if self.hash_move == Some(mv) || board.is_capture(mv) {
                        continue;
                    }
                    
                    let score = history.get_quiet(board, mv);
                    self.quiets.push(ScoredMove(mv, score));
                }
            }
            
            self.quiets.sort_by_key(|mv| mv.1);
            self.phase = Phase::YieldQuiets;
        }

        /*----------------------------------------------------------------*/

        if self.phase == Phase::YieldQuiets {
            if let Some(mv) = self.quiets.pop() {
                return Some(mv.0);
            }
            
            self.phase = Phase::YieldBadCaptures;
        }

        /*----------------------------------------------------------------*/

        if self.phase == Phase::YieldBadCaptures {
            if let Some(mv) = self.bad_captures.pop() {
                return Some(mv.0);
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
    GenCaptures,
    YieldCaptures,
    Finished,
}

#[derive(Debug, Clone)]
pub struct QMovePicker {
    phase: QPhase,
    piece_moves: ArrayVec<PieceMoves, 20>,
    evasions: ArrayVec<ScoredMove, MAX_MOVES>,
    captures: ArrayVec<ScoredMove, MAX_MOVES>,
}

impl QMovePicker {
    #[inline(always)]
    pub fn new() -> QMovePicker {
        QMovePicker {
            phase: QPhase::GenPieceMoves,
            piece_moves: ArrayVec::new(),
            evasions: ArrayVec::new(),
            captures: ArrayVec::new(),
        }
    }

    pub fn next(
        &mut self,
        pos: &mut Position,
        history: &History,
    ) -> Option<Move> {
        if self.phase == QPhase::GenPieceMoves {
            pos.board().gen_moves(|moves| {
                self.piece_moves.push(moves);
                false
            });
            
            if pos.in_check() {
                self.phase = QPhase::GenEvasions;
            } else {
                self.phase = QPhase::GenCaptures;
            }
        }
        
        if self.phase == QPhase::GenEvasions {
            let board = pos.board();

            for moves in self.piece_moves.iter().copied() {
                for mv in moves {
                    let score = history.get_move(board, mv);

                    self.evasions.push(ScoredMove(mv, score));
                }
            }

            self.evasions.sort_by_key(|mv| mv.1);
            self.phase = QPhase::YieldEvasions;
        }
        
        if self.phase == QPhase::YieldEvasions {
            if let Some(mv) = self.evasions.pop() {
                return Some(mv.0);
            }
            
            self.phase = QPhase::Finished;
        }
        
        if self.phase == QPhase::GenCaptures {
            let board = pos.board();

            for moves in self.piece_moves.iter().copied() {
                for mv in moves {
                    if !board.is_capture(mv) {
                        continue;
                    }

                    let see = see(board, mv);
                    let score = history.get_capture(board, mv);

                    if see >= 0 {
                        self.captures.push(ScoredMove(mv, score));
                    }
                }
            }
            
            self.captures.sort_by_key(|mv| mv.1);
            self.phase = QPhase::YieldCaptures;
        }

        /*----------------------------------------------------------------*/

        if self.phase == QPhase::YieldCaptures {
            if let Some(mv) = self.captures.pop() {
                return Some(mv.0);
            }
            
            self.phase = QPhase::Finished;
        }
        
        None
    }
}

//TODO: Handle Promotions
pub fn see(board: &Board, mv: Move) -> i16 {
    let (from, to) = (mv.from(), mv.to());
    let mut blockers = board.occupied() ^ from.bitboard();
    
    /*
    En passant only has to be handled for the first capture, because pawn double pushes
    can never capture a piece so they don't matter at all in SEE.
    */
    let first_capture = if board.is_en_passant(mv) {
        blockers ^= Square::new(
            board.en_passant().unwrap(),
            Rank::Fifth.relative_to(board.stm())
        ).bitboard();
        
        Piece::Pawn
    }  else {
        board.piece_on(to).unwrap()
    };
    
    let mut attackers = board.attackers(to, blockers) & blockers;
    let mut target_piece = board.piece_on(from).unwrap();
    let mut stm = !board.stm();
    let mut gains: ArrayVec<i16, 32> = ArrayVec::new();
    gains.push(first_capture.see_value());

    'see: loop {
        for &piece in Piece::ALL.iter() {
            let stm_attackers = attackers & board.color_pieces(piece, stm);
            
            if let Some(sq) = stm_attackers.try_next_square() {
                gains.push(target_piece.see_value());
                
                if target_piece == Piece::King {
                    break;
                }
                
                let bb = sq.bitboard();
                
                blockers ^= bb;
                attackers ^= bb;
                target_piece = piece;
                
                if matches!(piece, Piece::Rook | Piece::Queen) {
                    attackers |= rook_moves(sq, blockers) & blockers & board.orth_sliders();
                }

                if matches!(piece, Piece::Pawn | Piece::Bishop | Piece::Queen) {
                    attackers |= bishop_moves(sq, blockers) & blockers & board.diag_sliders();
                }
                
                stm = !stm;
                continue 'see;
            }
        }

        while gains.len() > 1 {
            let forced = gains.len() == 2;
            let their_gain = gains.pop().unwrap();
            let our_gain = gains.last_mut().unwrap();

            *our_gain -= their_gain;

            if !forced && *our_gain < 0 {
                *our_gain = 0;
            }
        }

        return gains.pop().unwrap();
    }
}

#[test]
fn test_see() {
    use cherry_core::*;
    let fens = &[
        "8/4k3/8/3n4/8/8/3R4/3K4 w - - 0 1",
        "8/4k3/1n6/3n4/8/8/3R4/3K4 w - - 0 1",
        "8/3r4/3q4/3r4/8/3Q3K/3R4/7k w - - 0 1",
        "8/8/b7/1q6/2b5/3Q3K/4B3/7k w - - 0 1",
    ];
    let expected = &[
        Piece::Knight.see_value(),
        Piece::Knight.see_value() - Piece::Rook.see_value(),
        0,
        0,
    ];
    
    let moves = &[
        Move::new(Square::D2, Square::D5, MoveFlag::None),
        Move::new(Square::D2, Square::D5, MoveFlag::None),
        Move::new(Square::D3, Square::D5, MoveFlag::None),
        Move::new(Square::D3, Square::C4, MoveFlag::None),
    ];

    for ((&fen, &expected), &mv) in fens.iter().zip(expected).zip(moves) {
        let board = Board::from_fen(fen, false).unwrap();
        
        assert!(see(&board, mv) >= expected);
        assert!(see(&board, mv) < (expected + 1));
    }
}