use std::sync::atomic::Ordering;
use arrayvec::ArrayVec;
use pyrrhic_rs::WdlProbeResult;
use cherry_core::*;
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
fn lmr<Node: NodeType>(depth: u8, moves_seen: u16) -> i32 {
    if Node::PV || depth < 3 {
        return 0;
    }

    LMR[depth as usize][moves_seen as usize]
}

const LMR: [[i32; MAX_MOVES]; MAX_DEPTH as usize] = {
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
            table[i][j] = (384f32 + 512f32 * iln(i as u8) * iln(j as u8)) as i32;

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
    cut_node: bool,
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
    if ply != 0 && skip_move.is_none()
        && depth >= shared_ctx.syzygy_depth.load(Ordering::Relaxed)
        && shared_ctx.syzygy.is_some() {
        if let Some(wdl) = Option::as_ref(&shared_ctx.syzygy)
            .and_then(|tb| probe_wdl(tb, pos.board())) {
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
    }

    let in_check = pos.in_check();
    let static_eval = match skip_move {
        Some(_) => ctx.search_stack[ply as usize].eval,
        None => tt_entry.and_then(|e| e.eval).unwrap_or_else(|| pos.eval())
    };
    
    ctx.search_stack[ply as usize].eval = static_eval;
    let (prev_eval, prev_ext, prev_reduction) = match ply {
        2.. => {
            let prev_stack = &ctx.search_stack[ply as usize - 2];
            (Some(prev_stack.eval), prev_stack.extension, prev_stack.reduction)
        },
        _ => (None, 0, 0)
    };

    let improving = prev_eval.is_some_and(|e| !in_check && static_eval > e);
    let w = &shared_ctx.weights;

    if !in_check && skip_move.is_none() && !alpha.is_decisive() && !beta.is_decisive(){
        /*
        Reverse Futility Pruning: Similar to Razoring, if the static evaluation of the position is *above*
        beta by a significant margin, we can assume that we can reach at least beta.
        */

        let rfp_mult = w.rfp_margin - w.rfp_tt * tt_entry.is_some() as i16;
        let rfp_margin = depth as i16 * rfp_mult - improving as i16 * rfp_mult * 2;
        if !Node::PV && depth < w.rfp_depth
            && !alpha.is_decisive() && static_eval > beta + rfp_margin {
            return (static_eval + beta) / 2
        }

        /*
        Razoring: If the static evaluation of the position is below alpha by a significant margin,
        skip searching this branch entirely and drop into the quiescence search.
        */
        let razor_margin = w.razor_margin * depth as i16;
        if !Node::PV && depth < w.razor_depth
            && static_eval < alpha - razor_margin {
            return q_search::<Node::Alt>(
                pos,
                ctx,
                shared_ctx,
                ply + 1,
                alpha,
                beta
            );
        }

        /*
        Null Move Pruning: In almost every position, there is a better legal move than doing nothing.
        If a reduced search after a null move fails high, we can be quite confident that the best legal move
        would also fail high. This can make the engine blind to zugzwang, so we do an additional verification search.
        */
        if Node::NMP && depth > w.nmp_depth && ctx.search_stack[ply as usize - 1].move_played.is_some()
            && static_eval >= beta && pos.non_pawn_material() && pos.null_move() {
            let nmp_depth = depth.saturating_sub(3 + depth / 3);

            ctx.search_stack[ply as usize].move_played = None;
            let score = -search::<Node::Alt>(
                pos,
                ctx,
                shared_ctx,
                nmp_depth,
                ply + 1,
                -beta,
                -beta + 1,
                !cut_node,
            );

            pos.unmake_null_move();

            if score >= beta && !score.is_decisive() {
                if depth < w.nmp_verification_depth {
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
                    false,
                );

                if v_score >= beta {
                    return score;
                }
            }
        }
    }

    if let Some(entry) = tt_entry {
        let iir = if (Node::PV || cut_node) && depth > w.iir_depth && entry.table_mv.is_none() {
            1 + (entry.depth >= depth) as i16
        } else {
            0
        };

        depth = (depth as i16 - iir) as u8;
    }

    let mut best_score = None;
    let mut quiets: ArrayVec<Move, MAX_MOVES> = ArrayVec::new();
    let mut captures: ArrayVec<Move, MAX_MOVES> = ArrayVec::new();
    let mut move_picker = MovePicker::new(best_move);
    let mut moves_seen = 0;
    
    let counter_move = (ply >= 1).then(|| ctx.search_stack[ply as usize - 1].move_played).flatten();
    let follow_up = (ply >= 2).then(|| ctx.search_stack[ply as usize - 2].move_played).flatten();

    while let Some(mv) = move_picker.next(pos, &ctx.history, counter_move, follow_up) {
        if skip_move == Some(mv) {
            continue;
        }

        if Node::PV {
            ctx.search_stack[ply as usize + 1].pv_len = 0;
        }

        let is_capture = pos.board().is_capture(mv);
        let nodes = ctx.nodes.local();
        let hist = ctx.history.get_move(
            pos.board(),
            mv,
            counter_move,
            follow_up
        );

        let mut extension: i16 = 0;
        let mut reduction: i32 = w.base_reduction; //base reduction to compensate for other reductions
        let mut score;
        
        /*
        Singular Extensions: If all moves but one fail low and just one fails high,
        then that move is singular and should be extended.
        */
        if let Some(entry) = tt_entry {
            if depth >= w.singular_depth && ply != 0
                && entry.table_mv == Some(mv) && entry.depth >= depth - 3
                && matches!(entry.flag, TTFlag::Exact | TTFlag::LowerBound)
                && !entry.score.is_decisive() {

                let s_beta = entry.score - depth as i16;
                let s_depth = depth / 2;

                ctx.search_stack[ply as usize].skip_move = Some(mv);
                ctx.search_stack[ply as usize].reduction = (depth / 2) as i32;

                let s_score = search::<Node::Alt>(
                    pos,
                    ctx,
                    shared_ctx,
                    s_depth,
                    ply,
                    s_beta - 1,
                    s_beta,
                    cut_node,
                );

                ctx.search_stack[ply as usize].skip_move = None;

                if s_score < s_beta {
                    extension += 1;

                    let double_margin = w.double_base + w.double_pv * Node::PV as i16;
                    if s_score < s_beta - double_margin {
                        extension += 1;
                    }

                    let triple_margin = w.triple_base + w.triple_pv * Node::PV as i16;
                    if s_score < s_beta - triple_margin {
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
                    //multi-cut
                    return s_score;
                } else if entry.score >= beta {
                    extension = -3;
                } else if cut_node {
                    extension = -2;
                }
            }
        }

        /*
        Late Move Reduction (LMR): Reduce the depth of moves ordered near the end.
        */
        reduction += lmr::<Node>(depth, moves_seen);
        reduction += w.non_pv_reduction * !Node::PV as i32;
        reduction += w.not_improving_reduction * improving as i32;
        reduction += w.cut_node_reduction * cut_node as i32;
        reduction -= hist as i32 / w.hist_reduction;

        if !Node::PV && ply != 0 && pos.non_pawn_material()
            && best_score.map_or(false, |s: Score| !s.is_decisive()) {

            if is_capture {
                //SEE pruning
                let see_margin = w.see_margin * depth as i16;
                if depth < w.see_depth
                    && move_picker.phase() == Phase::YieldBadCaptures
                    && pos.board().see(mv) < alpha - see_margin {
                    continue;
                }

            } else {
                //Late Move Pruning
                let lmp_margin = (3 + depth as u16 * depth as u16) / (2 - improving as u16);
                if moves_seen >= lmp_margin {
                    move_picker.skip_quiets();
                    continue;
                }

                let r_depth = (depth as i32).saturating_sub(reduction / 1024).clamp(1, MAX_DEPTH as i32) as u8;

                //History Pruning
                if hist < w.hist_margin * r_depth as i16 {
                    move_picker.skip_quiets();
                    continue;
                }

                //Futility Pruning
                let futile_margin = w.futile_base
                    + w.futile_margin * r_depth as i16
                    + w.futile_improving * improving as i16;
                if !pos.in_check() && r_depth < w.futile_depth && static_eval <= alpha - futile_margin {
                    move_picker.skip_quiets();
                    continue;
                }
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

        ctx.search_stack[ply as usize].extension = extension;

        let depth = (depth as i16 + extension).clamp(0, MAX_DEPTH as i16) as u8;
        if moves_seen == 0 {
            ctx.search_stack[ply as usize].reduction = 0;
            score = -search::<Node>(
                pos,
                ctx,
                shared_ctx,
                depth - 1,
                ply + 1,
                -beta,
                -alpha,
                false,
            );
        } else {
            ctx.search_stack[ply as usize].reduction = reduction / 1024;
            let r_depth = (depth as i32).saturating_sub(reduction / 1024).clamp(1, MAX_DEPTH as i32) as u8;

            score = -search::<Node::Alt>(
                pos,
                ctx,
                shared_ctx,
                r_depth - 1,
                ply + 1,
                -alpha - 1,
                -alpha,
                true,
            );

            if r_depth < depth && score > alpha {
                ctx.search_stack[ply as usize].reduction = 0;
                score = -search::<Node::Alt>(
                    pos,
                    ctx,
                    shared_ctx,
                    depth - 1,
                    ply + 1,
                    -alpha - 1,
                    -alpha,
                    !cut_node,
                );
            }

            if Node::PV && score > alpha {
                ctx.search_stack[ply as usize].reduction = 0;
                score = -search::<Node>(
                    pos,
                    ctx,
                    shared_ctx,
                    depth - 1,
                    ply + 1,
                    -beta,
                    -alpha,
                    false,
                );
            }
        }

        pos.unmake_move();
        moves_seen += 1;
        
        if ply == 0 {
            ctx.root_nodes[mv.from() as usize][mv.to() as usize] += ctx.nodes.local() - nodes;
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
        return if pos.in_check() {
            Score::new_mated(ply)
        } else {
            Score::ZERO
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

/*----------------------------------------------------------------*/

pub fn q_search<Node: NodeType>(
    pos: &mut Position,
    ctx: &mut ThreadContext,
    shared_ctx: &SharedContext,
    ply: u16,
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

    let mut best_move = None;
    let mut best_score = None;
    let mut move_picker = QMovePicker::new();
    let mut moves_seen = 0;

    let counter_move = (ply >= 1).then(|| ctx.search_stack[ply as usize - 1].move_played).flatten();
    let follow_up = (ply >= 2).then(|| ctx.search_stack[ply as usize - 2].move_played).flatten();

    while let Some(mv) = move_picker.next(pos, &ctx.history, counter_move, follow_up) {
        let is_check = pos.board().is_check(mv);

        if !alpha.is_decisive() && !is_check && !mv.is_promotion() {
            if moves_seen > 3 {
                continue;
            }

            /*
            Delta Pruning: Similar to Futility Pruning, but only in Quiescence Search.
            We test whether the captured piece + a safety margin (around 200 centipawns)
            is enough to raise alpha.

            For example if we're down a rook, don't bother testing pawn captures,
            because they are unlikely to matter for us. For safety reasons, we cannot
            apply this in the late endgame.
            */
            if let Some(victim) = pos.board().victim(mv) {
                let delta_margin = victim.see_value() + shared_ctx.weights.delta_margin;

                if !Node::PV && !pos.in_check()
                    && calc_phase(pos.board()) < TOTAL_PHASE / 2
                    && static_eval < alpha - delta_margin {
                    continue;
                }
            }
        }

        pos.make_move(mv);
        let score = -q_search::<Node>(
            pos,
            ctx,
            shared_ctx,
            ply + 1,
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