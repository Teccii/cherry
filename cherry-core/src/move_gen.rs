use crate::*;

/*----------------------------------------------------------------*/

trait PieceType {
    const PIECE: Piece;

    fn add_legals<
        F: FnMut(PieceMoves) -> bool,
        const IN_CHECK: bool
    >(board: &Board, mask: Bitboard, listener: &mut F) -> bool;
}

macro_rules! sliding_piece {
    ($name:ident, $piece:expr, $pseudo_legals:ident) => {
        struct $name;
        
        impl PieceType for $name {
            const PIECE: Piece = $piece;
            
            fn add_legals<
                F: FnMut(PieceMoves) -> bool,
                const IN_CHECK: bool
            >(board: &Board, mask: Bitboard, listener: &mut F) -> bool {
                let pieces = board.color_pieces(Self::PIECE, board.stm()) & mask;
                let target_squares = board.target_squares::<IN_CHECK>();
                let blockers = board.occupied();
        
                for piece in pieces & !board.pinned() {
                    let moves = $pseudo_legals(piece, blockers) & target_squares;
        
                    if !moves.is_empty() {
                        let abort = listener(PieceMoves {
                            piece: Self::PIECE,
                            from: piece,
                            to: moves,
                            en_passant: false,
                        });
        
                        if abort {
                            return true;
                        }
                    }
                }
        
                if !IN_CHECK {
                    let king = board.king(board.stm());
                    
                    for piece in pieces & board.pinned() {
                        let target_squares = target_squares & line(king, piece);
                        let moves = $pseudo_legals(piece, blockers) & target_squares;
        
                        if !moves.is_empty() {
                            let abort = listener(PieceMoves {
                                piece: Self::PIECE,
                                from: piece,
                                to: moves,
                                en_passant: false,
                            });
        
                            if abort {
                                return true;
                            }
                        }
                    }
                }
        
                false
            }
        }
    }
}

sliding_piece!(BishopPiece, Piece::Bishop, bishop_moves);
sliding_piece!(RookPiece, Piece::Rook, rook_moves);
sliding_piece!(QueenPiece, Piece::Queen, queen_moves);

struct KnightPiece;

impl PieceType for KnightPiece {
    const PIECE: Piece = Piece::Knight;

    fn add_legals<
        F: FnMut(PieceMoves) -> bool,
        const IN_CHECK: bool
    >(board: &Board, mask: Bitboard, listener: &mut F) -> bool {
        let pieces = board.color_pieces(Self::PIECE, board.stm()) & mask;
        let target_squares = board.target_squares::<IN_CHECK>();

        for piece in pieces & !board.pinned() {
            let moves = knight_moves(piece) & target_squares;

            if !moves.is_empty() {
                let abort = listener(PieceMoves {
                    piece: Self::PIECE,
                    from: piece,
                    to: moves,
                    en_passant: false,
                });

                if abort {
                    return true;
                }
            }
        }

        false
    }
}

struct PawnPiece;

impl PieceType for PawnPiece {
    const PIECE: Piece = Piece::Pawn;

    fn add_legals<
        F: FnMut(PieceMoves) -> bool,
        const IN_CHECK: bool
    >(board: &Board, mask: Bitboard, listener: &mut F) -> bool {
        let pieces = board.color_pieces(Self::PIECE, board.stm()) & mask;
        let target_squares = board.target_squares::<IN_CHECK>();
        let enemy_pieces = board.colors(!board.stm());
        let blockers = board.occupied();

        for piece in pieces & !board.pinned() {
            let moves = pawn_quiets(piece, board.stm(), blockers) | pawn_attacks(piece, board.stm()) & enemy_pieces;

            if !moves.is_empty() {
                let abort = listener(PieceMoves {
                    piece: Self::PIECE,
                    from: piece,
                    to: moves,
                    en_passant: false,
                });

                if abort {
                    return true;
                }
            }
        }

        if !IN_CHECK {
            let king = board.king(board.stm());

            for piece in pieces & board.pinned() {
                let target_squares = target_squares & line(king, piece);
                let moves = pawn_quiets(piece, board.stm(), blockers) | pawn_attacks(piece, board.stm()) & enemy_pieces;

                if !moves.is_empty() {
                    let abort = listener(PieceMoves {
                        piece: Self::PIECE,
                        from: piece,
                        to: moves,
                        en_passant: false,
                    });

                    if abort {
                        return true;
                    }
                }
            }
        }

        if let Some(file) = board.en_passant() {
            let king = board.king(board.stm());
            let (diag, orth) = (
                board.color_diag_sliders(!board.stm()),
                board.color_orth_sliders(!board.stm())
            );

            let dest = Square::new(file, Rank::Sixth.relative_to(board.stm()));
            let victim = Square::new(file, Rank::Fifth.relative_to(board.stm()));

            for piece in pawn_attacks(dest, !board.stm()) & pieces {
                let blockers = blockers ^ victim.bitboard() ^ piece.bitboard() ^ dest.bitboard();
                let on_ray = !(bishop_rays(king) & diag).is_empty();
                if on_ray && !(bishop_moves(king, blockers) & diag).is_empty() {
                    continue;
                }

                let on_ray = !(rook_rays(king) & orth).is_empty();
                if on_ray && !(rook_moves(king, blockers) & orth).is_empty() {
                    continue;
                }

                let abort = listener(PieceMoves {
                    piece: Self::PIECE,
                    from: piece,
                    to: dest.bitboard(),
                    en_passant: true
                });

                if abort {
                    return true;
                }
            }
        }

        false
    }
}

struct KingPiece;

impl PieceType for KingPiece {
    const PIECE: Piece = Piece::King;

    fn add_legals<
        F: FnMut(PieceMoves) -> bool,
        const IN_CHECK: bool
    >(board: &Board, mask: Bitboard, listener: &mut F) -> bool {
        let pieces = board.colors(board.stm());
        let king = board.king(board.stm());

        if !mask.has(king) {
            return false;
        }

        let mut moves = Bitboard::EMPTY;
        for sq in king_moves(king) & !pieces {
            if board.king_safe_on(sq) {
                moves |= sq.bitboard();
            }
        }

        if !IN_CHECK {
            let rights = board.castle_rights(board.stm());
            let back_rank = Rank::First.relative_to(board.stm());

            if let Some(rook) = rights.short && board.can_castle(rook, File::G, File::F) {
                moves |= Square::new(rook, back_rank).bitboard();
            }

            if let Some(rook) = rights.long && board.can_castle(rook, File::C, File::D) {
                moves |= Square::new(rook, back_rank).bitboard();
            }
        }

        if !moves.is_empty() {
            let abort = listener(PieceMoves {
                piece: Self::PIECE,
                from: king,
                to: moves,
                en_passant: false,
            });

            if abort {
                return true;
            }
        }

        false
    }
}

/*----------------------------------------------------------------*/

macro_rules! short_circuit {
    ($ret:expr, {$($cond:expr,)*}) => {
        $(if $cond {
            return $ret;
        })*
    }
}

impl Board {
    fn target_squares<const IN_CHECK: bool>(&self) -> Bitboard {
        let targets = if IN_CHECK {
            let checker = self.checkers().next_square();
            between(checker, self.king(self.stm())) | checker.bitboard()
        } else {
            Bitboard::FULL
        };

        targets & !self.colors(self.stm())
    }
    
    fn king_safe_on(&self, sq: Square) -> bool {
        let pieces = self.colors(!self.stm());
        let blockers = self.occupied() ^ self.king(self.stm()).bitboard() ^ sq.bitboard();
        
        short_circuit!(false, {
            (bishop_moves(sq, blockers) & pieces & self.diag_sliders()).is_empty(),
            (rook_moves(sq, blockers) & pieces & self.diag_sliders()).is_empty(),
            (knight_moves(sq) & pieces & self.pieces(Piece::Knight)).is_empty(),
            (king_moves(sq) & pieces & self.pieces(Piece::King)).is_empty(),
            (pawn_attacks(sq, self.stm()) & pieces & self.pieces(Piece::Pawn)).is_empty(),
        });
        
        true
    }
    
    fn can_castle(&self, rook: File, king_dest: File, rook_dest: File) -> bool {
        let king = self.king(self.stm());
        
        let back_rank = Rank::First.relative_to(self.stm());
        let rook = Square::new(rook, back_rank);
        let king_dest = Square::new(king_dest, back_rank);
        let rook_dest = Square::new(rook_dest, back_rank);
        
        let blockers = self.occupied() ^ king.bitboard();
        let check_safe = between(king, king_dest) | king_dest.bitboard();
        let check_empty = check_safe | between(king, rook) | rook_dest.bitboard();
        
        // !self.pinned().has(rook) &&
        (blockers & check_empty).is_empty() && check_safe.iter().all(|sq| self.king_safe_on(sq))
    }

    /*----------------------------------------------------------------*/

    fn add_legals<
        F: FnMut(PieceMoves) -> bool,
        const IN_CHECK: bool
    >(&self, mask: Bitboard, listener: &mut F) -> bool {
        short_circuit!(true, {
            PawnPiece::add_legals::<_, IN_CHECK>(self, mask, listener),
            KnightPiece::add_legals::<_, IN_CHECK>(self, mask, listener),
            BishopPiece::add_legals::<_, IN_CHECK>(self, mask, listener),
            RookPiece::add_legals::<_, IN_CHECK>(self, mask, listener),
            QueenPiece::add_legals::<_, IN_CHECK>(self, mask, listener),
            KingPiece::add_legals::<_, IN_CHECK>(self, mask, listener),
        });
        
        false
    }

    /*----------------------------------------------------------------*/
    
    #[inline(always)]
    pub fn gen_moves(&self, listener: impl FnMut(PieceMoves) -> bool) -> bool {
        self.gen_moves_for(Bitboard::FULL, listener)
    }
    
    pub fn gen_moves_for(&self, mask: Bitboard, mut listener: impl FnMut(PieceMoves) -> bool) -> bool {
        match self.checkers().popcnt() {
            0 => self.add_legals::<_, false>(mask, &mut listener),
            1 => self.add_legals::<_, true>(mask, &mut listener),
            _ => KingPiece::add_legals::<_, true>(self, mask, &mut listener),
        }
    }

    /*----------------------------------------------------------------*/
    
    pub fn is_legal(&self, mv: Move) -> bool {
        let (from, to, flag) = (mv.from(), mv.to(), mv.flag());
        let pieces = self.colors(self.stm());
        
        if !pieces.has(from) {
            return false;
        }
        
        let king = self.king(self.stm());
        if from == king {
            if matches!(flag, MoveFlag::EnPassant | MoveFlag::Promotion) {
                return false;
            }
            
            if self.checkers().is_empty() {
                let rights = self.castle_rights(self.stm());
                let back_rank = Rank::First.relative_to(self.stm());
                
                if let Some(rook) = rights.short {
                    let rook_sq = Square::new(rook, back_rank);
                    
                    if to == rook_sq && self.can_castle(rook, File::G, File::F) {
                        return flag == MoveFlag::Castling;
                    }
                }

                if let Some(rook) = rights.long {
                    let rook_sq = Square::new(rook, back_rank);

                    if to == rook_sq && self.can_castle(rook, File::C, File::D) {
                        return flag == MoveFlag::Castling;
                    }
                }
            }
            
            if !(king_moves(from) & !pieces).has(to) {
                return false;
            }
            
            return self.king_safe_on(to);
        }
        
        if self.pinned().has(from) && !line(king, from).has(to) {
            return false;
        }
        
        let target_squares = match self.checkers().popcnt() {
            0 => self.target_squares::<false>(),
            1 => self.target_squares::<true>(),
            _ => return false,
        };
        
        let piece = match self.piece_on(from) {
            Some(p) => p,
            None => return false,
        };
        
        if piece != Piece::Pawn && flag == MoveFlag::Promotion {
            return false;
        }
        
        match piece {
            Piece::Pawn => {
                let promotion_rank = Rank::Eighth.relative_to(self.stm());
                let promotion = mv.promotion();
                
                if to.rank() == promotion_rank  {
                    return promotion.is_some_and(|p| !matches!(p, Piece::Pawn | Piece::King));
                }
                
                let mut c = |moves: PieceMoves| moves.to.has(to);
                
                if self.checkers().is_empty() {
                    PawnPiece::add_legals::<_, false>(self, from.bitboard(), &mut c)
                } else {
                    PawnPiece::add_legals::<_, true>(self, from.bitboard(), &mut c)
                }
            },
            Piece::Knight => (target_squares & knight_moves(from)).has(to),
            Piece::Bishop => (target_squares & bishop_rays(from)).has(to) && (between(from, to) & self.occupied()).is_empty(),
            Piece::Rook => (target_squares & rook_rays(from)).has(to) && (between(from, to) & self.occupied()).is_empty(),
            Piece::Queen => (target_squares & queen_rays(from)).has(to) && (between(from, to) & self.occupied()).is_empty(),
            _ => false
        }
    }
}