use std::sync::atomic::Ordering;

use pyrrhic_rs::WdlProbeResult;
use smallvec::SmallVec;

use crate::*;

/*----------------------------------------------------------------*/

pub trait NodeType {
    const PV: bool;
    const ROOT: bool;
    type Next: NodeType;
}

pub struct Root;
pub struct PV;
pub struct NonPV;

impl NodeType for Root {
    const PV: bool = true;
    const ROOT: bool = true;
    type Next = PV;
}

impl NodeType for PV {
    const PV: bool = true;
    const ROOT: bool = false;
    type Next = PV;
}

impl NodeType for NonPV {
    const PV: bool = false;
    const ROOT: bool = false;
    type Next = NonPV;
}

/*----------------------------------------------------------------*/

#[inline]
fn adjust_eval(raw_eval: Score, corr: i32) -> Score {
    (raw_eval + corr).clamp(-Score::MAX_TB_WIN + 1, Score::MAX_TB_WIN - 1)
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
    cut_node: bool,
) -> Score {
    if !Node::ROOT
        && (thread.abort_now
            || shared
                .time_man
                .abort_search(thread.nodes.local(), thread.nodes.global()))
    {
        thread.abort_now = true;
        return Score::ZERO;
    }

    if Node::PV {
        thread.search_stack[ply as usize].pv.len = 0;
    }

    if !Node::ROOT && pos.is_draw() {
        return Score::ZERO;
    }

    if depth <= 0 || ply >= MAX_PLY {
        return q_search::<Node>(pos, thread, shared, ply, alpha, beta);
    }

    thread.sel_depth = thread.sel_depth.max(ply);

    if !Node::ROOT {
        thread.nodes.inc();
    }

    let skip_move = thread.search_stack[ply as usize].skip_move;
    let tt_entry = match skip_move {
        Some(_) => None,
        None => shared.ttable.fetch(pos.board(), ply),
    };
    let tt_pv = Node::PV || tt_entry.is_some_and(|e| e.pv);

    if !Node::PV
        && let Some(entry) = tt_entry
        && entry.depth as i32 >= depth / DEPTH_SCALE
    {
        match entry.flag {
            TTFlag::Exact => return entry.score,
            TTFlag::UpperBound =>
                if entry.score <= alpha {
                    return entry.score;
                },
            TTFlag::LowerBound =>
                if entry.score >= beta {
                    return entry.score;
                },
            TTFlag::None => unreachable!(),
        }
    }

    let in_check = pos.board().in_check();
    let (raw_eval, static_eval, _corr) = if !in_check && skip_move.is_none() {
        let raw_eval = tt_entry
            .map(|e| e.eval)
            .unwrap_or_else(|| pos.eval(&shared.nnue_weights));
        let corr = thread.history.corr(pos.board());
        let static_eval = adjust_eval(raw_eval, corr);

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

    let (mut syzygy_min, mut syzygy_max) = (-Score::MIN_MATE, Score::MIN_MATE);

    if !Node::ROOT
        && skip_move.is_none()
        && is_syzygy_enabled()
        && depth >= shared.syzygy_depth.load(Ordering::Relaxed) as i32 * DEPTH_SCALE
        && pos.board().halfmove_clock() == 0
        && pos.board().castle_rights(Color::White).is_none()
        && pos.board().castle_rights(Color::Black).is_none()
        && let Some(wdl) = probe_wdl(pos.board())
    {
        let (score, flag) = match wdl {
            WdlProbeResult::Win => (Score::tb_win(ply), TTFlag::LowerBound),
            WdlProbeResult::Loss => (Score::tb_loss(ply), TTFlag::UpperBound),
            _ => (Score::ZERO, TTFlag::Exact),
        };

        if flag == TTFlag::Exact
            || (flag == TTFlag::UpperBound && score <= alpha)
            || (flag == TTFlag::LowerBound && score >= beta)
        {
            let depth_bias = if Node::PV {
                W::tt_depth_pv_bias()
            } else {
                W::tt_depth_bias()
            };

            shared.ttable.store(
                pos.board(),
                ((depth + depth_bias).min(MAX_FRAC_DEPTH) / DEPTH_SCALE) as u8,
                ply,
                raw_eval,
                score,
                None,
                flag,
                tt_pv,
            );

            return score;
        }

        if Node::PV {
            if flag == TTFlag::UpperBound {
                syzygy_max = score;
            } else {
                if score > alpha {
                    alpha = score;
                }

                syzygy_min = score;
            }
        }
    }

    if !Node::PV && !in_check && skip_move.is_none() {
        let rfp_margin = W::rfp_margin(improving, depth);
        if depth < W::rfp_depth() && static_eval - rfp_margin >= beta {
            return if !static_eval.is_win() && !beta.is_win() {
                Score(static_eval.0 + W::rfp_lerp() * (beta.0 - static_eval.0) / 1024)
            } else {
                static_eval
            };
        }

        if depth >= W::nmp_depth()
            && pos.prev_move(1).is_some()
            && static_eval >= beta
            && pos.null_move()
        {
            shared.ttable.prefetch(pos.board());

            let nmp_depth = depth - W::nmp_reduction(depth);
            let score = -search::<NonPV>(
                pos,
                thread,
                shared,
                nmp_depth,
                ply + 1,
                -beta,
                -beta + 1,
                !cut_node,
            );
            pos.unmake_null_move();

            if thread.abort_now {
                return Score::ZERO;
            }

            if score >= beta {
                if depth <= W::nmp_verif_depth() {
                    return if score.is_win() { beta } else { score };
                }

                let v_score =
                    search::<NonPV>(pos, thread, shared, nmp_depth, ply, beta - 1, beta, true);

                if v_score >= beta {
                    return v_score;
                }
            }
        }
    }

    let mut best_move = None;
    let mut best_score = -Score::INFINITE;
    let mut moves_seen = 0;
    let mut flag = TTFlag::UpperBound;
    let mut move_picker = MovePicker::new(tt_entry.and_then(|e| e.mv));
    let mut tactics: SmallVec<[Move; 64]> = SmallVec::new();
    let mut quiets: SmallVec<[Move; 64]> = SmallVec::new();
    let cont_indices = ContIndices::new(&pos);

    let lmr_depth_bias = if tt_pv {
        W::lmr_depth_pv_bias()
    } else {
        W::lmr_depth_bias()
    };
    let lmr_lookup_depth = ((depth + lmr_depth_bias).min(MAX_FRAC_DEPTH) / DEPTH_SCALE) as u8;

    while let Some(ScoredMove(mv, _)) = move_picker.next(pos, &thread.history, &cont_indices) {
        if skip_move == Some(mv) {
            continue;
        }

        if Node::ROOT
            && ((!thread.root_moves.is_empty() && !thread.root_moves.contains(&mv))
                || thread.exclude_moves.contains(&mv))
        {
            continue;
        }

        let is_tactic = mv.is_tactic();
        let nodes = thread.nodes.local();
        let mut lmr = W::lmr(is_tactic, lmr_lookup_depth, moves_seen);
        let mut score;

        if !Node::ROOT && !best_score.is_loss() {
            if is_tactic {
                let see_margin = W::see_tactic_margin(depth);
                if !Node::PV
                    && depth <= W::see_tactic_depth()
                    && move_picker.stage() > Stage::YieldGoodTactics
                    && !pos.cmp_see(mv, see_margin)
                {
                    continue;
                }
            } else {
                let lmp_margin = W::lmp_margin(improving, depth);
                if moves_seen as i64 * 1024 >= lmp_margin {
                    move_picker.skip_quiets();
                }

                let lmr_depth = (depth - lmr).max(0);
                let fp_margin = W::fp_margin(improving, lmr_depth);
                if !Node::PV
                    && lmr_depth <= W::fp_depth()
                    && !in_check
                    && static_eval + fp_margin <= alpha
                {
                    move_picker.skip_quiets();
                }

                let see_margin = W::see_quiet_margin(lmr_depth);
                if !Node::PV && lmr_depth <= W::see_quiet_depth() && !pos.cmp_see(mv, see_margin) {
                    continue;
                }
            }
        }

        let mut ext = 0;
        if !Node::ROOT
            && depth >= W::singular_depth()
            && skip_move.is_none()
            && let Some(entry) = tt_entry
            && entry.mv == Some(mv)
            && entry.depth as i32 * DEPTH_SCALE + W::singular_tt_depth() >= depth
            && entry.flag != TTFlag::UpperBound
        {
            let s_beta = entry.score - depth * W::singular_beta_margin() / (DEPTH_SCALE * 64);
            let s_depth = depth * W::singular_search_depth() / DEPTH_SCALE;

            thread.search_stack[ply as usize].skip_move = Some(mv);
            let s_score = search::<NonPV>(
                pos,
                thread,
                shared,
                s_depth,
                ply,
                s_beta - 1,
                s_beta,
                cut_node,
            );
            thread.search_stack[ply as usize].skip_move = None;

            if s_score < s_beta {
                ext = W::singular_ext();

                if !Node::PV && s_score + W::singular_dext_margin() < s_beta {
                    ext = W::singular_dext();
                }
            } else if s_beta >= beta {
                return s_beta;
            } else if entry.score >= beta {
                ext = W::singular_tt_ext();
            } else if cut_node {
                ext = W::singular_cut_ext();
            }
        }

        pos.make_move(mv, &shared.nnue_weights);
        shared.ttable.prefetch(pos.board());

        let new_depth = (depth + ext - 1 * DEPTH_SCALE).min(MAX_FRAC_DEPTH);
        if moves_seen == 0 {
            score = -search::<Node::Next>(
                pos,
                thread,
                shared,
                new_depth,
                ply + 1,
                -beta,
                -alpha,
                !Node::PV && !cut_node,
            );
        } else {
            if depth >= W::lmr_depth() {
                lmr += W::cut_lmr() * cut_node as i32;
                lmr -= W::improving_lmr() * improving as i32;
                lmr += W::non_pv_lmr() * !Node::PV as i32;
                lmr -= W::tt_pv_lmr() * tt_pv as i32;
                lmr -= W::check_lmr() * pos.board().in_check() as i32;
            } else {
                lmr = 0;
            }

            let lmr_depth = (new_depth - lmr).max(1 * DEPTH_SCALE).min(new_depth);

            score = -search::<NonPV>(
                pos,
                thread,
                shared,
                lmr_depth,
                ply + 1,
                -alpha - 1,
                -alpha,
                true,
            );

            if lmr_depth < new_depth && score > alpha {
                score = -search::<NonPV>(
                    pos,
                    thread,
                    shared,
                    new_depth,
                    ply + 1,
                    -alpha - 1,
                    -alpha,
                    !cut_node,
                );
            }

            if Node::PV && score > alpha {
                score = -search::<PV>(
                    pos,
                    thread,
                    shared,
                    new_depth,
                    ply + 1,
                    -beta,
                    -alpha,
                    false,
                );
            }
        }
        pos.unmake_move();
        moves_seen += 1;

        if (Node::ROOT && moves_seen == 1) || (Node::PV && score > alpha) {
            let (parent, child) = thread.search_stack.split_at_mut(ply as usize + 1);
            let (parent, child) = (parent.last_mut().unwrap(), child.first().unwrap());

            parent.pv.update(mv, &child.pv);
        }

        if thread.abort_now {
            return Score::ZERO;
        }

        if Node::ROOT {
            thread.root_nodes[mv.src()][mv.dest()] += thread.nodes.local() - nodes;
        }

        if score > best_score {
            best_score = score;
        }

        if score > alpha {
            alpha = score;
            best_move = Some(mv);
            flag = TTFlag::Exact;
        }

        if score >= beta {
            flag = TTFlag::LowerBound;
            thread
                .history
                .update(pos.board(), &cont_indices, depth, mv, &quiets, &tactics);
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
            Score::mated(ply)
        } else {
            Score::ZERO
        };
    }

    let best_score = best_score.clamp(syzygy_min, syzygy_max);
    if skip_move.is_none() {
        let tt_depth_bias = if tt_pv {
            W::tt_depth_pv_bias()
        } else {
            W::tt_depth_bias()
        };

        shared.ttable.store(
            pos.board(),
            ((depth + tt_depth_bias).min(MAX_FRAC_DEPTH) / DEPTH_SCALE) as u8,
            ply,
            raw_eval,
            best_score,
            best_move,
            flag,
            tt_pv,
        );

        let static_eval = if !in_check {
            adjust_eval(raw_eval, thread.history.corr(pos.board()))
        } else {
            Score::NONE
        };

        if !in_check
            && best_move.is_none_or(|mv| !mv.is_tactic())
            && match flag {
                TTFlag::Exact => true,
                TTFlag::LowerBound => best_score > static_eval,
                TTFlag::UpperBound => best_score < static_eval,
                TTFlag::None => unreachable!(),
            }
        {
            thread
                .history
                .update_corr(pos.board(), depth, best_score, static_eval);
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
    if thread.abort_now
        || shared
            .time_man
            .abort_search(thread.nodes.local(), thread.nodes.global())
    {
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
        let raw_eval = pos.eval(&shared.nnue_weights);
        let corr = thread.history.corr(pos.board());

        return adjust_eval(raw_eval, corr);
    }

    thread.sel_depth = thread.sel_depth.max(ply);
    thread.nodes.inc();

    let tt_entry = shared.ttable.fetch(pos.board(), ply);
    if !Node::PV
        && let Some(entry) = tt_entry
    {
        match entry.flag {
            TTFlag::Exact => return entry.score,
            TTFlag::UpperBound =>
                if entry.score <= alpha {
                    return entry.score;
                },
            TTFlag::LowerBound =>
                if entry.score >= beta {
                    return entry.score;
                },
            TTFlag::None => unreachable!(),
        }
    }

    let in_check = pos.board().in_check();
    let static_eval;

    if in_check {
        static_eval = Score::mated(ply);
    } else {
        let raw_eval = tt_entry
            .map(|e| e.eval)
            .unwrap_or_else(|| pos.eval(&shared.nnue_weights));
        let corr = thread.history.corr(pos.board());
        static_eval = adjust_eval(raw_eval, corr);

        if static_eval >= beta {
            return static_eval;
        }

        if static_eval >= alpha {
            alpha = static_eval;
        }
    }

    let mut best_score = static_eval;
    let mut moves_seen = 0;
    let mut move_picker = MovePicker::new(None);
    let cont_indices = ContIndices::new(&pos);

    if !in_check {
        move_picker.skip_bad_tactics();
        move_picker.skip_quiets();
    }

    while let Some(ScoredMove(mv, _)) = move_picker.next(pos, &thread.history, &cont_indices) {
        pos.make_move(mv, &shared.nnue_weights);
        shared.ttable.prefetch(pos.board());
        let score = -q_search::<Node::Next>(pos, thread, shared, ply + 1, -beta, -alpha);
        pos.unmake_move();
        moves_seen += 1;

        if thread.abort_now {
            return Score::ZERO;
        }

        if Node::PV && score > alpha {
            let (parent, child) = thread.search_stack.split_at_mut(ply as usize + 1);
            let (parent, child) = (parent.last_mut().unwrap(), child.first().unwrap());

            parent.pv.update(mv, &child.pv);
        }

        if score > best_score {
            best_score = score;
        }

        if score > alpha {
            alpha = score;
        }

        if score >= beta {
            break;
        }
    }

    if moves_seen == 0 && in_check {
        return Score::mated(ply);
    }

    best_score
}
