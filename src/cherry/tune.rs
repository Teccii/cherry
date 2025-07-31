use crate::{EVAL_SCALE, HL, L1, QA, QB};
use bullet::{
    LocalSettings, TrainingSchedule, TrainingSteps,
    game::{
        formats::sfbinpack::{
            TrainingDataEntry,
            chess::{r#move::MoveType, piecetype::PieceType},
        },
        inputs,
    },
    value::{loader, ValueTrainerBuilder},
    trainer::save::SavedFormat,
    nn::optimiser,
    wdl,
    lr,
};

const INITIAL_LR: f32 = 0.001f32;
const FINAL_LR: f32 = 0.001f32 * 0.3f32.powi(5);
const SUPER_BATCHES: usize = 800;

pub fn tune(threads: usize, buffer_size: usize, queue_size: usize, file_paths: &[&str]) {
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

            l1.forward(ft_output)
        });

    let schedule = TrainingSchedule {
        net_id: String::from("cherry_768-256"),
        eval_scale: EVAL_SCALE as f32,
        wdl_scheduler: wdl::ConstantWDL { value: 0.75 },
        lr_scheduler: lr::CosineDecayLR {
            final_superbatch: SUPER_BATCHES,
            initial_lr: INITIAL_LR,
            final_lr: FINAL_LR,
        },
        steps: TrainingSteps {
            batch_size: 16384,
            batches_per_superbatch: 4096,
            end_superbatch: SUPER_BATCHES,
            start_superbatch: 1,
        },
        save_rate: 20,
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
                && entry.pos.piece_at(entry.mv.to()).piece_type() == PieceType::None
        }

        loader::SfBinpackLoader::new_concat_multiple(file_paths, buffer_size, threads, |_| true)
    };

    trainer.run(&schedule, &settings, &data_loader);
}