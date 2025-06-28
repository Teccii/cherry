use arrayvec::ArrayVec;
use cozy_chess::*;
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
    YieldKillers,
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
    killers: Killers,
    killer_index: usize,
    piece_moves: ArrayVec<PieceMoves, 18>,
    captures: ArrayVec<ScoredMove, MAX_MOVES>,
    bad_captures: ArrayVec<ScoredMove, MAX_MOVES>,
    quiets: ArrayVec<ScoredMove, MAX_MOVES>,
}

impl MovePicker {
    #[inline(always)]
    pub fn new(hash_move: Option<Move>, killers: Killers) -> MovePicker {
        MovePicker {
            phase: Phase::HashMove,
            hash_move,
            killers,
            killer_index: 0,
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
            Phase::YieldKillers | Phase::GenQuiets | Phase::YieldQuiets => Phase::YieldBadCaptures,
            _ => self.phase
        }
    }
    
    pub fn next(
        &mut self,
        pos: &mut Position,
        history: &History,
        counter_move: Option<MoveData>,
        follow_up: Option<MoveData>
    ) -> Option<Move> {
        
        if self.phase == Phase::HashMove {
            self.phase = Phase::GenPieceMoves;
            
            if self.hash_move.is_some() {
                return self.hash_move;
            }
        }

        /*----------------------------------------------------------------*/

        if self.phase == Phase::GenPieceMoves {
            self.phase = Phase::GenCaptures;
            
            pos.board().generate_moves(|moves| {
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
            
            self.phase = Phase::YieldKillers;
        }

        /*----------------------------------------------------------------*/

        if self.phase == Phase::YieldKillers {
            while self.killer_index < KILLER_COUNT {
                let next_killer = self.killers.get(self.killer_index).filter(|&mv| pos.board().is_legal(mv));
                self.killer_index += 1;
                
                if self.hash_move == next_killer || next_killer.is_none() {
                    continue;
                }
                
                return next_killer;
            }
            
            self.phase = Phase::GenQuiets;
        }
        
        /*----------------------------------------------------------------*/

        if self.phase == Phase::GenQuiets {
            let board = pos.board();

            for moves in self.piece_moves.iter().copied() {
                for mv in moves {
                    if self.hash_move == Some(mv) || self.killers.contains(mv) || board.is_capture(mv) {
                        continue;
                    }
                    
                    let score = history.get_quiet(board, mv)
                        + history.get_counter_move(board, mv, counter_move).unwrap_or_default()
                        + history.get_follow_up(board, mv, follow_up).unwrap_or_default();

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
    GenChecks,
    YieldChecks,
    Finished,
}

#[derive(Debug, Clone)]
pub struct QMovePicker {
    phase: QPhase,
    piece_moves: ArrayVec<PieceMoves, 18>,
    evasions: ArrayVec<ScoredMove, MAX_MOVES>,
    captures: ArrayVec<ScoredMove, MAX_MOVES>,
    checks: ArrayVec<ScoredMove, MAX_MOVES>,
}

impl QMovePicker {
    #[inline(always)]
    pub fn new() -> QMovePicker {
        QMovePicker {
            phase: QPhase::GenCaptures,
            piece_moves: ArrayVec::new(),
            evasions: ArrayVec::new(),
            captures: ArrayVec::new(),
            checks: ArrayVec::new(),
        }
    }

    pub fn next(
        &mut self,
        pos: &mut Position,
        qply: u16,
        history: &History,
        counter_move: Option<MoveData>,
        follow_up: Option<MoveData>,
    ) -> Option<Move> {
        if self.phase == QPhase::GenPieceMoves {
            pos.board().generate_moves(|moves| {
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
                    let score = history.get_move(board, mv, counter_move, follow_up);

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
                    if !board.is_quiet_capture(mv) {
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
            
            if qply < 6 {
                self.phase = QPhase::GenChecks;
            } else {
                self.phase = QPhase::Finished;
            }
        }
        
        if self.phase == QPhase::GenChecks {
            let board = pos.board();

            for moves in self.piece_moves.iter().copied() {
                for mv in moves {
                    if !board.is_check(mv) {
                        continue;
                    }

                    let score = history.get_move(board, mv, counter_move, follow_up);
                    
                    self.checks.push(ScoredMove(mv, score));
                }
            }

            self.checks.sort_by_key(|mv| mv.1);
            self.phase = QPhase::YieldChecks;
        }
        
        if self.phase == QPhase::YieldChecks {
            if let Some(mv) = self.checks.pop() {
                return Some(mv.0);
            }
            
            self.phase = QPhase::Finished;
        }
        
        None
    }
}

//TODO: Handle Promotions
pub fn see(board: &Board, mv: Move) -> i16 {
    let mut blockers = board.occupied() ^ mv.from.bitboard();
    
    /*
    En passant only has to be handled for the first capture, because pawn double pushes
    can never capture a piece so they don't matter at all in SEE.
    */
    let first_capture = if board.is_en_passant(mv) {
        blockers ^= mv.to.shift_rel::<Down>(board.side_to_move()).unwrap().bitboard();
        
        Piece::Pawn
    }  else {
        board.piece_on(mv.to).unwrap()
    };
    
    let mut attackers =
        get_king_moves(mv.to) & blockers & board.pieces(Piece::King)
        | get_knight_moves(mv.to) & blockers & board.pieces(Piece::Knight)
        | get_bishop_moves(mv.to, blockers) & blockers & board.diag_sliders()
        | get_rook_moves(mv.to, blockers) & blockers & board.orth_sliders()
        | get_pawn_attacks(mv.to, Color::Black) & blockers & board.colored_pieces(Color::White, Piece::Pawn)
        | get_pawn_attacks(mv.to, Color::White) & blockers & board.colored_pieces(Color::Black, Piece::Pawn);

    let mut target_piece = board.piece_on(mv.from).unwrap();
    let mut stm = !board.side_to_move();
    let mut gains: ArrayVec<i16, 32> = ArrayVec::new();
    gains.push(piece_value(first_capture));

    'see: loop {
        for &piece in Piece::ALL.iter() {
            let stm_attackers = attackers & board.colored_pieces(stm, piece);
            
            if let Some(sq) = stm_attackers.next_square() {
                gains.push(piece_value(target_piece));
                
                if target_piece == Piece::King {
                    break;
                }
                
                let bb = sq.bitboard();
                
                blockers ^= bb;
                attackers ^= bb;
                target_piece = piece;
                
                if matches!(piece, Piece::Rook | Piece::Queen) {
                    attackers |= get_rook_moves(sq, blockers) & blockers & board.orth_sliders();
                }

                if matches!(piece, Piece::Pawn | Piece::Bishop | Piece::Queen) {
                    attackers |= get_bishop_moves(sq, blockers) & blockers & board.diag_sliders();
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

pub fn cmp_see(board: &Board, mv: Move, threshold: i16) -> bool {
    true
}

#[test]
fn test_see() {
    use cozy_chess::Square;
    let fens = &[
        "8/4k3/8/3n4/8/8/3R4/3K4 w - - 0 1",
        "8/4k3/1n6/3n4/8/8/3R4/3K4 w - - 0 1",
        "8/3r4/3q4/3r4/8/3Q3K/3R4/7k w - - 0 1",
        "8/8/b7/1q6/2b5/3Q3K/4B3/7k w - - 0 1",
    ];
    let expected = &[
        piece_value(Piece::Knight),
        piece_value(Piece::Knight) - piece_value(Piece::Rook),
        0,
        0,
    ];
    
    let moves = &[
        Move { from: Square::D2, to: Square::D5, promotion: None },
        Move { from: Square::D2, to: Square::D5, promotion: None },
        Move { from: Square::D3, to: Square::D5, promotion: None },
        Move { from: Square::D3, to: Square::C4, promotion: None },
    ];

    for ((&fen, &expected), &mv) in fens.iter().zip(expected).zip(moves) {
        let board = Board::from_fen(fen, false).unwrap();
        
        assert!(see(&board, mv) >= expected);
        assert!(see(&board, mv) < (expected + 1));
        
        //println!("fen: {} move: {} see: {} expected: {}", fen, mv, see(&board, mv), expected);
    }
}