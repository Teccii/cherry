use smallvec::SmallVec;
use crate::*;

/*----------------------------------------------------------------*/

pub trait NodeType {
    const PV: bool;
}

pub struct PV;
pub struct NonPV;

impl NodeType for PV {
    const PV: bool = true;
}

impl NodeType for NonPV {
    const PV: bool = false;
}

/*----------------------------------------------------------------*/

pub fn search<Node: NodeType>(
    pos: &mut Position,
    thread: &mut ThreadData,
    shared: &SharedData,
    depth: i32,
    ply: u16,
    mut alpha: Score,
    beta: Score,
) -> Score {
    if ply != 0 && (thread.abort_now || (thread.nodes.local() % 1024 == 0 && shared.time_man.abort_search(thread.nodes.global()))) {
        thread.abort_now = true;

        return Score::ZERO;
    }

    if Node::PV {
        thread.search_stack[ply as usize].pv.len = 0;
    }

    if ply != 0 && pos.is_draw() {
        return Score::ZERO;
    }

    if depth <= 0 || ply >= MAX_PLY {
        return q_search::<Node>(pos, thread, shared, ply, alpha, beta);
    }

    thread.sel_depth = thread.sel_depth.max(ply);
    thread.nodes.inc();

    let mut best_move = None;
    let skip_move = thread.search_stack[ply as usize].skip_move;
    let tt_entry = match skip_move {
        Some(_) => None,
        None => shared.ttable.fetch(pos.board(), ply),
    };
    let tt_pv = Node::PV || tt_entry.is_some_and(|e| e.pv);

    if let Some(entry) = tt_entry {
        best_move = entry.mv;

        if !Node::PV && entry.depth as i32 >= depth / DEPTH_SCALE {
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
        }
    }

    let in_check = pos.board().in_check();
    let (raw_eval, static_eval, _corr) = if !in_check && skip_move.is_none() {
        let raw_eval = tt_entry.map(|e| e.eval).unwrap_or_else(|| pos.eval(&shared.nnue_weights));
        let corr = thread.history.get_corr(pos.board());
        let static_eval = Score::clamp(raw_eval + corr as i16, -Score::MIN_TB_WIN, Score::MIN_TB_WIN);

        (raw_eval, static_eval, corr)
    } else {
        (Score::NONE, Score::NONE, 0)
    };

    let improving = !in_check && skip_move.is_none() && {
        let ss = &thread.search_stack;
        let prev2 = ply.wrapping_sub(2) as usize;
        let prev4 = ply.wrapping_sub(4) as usize;

        if ply >= 2 && ss[prev2].static_eval != Score::NONE {
            static_eval > ss[prev2].static_eval
        } else if ply >= 4 && ss[prev4].static_eval != Score::NONE {
            static_eval > ss[prev4].static_eval
        } else {
            true
        }
    };

    thread.search_stack[ply as usize].static_eval = static_eval;

    if !Node::PV && !in_check && skip_move.is_none() {
        let (rfp_depth, rfp_base, rfp_scale, rfp_lerp) = (
            W::rfp_depth()[improving as usize],
            W::rfp_base()[improving as usize],
            W::rfp_scale()[improving as usize],
            W::rfp_lerp()[improving as usize],
        );
        let rfp_margin = (rfp_base + rfp_scale * depth / DEPTH_SCALE) as i16;
        if depth < rfp_depth && static_eval - rfp_margin >= beta {
            return if !static_eval.is_win() && !beta.is_win() {
                let (static_eval, beta) = (i32::from(static_eval.0), i32::from(beta.0));

                Score::new((static_eval + rfp_lerp * (beta - static_eval) / 1024) as i16)
            } else {
                static_eval
            };
        }

        let (nmp_depth, nmp_base, nmp_scale) = (
            W::nmp_depth()[improving as usize],
            W::nmp_base()[improving as usize],
            W::nmp_scale()[improving as usize],
        );
        if depth >= nmp_depth
            && thread.search_stack[ply as usize - 1].move_played.is_some()
            && static_eval >= beta
            && pos.null_move() {
            let nmp_reduction = (nmp_base + nmp_scale * depth as i64 / DEPTH_SCALE as i64) as i32;

            thread.search_stack[ply as usize].move_played = None;
            let score = -search::<Node>(pos, thread, shared, depth - nmp_reduction, ply + 1, -beta, -beta + 1);
            pos.unmake_null_move();

            if thread.abort_now {
                return Score::INFINITE;
            }

            if score >= beta {
                return beta;
            }
        }
    }

    let mut best_score = None;
    let mut moves_seen = 0;
    let mut flag = TTFlag::UpperBound;
    let mut move_picker = MovePicker::new(best_move);
    let mut tactics: SmallVec<[Move; 64]> = SmallVec::new();
    let mut quiets: SmallVec<[Move; 64]> = SmallVec::new();
    let cont_indices = ContIndices::new(&thread.search_stack, ply);

    while let Some(ScoredMove(mv, _)) = move_picker.next(pos, &thread.history, &cont_indices) {
        if skip_move == Some(mv) {
            continue;
        }

        let is_tactic = mv.is_tactic();
        let nodes = thread.nodes.local();
        let lmr = get_lmr(is_tactic, improving, (depth / DEPTH_SCALE) as u8, moves_seen);
        let mut score;

        if !Node::PV && best_score.map_or(false, |s: Score| !s.is_loss()) {
            if is_tactic {
                let (see_depth, see_base, see_scale) = (
                    W::see_tactic_depth()[improving as usize],
                    W::see_tactic_base()[improving as usize],
                    W::see_tactic_scale()[improving as usize],
                );
                let see_margin = (see_base + see_scale * depth / DEPTH_SCALE) as i16;
                if depth <= see_depth
                    && move_picker.stage() > Stage::YieldGoodTactics
                    && !pos.board().cmp_see(mv, see_margin) {
                    continue;
                }
            } else {
                let (lmp_base, lmp_scale) = (W::lmp_base()[improving as usize], W::lmp_scale()[improving as usize]);
                let lmp_margin = lmp_base + lmp_scale * depth as i64 * depth as i64 / (DEPTH_SCALE as i64 * DEPTH_SCALE as i64);
                if moves_seen as i64 * 1024 >= lmp_margin {
                    move_picker.skip_quiets();
                }
                
                let lmr_depth = (depth - lmr).max(0);

                let (futile_depth, futile_base, futile_scale) = (
                    W::futile_depth()[improving as usize],
                    W::futile_base()[improving as usize],
                    W::futile_scale()[improving as usize],
                );
                let futile_margin = (futile_base + futile_scale * lmr_depth / DEPTH_SCALE) as i16;
                if lmr_depth <= futile_depth && !in_check && static_eval + futile_margin <= alpha {
                    move_picker.skip_quiets();
                }

                let (see_depth, see_base, see_scale) = (
                    W::see_quiet_depth()[improving as usize],
                    W::see_quiet_base()[improving as usize],
                    W::see_quiet_scale()[improving as usize]
                );
                let see_margin = (see_base + see_scale * lmr_depth / DEPTH_SCALE) as i16;
                if lmr_depth <= see_depth
                    && move_picker.stage() > Stage::YieldGoodTactics
                    && !pos.board().cmp_see(mv, see_margin) {
                    continue;
                }
            }
        }

        let mut ext = 0;

        if ply != 0
            && depth >= W::singular_depth()[tt_pv as usize]
            && skip_move.is_none()
            && let Some(entry) = tt_entry
            && entry.mv == Some(mv)
            && entry.depth as i32 * DEPTH_SCALE + W::singular_tt_depth()[tt_pv as usize] >= depth
            && entry.flag != TTFlag::UpperBound {
            let s_beta = entry.score - (depth * W::singular_beta_margin()[tt_pv as usize] / (DEPTH_SCALE * 64)) as i16;
            let s_depth = depth * W::singular_search_depth()[tt_pv as usize] / DEPTH_SCALE;

            thread.search_stack[ply as usize].skip_move = Some(mv);
            let s_score = search::<NonPV>(pos, thread, shared, s_depth, ply, s_beta - 1, s_beta);
            thread.search_stack[ply as usize].skip_move = None;

            if s_score < s_beta {
                ext += W::singular_ext()[tt_pv as usize];
                if !Node::PV && s_score + W::singular_dext_margin()[tt_pv as usize] < s_beta {
                    ext += W::singular_dext()[tt_pv as usize];
                }
            } else if s_beta >= beta {
                return s_beta;
            } else if entry.score >= beta {
                ext = W::singular_neg_ext()[tt_pv as usize];
            }
        }

        thread.search_stack[ply as usize].move_played = Some(MoveData::new(pos.board(), mv));
        pos.make_move(mv, &shared.nnue_weights);

        let new_depth = depth + ext - 1 * DEPTH_SCALE;
        if moves_seen == 0 {
            score = -search::<Node>(pos, thread, shared, new_depth, ply + 1, -beta, -alpha);
        } else {
            score = -search::<NonPV>(pos, thread, shared, new_depth - lmr, ply + 1, -alpha - 1, -alpha);

            if lmr > 0 && score > alpha {
                score = -search::<NonPV>(pos, thread, shared, new_depth, ply + 1, -alpha - 1, -alpha);
            }

            if Node::PV && score > alpha {
                score = -search::<PV>(pos, thread, shared, new_depth, ply + 1, -beta, -alpha);
            }
        }
        pos.unmake_move();
        moves_seen += 1;

        if thread.abort_now {
            return Score::ZERO;
        }

        if ply == 0 {
            thread.root_nodes[mv.src()][mv.dest()] += thread.nodes.local() - nodes;
        }

        if Node::PV && (moves_seen == 1 || score > alpha) {
            let (parent, child) = thread.search_stack.split_at_mut(ply as usize + 1);
            let (parent, child) = (parent.last_mut().unwrap(), child.first().unwrap());

            parent.pv.update(mv, &child.pv);
        }

        if best_score.is_none() || score > best_score.unwrap() {
            best_score = Some(score);
        }

        if score > alpha {
            alpha = score;
            best_move = Some(mv);
            flag = TTFlag::Exact;
        }

        if score >= beta {
            flag = TTFlag::LowerBound;
            thread.history.update(pos.board(), &cont_indices, depth, mv, &tactics, &quiets);
            break;
        }

        if best_move != Some(mv) {
            if is_tactic {
                tactics.push(mv);
            } else {
                quiets.push(mv);
            }
        }
    }

    if moves_seen == 0 {
        return if skip_move.is_some() {
            alpha
        } else if in_check {
            Score::new_mated(ply)
        } else {
            Score::ZERO
        };
    }

    let best_score = best_score.unwrap();
    if skip_move.is_none() {
        shared.ttable.store(
            pos.board(),
            ((depth + W::tt_depth_bias()[Node::PV as usize]) / DEPTH_SCALE) as u8,
            ply,
            raw_eval,
            best_score,
            best_move,
            flag,
            tt_pv
        );

        if !in_check && best_move.is_none_or(|mv| !mv.is_tactic()) && match flag {
            TTFlag::Exact => true,
            TTFlag::LowerBound => best_score > static_eval,
            TTFlag::UpperBound => best_score < static_eval,
            TTFlag::None => unreachable!(),
        } {
            thread.history.update_corr(pos.board(), depth, best_score, static_eval);
        }
    }

    best_score
}

/*----------------------------------------------------------------*/

fn q_search<Node: NodeType>(
    pos: &mut Position,
    thread: &mut ThreadData,
    shared: &SharedData,
    ply: u16,
    mut alpha: Score,
    beta: Score,
) -> Score {
    if thread.abort_now || (thread.nodes.local() % 1024 == 0 && shared.time_man.abort_search(thread.nodes.global())) {
        thread.abort_now = true;

        return Score::ZERO;
    }

    if Node::PV {
        thread.search_stack[ply as usize].pv.len = 0;
    }

    if pos.is_draw() {
        return Score::ZERO;
    }

    if ply >= MAX_PLY {
        return pos.eval(&shared.nnue_weights);
    }

    thread.sel_depth = thread.sel_depth.max(ply);
    thread.nodes.inc();

    let tt_entry = shared.ttable.fetch(pos.board(), ply);
    if let Some(entry) = tt_entry {
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
    }

    let in_check = pos.board().in_check();
    if !in_check {
        let raw_eval = tt_entry.map(|e| e.eval).unwrap_or_else(|| pos.eval(&shared.nnue_weights));
        let corr = thread.history.get_corr(pos.board());
        let static_eval = Score::clamp(raw_eval + corr as i16, -Score::MIN_TB_WIN, Score::MIN_TB_WIN);

        if static_eval >= beta {
            return static_eval;
        }

        if static_eval >= alpha {
            alpha = static_eval;
        }
    }

    let mut best_score = None;
    let mut moves_seen = 0;
    let mut move_picker = MovePicker::new(None);
    let cont_indices = ContIndices::new(&thread.search_stack, ply);

    if !in_check {
        move_picker.skip_bad_tactics();
        move_picker.skip_quiets();
    }

    while let Some(ScoredMove(mv, _)) = move_picker.next(pos, &thread.history, &cont_indices) {
        thread.search_stack[ply as usize].move_played = Some(MoveData::new(pos.board(), mv));
        pos.make_move(mv, &shared.nnue_weights);
        let score = -q_search::<Node>(pos, thread, shared, ply + 1, -beta, -alpha);
        pos.unmake_move();
        moves_seen += 1;

        if thread.abort_now {
            return Score::ZERO;
        }

        if Node::PV && (moves_seen == 1 || score > alpha) {
            let (parent, child) = thread.search_stack.split_at_mut(ply as usize + 1);
            let (parent, child) = (parent.last_mut().unwrap(), child.first().unwrap());

            parent.pv.update(mv, &child.pv);
        }

        if best_score.is_none() || score > best_score.unwrap() {
            best_score = Some(score);
        }

        if score > alpha {
            alpha = score;
        }

        if score >= beta {
            break;
        }
    }

    if moves_seen == 0 && in_check {
        return Score::new_mated(ply);
    }

    best_score.unwrap_or(alpha)
}