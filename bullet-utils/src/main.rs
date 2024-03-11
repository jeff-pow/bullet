#![feature(buf_read_has_data_left)]
mod convert;
mod interleave;
mod shuffle;
mod validate;

use crate::shuffle::ShuffleOptions;
use structopt::StructOpt;

#[derive(StructOpt)]
pub enum Options {
    Convert(convert::ConvertOptions),
    Interleave(interleave::InterleaveOptions),
    Shuffle(shuffle::ShuffleOptions),
    Validate(validate::ValidateOptions),
}

fn main() {
    let s = ShuffleOptions {
        input: "sample.bin".into(),
        output: "bruh.bin".into(),
        mem_used: 1_000_000,
    };
    s.run();
}
