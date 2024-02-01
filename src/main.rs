use clap::Parser;
use std::io::Error;
use std::path::PathBuf;
use std::time::Instant;

use sc_keymap_rs::{
    sc::parse_keybind_xml,
    template_gen::generate_sc_template,
    vkb::{self, parse_and_check_vkb_both_sticks},
};

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
    /// NOTE: side-specific! (obviously)
    #[clap(long)]
    pub vkb_report_path: Option<PathBuf>,

    /// Optional path to a csv button_id -> user provided description
    #[clap(long)]
    pub vkb_user_provided_data_path: Option<PathBuf>,

    /// Optional path to a "vkb_template_params.json" cf `TemplateJsonParamaters`
    /// NOTE: side-specific!
    #[clap(long)]
    pub vkb_template_params_path: Option<PathBuf>,

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

    ////////////////////////////////////////////////////////////////////////////
    // First step: parse the GAME mapping
    // NOTE: 1 mapping, irregardless of the number of physical sticks/devices

    let sc_bindings_to_ignore = match args.sc_bindings_to_ignore_path {
        Some(sc_bindings_to_ignore_path) => {
            let rdr = csv::Reader::from_path(sc_bindings_to_ignore_path).unwrap();
            Some(rdr)
        }
        None => None,
    };

    let game_buttons_mapping = match args.sc_mapping {
        Some(sc_mapping) => {
            parse_keybind_xml::parse_keybind(sc_mapping, sc_bindings_to_ignore).ok()
        }
        None => {
            println!("SKIP : no sc_mapping path given");
            None
        }
    };

    ////////////////////////////////////////////////////////////////////////////
    // Second step: parse the DEVICES mapping
    // NOTE: many mappings, one per physical sticks/devices

    let joysticks_mappings = match &args.vkb_report_path {
        Some(vkb_report_path) => parse_and_check_vkb_both_sticks(
            vkb_report_path.clone(),
            args.vkb_user_provided_data_path,
        )
        .ok(),
        None => {
            println!("SKIP : no vkb_reports_paths given");
            None
        }
    };

    ////////////////////////////////////////////////////////////////////////////
    // Last step:
    // We have the ONE game mappings, and the many devices mappings
    //

    match (game_buttons_mapping, joysticks_mappings) {
        (Some(game_buttons_mapping), Some(joysticks_mappings)) => {
            // svg_parse::svg_parse(
            //     vkb_template_path,
            //     args.vkb_output_png_path
            //         .expect("vkb_template_path set but missing vkb_output_png_path"),
            // );

            generate_sc_template(
                game_buttons_mapping,
                joysticks_mappings,
                args.vkb_template_params_path
                    .expect("missing --vkb-template-params-path"),
            );
        }
        _ => {
            // missing stuff; nothing to do
            println!("SKIP : missing game and/or devices mappings and/or arg --vkb-template-path; nothing to do...");
        }
    }

    Ok(())
}
