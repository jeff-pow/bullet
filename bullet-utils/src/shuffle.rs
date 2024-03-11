use std::hash::Hash;
use std::io::BufRead;
use std::{
    fs::{self, create_dir, remove_dir_all, File},
    io::{BufReader, BufWriter, Read, Result, Seek, Write},
    path::{Path, PathBuf},
    time::Instant,
};

use bullet_core::{util, Rand};
use bulletformat::ChessBoard;
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct ShuffleOptions {
    // #[structopt(required = true, short, long)]
    pub input: PathBuf,
    // #[structopt(required = true, short, long)]
    pub output: PathBuf,
    // #[structopt(required = true, short, long)]
    /// Maximum RAM the program should use for its buffer, in MB
    pub mem_used: usize,
}

const TMP_PATH: &str = "../../tmp";
const CHESS_BOARD_SIZE: usize = std::mem::size_of::<ChessBoard>();

impl ShuffleOptions {
    pub fn run(&self) {
        let input_size_mb = fs::metadata(self.input.clone())
            .expect("Input file is valid")
            .len() as usize;
        let num_tmp_files = input_size_mb / self.mem_used + 1;
        dbg!(num_tmp_files);
        dbg!(input_size_mb / CHESS_BOARD_SIZE);

        if !Path::new(TMP_PATH).exists() {
            create_dir(TMP_PATH).expect("Path could be created");
        }
        let mut temp_files = (0..num_tmp_files)
            .map(|idx| {
                let output_file = format!("{}/part_{}.bin", TMP_PATH, idx + 1);
                File::create(output_file).unwrap()
            })
            .collect::<Vec<_>>();

        println!("# [Shuffling Data]");
        let time = Instant::now();
        assert!(self.split_file(&mut temp_files).is_ok());
        assert!(rewind_files(&mut temp_files).is_ok());
        let mut temp_files = (0..num_tmp_files)
            .map(|idx| {
                let output_file = format!("{}/part_{}.bin", TMP_PATH, idx + 1);
                File::open(output_file).unwrap()
            })
            .collect::<Vec<_>>();

        println!("# [Finished splitting data. Shuffling...]");
        assert!(self.output(&mut temp_files).is_ok());

        println!("> Took {:.2} seconds.", time.elapsed().as_secs_f32());
    }

    fn split_file(&self, temp_files: &mut [File]) -> Result<()> {
        let mut input = File::open(self.input.clone()).unwrap();

        dbg!(self.actual_buffer_size());

        for file in temp_files.iter_mut() {
            let mut buffer = vec![0u8; self.actual_buffer_size()];
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
                .map(|&p| {
                    if p == 0 {
                        0.
                    } else {
                        p as f64 / remaining
                    }
                })
                .collect::<Vec<_>>();

            let idx = pick_index(&probs, &mut rng);
            populations[idx] -= 1;
            let mut buffer = [0; CHESS_BOARD_SIZE];
            temp_files[idx].read_exact(&mut buffer).expect("Read bruh");
            assert_eq!(CHESS_BOARD_SIZE, output.write(&buffer).expect("Write bruh"));
        }
        Ok(())
    }

    fn actual_buffer_size(&self) -> usize {
        self.mem_used / CHESS_BOARD_SIZE * CHESS_BOARD_SIZE
    }
}

fn pick_index(probs: &[f64], rng: &mut Rand) -> usize {
    assert!((1.0 - probs.iter().sum::<f64>()).abs() < f64::EPSILON);
    let rand_num = f64::from(rng.rand(1.0)).abs();
    assert!(0. <= rand_num && rand_num <= 1.0);

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

fn rewind_files(files: &mut [File]) -> Result<()> {
    for file in files {
        file.seek(std::io::SeekFrom::Start(0))?;
    }
    Ok(())
}
