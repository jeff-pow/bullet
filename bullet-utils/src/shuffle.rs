use std::{
    fs::{self, create_dir, File},
    io::{BufWriter, Read, Result, Write},
    path::{Path, PathBuf},
    time::Instant,
};

use bullet_core::{util, Rand};
use bulletformat::ChessBoard;
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct ShuffleOptions {
    #[structopt(required = true, short, long)]
    pub input: PathBuf,
    #[structopt(required = true, short, long)]
    pub output: PathBuf,
    #[structopt(required = true, short, long)]
    pub mem_used_mb: usize,
    #[structopt(required = true, short, long)]
    pub tmp_dir: PathBuf,
}

const CHESS_BOARD_SIZE: usize = std::mem::size_of::<ChessBoard>();
const MIN_TMP_FILES: usize = 4;

impl ShuffleOptions {
    pub fn run(&self) {
        let input_size = fs::metadata(self.input.clone())
            .expect("Input file is valid")
            .len() as usize;
        let num_tmp_files = (input_size / (self.mem_used_mb * 1000) + 1).max(MIN_TMP_FILES);

        if !Path::new(&self.tmp_dir).exists() {
            create_dir(self.tmp_dir.clone()).expect("Path could be created");
        }
        let mut temp_files = (0..num_tmp_files)
            .map(|idx| {
                let output_file =
                    format!("{}/part_{}.bin", self.tmp_dir.to_str().unwrap(), idx + 1);
                File::create(output_file).unwrap()
            })
            .collect::<Vec<_>>();

        println!("# [Shuffling Data]");
        let time = Instant::now();
        assert!(self.split_file(&mut temp_files, input_size).is_ok());
        let mut temp_files = (0..num_tmp_files)
            .map(|idx| {
                let output_file =
                    format!("{}/part_{}.bin", self.tmp_dir.to_str().unwrap(), idx + 1);
                File::open(output_file).unwrap()
            })
            .collect::<Vec<_>>();

        println!("# [Finished splitting data. Shuffling...]");
        assert!(self.output(&mut temp_files).is_ok());

        println!("> Took {:.2} seconds.", time.elapsed().as_secs_f32());
    }

    fn split_file(&self, temp_files: &mut [File], input_size: usize) -> Result<()> {
        let mut input = File::open(self.input.clone()).unwrap();

        let buff_size = self.actual_buffer_size(temp_files.len(), input_size);

        for file in temp_files.iter_mut() {
            let mut buffer = vec![0u8; buff_size];
            let bytes_read = input.read(&mut buffer)?;

            let data = util::to_slice_with_lifetime_mut(&mut buffer[0..bytes_read]);
            shuffle_positions(data);
            let data_slice = util::to_slice_with_lifetime(data);
            assert_eq!(0, bytes_read % CHESS_BOARD_SIZE);

            let mut writer = BufWriter::new(file);
            assert!(writer.write(&data_slice[0..bytes_read]).is_ok());
        }

        Ok(())
    }

    fn output(&self, temp_files: &mut [File]) -> Result<()> {
        let mut populations = temp_files
            .iter()
            .map(|file| file.metadata().expect("File is valid").len() as usize / CHESS_BOARD_SIZE)
            .collect::<Vec<_>>();
        dbg!(&populations);
        let mut rng = Rand::new(0xBEEF);
        let mut output =
            BufWriter::new(File::create(&self.output).expect("Provide a correct path!"));

        loop {
            if populations.iter().all(|&p| p == 0) {
                break;
            }
            let remaining = populations.iter().sum::<usize>() as f64;
            let probs = populations
                .iter()
                .map(|&p| p as f64 / remaining)
                .collect::<Vec<_>>();

            let idx = pick_index(&probs, &mut rng);
            populations[idx] -= 1;
            let mut buffer = [0; CHESS_BOARD_SIZE];
            temp_files[idx]
                .read_exact(&mut buffer)
                .expect("Chess position couldn't be read from tmp file.");
            assert_eq!(
                CHESS_BOARD_SIZE,
                output
                    .write(&buffer)
                    .expect("Chess position couldn't be written to output.")
            );
        }
        Ok(())
    }

    /// Input size should be in bytes
    fn actual_buffer_size(&self, num_tmp_files: usize, input_size: usize) -> usize {
        input_size / num_tmp_files / CHESS_BOARD_SIZE * CHESS_BOARD_SIZE + CHESS_BOARD_SIZE
    }
}

fn pick_index(probs: &[f64], rng: &mut Rand) -> usize {
    let rand_num = f64::from(rng.rand(1.0)).abs();
    assert!((0. ..=1.0).contains(&rand_num));

    let mut total_prob = 0.0;
    for (i, &prob) in probs.iter().enumerate() {
        total_prob += prob;
        if rand_num < total_prob {
            return i;
        }
    }
    unreachable!()
}

fn shuffle_positions(data: &mut [ChessBoard]) {
    let mut rng = Rand::default();

    for i in (0..data.len()).rev() {
        let idx = rng.rand_int() as usize % (i + 1);
        data.swap(idx, i);
    }
}
