use crate::*;

impl Board {
    pub fn cmp_see(&self, mv: Move, threshold: i16) -> bool {
        if mv.is_castling() {
            return threshold <= 0;
        }

        let (from, to, flag) = (mv.from(), mv.to(), mv.flag());

        let next_victim = mv.promotion().unwrap_or_else(|| self.piece_on(from).unwrap());
        let mut balance = -threshold + match flag {
            MoveFlag::Normal | MoveFlag::DoublePush => 0,
            MoveFlag::EnPassant => W::see_value(Piece::Pawn),
            MoveFlag::Capture => W::see_value(self.piece_on(to).unwrap()),
            _ if mv.is_capture_promotion() => W::see_value(self.piece_on(to).unwrap())
                + W::see_value(mv.promotion().unwrap())
                - W::see_value(Piece::Pawn),
            _ if mv.is_promotion() => W::see_value(mv.promotion().unwrap()) - W::see_value(Piece::Pawn),
            _ => unreachable!(),
        };

        //best case fail
        if balance < 0 {
            return false;
        }

        balance -= W::see_value(next_victim);

        //worst case pass
        if balance >= 0 {
            return true;
        }

        let board = Vec512::mask8(!from.bitboard().0, self.board.inner);
        let (ray_coords, ray_valid) = superpiece_rays(to);
        let ray_places = Vec512::permute8(ray_coords, board);
        let attackers = ray_valid & attackers_from_rays(ray_places);
        let color = ray_places.msb8();

        let mut occupied = ray_places.nonzero8() & ray_valid;
        let mut stm = !self.stm;

        if mv.is_en_passant() {
            occupied &= match self.stm {
                Color::White => 0xFFFFFFFDFFFFFFFF,
                Color::Black => 0xFFFFFFFFFFFFFFFD,
            };
        }

        let bit_pieces = Vec512::permute8_128(
            Vec512::mask8(attackers, Vec512::shr16::<4>(ray_places) & Vec512::splat8(0x0F)),
            Vec128::from([0x00, 0x80, 0x01, 0x02, 0x00, 0x8, 0x10, 0x20, 0x00, 0x80, 0x01, 0x02, 0x00, 0x08, 0x10, 0x20])
        );
        let piece_rays_vec = Vec512::permute8(
            Vec512::from([
                0x00, 0x08, 0x10, 0x18, 0x20, 0x28, 0x30, 0x38,
                0x01, 0x09, 0x11, 0x19, 0x21, 0x29, 0x31, 0x39,
                0x02, 0x0A, 0x12, 0x1A, 0x22, 0x2A, 0x32, 0x3A,
                0x03, 0x0B, 0x13, 0x1B, 0x23, 0x2B, 0x33, 0x3B,
                0x04, 0x0C, 0x14, 0x1C, 0x24, 0x2C, 0x34, 0x3C,
                0x05, 0x0D, 0x15, 0x1D, 0x25, 0x2D, 0x35, 0x3D,
                0x06, 0x0E, 0x16, 0x1E, 0x26, 0x2E, 0x36, 0x3E,
                0x07, 0x0F, 0x17, 0x1F, 0x27, 0x2F, 0x37, 0x3F,
            ]),
            Vec512::gf2p8matmul8(
                Vec512::gf2p8matmul8(Vec512::splat64(0x8040201008040201), bit_pieces),
                Vec512::splat64(0x8040201008040201)
            )
        );
        let piece_rays = unsafe { core::mem::transmute::<Vec512, [u64; 8]>(piece_rays_vec) };

        #[inline]
        fn next_attackers(occupied: u64, attackers: u64, color: u64, stm: Color) -> Bitboard {
            superpiece_attacks(occupied, occupied) & attackers & (!color ^ match stm {
                Color::White => Bitboard::EMPTY,
                Color::Black => Bitboard::FULL,
            })
        }

        let mut current = next_attackers(occupied, attackers, color, stm);
        while !current.is_empty() {
            let next = (piece_rays_vec & Vec512::splat64(current.0)).nonzero64().trailing_zeros();
            let piece = Piece::from_bits(((next + 2) % 8) as u8).unwrap();
            let br = piece_rays[next as usize] & current;
            occupied ^= br.0 & br.0.wrapping_neg();

            balance = -balance - 1 - W::see_value(piece);
            stm = !stm;

            if piece == Piece::King {
                if !next_attackers(occupied, attackers, color, stm).is_empty() {
                    stm = !stm;
                }

                break;
            }

            if balance >= 0 {
                break;
            }

            current = next_attackers(occupied, attackers, color, stm);
        }

        stm != self.stm
    }
}