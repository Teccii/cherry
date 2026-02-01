#[cfg(test)]
mod tests {
    use crate::*;

    #[inline]
    fn all_moves() -> Vec<Move> {
        let mut moves = Vec::new();

        for &src in &Square::ALL {
            let superpiece_rays = queen_rays(src) | knight_attacks(src);

            moves.extend(
                superpiece_rays
                    .into_iter()
                    .map(|dest| Move::new(src, dest, MoveFlag::Normal)),
            );
            moves.extend(
                superpiece_rays
                    .into_iter()
                    .map(|dest| Move::new(src, dest, MoveFlag::Capture)),
            );
        }

        for &color in &Color::ALL {
            moves.extend(File::ALL.iter().map(|&f| {
                let src = Square::new(f, Rank::Second.relative_to(color));
                let dest = Square::new(f, Rank::Fourth.relative_to(color));

                Move::new(src, dest, MoveFlag::DoublePush)
            }));

            moves.extend(File::ALL.iter().flat_map(|&f| {
                let src = Square::new(f, Rank::Fifth.relative_to(color));
                let dest = pawn_attacks(src, color);

                dest.into_iter()
                    .map(move |dest| Move::new(src, dest, MoveFlag::EnPassant))
            }));

            moves.extend(File::ALL.iter().flat_map(|&f| {
                let src = Square::new(f, Rank::Seventh.relative_to(color));
                let dest = Square::new(f, Rank::Eighth.relative_to(color));

                [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight]
                    .map(|piece| Move::new(src, dest, MoveFlag::promotion(piece).unwrap()))
            }));

            moves.extend(File::ALL.iter().flat_map(|&f| {
                let src = Square::new(f, Rank::Fifth.relative_to(color));
                let dest = pawn_attacks(src, color);

                dest.into_iter().flat_map(move |dest| {
                    [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight].map(|piece| {
                        Move::new(src, dest, MoveFlag::capture_promotion(piece).unwrap())
                    })
                })
            }));

            let back_rank = Rank::First.relative_to(color);
            for &src_file in &[File::B, File::C, File::D, File::E, File::G] {
                for &dest_file in &File::ALL {
                    if src_file == dest_file {
                        continue;
                    }

                    let (src, dest) = (
                        Square::new(src_file, back_rank),
                        Square::new(dest_file, back_rank),
                    );
                    let flag = if src_file < dest_file {
                        MoveFlag::ShortCastling
                    } else {
                        MoveFlag::LongCastling
                    };

                    moves.push(Move::new(src, dest, flag));
                }
            }
        }

        moves
    }

    #[inline]
    fn perft(board: &Board, depth: u8) -> u64 {
        if depth == 0 {
            return 1;
        }

        let mut nodes = 0;
        let move_list = board.gen_moves();

        if depth == 1 {
            nodes += move_list.len() as u64;
        } else {
            for pseudo_mv in all_moves() {
                assert_eq!(
                    board.is_legal(pseudo_mv),
                    move_list.contains(&pseudo_mv),
                    "{} | {} | {:?}",
                    board.to_fen(true),
                    pseudo_mv.display(&board, true),
                    pseudo_mv.flag()
                );
            }

            for &mv in move_list.iter() {
                let mut board = board.clone();
                board.make_move(mv);

                nodes += perft(&board, depth - 1);
            }
        }

        nodes
    }

    macro_rules! perft_test {
        ($name:ident: $board:expr; $($nodes:expr),*) => {
            #[test]
            fn $name() {
                const NODES: &'static [u64] = &[$($nodes),*];

                let board = Board::from_fen($board).unwrap();
                for (depth, &nodes) in NODES.iter().enumerate() {
                    let perft_nodes = perft(&board, depth as u8);
                    assert_eq!(perft_nodes, nodes, "Depth: {} Expected: {} Got: {}", depth, nodes, perft_nodes);
                }
            }
        };
    }

    perft_test!(
        perft_startpos: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        1,
        20,
        400,
        8902,
        197281,
        4865609,
        119060324
    );

    perft_test!(
        perft_kiwipete:  "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        1,
        48,
        2039,
        97862,
        4085603,
        193690690
    );

    perft_test!(
        perft_pos3: "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
        1,
        14,
        191,
        2812,
        43238,
        674624,
        11030083,
        178633661
    );

    perft_test!(
        perft_pos4: "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
        1,
        6,
        264,
        9467,
        422333,
        15833292
    );

    perft_test!(
        perft_pos5: "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
        1,
        44,
        1486,
        62379,
        2103487,
        89941194
    );

    perft_test!(
        perft_pos6: "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10";
        1,
        46,
        2079,
        89890,
        3894594,
        164075551
    );

    perft_test!(
        perft960_position333: "1rqbkrbn/1ppppp1p/1n6/p1N3p1/8/2P4P/PP1PPPP1/1RQBKRBN w FBfb - 0 9";
        1,
        29,
        502,
        14569,
        287739,
        8652810,
        191762235
    );

    perft_test!(
        perft960_position404: "rbbqn1kr/pp2p1pp/6n1/2pp1p2/2P4P/P7/BP1PPPP1/R1BQNNKR w HAha - 0 9";
        1,
        27,
        916,
        25798,
        890435,
        26302461,
        924181432
    );

    perft_test!(
        perft960_position789: "rqbbknr1/1ppp2pp/p5n1/4pp2/P7/1PP5/1Q1PPPPP/R1BBKNRN w GAga - 0 9";
        1,
        24,
        600,
        15347,
        408207,
        11029596,
        308553169
    );

    perft_test!(
        perft960_position726: "rkb2bnr/pp2pppp/2p1n3/3p4/q2P4/5NP1/PPP1PP1P/RKBNQBR1 w Aha - 0 9";
        1,
        29,
        861,
        24504,
        763454,
        22763215,
        731511256
    );
}
