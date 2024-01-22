use clap::Parser;
use std::io::Error;
use std::path::PathBuf;
use std::time::Instant;

mod parse_keybind_xml;
mod vkb;
// mod pdf_form;
// mod pdf_merge;
// mod pdf_parse;
// mod svg_parse;

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
    #[clap(long)]
    pub sc_mapping: Option<PathBuf>,

    #[clap(long)]
    pub vkb_report: Option<PathBuf>,

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
    env_logger::init();

    let args = Args::parse_args();

    let start_time = Instant::now();
    // let input_paths: Vec<_> = args
    //     .input_paths
    //     .iter()
    //     .map(|input_path| {
    //         PathBuf::from(
    //             shellexpand::full(input_path.to_str().unwrap())
    //                 .unwrap()
    //                 .to_string(),
    //         )
    //     })
    //     .collect();
    // println!("input_paths : {:?}", input_paths);

    // TODO https://www.dariocancelliere.it/blog/2020/09/29/pdf-manipulation-with-rust-and-considerations
    // "Filling form fields"

    // TODO read multiple pdfs
    // pdf_merge::merge(pdf_paths.clone())?;

    // for pdf_path in pdf_paths {
    //     pdf_form::list_forms(&pdf_path);
    // }

    match args.sc_mapping {
        Some(sc_mapping) => parse_keybind_xml::parse_keybind(sc_mapping).unwrap(),
        None => println!("SKIP : no sc_mapping path given"),
    }

    // svg_parse::svg_parse(&input_paths[0], "merged.png".into());

    // pdf_parse::pdf_read(input_paths[0].clone().into(), "output.txt".into());

    match args.vkb_report {
        Some(vkb_report) => {
            let vkb_report = vkb::parse_report(vkb_report).unwrap();
            println!("vkb_report : {:#?}", vkb_report);
        }
        None => println!("SKIP : no vkb_report path given"),
    }

    Ok(())
}
