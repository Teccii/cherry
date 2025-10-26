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
    if ply != 0 && (thread.abort_now || shared.time_man.abort_search(thread.nodes.global())) {
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
    let tt_entry = shared.ttable.fetch(pos.board());

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
    let static_eval = tt_entry.map(|e| e.eval).unwrap_or_else(|| pos.eval(&shared.nnue_weights));

    if !Node::PV && !in_check {
        let rfp_margin = (W::rfp_margin() * depth / DEPTH_SCALE) as i16;
        if depth < W::rfp_depth() && static_eval >= beta + rfp_margin {
            return static_eval;
        }

        if depth >= W::nmp_depth()
            && thread.search_stack[ply as usize - 1].move_played.is_some()
            && pos.null_move() {

            thread.search_stack[ply as usize].move_played = None;
            let score = -search::<Node>(pos, thread, shared, depth - W::nmp_reduction(), ply + 1, -beta, -beta + 1);
            pos.unmake_null_move();

            if thread.abort_now {
                return Score::INFINITE;
            }

            if score >= beta {
                return beta;
            }
        }
    }

    let lmp_margin = W::lmp_base() + W::lmp_margin() * depth as i64 * depth as i64 / (DEPTH_SCALE as i64 * 1024);
    let see_margins = [
        (W::see_quiet_margin() * depth as i64 * depth as i64 / (DEPTH_SCALE as i64 * DEPTH_SCALE as i64)) as i16,
        (W::see_tactic_margin() * depth / DEPTH_SCALE) as i16,
    ];

    let mut best_score = None;
    let mut moves_seen = 0;
    let mut move_picker = MovePicker::new(best_move);
    let mut tactics: SmallVec<[Move; 64]> = SmallVec::new();
    let mut quiets: SmallVec<[Move; 64]> = SmallVec::new();
    let mut flag = TTFlag::UpperBound;

    while let Some(ScoredMove(mv, _)) = move_picker.next(pos, &thread.history) {
        let is_tactic = mv.is_tactic();
        let mut score;

        if !Node::PV && ply != 0 && best_score.map_or(false, |s: Score| !s.is_loss()) {
            if !is_tactic {
                if moves_seen as i64 * 1024 >= lmp_margin {
                    move_picker.skip_quiets();
                }

                let futile_margin = (W::futile_base() + W::futile_margin() * depth / DEPTH_SCALE) as i16;
                if depth <= W::futile_depth() && !in_check && static_eval <= alpha - futile_margin {
                    move_picker.skip_quiets();
                }
            }

            if depth <= W::see_depth()
                && move_picker.stage() > Stage::YieldGoodTactics
                && !pos.board().cmp_see(mv, see_margins[is_tactic as usize]) {
                continue;
            }
        }

        thread.search_stack[ply as usize].move_played = Some(mv);
        pos.make_move(mv, &shared.nnue_weights);

        if moves_seen == 0 {
            score = -search::<Node>(pos, thread, shared, depth - 1 * DEPTH_SCALE, ply + 1, -beta, -alpha);
        } else {
            let lmr = get_lmr(is_tactic, (depth / DEPTH_SCALE) as u8, moves_seen);

            score = -search::<NonPV>(pos, thread, shared, depth - lmr - 1 * DEPTH_SCALE, ply + 1, -alpha - 1, -alpha);

            if lmr > 0 && score > alpha {
                score = -search::<NonPV>(pos, thread, shared, depth - 1 * DEPTH_SCALE, ply + 1, -alpha - 1, -alpha);
            }

            if Node::PV && score > alpha {
                score = -search::<PV>(pos, thread, shared, depth - 1 * DEPTH_SCALE, ply + 1, -beta, -alpha);
            }
        }
        pos.unmake_move();
        moves_seen += 1;

        if ply == 0 && moves_seen == 1 {
            let (parent, child) = thread.search_stack.split_at_mut(ply as usize + 1);
            let (parent, child) = (parent.last_mut().unwrap(), child.first().unwrap());

            parent.pv.update(mv, &child.pv);
        }

        if thread.abort_now {
            return Score::ZERO;
        }

        if best_score.is_none() || score > best_score.unwrap() {
            best_score = Some(score);
        }

        if score > alpha {
            alpha = score;
            best_move = Some(mv);
            flag = TTFlag::Exact;

            if Node::PV && !thread.abort_now {
                let (parent, child) = thread.search_stack.split_at_mut(ply as usize + 1);
                let (parent, child) = (parent.last_mut().unwrap(), child.first().unwrap());

                parent.pv.update(mv, &child.pv);
            }
        }

        if score >= beta {
            flag = TTFlag::LowerBound;

            if !thread.abort_now {
                thread.history.update(pos.board(), mv, &tactics, &quiets, depth);
            }

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
        return if in_check {
            Score::new_mated(ply)
        } else {
            Score::ZERO
        };
    }

    let best_score = best_score.unwrap();
    shared.ttable.store(
        pos.board(),
        (depth / DEPTH_SCALE) as u8,
        static_eval,
        best_score,
        best_move,
        flag,
        Node::PV || tt_entry.is_some_and(|e| e.pv)
    );

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
    if thread.abort_now || shared.time_man.abort_search(thread.nodes.global()) {
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

    let in_check = pos.board().in_check();
    let static_eval = pos.eval(&shared.nnue_weights);

    if !in_check {
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

    if !in_check {
        move_picker.skip_bad_tactics();
        move_picker.skip_quiets();
    }

    while let Some(ScoredMove(mv, _)) = move_picker.next(pos, &thread.history) {
        thread.search_stack[ply as usize].move_played = Some(mv);
        pos.make_move(mv, &shared.nnue_weights);
        let score = -q_search::<Node>(pos, thread, shared, ply + 1, -beta, -alpha);
        pos.unmake_move();
        moves_seen += 1;

        if thread.abort_now {
            return Score::ZERO;
        }

        if best_score.is_none() || score > best_score.unwrap() {
            best_score = Some(score);
        }

        if score > alpha {
            alpha = score;

            if Node::PV && !thread.abort_now {
                let (parent, child) = thread.search_stack.split_at_mut(ply as usize + 1);
                let (parent, child) = (parent.last_mut().unwrap(), child.first().unwrap());

                parent.pv.update(mv, &child.pv);
            }
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