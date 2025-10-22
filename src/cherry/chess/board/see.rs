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

        let mut stm = !self.stm;
        let board = Vec512::mask8(!from.bitboard().0, self.board.inner);
        let board = Vec512::blend8(to.bitboard().0, board, Vec512::splat8(self.get(from).into_inner()));

        let (ray_coords, ray_valid) = geometry::superpiece_rays(to);
        let ray_places = Vec512::permute8(ray_coords, board);
        let ray_attackers = ray_valid & geometry::attackers_from_rays(ray_places);
        let mut ray_occupied = ray_valid & ray_places.nonzero8();
        let ray_colors = ray_valid & ray_places.msb8();

        if mv.is_en_passant() {
            ray_occupied &= match self.stm {
                Color::White => 0xFFFFFFFDFFFFFFFF,
                Color::Black => 0xFFFFFFFFFFFFFFFD,
            };
        }

        #[cfg(target_feature = "avx512f")]
        let piece_rays_vec = {
            let bits_to_piece = Vec512::permute8_128(
                Vec512::mask8(ray_attackers, Vec512::shr16::<4>(ray_places & Vec512::splat8(Place::PIECE_MASK | Place::COLOR_MASK))),
                Vec128::from([
                    0x00, 0x80, 0x01, 0x02, 0x00, 0x08, 0x10, 0x20,
                    0x00, 0x80, 0x01, 0x02, 0x00, 0x08, 0x10, 0x20
                ])
            );

            Vec512::permute8(
                Vec512::from([
                    0x00, 0x08, 0x10, 0x18, 0x20, 0x28, 0x30, 0x38,
                    0x01, 0x09, 0x11, 0x19, 0x21, 0x29, 0x31, 0x39,
                    0x02, 0x0A, 0x12, 0x1A, 0x22, 0x2A, 0x32, 0x3A,
                    0x03, 0x0B, 0x13, 0x1B, 0x23, 0x2B, 0x33, 0x3B,
                    0x04, 0x0C, 0x14, 0x1C, 0x24, 0x2C, 0x34, 0x3C,
                    0x05, 0x0D, 0x15, 0x1D, 0x25, 0x2D, 0x35, 0x3D,
                    0x06, 0x0E, 0x16, 0x1E, 0x26, 0x2E, 0x36, 0x3E,
                    0x07, 0x0F, 0x17, 0x1F, 0x27, 0x2F, 0x37, 0x3F
                ]),
                Vec512::gf2p8matmul8(
                    Vec512::gf2p8matmul8(Vec512::splat64(0x8040201008040201), bits_to_piece),
                    Vec512::splat64(0x8040201008040201)
                )
            )
        };

        #[cfg(all(target_feature = "avx2", not(target_feature = "avx512f")))]
        let piece_rays_vec = {
            let ray_pieces = Vec512::mask8(ray_attackers, Vec512::shr16::<4>(ray_places & Vec512::splat8(Place::PIECE_MASK)));

            Vec512::from([
                Vec512::eq8(ray_pieces, Vec512::splat8(Piece::Pawn.bits())),
                Vec512::eq8(ray_pieces, Vec512::splat8(Piece::Knight.bits())),
                0,
                Vec512::eq8(ray_pieces, Vec512::splat8(Piece::Bishop.bits())),
                Vec512::eq8(ray_pieces, Vec512::splat8(Piece::Rook.bits())),
                Vec512::eq8(ray_pieces, Vec512::splat8(Piece::Queen.bits())),
                0,
                Vec512::eq8(ray_pieces, Vec512::splat8(Piece::King.bits())),
            ])
        };

        let piece_rays: [u64; 8] = unsafe { core::mem::transmute(piece_rays_vec) };

        #[inline]
        fn next_attackers(occupied: u64, attackers: u64, colors: u64, stm: Color) -> u64 {
            let closest = geometry::superpiece_attacks(occupied, occupied);
            let colors = match stm {
                Color::White => !colors,
                Color::Black => colors,
            };

            closest & colors & attackers
        }

        loop {
            let current_attackers = next_attackers(ray_occupied, ray_attackers, ray_colors, stm);

            if current_attackers == 0 {
                break;
            }

            let next = (piece_rays_vec & Vec512::splat64(current_attackers)).nonzero64().trailing_zeros();
            let piece = Piece::from_bits(((next + 2) % 8) as u8).unwrap();
            let br = piece_rays[next as usize] & current_attackers;
            ray_occupied ^= br & br.wrapping_neg();

            balance = -balance - 1 - W::see_value(piece);
            stm = !stm;

            if piece == Piece::King {
                if next_attackers(ray_occupied, ray_attackers, ray_colors, stm) != 0 {
                    stm = !stm;
                }

                break;
            }

            if balance >= 0 {
                break;
            }
        }

        stm != self.stm
    }
}