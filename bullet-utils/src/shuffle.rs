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
    #[structopt(required = true, short, long)]
    input: PathBuf,
    #[structopt(required = true, short, long)]
    output: PathBuf,
    #[structopt(required = true, short, long)]
    /// Maximum RAM the program should use for its buffer, in MB
    mem_used_mb: usize,
}

const TMP_PATH: &str = "../../tmp";
const CHESS_BOARD_SIZE: usize = std::mem::size_of::<ChessBoard>();

impl ShuffleOptions {
    pub fn run(&self) {
        let input_size_mb = fs::metadata(self.input.clone())
            .expect("Input file is valid")
            .len() as usize
            / 1024;
        let num_tmp_files = input_size_mb / self.mem_used_mb;

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

        println!("# [Finished splitting data. Shuffling...]");
        assert!(self.output(&mut temp_files).is_ok());

        println!("> Took {:.2} seconds.", time.elapsed().as_secs_f32());
        assert!(remove_dir_all(TMP_PATH).is_ok());
    }

    fn split_file(&self, temp_files: &mut [File]) -> Result<()> {
        let mut input = BufReader::new(File::open(self.input.clone())?);

        let mut buffer = vec![0u8; self.mem_used_mb * 1024];

        for file in temp_files.iter_mut() {
            let bytes_read = input.read(&mut buffer).expect("Read was valid");
            let data = util::to_slice_with_lifetime_mut(&mut buffer);
            shuffle_positions(data);
            let data_slice = util::to_slice_with_lifetime(data);
            assert_eq!(bytes_read % CHESS_BOARD_SIZE, 0);
            let mut writer = BufWriter::new(file);
            assert!(writer.write_all(data_slice).is_ok());
            buffer.clear();
        }

        Ok(())
    }

    fn output(&self, temp_files: &mut [File]) -> Result<()> {
        let mut populations = temp_files
            .iter()
            .map(|file| file.metadata().expect("File is valid").len() as usize % CHESS_BOARD_SIZE)
            .collect::<Vec<_>>();
        let mut rng = Rand::default();
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
            assert_eq!(CHESS_BOARD_SIZE, temp_files[idx].read(&mut buffer)?);
            assert_eq!(CHESS_BOARD_SIZE, output.write(&buffer)?);
        }
        Ok(())
    }
}

fn pick_index(probs: &[f64], rng: &mut Rand) -> usize {
    assert!((1.0 - probs.iter().sum::<f64>()).abs() < f64::EPSILON);
    let rand_num = f64::from(rng.rand(1.0));

    let mut total_prob = 0.0;
    for (i, &prob) in probs.iter().enumerate() {
        total_prob += prob;
        if rand_num <= total_prob {
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
