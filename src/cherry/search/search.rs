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

        return Score::INFINITE;
    }

    if Node::PV {
        thread.search_stack[ply as usize].pv.len = 0;
    }

    if ply != 0 && pos.is_draw() {
        return Score::ZERO;
    }

    if depth <= 0 || ply >= MAX_PLY {
        return pos.eval(&shared.nnue_weights);
    }

    thread.sel_depth = thread.sel_depth.max(ply);
    thread.nodes.inc();

    let in_check = pos.board().in_check();

    let mut best_move = None;
    let mut best_score = None;
    let mut moves_seen = 0;
    let mut move_picker = MovePicker::new();
    let mut tactics: SmallVec<[Move; 64]> = SmallVec::new();
    let mut quiets: SmallVec<[Move; 64]> = SmallVec::new();

    while let Some(ScoredMove(mv, _)) = move_picker.next(pos, &thread.history) {
        pos.make_move(mv, &shared.nnue_weights);
        let score = -search::<Node>(pos, thread, shared, depth - 1 * DEPTH_SCALE, ply + 1, -beta, -alpha);
        pos.unmake_move();

        moves_seen += 1;

        if ply == 0 && moves_seen == 1 {
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

            if Node::PV && !thread.abort_now {
                let (parent, child) = thread.search_stack.split_at_mut(ply as usize + 1);
                let (parent, child) = (parent.last_mut().unwrap(), child.first().unwrap());

                parent.pv.update(mv, &child.pv);
            }
        }

        if score >= beta {
            if !thread.abort_now {
                thread.history.update(pos.board(), mv, &tactics, &quiets, depth);
            }

            break;
        }

        if best_move != Some(mv) {
            if mv.is_tactic() {
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

    if ply == 0 {
        thread.nodes.flush();
    }

    best_score.unwrap_or(alpha)
}