use bullet::{
    game::{
        formats::sfbinpack::{
            chess::{
                piecetype::PieceType,
                r#move::MoveType,
            },
            TrainingDataEntry,
        },
        inputs,
    },
    value::{loader, ValueTrainerBuilder},
    trainer::save::SavedFormat,
    nn::optimiser,
    wdl,
    lr,
    LocalSettings,
    TrainingSchedule,
    TrainingSteps,
};
use crate::{HL, L1, QA, QB, EVAL_SCALE};

pub fn tune(
    threads: usize,
    buffer_size: usize,
    queue_size: usize,
    file_paths: &[&str]
) {
    let mut trainer = ValueTrainerBuilder::default()
        .dual_perspective()
        .optimiser(optimiser::AdamW)
        .inputs(inputs::Chess768)
        .save_format(&[
            SavedFormat::id("ftw").quantise::<i16>(QA),
            SavedFormat::id("ftb").quantise::<i16>(QA),
            SavedFormat::id("l1w").quantise::<i16>(QB),
            SavedFormat::id("l1b").quantise::<i16>(QA * QB),
        ])
        .loss_fn(|output, target| output.sigmoid().squared_error(target))
        .build(|builder, stm, nstm| {
            let ft = builder.new_affine("ft", 768, HL);
            let l1 = builder.new_affine("l1", L1, 1);

            let stm = ft.forward(stm).screlu();
            let nstm = ft.forward(nstm).screlu();
            let ft_output = stm.concat(nstm);

            l1.forward(ft_output);
        });

    let schedule = TrainingSchedule {
        net_id: "cherry_768x2-1024",
        eval_scale: EVAL_SCALE,
        wdl_scheduler: wdl::ConstantWDL { value: 0.75 },
        lr_scheduler: lr::StepLR { start: 0.001, gamma: 0.1, step: 18 },
        steps: TrainingSteps {
            batch_size: 16384,
            batches_per_superbatch: 4096,
            start_superbatch: 1,
            end_superbatch: 64,
        },
        save_rate: 10,
    };

    let settings = LocalSettings {
        threads,
        test_set: None,
        output_directory: "data/training/checkpoints",
        batch_queue_size: queue_size,
    };

    let data_loader = {
        fn filter(entry: &TrainingDataEntry) -> bool {
            entry.ply >= 12
                && entry.score.unsigned_abs() <= 10000
                && !entry.pos.is_checked(entry.pos.side_to_move())
                && entry.mv.mtype() == MoveType::Normal
                && entry.pos.piece_at(entry.mv.to()) == PieceType::None
        }
        
        loader::SfBinpackLoader::new_concat_multiple(
            file_paths,
            buffer_size,
            threads,
            filter,
        )
    };

    trainer.run(&schedule, &settings, &data_loader);
}