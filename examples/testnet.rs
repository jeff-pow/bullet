/*
This is used to confirm non-functional changes for bullet.
*/
use bullet_lib::{
    inputs, outputs, Activation, LocalSettings, LrScheduler, TrainerBuilder, TrainingSchedule,
    WdlScheduler,
};

fn main() {
    let mut trainer = TrainerBuilder::default()
        .quantisations(&[181, 64])
        .input(inputs::Chess768)
        .output_buckets(outputs::Single)
        .feature_transformer(32)
        .activate(Activation::SCReLU)
        .add_layer(1)
        .build();

    trainer.load_from_checkpoint("checkpoints/testnet");

    let schedule = TrainingSchedule {
        net_id: "testnet".to_string(),
        eval_scale: 400.0,
        ft_regularisation: 0.0,
        batch_size: 16_384,
        batches_per_superbatch: 1,
        start_superbatch: 1,
        end_superbatch: 5,
        wdl_scheduler: WdlScheduler::Constant { value: 0.2 },
        lr_scheduler: LrScheduler::Constant { value: 0.001 },
        save_rate: 10,
    };

    let settings = LocalSettings {
        threads: 4,
        data_file_paths: vec!["../../data/batch1.data"],
        output_directory: "checkpoints",
    };

    trainer.run(&schedule, &settings);
}
