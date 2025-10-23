use crate::*;

/*----------------------------------------------------------------*/

pub trait SearchInfo {
    fn new(frc: bool) -> Self;

    fn update(
        &mut self,
        board: &Board,
        thread: &ThreadData,
        shared: &SharedData,
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
        score: Score,
        depth: u8,
    ) {
        let nodes = thread.nodes.global();
        let time = shared.time_man.elapsed();

        println!(
            "info depth {} seldepth {} score {} time {} nodes {} nps {} pv {}",
            depth,
            thread.sel_depth,
            score,
            time,
            nodes,
            nodes / time.max(1) * 1000,
            thread.root_pv.display(board, self.frc)
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
        _: Score,
        _: u8,
    ) { }
}