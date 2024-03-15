use clap::Parser;
use sc_keymap_rs::{generate_html, Error};
use std::path::PathBuf;

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

    /// usually "1" or "2"; For Star Citizen, it is e.g. "options type="joystick" instance=" in the exported xml
    /// It is an CLI arg b/c it can change when rebooting the computer, replugging, etc
    #[clap(long)]
    pub game_device_id: Option<u8>,

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
        Some(sc_mapping) => sc_keymap_rs::sc_parse_keybind(sc_mapping, sc_bindings_to_ignore).ok(),
        None => {
            println!("SKIP : no sc_mapping path given");
            None
        }
    };

    ////////////////////////////////////////////////////////////////////////////
    // Second step: parse the DEVICES mapping
    // NOTE: many mappings, one per physical sticks/devices

    let joysticks_mappings = match &args.vkb_report_path {
        Some(vkb_report_path) => sc_keymap_rs::vkb_parse_and_check_both_sticks(
            vkb_report_path.clone(),
            &args.vkb_user_provided_data_path,
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

    match (
        game_buttons_mapping,
        joysticks_mappings,
        args.game_device_id,
    ) {
        (Some(game_buttons_mapping), Some(joysticks_mappings), Some(game_device_id)) => {
            // TODO add arg and handle HTML gen separately
            // sc_keymap_rs::generate_template(
            //     &game_buttons_mapping,
            //     &joysticks_mappings,
            //     &args
            //         .vkb_template_params_path
            //         .as_ref()
            //         .expect("missing --vkb-template-params-path"),
            //     game_device_id,
            // )?;
            generate_html(
                &game_buttons_mapping,
                &joysticks_mappings,
                &args
                    .vkb_template_params_path
                    .expect("missing --vkb-template-params-path"),
                game_device_id,
            )?;
        }
        _ => {
            // missing stuff; nothing to do
            println!("SKIP : missing game and/or devices mappings and/or arg --vkb-template-path and/or --game-device-id ; nothing to do...");
        }
    }

    Ok(())
}
