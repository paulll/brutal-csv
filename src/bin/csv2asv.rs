//! Transform any CSV file into ASV file,
//! dropping empty columns. 

use std::io::{BufWriter, Seek};
use std::process::exit;
use clap::{Parser};
use clio::*;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Input CSV file
    #[clap(short, long, value_parser)]
    input: Input,

    /// Output ASV file
    #[clap(short, long, value_parser)]
    output: Output
}

#[allow(unused_mut)]
fn main() {
    let mut cli = Args::parse();

    #[cfg(feature = "progress")]
    let progress = if let Some(len) = cli.input.len() {
        indicatif::ProgressBar::new(len)
    } else {
        indicatif::ProgressBar::new_spinner()
    };

    let mut detector = brutal_csv::CsvSniffer::new();
    let mut reader = cli.input.clone();
    #[cfg(feature = "progress")]
    let mut reader = progress.wrap_read(reader);

    detector.process(&mut reader);
    let dialects = detector.dialects();

    if let Some(dialect) = dialects.iter().max() {
        cli.input.rewind().expect("To transform we need two full passes over stream, so it must be rewindable, so pipes don't work.");
        progress.reset();

        eprintln!("{:#?}", dialect);

        let reader = cli.input;
        #[cfg(feature = "progress")]
        let mut reader = progress.wrap_read(reader);
        dialect.to_asv(reader, BufWriter::new(cli.output));
    } else {
        eprintln!("No valid dialects found");
        exit(1);
    }
}
