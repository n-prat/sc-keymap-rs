use clap::Parser;
use std::io::Error;
use std::path::PathBuf;
use std::time::Instant;

mod edit;
mod merge;

/// https://github.com/J-F-Liu/lopdf/blob/master/examples/extract_toc.rs
///
#[derive(Parser, Debug)]
#[clap(
    author,
    version,
    about,
    long_about = "Extract TOC and write to file.",
    arg_required_else_help = true
)]
pub struct Args {
    #[clap(num_args = 1..)]
    pub pdf_paths: Vec<PathBuf>,

    /// Optional output directory. If omitted the directory of the PDF file will be used.
    #[clap(short, long)]
    pub output: Option<PathBuf>,

    /// Optional pretty print output.
    #[clap(short, long)]
    pub pretty: bool,
}

impl Args {
    pub fn parse_args() -> Self {
        Args::parse()
    }
}

fn main() -> Result<(), Error> {
    let args = Args::parse_args();

    let start_time = Instant::now();
    let pdf_paths: Vec<_> = args
        .pdf_paths
        .iter()
        .map(|pdf_path| {
            PathBuf::from(
                shellexpand::full(pdf_path.to_str().unwrap())
                    .unwrap()
                    .to_string(),
            )
        })
        .collect();
    println!("pdf_paths : {:?}", pdf_paths);

    // TODO https://www.dariocancelliere.it/blog/2020/09/29/pdf-manipulation-with-rust-and-considerations
    // "Filling form fields"

    // TODO read multiple pdfs
    merge::merge(pdf_paths.clone())?;

    for pdf_path in pdf_paths {
        edit::list_forms(&pdf_path);
    }

    Ok(())
}
