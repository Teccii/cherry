use crate::*;

/*----------------------------------------------------------------*/

pub trait SearchInfo {
    fn new(frc: bool) -> Self;

    fn update(
        &mut self,
        board: &Board,
        thread: &ThreadData,
        shared: &SharedData,
        multipv: usize,
        pv_index: usize,
        pv: &PrincipalVariation,
        bound: TTFlag,
        score: Score,
        depth: u8,
    );
}

/*----------------------------------------------------------------*/

pub struct UciInfo {
    frc: bool,
}
pub struct NoInfo;

/*----------------------------------------------------------------*/

impl SearchInfo for UciInfo {
    #[inline]
    fn new(frc: bool) -> Self {
        Self { frc }
    }

    fn update(
        &mut self,
        board: &Board,
        thread: &ThreadData,
        shared: &SharedData,
        multipv: usize,
        pv_index: usize,
        pv: &PrincipalVariation,
        bound: TTFlag,
        score: Score,
        depth: u8,
    ) {
        let nodes = thread.nodes.global();
        let time = shared.time_man.elapsed();

        println!(
            "info depth {} seldepth {} {}score {} {}time {} nodes {} nps {} pv {}",
            depth,
            thread.sel_depth,
            if multipv > 1 {
                format!("multipv {} ", pv_index + 1)
            } else {
                String::new()
            },
            score,
            match bound {
                TTFlag::Exact => "",
                TTFlag::UpperBound => "upperbound ",
                TTFlag::LowerBound => "lowerbound ",
                TTFlag::None => "",
            },
            time,
            nodes,
            ((nodes as f64) / (time.max(1) as f64) * 1000.0) as u64,
            pv.display(board, self.frc)
        );
    }
}

/*----------------------------------------------------------------*/

impl SearchInfo for NoInfo {
    #[inline]
    fn new(_: bool) -> Self {
        Self
    }

    fn update(
        &mut self,
        _: &Board,
        _: &ThreadData,
        _: &SharedData,
        _: usize,
        _: usize,
        _: &PrincipalVariation,
        _: TTFlag,
        _: Score,
        _: u8,
    ) {
    }
}
