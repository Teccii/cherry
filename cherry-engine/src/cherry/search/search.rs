use std::sync::atomic::Ordering;
use arrayvec::ArrayVec;
use cozy_chess::*;
use pyrrhic_rs::WdlProbeResult;
use crate::*;

/*----------------------------------------------------------------*/

pub trait NodeType {
    const PV: bool;
    const NMP: bool;

    type Alt: NodeType;
}

pub struct PV;

impl NodeType for PV {
    const PV: bool = true;
    const NMP: bool = false;

    type Alt = NonPV;
}

pub struct NonPV;

impl NodeType for NonPV {
    const PV: bool = false;
    const NMP: bool = true;

    type Alt = NoNMP;
}

pub struct NoNMP;

impl NodeType for NoNMP {
    const PV: bool = false;
    const NMP: bool = false;

    type Alt = NoNMP;
}

/*----------------------------------------------------------------*/

#[inline(always)]
fn razor<Node: NodeType>(depth: u8, eval: Score, alpha: Score) -> bool {
    !Node::PV && depth < 4 && eval < alpha - razor_margin(depth)
}

#[inline(always)]
fn razor_margin(depth: u8) -> i16 {
    337 * depth as i16
}

/*----------------------------------------------------------------*/

#[inline(always)]
fn rev_futile<Node: NodeType>(
    entry: Option<TTData>,
    depth: u8,
    eval: Score,
    alpha: Score,
    beta: Score,
    improving: bool,
    opponent_worsening: bool
) -> bool {
    !Node::PV
        && depth < 12
        && !alpha.is_decisive()
        && eval > beta + rev_futile_margin(entry, depth, improving, opponent_worsening)
}

#[inline(always)]
fn rev_futile_margin(entry: Option<TTData>, depth: u8, improving: bool, opponent_worsening: bool) -> i16 {
    let mult = 93 - 20 * entry.is_some() as i16;

    depth as i16 * mult
        - improving as i16 * mult * 2
        - opponent_worsening as i16 * mult / 3
}

/*----------------------------------------------------------------*/

#[inline(always)]
fn nmp<Node: NodeType>(
    board: &Board,
    depth: u8,
    eval: Score,
    beta: Score,
    improving: bool
) -> bool {
    Node::NMP
        && depth > 4
        && eval >= beta - nmp_margin(depth, improving)
        && board.occupied() != (board.pieces(Piece::Pawn) | board.pieces(Piece::King))
}

#[inline(always)]
fn nmp_margin(depth: u8, improving: bool) -> i16 {
    389 + 19 * depth as i16 + 55 * improving as i16
}

#[inline(always)]
fn nmp_depth(depth: u8) -> u8 {
    depth.saturating_sub(3 + depth / 3).clamp(1, MAX_DEPTH)
}

/*----------------------------------------------------------------*/

#[inline(always)]
fn iir<Node: NodeType>(entry: TTData, depth: u8) -> i16 {
    if Node::PV && depth > 6 && entry.table_mv.is_none() {
        return 1 + (entry.depth >= depth) as i16;
    }
    
    0
}

/*----------------------------------------------------------------*/

#[inline(always)]
fn shallow_quiet<Node: NodeType>(board: &Board, ply: u16, is_capture: bool, best_score: Option<Score>) -> bool {
    !Node::PV
        && ply != 0
        && !is_capture
        && best_score.map_or(false, |s| !s.is_decisive())
        && board.occupied() != (board.pieces(Piece::King) | board.pieces(Piece::Pawn))
}

#[inline(always)]
fn lmp(depth: u8, moves_seen: u16, improving: bool) -> bool {
    let margin = (3 + depth as u16 * depth as u16) / (2 - improving as u16);
    
    moves_seen >= margin
}

#[inline(always)]
fn hist_margin(depth: u8) -> i16 {
    -4300 * depth as i16
}

#[inline(always)]
fn futile(
    board: &Board,
    lmr_depth: u8,
    eval: Score,
    alpha: Score,
    improving: bool,
    opponent_worsening: bool
) -> bool {
    !board.in_check()
        && lmr_depth < 12
        && eval <= alpha - futile_margin(lmr_depth, improving, opponent_worsening)
}

#[inline(always)]
fn futile_margin(
    lmr_depth: u8,
    improving: bool,
    opponent_worsening: bool
) -> i16 {
    46 + 107 * lmr_depth as i16
        + 102 * improving as i16
        + 87 * opponent_worsening as i16
}

/*----------------------------------------------------------------*/

#[inline(always)]
fn singular(entry: TTData, mv: Move, depth: u8, ply: u16) -> bool {
    depth >= 6
        && ply != 0
        && entry.table_mv == Some(mv)
        && entry.depth >= depth - 3
        && matches!(entry.flag, TTFlag::Exact | TTFlag::LowerBound)
        && !entry.score.is_decisive()
}

#[inline(always)]
fn double_margin<Node: NodeType>() -> i16 {
    -4 + 244 * Node::PV as i16
}

#[inline(always)]
fn triple_margin<Node: NodeType>() -> i16 {
    84 + 269 * Node::PV as i16
}

/*----------------------------------------------------------------*/

#[inline(always)]
fn lmr<Node: NodeType>(board: &Board, mv: Move, depth: u8, moves_seen: u16) -> i16 {
    if Node::PV || depth < 3 {
        return 0;
    }

    if board.is_quiet_capture(mv) {
        return 2;
    } else if board.is_capture(mv) {
        return 3;
    }

    LMR[depth as usize][moves_seen as usize]
}

const LMR: [[i16; MAX_MOVES]; MAX_DEPTH as usize] = {
    const fn iln(x: u8) -> f32 {
        let ilog2 = x.checked_ilog2();

        if let Some(value) = ilog2 {
            //ln(x) = log2(x) * ln(2)
            return value as f32 * std::f32::consts::LN_2;
        }

        0.0
    }
    
    let mut table = [[0; MAX_MOVES]; MAX_DEPTH as usize];
    let mut i = 0;

    while i < MAX_DEPTH as usize {
        let mut j = 0;
        while j < MAX_MOVES {
            table[i][j] = (0.5 + iln(i as u8) * iln(j as u8) / 2.5) as i16;

            j += 1;
        }

        i += 1
    }

    table
};

/*----------------------------------------------------------------*/

pub fn search<Node: NodeType>(
    pos: &mut Position,
    ctx: &mut ThreadContext,
    shared_ctx: &SharedContext,
    mut depth: u8,
    ply: u16,
    mut alpha: Score,
    mut beta: Score,
) -> Score {
    if ply != 0 && (ctx.abort_now || shared_ctx.time_man.abort_search(ctx.nodes.global())) {
        ctx.abort_now();
        return Score::INFINITE;
    }

    ctx.nodes.inc();
    ctx.update_sel_depth(ply);

    if ply != 0 && pos.is_draw() {
        return Score::ZERO;
    }

    if depth == 0 || ply >= MAX_PLY {
        return q_search::<Node>(
            pos,
            ctx,
            shared_ctx,
            ply,
            0,
            alpha,
            beta
        );
    }

    if ply != 0 {
        /*
        Mate Distance Pruning:
        If a forced mate has already been found, prune branches where a shorter mate is not possible.
        This is assuming that we are in a winning position, and that our alpha is Score::new_mate(ply).
        So if we're searching a line that is equal to or longer than ply, we can safely prune that branch,
        because there's no way we will ever improve if we continue searching down that line.
        */
        alpha = alpha.max(Score::new_mated(ply));
        beta = beta.min(Score::new_mate(ply + 1));

        if alpha >= beta {
            return alpha;
        }
    }

    let mut best_move = None;
    let initial_alpha = alpha;

    let skip_move = ctx.search_stack[ply as usize].skip_move;
    let mut tt_entry = skip_move.and_then(|_| shared_ctx.t_table.probe(pos.board()));

    if let Some(entry) = tt_entry {
        ctx.tt_hits.inc();
        best_move = entry.table_mv.filter(|&mv| pos.board().is_legal(mv));

        if entry.table_mv.is_some() && best_move.is_none() {
            //We can't trust this entry if the move is invalid
            tt_entry = None;
        }

        if !Node::PV && entry.depth >= depth {
            let score = entry.score;

            match entry.flag {
                TTFlag::Exact => return score,
                TTFlag::UpperBound => {
                    if score <= alpha {
                        return score;
                    }
                }
                TTFlag::LowerBound => {
                    if score >= beta {
                        return score;
                    }
                },
                TTFlag::None => unreachable!()
            }
        }
    } else {
        ctx.tt_misses.inc();
    }
    
    let (mut syzygy_max, mut syzygy_min) = (Score::MAX_MATE, -Score::MAX_MATE);
    if ply != 0
        && skip_move.is_none()
        && depth >= shared_ctx.syzygy_depth.load(Ordering::Relaxed)
        && shared_ctx.syzygy.is_some()
        && let Some(wdl) = Option::as_ref(&shared_ctx.syzygy).and_then(|tb| probe_wdl(tb, pos.board())) {
        ctx.tb_hits.inc();
        
        let tb_score = match wdl {
            WdlProbeResult::Win => Score::new_tb_win(ply),
            WdlProbeResult::Loss => Score::new_tb_loss(ply),
            _ => Score::ZERO
        };
        
        let tb_bound = match wdl {
            WdlProbeResult::Win => TTFlag::LowerBound,
            WdlProbeResult::Loss => TTFlag::UpperBound,
            _ => TTFlag::Exact,
        };
        
        if tb_bound == TTFlag::Exact
            || (tb_bound == TTFlag::LowerBound && tb_score >= beta)
            || (tb_bound == TTFlag::UpperBound && tb_score <= alpha) {
            shared_ctx.t_table.store(
                pos.board(),
                depth,
                tb_score,
                None,
                None,
                tb_bound
            );
            
            return tb_score;
        }
        
        if Node::PV && tb_bound == TTFlag::LowerBound {
            alpha = alpha.max(tb_score);
            syzygy_min = tb_score;
        }
        
        if Node::PV && tb_bound == TTFlag::UpperBound {
            syzygy_max = tb_score;
        }
    }

    let in_check = pos.in_check();
    let static_eval = match skip_move {
        Some(_) => ctx.search_stack[ply as usize].eval,
        None => tt_entry.and_then(|e| e.eval).unwrap_or_else(|| pos.eval())
    };
    ctx.search_stack[ply as usize].eval = static_eval;
    
    let prev_eval = match ply {
        2.. => Some(ctx.search_stack[ply as usize - 2].eval),
        _ => None
    };
    let opponent_eval = match ply {
        1.. => Some(ctx.search_stack[ply as usize - 1].eval),
        _ => None
    };

    let mut improving = prev_eval.is_some_and(|e| !in_check && static_eval > e);
    let opponent_worsening = opponent_eval.is_some_and(|e| static_eval > -e);

    if !in_check && skip_move.is_none() && !alpha.is_decisive() && !beta.is_decisive(){
        /*
        Reverse Futility Pruning: Similar to Razoring, if the static evaluation of the position is *above*
        beta by a significant margin, we can assume that we can reach at least beta.
        */
        if rev_futile::<Node>(tt_entry, depth, static_eval, alpha, beta, improving, opponent_worsening) {
            return (static_eval + beta) / 2
        }

        /*
        Razoring: If the static evaluation of the position is below alpha by a significant margin,
        skip searching this branch entirely and drop into the quiescence search.
        */
        if razor::<Node>(depth, static_eval, alpha) {
            return q_search::<Node::Alt>(
                pos,
                ctx,
                shared_ctx,
                ply + 1,
                0,
                alpha,
                beta
            );
        }

        /*
        Null Move Pruning: In almost every position, there is a better legal move than doing nothing.
        If a reduced search after a null move fails high, we can be quite confident that the best legal move
        would also fail high. This can make the engine blind to zugzwang, so we do an additional verification search.
        */
        if nmp::<Node>(pos.board(), depth, static_eval, beta, improving) && pos.null_move() {
            let nmp_depth = nmp_depth(depth);

            ctx.search_stack[ply as usize].move_played = None;

            let score = -search::<Node::Alt>(
                pos,
                ctx,
                shared_ctx,
                nmp_depth,
                ply + 1,
                -beta,
                -beta + 1
            );

            pos.unmake_null_move();

            if score >= beta && !score.is_decisive() {
                if depth < 12 {
                    return score;
                }

                let v_score = search::<Node::Alt>(
                    pos,
                    ctx,
                    shared_ctx,
                    nmp_depth,
                    ply + 1,
                    beta - 1,
                    beta,
                );

                if v_score >= beta {
                    return score;
                }
            }
        }
    }
    
    improving |= static_eval >= beta + 94;
    
    if let Some(entry) = tt_entry {
        depth = (depth as i16 - iir::<Node>(entry, depth)) as u8;
    }

    let mut best_score = None;
    let mut quiets: ArrayVec<Move, MAX_MOVES> = ArrayVec::new();
    let mut captures: ArrayVec<Move, MAX_MOVES> = ArrayVec::new();
    let mut move_picker = MovePicker::new(best_move, ctx.search_stack[ply as usize].killers);
    let mut moves_seen = 0;
    
    let counter_move = (ply >= 1).then(|| ctx.search_stack[ply as usize - 1].move_played).flatten();
    let follow_up = (ply >= 2).then(|| ctx.search_stack[ply as usize - 2].move_played).flatten();
    
    ctx.search_stack[ply as usize + 1].killers.clear();

    while let Some(mv) = move_picker.next(pos, &ctx.history, counter_move, follow_up) {
        if skip_move == Some(mv) {
            continue;
        }

        if Node::PV {
            ctx.search_stack[ply as usize + 1].pv_len = 0;
        }

        let is_capture = pos.board().is_capture(mv);
        let nodes = ctx.nodes.local();
        let stat_score = if is_capture {
            ctx.history.get_capture(pos.board(), mv)
        } else {
            ctx.history.get_quiet(pos.board(), mv)
                + ctx.history.get_counter_move(pos.board(), mv, counter_move).unwrap_or_default()
                + ctx.history.get_follow_up(pos.board(), mv, follow_up).unwrap_or_default()
        };
        
        let mut extension: i16 = 0;
        let mut reduction: i16 = 0;
        let mut score;
        
        /*
        Singular Extensions: If all moves but one fail low and just one fails high,
        then that move is singular and should be extended.
        */
        if let Some(entry) = tt_entry && singular(entry, mv, depth, ply) {
            let s_beta = entry.score - depth as i16;
            let s_depth = depth / 2;

            ctx.search_stack[ply as usize].skip_move = Some(mv);

            let s_score = search::<Node::Alt>(
                pos,
                ctx,
                shared_ctx,
                s_depth,
                ply,
                s_beta - 1,
                s_beta,
            );

            ctx.search_stack[ply as usize].skip_move = None;

            if s_score < s_beta {
                extension += 1;

                if s_score < s_beta - double_margin::<Node>() {
                    extension += 1;
                }

                if s_score < s_beta - triple_margin::<Node>() {
                    extension += 1;
                }
                
                ctx.history.update(
                    pos.board(),
                    mv,
                    counter_move,
                    follow_up,
                    &[],
                    &[],
                    depth
                );
            } else if s_score >= beta && !s_score.is_decisive() {
                return s_score;
            } else if entry.score >= beta {
                extension = -2;
            }
        }

        /*
        Late Move Reduction (LMR): Reduce the depth of moves ordered near the end.
        */
        reduction += lmr::<Node>(pos.board(), mv, depth, moves_seen);
        reduction += !Node::PV as i16 + !improving as i16;
        reduction -= ctx.search_stack[ply as usize].killers.contains(mv) as i16;

        if shallow_quiet::<Node>(pos.board(), ply, is_capture, best_score) {
            //Late Move Pruning
            if lmp(depth, moves_seen, improving) {
                move_picker.skip_quiets();
                continue;
            }

            let r_depth = (depth as i16).saturating_sub(reduction).clamp(1, MAX_DEPTH as i16) as u8;

            //History Pruning
            if stat_score < hist_margin(r_depth) {
                move_picker.skip_quiets();
                continue;
            }

            //Futility Pruning
            if futile(
                pos.board(),
                r_depth,
                static_eval,
                alpha,
                improving,
                opponent_worsening
            ) {
                move_picker.skip_quiets();
                continue;
            }
        }

        ctx.search_stack[ply as usize].move_played = Some(MoveData::new(pos.board(), mv));
        pos.make_move(mv);

        /*
        Check Extension: Extend the search if we give check.
        */
        if pos.in_check() {
            extension += 1;
        }

        let depth = (depth as i16 + extension).clamp(0, MAX_DEPTH as i16) as u8;
        if moves_seen == 0 {
            score = -search::<Node>(
                pos,
                ctx,
                shared_ctx,
                depth - 1, 
                ply + 1,
                -beta,
                -alpha
            );
        } else {
            let r_depth = (depth as i16).saturating_sub(reduction).clamp(1, MAX_DEPTH as i16) as u8;

            score = -search::<Node::Alt>(
                pos,
                ctx,
                shared_ctx,
                r_depth - 1,
                ply + 1,
                -alpha - 1,
                -alpha
            );

            if r_depth < depth && score > alpha {
                score = -search::<Node::Alt>(
                    pos,
                    ctx,
                    shared_ctx,
                    depth - 1,
                    ply + 1,
                    -alpha - 1,
                    -alpha
                );
            }

            if Node::PV && score > alpha {
                score = -search::<Node>(
                    pos,
                    ctx,
                    shared_ctx,
                    depth - 1,
                    ply + 1,
                    -beta,
                    -alpha
                );
            }
        }

        pos.unmake_move();
        moves_seen += 1;
        
        if ply == 0 {
            ctx.move_nodes[mv.from as usize][mv.to as usize] += ctx.nodes.local() - nodes;
        }

        if best_score.is_none() || score > best_score.unwrap() {
            best_score = Some(score);
        }

        if score > alpha {
            alpha = score;
            best_move = Some(mv);

            if Node::PV && !ctx.abort_now {
                let child = &ctx.search_stack[ply as usize + 1];
                let (child_pv, len) = (child.pv, child.pv_len);

                ctx.search_stack[ply as usize].update_pv(mv, &child_pv[..len]);
            }
        }

        if score >= beta {
            if !ctx.abort_now {
                if !is_capture {
                    ctx.search_stack[ply as usize].killers.push(mv);
                }

                ctx.history.update(
                    pos.board(),
                    mv,
                    counter_move,
                    follow_up,
                    &quiets,
                    &captures,
                    depth
                );
            }
            
            break;
        }
        
        if Some(mv) != best_move {
            if is_capture {
                captures.push(mv);
            } else {
                quiets.push(mv);
            }
        }
    }

    if moves_seen == 0 {
        return if pos.board().checkers().is_empty() {
            Score::ZERO
        } else {
            Score::new_mated(ply)
        };
    }

    if ply == 0 {
        ctx.nodes.flush();
        ctx.tt_hits.flush();
        ctx.tt_misses.flush();
    }

    let best_score = best_score.unwrap().clamp(syzygy_min, syzygy_max);
    
    if skip_move.is_none() && !ctx.abort_now {
        let flag = match () {
            _ if best_score <= initial_alpha => TTFlag::UpperBound,
            _ if best_score >= beta => TTFlag::LowerBound,
            _ => TTFlag::Exact,
        };

        shared_ctx.t_table.store(
            pos.board(),
            depth,
            best_score,
            Some(static_eval),
            best_move,
            flag
        );
    }

    best_score
}

/*----------------------------------------------------------------*/

#[inline(always)]
fn delta<Node: NodeType>(board: &Board, piece: Piece, eval: Score, alpha: Score) -> bool {
    !Node::PV
        && !board.in_check()
        && calc_phase(board) < TOTAL_PHASE * 3 / 4
        && eval < alpha - delta_margin(piece)
}

#[inline(always)]
fn delta_margin(piece: Piece) -> i16 {
    piece_value(piece) + 2 * PAWN_VALUE.0
}

/*----------------------------------------------------------------*/

pub fn q_search<Node: NodeType>(
    pos: &mut Position,
    ctx: &mut ThreadContext,
    shared_ctx: &SharedContext,
    ply: u16,
    qply: u16,
    mut alpha: Score,
    beta: Score,
) -> Score {
    if ctx.abort_now || shared_ctx.time_man.abort_search(ctx.nodes.global()) {
        ctx.abort_now();

        return Score::INFINITE;
    }

    ctx.qnodes.inc();
    ctx.nodes.inc();
    ctx.update_sel_depth(ply);

    if pos.is_draw() {
        return Score::ZERO;
    }
    
    let initial_alpha = alpha;

    let tt_entry = shared_ctx.t_table.probe(pos.board());
    if let Some(entry) = tt_entry {
        ctx.tt_hits.inc();

        let score = entry.score;
        match entry.flag {
            TTFlag::Exact => return score,
            TTFlag::UpperBound => if score <= alpha {
                return score;
            },
            TTFlag::LowerBound => if score >= beta {
                return score;
            },
            TTFlag::None => unreachable!()
        }
    } else {
        ctx.tt_misses.inc();
    }

    let static_eval = tt_entry.and_then(|e| e.eval).unwrap_or_else(|| pos.eval());

    if ply >= MAX_PLY {
        return static_eval;
    }
    
    if !pos.in_check() {
        if static_eval >= beta {
            return static_eval;
        }

        if static_eval >= alpha {
            alpha = static_eval;
        }
    }

    let mut best_score = None;
    let mut best_move = None;
    let mut move_picker = QMovePicker::new();
    let mut moves_seen = 0;

    let counter_move = (ply >= 1).then(|| ctx.search_stack[ply as usize - 1].move_played).flatten();
    let follow_up = (ply >= 2).then(|| ctx.search_stack[ply as usize - 2].move_played).flatten();

    while let Some(mv) = move_picker.next(pos, qply, &ctx.history, counter_move, follow_up) {
        /*
        Delta Pruning: Similar to Futility Pruning, but only in Quiescence Search .
        We test whether the captured piece + a safety margin (around 200 centipawns)
        is enough to raise alpha.

        For example if we're down a rook, don't bother testing pawn captures,
        because they are unlikely to matter for us. For safety reasons, we cannot
        apply this in the late endgame.
        */
        if let Some(target) = pos.board().capture_piece(mv) && delta::<Node>(
            pos.board(),
            target,
            static_eval,
            alpha
        ) {
            continue;
        }

        pos.make_move(mv);
        let score = -q_search::<Node>(
            pos,
            ctx,
            shared_ctx,
            ply + 1,
            qply + 1,
            -beta,
            -alpha
        );
        pos.unmake_move();
        moves_seen += 1;

        if best_score.is_none() || score > best_score.unwrap() {
            best_score = Some(score);
        }

        if score > alpha {
            alpha = score;
            best_move = Some(mv);
        }

        if score >= beta {
            break;
        }
    }

    if !ctx.abort_now && let Some(best_score) = best_score {
        let flag = match () {
            _ if best_score <= initial_alpha => TTFlag::UpperBound,
            _ if best_score >= beta => TTFlag::LowerBound,
            _ => TTFlag::Exact,
        };

        shared_ctx.t_table.store(
            pos.board(),
            0,
            best_score,
            Some(static_eval),
            best_move,
            flag
        );
    }

    best_score.unwrap_or(alpha)
}

/*----------------------------------------------------------------*/

pub fn det_q_search(pos: &mut Position, ply: u16, mut alpha: Score, beta: Score) -> Score {
    let static_eval = pos.eval();
    
    if ply >= MAX_PLY || static_eval >= beta {
        return static_eval;
    }
    
    if static_eval > alpha {
        alpha = static_eval;
    }

    let mut moves = Vec::new();
    pos.board().generate_moves(|piece_moves| {
        for mv in piece_moves {
            if !pos.board().is_quiet_capture(mv) {
                continue;
            }

            moves.push(mv);
        }

        false
    });

    for mv in moves {
        pos.make_move(mv);
        let score = -det_q_search(pos, ply + 1, -beta, -alpha);
        pos.unmake_move();

        if score >= beta {
            return score;
        }

        if score > alpha {
            alpha = score;
        }
    }

    alpha
}