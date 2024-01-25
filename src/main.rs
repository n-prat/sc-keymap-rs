use clap::Parser;
use std::io::Error;
use std::path::PathBuf;
use std::time::Instant;

use sc_keymap_rs::{sc::parse_keybind_xml, svg_parse, vkb};

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

    #[clap(long)]
    pub vkb_template_path: Option<PathBuf>,

    /// Optional path to a csv button_id -> user provided description
    #[clap(long)]
    pub vkb_user_provided_data_path: Option<PathBuf>,

    /// Optional output png path; only applicable if `vkb_template_path`
    #[clap(short, long)]
    pub vkb_output_png_path: Option<PathBuf>,

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

    let game_buttons_mapping = match args.sc_mapping {
        Some(sc_mapping) => parse_keybind_xml::parse_keybind(sc_mapping).ok(),
        None => {
            println!("SKIP : no sc_mapping path given");
            None
        }
    };

    let vkb_user_provided_data = match args.vkb_user_provided_data_path {
        Some(vkb_user_provided_data_path) => {
            let mut rdr = csv::Reader::from_path(vkb_user_provided_data_path).unwrap();
            Some(rdr)
        }
        None => None,
    };

    match args.vkb_template_path {
        Some(vkb_template_path) => {
            svg_parse::svg_parse(
                vkb_template_path,
                args.vkb_output_png_path
                    .expect("vkb_template_path set but missing vkb_output_png_path"),
            );
        }
        None => println!("SKIP : no vkb_template_path path given"),
    }

    // pdf_parse::pdf_read(input_paths[0].clone().into(), "output.txt".into());

    match args.vkb_report {
        Some(vkb_report) => {
            let vkb_report = vkb::parse_report(vkb_report).unwrap();
            log::info!("vkb_report : {:#?}", vkb_report);

            let vkb_buttons = vkb::check_report(vkb_report, vkb_user_provided_data);
            log::info!("vkb_buttons : {:#?}", vkb_buttons);
        }
        None => println!("SKIP : no vkb_report path given"),
    }

    Ok(())
}
