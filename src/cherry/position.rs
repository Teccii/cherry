use crate::*;

/*----------------------------------------------------------------*/

#[derive(Clone)]
pub struct Position {
    current: Board,
    boards: Vec<Board>,
    moves: Vec<Option<MoveData>>,
    nnue: Nnue,
}

impl Position {
    #[inline]
    pub fn new(board: Board, weights: &NetworkWeights) -> Position {
        let nnue = Nnue::new(&board, weights);

        Position {
            current: board,
            boards: Vec::with_capacity(MAX_PLY as usize),
            moves: Vec::with_capacity(MAX_PLY as usize),
            nnue,
        }
    }

    #[inline]
    pub fn set_board(&mut self, board: Board, weights: &NetworkWeights) {
        self.nnue.full_reset(&board, weights);
        self.boards.clear();
        self.moves.clear();
        self.current = board;
    }

    #[inline]
    pub fn reset(&mut self, weights: &NetworkWeights) {
        self.nnue.full_reset(&self.current, weights);
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn board(&self) -> &Board {
        &self.current
    }

    #[inline]
    pub fn prev_move(&self, ply: usize) -> Option<MoveData> {
        if self.moves.is_empty() {
            return None;
        }

        let ply = ply as isize - 1; //countermove is last
        let last_index = self.moves.len() as isize - 1;
        let index = last_index - ply;

        (index >= 0)
            .then(|| self.moves.get(index as usize).and_then(|&m| m))
            .flatten()
    }

    #[inline]
    pub fn stm(&self) -> Color {
        self.current.stm()
    }

    #[inline]
    pub fn hash(&self) -> u64 {
        self.current.hash()
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn make_move(&mut self, mv: Move, weights: &NetworkWeights) {
        self.boards.push(self.current.clone());
        self.moves.push(Some(MoveData::new(&self.current, mv)));
        self.current.make_move(mv);

        let prev_board = self.boards.last().unwrap();
        self.nnue.make_move(prev_board, &self.current, weights, mv);
    }

    #[inline]
    pub fn null_move(&mut self) -> bool {
        self.boards.push(self.current.clone());
        self.moves.push(None);

        if self.current.null_move() {
            return true;
        }

        self.boards.pop().unwrap();
        self.moves.pop().unwrap();
        false
    }

    #[inline]
    pub fn unmake_move(&mut self) {
        self.current = self.boards.pop().unwrap();
        self.moves.pop().unwrap();
        self.nnue.unmake_move();
    }

    #[inline]
    pub fn unmake_null_move(&mut self) {
        self.current = self.boards.pop().unwrap();
        self.moves.pop().unwrap();
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn eval(&mut self, weights: &NetworkWeights) -> Score {
        self.nnue.apply_updates(&self.current, weights);

        let bucket = OUTPUT_BUCKETS[self.current.occupied().popcnt()];
        let mut eval = self.nnue.eval(weights, bucket, self.stm());

        let material = W::pawn_mat_scale() * self.current.pieces(Piece::Pawn).popcnt() as i32
            + W::knight_mat_scale() * self.current.pieces(Piece::Knight).popcnt() as i32
            + W::bishop_mat_scale() * self.current.pieces(Piece::Bishop).popcnt() as i32
            + W::rook_mat_scale() * self.current.pieces(Piece::Rook).popcnt() as i32
            + W::queen_mat_scale() * self.current.pieces(Piece::Queen).popcnt() as i32;
        eval = (i32::from(eval) * (W::mat_scale_base() + material) / 32768) as i16;

        Score::new(eval.clamp(-Score::MAX_TB_WIN.0 + 1, Score::MAX_TB_WIN.0 - 1))
    }

    #[inline]
    pub fn cmp_see(&self, mv: Move, threshold: i16) -> bool {
        if mv.is_castling() {
            return threshold <= 0;
        }

        let board = self.board();
        let (src, dest, flag) = (mv.src(), mv.dest(), mv.flag());

        let next_victim = mv
            .promotion()
            .unwrap_or_else(|| board.piece_on(src).unwrap());
        let mut balance = -threshold
            + match flag {
                MoveFlag::Normal | MoveFlag::DoublePush => 0,
                MoveFlag::EnPassant => W::see_value(Piece::Pawn),
                MoveFlag::Capture => W::see_value(board.piece_on(dest).unwrap()),
                _ if mv.is_capture_promotion() =>
                    W::see_value(board.piece_on(dest).unwrap())
                        + W::see_value(mv.promotion().unwrap())
                        - W::see_value(Piece::Pawn),
                _ if mv.is_promotion() =>
                    W::see_value(mv.promotion().unwrap()) - W::see_value(Piece::Pawn),
                _ => unreachable!(),
            };

        if balance < 0 {
            return false;
        }

        balance -= W::see_value(next_victim);

        if balance >= 0 {
            return true;
        }

        let src_place = board.get(src);
        let mut see_board: Byteboard = self.current.inner;
        see_board.set(src, Place::EMPTY);
        see_board.set(dest, src_place);

        if flag == MoveFlag::EnPassant {
            let victim_sq = dest.offset(0, -board.stm().sign() as i8);
            see_board.set(victim_sq, Place::EMPTY);
        }

        let mut stm = !board.stm();

        let (ray_perm, ray_valid) = ray_perm(dest);
        let ray_places = see_board.permute(ray_perm).mask(Mask8x64::from(ray_valid));

        let colors = ray_places.msb().to_bitmask();
        let mut blockers = ray_places.nonzero().to_bitmask();
        let attackers = ray_attackers(ray_places);

        let ray_pieces = (ray_places & u8x64::splat(Place::PIECE_MASK)).mask(attackers);
        let ray_pieces_vec = u64x8::from([
            u8x64::eq(ray_pieces, u8x64::splat(Piece::Pawn.bits() << 4)).to_bitmask(),
            u8x64::eq(ray_pieces, u8x64::splat(Piece::Knight.bits() << 4)).to_bitmask(),
            u8x64::eq(ray_pieces, u8x64::splat(Piece::Bishop.bits() << 4)).to_bitmask(),
            u8x64::eq(ray_pieces, u8x64::splat(Piece::Rook.bits() << 4)).to_bitmask(),
            u8x64::eq(ray_pieces, u8x64::splat(Piece::Queen.bits() << 4)).to_bitmask(),
            u8x64::eq(ray_pieces, u8x64::splat(Piece::King.bits() << 4)).to_bitmask(),
            0,
            0,
        ]);
        let ray_pieces: [u64; 8] = unsafe { core::mem::transmute(ray_pieces_vec) };
        let attackers = attackers.to_bitmask();

        #[inline]
        fn next_attackers(
            stm: Color,
            blockers: u64,
            ray_valid: u64,
            attackers: u64,
            colors: u64,
        ) -> u64 {
            let closest_blockers = extend_bitrays(blockers, ray_valid) & blockers;
            let colors = match stm {
                Color::White => !colors,
                Color::Black => colors,
            };

            closest_blockers & attackers & colors
        }

        loop {
            let current_attackers = next_attackers(stm, blockers, ray_valid, attackers, colors);

            if current_attackers == 0 {
                break;
            }

            let next_piece = Piece::index(
                (ray_pieces_vec & u64x8::splat(current_attackers))
                    .nonzero()
                    .to_bitmask()
                    .trailing_zeros() as usize,
            );
            let piece_blockers = ray_pieces[next_piece as usize] & current_attackers;

            blockers ^= piece_blockers & piece_blockers.wrapping_neg();
            balance = -balance - 1 - W::see_value(next_piece);
            stm = !stm;

            if next_piece == Piece::King {
                if next_attackers(stm, blockers, ray_valid, attackers, colors) != 0 {
                    stm = !stm;
                }

                break;
            }

            if balance >= 0 {
                break;
            }
        }

        stm != board.stm()
    }

    /*----------------------------------------------------------------*/

    #[inline]
    pub fn is_draw(&self) -> bool {
        self.insufficient_material()
            || self.repetition()
            || self.current.status() == BoardStatus::Draw
    }

    #[inline]
    pub fn insufficient_material(&self) -> bool {
        match self.current.occupied().popcnt() {
            2 => true,
            3 =>
                (self.current.pieces(Piece::Knight) | self.current.pieces(Piece::Bishop)).popcnt()
                    > 0,
            n => {
                let bishops = self.current.pieces(Piece::Bishop);
                if bishops.popcnt() != n - 2 {
                    return false;
                }

                let dark_bishops = bishops.is_subset(Bitboard::DARK_SQUARES);
                let light_bishops = bishops.is_subset(Bitboard::LIGHT_SQUARES);
                dark_bishops || light_bishops
            }
        }
    }

    #[inline]
    pub fn repetition(&self) -> bool {
        let hash = self.hash();
        let hm = self.current.halfmove_clock() as usize;

        if hm < 4 {
            return false;
        }

        self.boards
            .iter()
            .rev()
            .skip(3)
            .take(hm.saturating_sub(3))
            .step_by(2)
            .any(|b| b.hash() == hash)
    }
}
