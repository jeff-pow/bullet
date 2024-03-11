#![feature(buf_read_has_data_left)]
mod convert;
mod interleave;
mod shuffle;
mod validate;

use structopt::StructOpt;

#[derive(StructOpt)]
pub enum Options {
    Convert(convert::ConvertOptions),
    Interleave(interleave::InterleaveOptions),
    Shuffle(shuffle::ShuffleOptions),
    Validate(validate::ValidateOptions),
}

fn main() {
    match Options::from_args() {
        Options::Convert(options) => options.run(),
        Options::Interleave(options) => options.run(),
        Options::Shuffle(options) => options.run(),
        Options::Validate(options) => options.run(),
    }
    // let s = ShuffleOptions {
    //     input: "sample.bin".into(),
    //     output: "bruh.bin".into(),
    //     mem_used: 1_000_000,
    //     tmp_dir: "tmp".into(),
    // };
    // s.run();
}
