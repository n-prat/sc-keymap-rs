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
    pub sc_bindings_to_ignore_path: Option<PathBuf>,

    #[clap(long)]
    pub sc_mapping: Option<PathBuf>,

    /// About the order: it SHOULD match the INSTANCE ID in the game mappings
    /// example:
    /// ```xml
    /// <options type="joystick" instance="1"
    ///     Product=" VKBsim Gladiator EVO  R    {0200231D-0000-0000-0000-504944564944}">
    ///     <flight_move_yaw invert="1" />
    /// </options>
    /// <options type="joystick" instance="2"
    ///     Product=" VKBsim Gladiator EVO  L    {0201231D-0000-0000-0000-504944564944}" />
    /// <modifiers />
    /// ```
    /// -> You SHOULD pass the files as --vkb-reports-paths=path_to_R_stick.fp3,path_to_L_stick.fp3
    ///
    /// NOTE: the order in eg
    /// ```xml
    /// <devices>
    ///     <keyboard instance="1" />
    ///     <mouse instance="1" />
    ///     <joystick instance="1" />
    ///     <joystick instance="2" />
    /// </devices>
    /// ```
    /// does NOT matter!
    ///
    #[clap(long, value_delimiter = ',')]
    pub vkb_reports_paths: Option<Vec<PathBuf>>,

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

    let _start_time = Instant::now();
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

    let sc_bindings_to_ignore = match args.sc_bindings_to_ignore_path {
        Some(sc_bindings_to_ignore_path) => {
            let rdr = csv::Reader::from_path(sc_bindings_to_ignore_path).unwrap();
            Some(rdr)
        }
        None => None,
    };

    let _game_buttons_mapping = match args.sc_mapping {
        Some(sc_mapping) => {
            parse_keybind_xml::parse_keybind(sc_mapping, sc_bindings_to_ignore).ok()
        }
        None => {
            println!("SKIP : no sc_mapping path given");
            None
        }
    };

    ////////////////////////////////////////////////////////////////////////////

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

    let joysticks_mappings = match args.vkb_reports_paths {
        Some(vkb_reports_paths) => {
            let mut res = vec![];
            for vkb_report_path in vkb_reports_paths {
                let vkb_user_provided_data = match args.vkb_user_provided_data_path {
                    Some(ref vkb_user_provided_data_path) => {
                        let rdr = csv::Reader::from_path(vkb_user_provided_data_path).unwrap();
                        Some(rdr)
                    }
                    None => None,
                };

                let vkb_report = vkb::parse_report(vkb_report_path).unwrap();
                log::info!("vkb_report : {:#?}", vkb_report);

                let vkb_buttons = vkb::check_report(vkb_report, vkb_user_provided_data);
                log::info!("vkb_buttons : {:#?}", vkb_buttons);

                res.push(vkb_buttons);
            }

            Some(res)
        }
        None => {
            println!("SKIP : no vkb_reports_paths given");
            None
        }
    };

    match joysticks_mappings {
        Some(joysticks_mappings) => {
            if joysticks_mappings.len() == 2 {
                if joysticks_mappings[0] != joysticks_mappings[1] {
                    log::warn!("2 joystick mappings processed -> they are different!");
                }
            }
        }
        None => {}
    }

    Ok(())
}
