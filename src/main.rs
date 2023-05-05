use clap::Parser;
use std::io::Error;
use std::path::PathBuf;
use std::time::Instant;

mod parse_keybind_xml;
mod pdf_form;
mod pdf_merge;
mod pdf_parse;
mod svg_parse;

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
    pub input_paths: Vec<PathBuf>,

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
    let input_paths: Vec<_> = args
        .input_paths
        .iter()
        .map(|input_path| {
            PathBuf::from(
                shellexpand::full(input_path.to_str().unwrap())
                    .unwrap()
                    .to_string(),
            )
        })
        .collect();
    println!("input_paths : {:?}", input_paths);

    // TODO https://www.dariocancelliere.it/blog/2020/09/29/pdf-manipulation-with-rust-and-considerations
    // "Filling form fields"

    // TODO read multiple pdfs
    // pdf_merge::merge(pdf_paths.clone())?;

    // for pdf_path in pdf_paths {
    //     pdf_form::list_forms(&pdf_path);
    // }

    parse_keybind_xml::parse_keybind("./data/layout_vkb_custom_v1_exported.xml".into()).unwrap();

    // svg_parse::svg_parse(&input_paths[0], "merged.png".into());

    pdf_parse::pdf_read("data/report_L.pdf".into(), "output.txt".into());

    Ok(())
}
