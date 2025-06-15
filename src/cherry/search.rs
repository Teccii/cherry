use arrayvec::ArrayVec;
use cozy_chess::*;
use super::*;

/*----------------------------------------------------------------*/

macro_rules! params {
    ($($name:ident: $ty:ty = $default:expr,)*) => {
        #[derive(Debug, Copy, Clone)]
        pub struct SearchParams {
            $(pub $name: $ty),*
        }
        
        impl SearchParams {
            #[inline(always)]
            pub fn new($($name: $ty),*) -> SearchParams {
                SearchParams { $($name),* }
            }
        }
        
        impl Default for SearchParams {
            #[inline(always)]
            fn default() -> Self {
                SearchParams {
                    $($name: $default),*
                }
            }
        }
    }
}

params! {
    razor_margin: i16 = 257,
    rev_futile_margin_depth: i16 = 71,
    rev_futile_margin_improving: i16 = 43,
    double_margin: i16 = 47,
    triple_margin: i16 = 147,
}

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
    !Node::PV && eval < alpha - razor_margin(depth)
}

#[inline(always)]
fn razor_margin(depth: u8) -> i16 {
    257 * depth as i16
}

#[inline(always)]
fn rev_futile<Node: NodeType>(depth: u8, eval: Score, alpha: Score, beta: Score, improving: bool) -> bool {
    !Node::PV && !alpha.is_mate() && eval > beta + rev_futile_margin(depth, improving)
}

#[inline(always)]
fn rev_futile_margin(depth: u8, improving: bool) -> i16 {
    depth as i16 * 71 - improving as i16 * 43
}

#[inline(always)]
fn nmp<Node: NodeType>(board: &Board, depth: u8, eval: Score, beta: Score) -> bool {
    Node::NMP
        && depth > 4
        && eval >= beta
        && board.occupied() != (board.pieces(Piece::Pawn) | board.pieces(Piece::King))
}

#[inline(always)]
fn nmp_depth(depth: u8) -> u8 {
    depth.saturating_sub(4 + depth / 3).max(1)
}

#[inline(always)]
fn iir<Node: NodeType>(entry: TTData, depth: u8) -> i16 {
    if Node::PV && depth > 5 && entry.table_mv.is_none() {
        return 1 + (entry.depth >= depth) as i16;
    }
    
    0
}

#[inline(always)]
fn singular(entry: TTData, mv: Move, depth: u8, ply: u16) -> bool {
    depth >= 6
        && ply != 0
        && entry.table_mv == Some(mv)
        && entry.depth >= depth - 3
        && matches!(entry.flag, TTFlag::Exact | TTFlag::LowerBound)
        && !entry.score.is_mate()
}

const DOUBLE_MARGIN: i16 = 51;
const TRIPLE_MARGIN: i16 = 147;

#[inline(always)]
fn lmr<Node: NodeType>(board: &Board, mv: Move, depth: u8, moves_seen: u8) -> i16 {
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
    if Node::PV {
        ctx.search_stack[ply as usize].pv_len = 0;
    }
    
    if ply != 0 && (ctx.abort_now || shared_ctx.time_man.abort_search(ctx.nodes.global())) {
        ctx.abort_now();
        return Score::INFINITE;
    }

    ctx.nodes.inc();
    ctx.update_sel_depth(ply);

    if ply != 0 && pos.is_draw(ply) {
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
    let mut tt_entry = skip_move.and_then(|_| shared_ctx.t_table.get(pos.board()));

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

    let in_check = pos.in_check();
    let static_eval = match skip_move {
        Some(_) => ctx.search_stack[ply as usize].eval,
        None => tt_entry.map(|e| e.eval).unwrap_or_else(|| pos.eval(ply))
    };
    ctx.search_stack[ply as usize].eval = static_eval;
    
    let prev_eval = match ply {
        2.. => Some(ctx.search_stack[ply as usize - 2].eval),
        _ => None
    };
    let improving = prev_eval.is_some_and(|e| in_check && static_eval > e);

    if !in_check && skip_move.is_none() && !alpha.is_mate() && !beta.is_mate(){
        /*
        Reverse Futility Pruning: Similar to Razoring, if the static evaluation of the position is *above*
        beta by a significant margin, we can assume that we can reach at least beta.
        */
        if rev_futile::<Node>(depth, static_eval, alpha, beta, improving) {
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
        if nmp::<Node>(pos.board(), depth, static_eval, beta) && pos.null_move() {
            let nmp_depth = nmp_depth(depth);
            let mut score = -search::<Node::Alt>(
                pos,
                ctx,
                shared_ctx,
                nmp_depth,
                ply + 1,
                -beta,
                -beta + 1
            );

            pos.unmake_move();

            if score >= beta && !score.is_mate() {
                if depth < 12 {
                    return score;
                }

                score = search::<Node::Alt>(
                    pos,
                    ctx,
                    shared_ctx,
                    nmp_depth,
                    ply + 1,
                    beta - 1,
                    beta,
                );

                if score >= beta {
                    return score;
                }
            }
        }
    }
    
    if let Some(entry) = tt_entry {
        depth = (depth as i16 - iir::<Node>(entry, depth)) as u8;
    }

    let mut best_score = None;
    let mut quiets: ArrayVec<Move, MAX_MOVES> = ArrayVec::new();
    let mut captures: ArrayVec<Move, MAX_MOVES> = ArrayVec::new();
    let mut move_picker = MovePicker::new(best_move, ctx.search_stack[ply as usize].killers);
    let mut moves_seen = 0;

    ctx.search_stack[ply as usize + 1].killers.clear();

    while let Some(mv) = move_picker.next(pos, &ctx.history) {
        if skip_move == Some(mv) {
            continue;
        }

        let is_capture = pos.board().is_capture(mv);
        let nodes = ctx.nodes.local();
        
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

                if !Node::PV && s_score < s_beta - DOUBLE_MARGIN {
                    extension += 1;
                }

                if !Node::PV && !is_capture && s_score < s_beta - TRIPLE_MARGIN {
                    extension += 1;
                }

            } else if s_score >= beta && !s_score.is_mate() {
                return s_score;
            } else if entry.score >= beta {
                extension = -2;
            }
        }

        /*
        Late Move Reduction (LMR): Reduce the depth of moves ordered near the end.
        */
        reduction += lmr::<Node>(pos.board(), mv, depth, moves_seen);
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
                if pos.board().is_quiet(mv) {
                    ctx.search_stack[ply as usize].killers.push(mv);
                }

                ctx.history.update(
                    pos.board(),
                    mv,
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

    let best_score = best_score.unwrap();
    
    if skip_move.is_none() && !ctx.abort_now {
        let flag = match () {
            _ if best_score <= initial_alpha => TTFlag::UpperBound,
            _ if best_score >= beta => TTFlag::LowerBound,
            _ => TTFlag::Exact,
        };

        shared_ctx.t_table.set(
            pos.board(),
            depth,
            best_score,
            static_eval,
            best_move,
            flag
        );
    }

    best_score
}

/*----------------------------------------------------------------*/

fn delta<Node: NodeType>(pos: &Position, piece: Piece, sq: Square, eval: Score, alpha: Score) -> bool {
    let phase = calc_phase(pos.board());

    !Node::PV
        && phase < TAPER_SCALE * 3 / 4
        && !pos.in_check()
        && eval < alpha - delta_margin(pos, piece, sq, phase)
}

fn delta_margin(pos: &Position, piece: Piece, sq: Square, phase: u16) -> Score {
    let weights = pos.evaluator().weights();
    let value = piece_value(weights, !pos.stm(), piece, sq);

    (value + weights.pawn_value * 2).scale(phase)
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

    if pos.is_draw(ply) {
        return Score::ZERO;
    }
    
    let initial_alpha = alpha;

    let tt_entry = shared_ctx.t_table.get(pos.board());
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

    let static_eval = tt_entry.map(|e| e.eval).unwrap_or_else(|| pos.eval(ply));

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

    while let Some(mv) = move_picker.next(pos, qply, &ctx.history) {
        /*
        Delta Pruning: Similar to Futility Pruning, but only in Quiescence Search .
        We test whether the captured piece + a safety margin (around 200 centipawns)
        is enough to raise alpha.

        For example if we're down a rook, don't bother testing pawn captures,
        because they are unlikely to matter for us. For safety reasons, we cannot
        apply this in the late endgame.
        */
        if let Some(target) = pos.board().capture_piece(mv) && delta::<Node>(
            pos,
            target,
            pos.board().capture_square(mv).unwrap(),
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

    if !ctx.abort_now {
        if let Some(best_score) = best_score{
            let flag = match () {
                _ if best_score <= initial_alpha => TTFlag::UpperBound,
                _ if best_score >= beta => TTFlag::LowerBound,
                _ => TTFlag::Exact,
            };

            shared_ctx.t_table.set(
                pos.board(),
                0,
                best_score,
                static_eval,
                best_move,
                flag
            );
        }
    }

    best_score.unwrap_or(alpha)
}
