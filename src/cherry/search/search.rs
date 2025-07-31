use arrayvec::ArrayVec;
use pyrrhic_rs::WdlProbeResult;
use crate::*;

/*----------------------------------------------------------------*/

pub trait NodeType {
    const PV: bool;
    const NMP: bool;
}

pub struct PV;
pub struct NonPV;
pub struct NoNMP;

impl NodeType for PV {
    const PV: bool = true;
    const NMP: bool = false;
}

impl NodeType for NonPV {
    const PV: bool = false;
    const NMP: bool = true;
}

impl NodeType for NoNMP {
    const PV: bool = false;
    const NMP: bool = false;
}

/*----------------------------------------------------------------*/

pub fn search<Node: NodeType>(
    pos: &mut Position,
    ctx: &mut ThreadContext,
    shared_ctx: &SharedContext,
    depth: u8,
    ply: u16,
    mut alpha: Score,
    mut beta: Score,
    cut_node: bool,
) -> Score {
    if ply != 0 && (ctx.abort_now || shared_ctx.time_man.abort_search(ctx.nodes.global())) {
        ctx.abort_now();
        return Score::INFINITE;
    }

    ctx.ss[ply as usize].pv.len = 0;

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

    ctx.nodes.inc();
    ctx.update_sel_depth(ply);

    if ply != 0 {
        if pos.is_draw() {
            return Score::ZERO;
        }

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

    let initial_alpha = alpha;
    let skip_move = ctx.ss[ply as usize].skip_move;
    let mut tt_entry = match skip_move {
        Some(_) => None,
        None => shared_ctx.t_table.probe(pos.board())
    };
    let mut best_move = None;

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
                TTBound::Exact => return score,
                TTBound::UpperBound => if score <= alpha {
                    return score;
                }
                TTBound::LowerBound => if score >= beta {
                    return score;
                },
                TTBound::None => unreachable!()
            }
        }
    } else {
        ctx.tt_misses.inc();
    }

    let (mut syzygy_max, mut syzygy_min) = (Score::MAX_MATE, -Score::MAX_MATE);
    if shared_ctx.syzygy.is_some()
        && ply != 0 && skip_move.is_none()
        && depth >= shared_ctx.syzygy_depth
        && pos.board().halfmove_clock() == 0
        && !pos.can_castle() {
        if let Some(wdl) = Option::as_ref(&shared_ctx.syzygy)
            .and_then(|tb| probe_wdl(tb, pos.board())) {
            ctx.tb_hits.inc();

            let tb_score = match wdl {
                WdlProbeResult::Win => Score::new_tb_win(ply),
                WdlProbeResult::Loss => Score::new_tb_loss(ply),
                _ => Score::ZERO
            };

            let tb_bound = match wdl {
                WdlProbeResult::Win => TTBound::LowerBound,
                WdlProbeResult::Loss => TTBound::UpperBound,
                _ => TTBound::Exact,
            };

            if tb_bound == TTBound::Exact
                || (tb_bound == TTBound::LowerBound && tb_score >= beta)
                || (tb_bound == TTBound::UpperBound && tb_score <= alpha) {
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

            if Node::PV && tb_bound == TTBound::LowerBound {
                alpha = alpha.max(tb_score);
                syzygy_min = tb_score;
            }

            if Node::PV && tb_bound == TTBound::UpperBound {
                syzygy_max = tb_score;
            }
        }
    }

    let in_check = pos.in_check();
    let corr = ctx.history.get_corr(pos.board(), &shared_ctx.weights);
    let raw_eval = match skip_move {
        Some(_) => ctx.ss[ply as usize].eval,
        None => tt_entry.and_then(|e| e.eval).unwrap_or_else(|| pos.eval(#[cfg(feature = "nnue")] &shared_ctx.nnue_weights)),
    };
    let static_eval = raw_eval + corr;
    let prev_eval = (ply >= 2).then(|| ctx.ss[ply as usize - 2].eval);
    let improving = prev_eval.is_some_and(|e| !in_check && raw_eval > e);
    let tt_pv = tt_entry.is_some_and(|e| e.flag == TTBound::Exact);
    let w = &shared_ctx.weights;

    ctx.ss[ply as usize].eval = raw_eval;
    ctx.ss[ply as usize].tt_pv = tt_pv;

    if !Node::PV && !in_check && skip_move.is_none() {
        /*
        Reverse Futility Pruning: Similar to Razoring, if the static evaluation of the position is *above*
        beta by a significant margin, we can assume that we can reach at least beta.
        */

        if depth < w.rfp_depth && static_eval >= beta + w.rfp_margin * depth as i16 {
            return (static_eval + beta) / 2
        }

        /*
        Null Move Pruning: In almost every position, there is a better legal move than doing nothing.
        If a reduced search after a null move fails high, we can be quite confident that the best legal move
        would also fail high. This can make the engine blind to zugzwang, so we do an additional verification search.
        */
        if Node::NMP && depth > w.nmp_depth
            && ctx.ss[ply as usize - 1].move_played.is_some()
            && static_eval >= beta
            && tt_entry.is_none_or(|e| e.flag != TTBound::UpperBound || e.score >= beta)
            && pos.non_pawn_material()
            && pos.null_move() {
            ctx.ss[ply as usize].move_played = None;
            shared_ctx.t_table.prefetch(pos.board());

            let nmp_depth = depth.saturating_sub(4 + depth / 3);
            let score = -search::<NoNMP>(
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
            if score >= beta {
                return beta;
            }
        }
    }
    
    let mut best_score = None;
    let mut moves_seen = 0;
    let mut move_exists = false;
    let mut quiets: ArrayVec<Move, MAX_MOVES> = ArrayVec::new();
    let mut tactics: ArrayVec<Move, MAX_MOVES> = ArrayVec::new();
    let mut move_picker = MovePicker::new(best_move);
    let cont_indices = ContIndices::new(&ctx.ss, ply);

    while let Some(mv) = move_picker.next(pos, &ctx.history, &cont_indices, w) {
        if skip_move == Some(mv) {
            continue;
        }

        move_exists = true;
        if ply == 0 && (!shared_ctx.root_moves.is_empty() && !shared_ctx.root_moves.contains(&mv)){
            continue;
        }

        let nodes = ctx.nodes.local();
        let is_tactical = pos.board().is_tactical(mv);
        let stat_score = if is_tactical {
            ctx.history.get_tactical(pos.board(), mv)
        } else {
            ctx.history.get_non_tactical(pos.board(), mv, &cont_indices, w)
        };
        
        ctx.ss[ply as usize].stat_score = stat_score;

        /*
        Late Move Reductions (LMR): Reduce the depth of moves ordered near the end.
        */
        let mut reduction = if is_tactical {
            shared_ctx.lmr_tactical.get(depth as usize, moves_seen as usize)
        } else {
            shared_ctx.lmr_quiet.get(depth as usize, moves_seen as usize)
        };
        let mut extension: i16 = 0;
        let mut score;

        if !Node::PV && ply != 0 && pos.non_pawn_material()
            && best_score.map_or(false, |s: Score| !s.is_decisive()) {

            if is_tactical {
                /*
                Tactical SEE Pruning: Skip tactical moves whose SEE score
                is below a depth-dependent margin.
                */
                let see_margin = w.see_margin * depth as i16 * depth as i16 - (stat_score / w.see_hist) as i16;
                if depth < w.see_depth
                    && move_picker.phase() == Phase::YieldBadTactics
                    && !pos.board().cmp_see(mv, see_margin) {
                    continue;
                }
            } else {
                /*
                Late Move Pruning: Start skipping quiet moves after
                a depth-dependent amount of moves.
                */
                let lmp_margin = (3 + depth as u16 * depth as u16) / (2 - improving as u16);
                if moves_seen >= lmp_margin {
                    move_picker.skip_quiets();
                }

                let r_depth = (depth as i32).saturating_sub(reduction / 1024).clamp(1, MAX_DEPTH as i32) as u8;

                /*
                History Pruning: Skip quiet moves whose history score
                is below an LMR depth-dependent margin.
                */
                if r_depth < w.hist_depth && stat_score < w.hist_margin * r_depth as i32 {
                    move_picker.skip_quiets();
                    continue;
                }

                /*
                Futility Pruning: Skip quiet moves if
                the static evaluation is below alpha by an
                LMR depth-dependent margin.
                */
                let futile_margin = w.futile_base + w.futile_margin * r_depth as i16;
                if r_depth < w.futile_depth && static_eval <= alpha - futile_margin {
                    move_picker.skip_quiets();
                }

                /*
                Quiet SEE Pruning: Skip quiet moves whose SEE score
                is below a depth-dependent margin. SEE works for quiets as
                well because for example, you can just lose your queen after
                moving it to an attacked square.
                */
                let see_margin = w.see_margin * r_depth as i16 * r_depth as i16;
                if r_depth < w.see_depth && !pos.board().cmp_see(mv, see_margin) {
                    continue;
                }
            }
        }

        ctx.ss[ply as usize].move_played = Some(MoveData::new(pos.board(), mv));
        pos.make_move(mv);
        shared_ctx.t_table.prefetch(pos.board());

        /*
        Check Extension: Extend the search if we give check.
        */
        if pos.in_check() {
            extension += 1;
        }

        ctx.ss[ply as usize].extension = extension;
        let depth = (depth as i16 + extension).clamp(0, MAX_DEPTH as i16) as u8;

        if moves_seen == 0 {
            ctx.ss[ply as usize].reduction = 0;
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
            reduction += w.tt_pv_reduction * tt_pv as i32;
            reduction += w.non_pv_reduction * !Node::PV as i32;
            reduction += w.not_improving_reduction * !improving as i32;
            reduction += w.cut_node_reduction * cut_node as i32;
            reduction -= stat_score / w.hist_reduction;
            reduction /= REDUCTION_SCALE;

            let r_depth = (depth as i32).saturating_sub(reduction).clamp(1, MAX_DEPTH as i32) as u8;

            ctx.ss[ply as usize].reduction = reduction;
            score = -search::<NonPV>(
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
                ctx.ss[ply as usize].reduction = 0;
                score = -search::<NonPV>(
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
                ctx.ss[ply as usize].reduction = 0;
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
        }

        if moves_seen == 1 || score > alpha {
            let child = &ctx.ss[ply as usize + 1];
            let (child_pv, len) = (child.pv.moves, child.pv.len);

            ctx.ss[ply as usize].pv.update(mv, &child_pv[..len]);
        }

        if score >= beta {
            if !ctx.abort_now {
                ctx.history.update(pos.board(), &cont_indices, w, mv, &quiets, &tactics, depth);
            }
            
            break;
        }
        
        if Some(mv) != best_move {
            if is_tactical {
                tactics.push(mv);
            } else {
                quiets.push(mv);
            }
        }
    }

    if !move_exists {
        return if skip_move.is_some() {
            alpha
        } else if pos.in_check() {
            Score::new_mated(ply)
        } else {
            Score::ZERO
        };
    }

    if ply == 0 {
        ctx.nodes.flush();
        ctx.qnodes.flush();
        ctx.tt_hits.flush();
        ctx.tt_misses.flush();
        ctx.tb_hits.flush();
    }

    let best_score = best_score.unwrap().clamp(syzygy_min, syzygy_max);
    if skip_move.is_none() && !ctx.abort_now {
        let flag = match () {
            _ if best_score <= initial_alpha => TTBound::UpperBound,
            _ if best_score >= beta => TTBound::LowerBound,
            _ => TTBound::Exact,
        };

        let is_tactic = best_move.is_some_and(|mv| !pos.board().is_tactical(mv));
        if !in_check && !is_tactic && flag == TTBound::Exact && best_score != static_eval {
            ctx.history.update_corr(pos.board(), depth, best_score, static_eval);
        }

        shared_ctx.t_table.store(
            pos.board(),
            depth,
            best_score,
            Some(raw_eval),
            best_move,
            flag
        );
    }

    best_score
}

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

    if ply >= MAX_PLY {
        return pos.eval(#[cfg(feature = "nnue")] &shared_ctx.nnue_weights) + ctx.history.get_corr(pos.board(), &shared_ctx.weights);
    }

    let tt_entry = shared_ctx.t_table.probe(pos.board());
    let initial_alpha = alpha;

    if let Some(entry) = tt_entry {
        ctx.tt_hits.inc();

        if !Node::PV {
            let score = entry.score;
            match entry.flag {
                TTBound::Exact => return score,
                TTBound::UpperBound => if score <= alpha {
                    return score;
                },
                TTBound::LowerBound => if score >= beta {
                    return score;
                },
                TTBound::None => unreachable!()
            }
        }
    } else {
        ctx.tt_misses.inc();
    }

    let in_check = pos.in_check();
    let corr = ctx.history.get_corr(pos.board(), &shared_ctx.weights);
    let raw_eval = tt_entry.and_then(|e| e.eval).unwrap_or_else(|| pos.eval(#[cfg(feature = "nnue")] &shared_ctx.nnue_weights));
    let static_eval = raw_eval + corr;

    if !in_check {
        if static_eval >= beta {
            return static_eval;
        }

        if static_eval >= alpha {
            alpha = static_eval;
        }
    }

    let mut best_move = None;
    let mut best_score = None;
    let mut moves_seen = 0;
    let mut move_picker = QMovePicker::new();
    let cont_indices = ContIndices::new(&ctx.ss, ply);

    while let Some(mv) = move_picker.next(pos, &ctx.history, &cont_indices, &shared_ctx.weights) {
        if !pos.board().cmp_see(mv, 0) {
            continue;
        }

        pos.make_move(mv);
        shared_ctx.t_table.prefetch(pos.board());

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

    if moves_seen == 0 && in_check {
        return Score::new_mated(ply);
    }

    if let Some(best_score) = best_score && !ctx.abort_now {
        let flag = match () {
            _ if best_score <= initial_alpha => TTBound::UpperBound,
            _ if best_score >= beta => TTBound::LowerBound,
            _ => TTBound::Exact,
        };

        shared_ctx.t_table.store(
            pos.board(),
            0,
            best_score,
            Some(raw_eval),
            best_move,
            flag
        );
    }

    best_score.unwrap_or(alpha)
}