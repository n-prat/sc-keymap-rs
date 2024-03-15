use image::imageops;
use rusttype::Font;
use rusttype::Scale;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::sc::parse_keybind_xml;
use crate::vkb::vkb_button::JoystickButtonsMapping;
use crate::Error;

/// Combine a game keybinds mapping and a physical joystick configuration and generates a .png
///
/// params:
/// - `game_device_id`: usually "1" or "2"; For Star Citizen, it is e.g. "options type="joystick" instance=" in the exported xml
///
/// # Errors
/// - the various files could not be read
/// - the positions/sizes/etc in `vkb_template_params.json` are not correct
/// - etc
///
#[allow(clippy::too_many_lines)]
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
pub fn generate_template(
    game_buttons_mapping: &parse_keybind_xml::GameButtonsMapping,
    joysticks_mappings: &JoystickButtonsMapping,
    json_template_params_path: &PathBuf,
    game_device_id: u8,
) -> Result<(), Error> {
    const WIDTH: u32 = 4000;
    const HEIGHT: u32 = 2000;
    const FULL_PNG_RESIZED_WIDTH: i32 = 1800;
    const FULL_PNG_RESIZED_HEIGHT: i32 = 1800;
    const SIDE_PNG_RESIZED_WIDTH: i32 = 1200;
    const SIDE_PNG_RESIZED_HEIGHT: i32 = 1200;
    const BOX_LENGTH: i32 = 500;
    const BOX_HEIGHT: i32 = 110;
    const PADDING_H: i32 = 10;
    const PADDING_V: i32 = 10;

    let json_params = read_json_template(json_template_params_path)?;

    ////////////////////////////////////////////////////////////////////////////
    let image_full_front = image::open(json_params.path_to_full_png.clone()).map_err(|_err| {
        Error::Other(format!(
            "failed to open path_to_full_png {:?}",
            json_params.path_to_full_png
        ))
    })?;

    let image_back = image::open(json_params.path_to_side_png.clone()).map_err(|_err| {
        Error::Other(format!(
            "failed to open path_to_side_png {:?}",
            json_params.path_to_side_png
        ))
    })?;

    // Create a new RgbaImage with dimensions based on the larger of the two images
    let mut final_image = image::RgbaImage::new(WIDTH, HEIGHT);

    ////////////////////////////////////////////////////////////////////////////

    // Draw the main image; usually this is the "front" or "3/4 front" view
    let image_full_front = imageops::resize(
        &image_full_front,
        FULL_PNG_RESIZED_WIDTH as u32,
        FULL_PNG_RESIZED_HEIGHT as u32,
        imageops::FilterType::Nearest,
    );
    let full_png_top_left_position: (i32, i32) =
        ((WIDTH as f32 * 0.3) as i32, (HEIGHT as f32 * 0.0) as i32);
    image::imageops::overlay(
        &mut final_image,
        &image_full_front,
        full_png_top_left_position.0.into(),
        full_png_top_left_position.1.into(),
    );
    let full_png_center_position = transform_relative_coords_to_absolute(
        (FULL_PNG_RESIZED_WIDTH / 2, FULL_PNG_RESIZED_HEIGHT / 2),
        full_png_top_left_position,
    );

    // Draw the side/back
    let image_back = imageops::resize(
        &image_back,
        SIDE_PNG_RESIZED_WIDTH as u32,
        SIDE_PNG_RESIZED_HEIGHT as u32,
        imageops::FilterType::Nearest,
    );
    let side_png_top_left_position: (i32, i32) =
        ((WIDTH as f32 * 0.05) as i32, (HEIGHT as f32 * 0.3) as i32);
    image::imageops::overlay(
        &mut final_image,
        &image_back,
        side_png_top_left_position.0.into(),
        side_png_top_left_position.1.into(),
    );
    let side_png_center_position = transform_relative_coords_to_absolute(
        (SIDE_PNG_RESIZED_WIDTH / 2, SIDE_PNG_RESIZED_HEIGHT / 2),
        side_png_top_left_position,
    );

    ////////////////////////////////////////////////////////////////////////////
    // main drawing code

    let line_color = image::Rgba([0, 255, 0, 255]);

    // Load a system font (replace with the path to your TTF or OTF font file)
    let font_data = include_bytes!("../data/BF_Modernista-Regular.ttf");
    let font = rusttype::Font::try_from_bytes(font_data)
        .ok_or_else(|| Error::Other("Failed to load font".to_string()))?;

    // Draw boxes in a 4-way pattern with customizable color and stroke thickness

    for button_param in &json_params.buttons_params {
        let mut keybind_lines: Vec<String> = vec![];

        for physical_name in &button_param.physical_names {
            // First: get the corresponding VIRTUAL button ID from "physical_name" in json
            // TODO(2-sticks) handle two sticks
            let virtual_buttons =
                joysticks_mappings.get_virtual_button_ids_from_info_or_user_desc(physical_name)?;

            // Next: get the game binding from this virtual_button_id
            let mut actions_names: String = String::new();
            for virtual_button in virtual_buttons {
                match virtual_button {
                    crate::button::VirtualButtonOrSpecial::Virtual(virtual_button) => {
                        let modifier: String = match &virtual_button.kind {
                            crate::button::VirtualButtonKind::Momentary(shift) => match shift {
                                Some(shift_kind) => match shift_kind {
                                    crate::button::VirtualShiftKind::Shift1 => {
                                        "[SHIFT1] ".to_string()
                                    }
                                    crate::button::VirtualShiftKind::Shift2 => {
                                        "[SHIFT2] ".to_string()
                                    }
                                },
                                None => String::new(),
                            },
                            crate::button::VirtualButtonKind::Tempo(tempo) => match tempo {
                                crate::button::VirtualTempoKind::Short => "[SHORT] ".to_string(),
                                crate::button::VirtualTempoKind::Long => "[LONG] ".to_string(),
                                crate::button::VirtualTempoKind::Double => "[DOUBLE] ".to_string(),
                            },
                        };

                        let mut action_name_with_modifier = modifier;

                        match game_buttons_mapping.get_action_from_virtual_button_id(
                            *virtual_button.get_id(),
                            game_device_id,
                        ) {
                            Some(act_names) => {
                                action_name_with_modifier.push_str(&act_names.join("\n"));
                            }
                            None => action_name_with_modifier.push_str("NO BINDING"),
                        }

                        actions_names.push_str(&action_name_with_modifier);

                        actions_names.push('\n');
                    }
                    crate::button::VirtualButtonOrSpecial::Special(special_kind) => {
                        match special_kind {
                            crate::button::SpecialButtonKind::Shift1 => {
                                actions_names.push_str("SHIFT1");
                            }
                            crate::button::SpecialButtonKind::Shift2 => {
                                actions_names.push_str("SHIFT2");
                            }
                        }
                    }
                }
            }

            keybind_lines.push(actions_names);
        }

        let reference_point = if button_param.is_using_full_png_center_as_reference {
            full_png_center_position
        } else {
            side_png_center_position
        };

        match button_param.physical_names.len() {
            1 | 2 | 5 | 8 | 3 => {
                draw_boxes(
                    &mut final_image,
                    button_param.physical_names.len(),
                    // image::Rgba([240, 240, 240, 240]),
                    image::Rgba([50, 50, 50, 220]),
                    2,
                    transform_relative_coords_to_absolute(
                        reference_point,
                        button_param.desired_box_position_relative_to_center_full_png,
                    ),
                    BOX_LENGTH,
                    BOX_HEIGHT,
                    PADDING_H,
                    PADDING_V,
                    &font,
                    // image::Rgba([26, 26, 26, 255]),
                    image::Rgba([220, 220, 220, 255]),
                    24,
                    &keybind_lines,
                )?;

                draw_thicker_line_mut(
                    &mut final_image,
                    transform_relative_coords_to_absolute(
                        reference_point,
                        button_param.connector_start_line_position_relative_to_center_full_png,
                    ),
                    transform_relative_coords_to_absolute(
                        reference_point,
                        button_param.connector_end_line_position_relative_to_center_full_png,
                    ),
                    4,
                    line_color,
                );
            }
            _ => {
                unimplemented!("NOT SUPPORTED")
            }
        }
    }

    ////////////////////////////////////////////////////////////////////////////

    // Save the final image
    final_image
        .save(json_params.path_to_output_png.clone())
        .map_err(|_err| {
            Error::Other(format!(
                "could not write image to {:?}",
                json_params.path_to_output_png
            ))
        })?;

    Ok(())
}

pub(crate) fn read_json_template(
    json_template_path: &PathBuf,
) -> Result<TemplateJsonParamaters, Error> {
    let json_params: TemplateJsonParamaters = serde_json::from_reader(std::io::BufReader::new(
        std::fs::File::open(json_template_path.clone()).map_err(|_err| {
            Error::Other(format!(
                "failed to open json_template_params_path {json_template_path:?}"
            ))
        })?,
    ))
    .map_err(|_err| Error::Other(format!("serde_json error for {json_template_path:?}")))?;
    log::debug!("json_params : {json_params:?}");
    Ok(json_params)
}

#[derive(Debug, Clone)]
struct TextParameters<'a> {
    text: String,
    text_size: u32,
    text_color: image::Rgba<u8>,
    font: &'a Font<'static>,
}

/// `https://chat.openai.com`
#[derive(Debug, Clone)]
struct BoxParameters<'a> {
    position: (i32, i32),
    size: (u32, u32),
    color: image::Rgba<u8>,
    stroke_thickness: i32,
    text_params: Option<TextParameters<'a>>,
}

/// Draw a thick line
/// `https://chat.openai.com`
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
fn draw_thicker_line_mut(
    image: &mut image::RgbaImage,
    start: (i32, i32),
    end: (i32, i32),
    thickness: i32,
    color: image::Rgba<u8>,
) {
    let half_thickness = thickness / 2;
    for offset in -half_thickness..=half_thickness {
        imageproc::drawing::draw_line_segment_mut(
            image,
            (start.0 as f32, (start.1 + offset) as f32),
            (end.0 as f32, (end.1 + offset) as f32),
            color,
        );
    }
}

/// `https://chat.openai.com`
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
fn draw_box(image: &mut image::RgbaImage, parameters: BoxParameters<'_>) {
    imageproc::drawing::draw_filled_rect_mut(
        image,
        imageproc::rect::Rect::at(parameters.position.0, parameters.position.1)
            .of_size(parameters.size.0, parameters.size.1),
        parameters.color,
    );

    // Draw border with customizable thickness
    let half_thickness = parameters.stroke_thickness / 2;
    for offset in -half_thickness..=half_thickness {
        imageproc::drawing::draw_line_segment_mut(
            image,
            (
                (parameters.position.0 - half_thickness) as f32,
                (parameters.position.1 + offset) as f32,
            ),
            (
                (parameters.position.0 + parameters.size.0 as i32 + half_thickness) as f32,
                (parameters.position.1 + offset) as f32,
            ),
            parameters.color,
        );
        imageproc::drawing::draw_line_segment_mut(
            image,
            (
                (parameters.position.0 + offset) as f32,
                (parameters.position.1 - half_thickness) as f32,
            ),
            (
                (parameters.position.0 + offset) as f32,
                (parameters.position.1 + parameters.size.1 as i32 + half_thickness) as f32,
            ),
            parameters.color,
        );
    }

    if let Some(text_params) = parameters.text_params {
        let scale = Scale::uniform(text_params.text_size as f32);
        // height: use the max height
        let text_height =
            imageproc::drawing::text_size(scale, text_params.font, &text_params.text).1;
        // width: use the longest b/w every lines
        let mut max_text_width: i32 = 0;
        for line in text_params.text.split('\n') {
            max_text_width =
                max_text_width.max(imageproc::drawing::text_size(scale, text_params.font, line).0);
        }

        // Center the text, both horizontally and vertically
        for (line_no, line) in text_params.text.split('\n').enumerate() {
            imageproc::drawing::draw_text_mut(
                image,
                text_params.text_color,
                parameters.position.0 + parameters.size.0 as i32 / 2 - max_text_width / 2,
                // text_size.1 / 4 b/c 2 would make the bottom of the text on the bottom of the box
                parameters.position.1 + parameters.size.1 as i32 / 4 - text_height as i32 / 2
                    + line_no as i32 * text_height as i32,
                scale,
                text_params.font,
                line,
            );
        }
    }
}

/// `https://chat.openai.com`
///
/// `texts`: order is important; it MUST match with how "`vkb_template_params.json`" is handled
/// - 2 boxes: vertical, top -> bottom
/// - 3 boxes: horizontal, left -> right
/// - 5 boxes: clockwise, starts from NORTH: N -> E -> S -> W, then center (ie the ministick "push"/"click"/"press")
/// - 8 boxes: clockwise, starts from NORTH = N -> NE -> E -> SE -> S -> SW -> W -> NW
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn draw_boxes(
    image: &mut image::RgbaImage,
    pattern: usize,
    color: image::Rgba<u8>,
    stroke_thickness: i32,
    start_position: (i32, i32),
    small_box_length: i32,
    small_box_height: i32,
    padding_h: i32,
    padding_v: i32,
    font: &Font<'static>,
    text_color: image::Rgba<u8>,
    text_size: u32,
    texts: &[String],
) -> Result<(), Error> {
    assert_eq!(texts.len(), pattern);

    let draw_parameters = |x, y, txt: &str| {
        Ok(BoxParameters {
            position: (x, y),
            size: (
                small_box_length
                    .try_into()
                    .map_err(Error::TryFromIntError)?,
                small_box_height
                    .try_into()
                    .map_err(Error::TryFromIntError)?,
            ),
            color,
            stroke_thickness,
            text_params: Some(TextParameters {
                text: txt.to_string(),
                text_size,
                text_color,
                font,
            }),
        })
    };

    let mut draw_4_in_cross = |text_a, text_b, text_c, text_d, text_e| {
        // top center
        draw_box(
            image,
            draw_parameters(start_position.0, start_position.1, text_a)?,
        );
        // right, vertically in between "top center" and "bottom center"
        draw_box(
            image,
            draw_parameters(
                start_position.0 + small_box_length + padding_h,
                start_position.1 + small_box_height + padding_v,
                text_b,
            )?,
        );
        // bottom center
        draw_box(
            image,
            draw_parameters(
                start_position.0,
                start_position.1 + 2 * (small_box_height + padding_v),
                text_c,
            )?,
        );
        // left, vertically in between "top center" and "bottom center"
        draw_box(
            image,
            draw_parameters(
                start_position.0 - small_box_length - padding_h,
                start_position.1 + small_box_height + padding_v,
                text_d,
            )?,
        );
        // center center, "push"/"click"/"press"
        draw_box(
            image,
            draw_parameters(
                start_position.0,
                start_position.1 + small_box_height + padding_v,
                text_e,
            )?,
        );

        Ok::<(), Error>(())
    };

    match pattern {
        2 => {
            draw_box(
                image,
                draw_parameters(start_position.0, start_position.1, &texts[0])?,
            );
            draw_box(
                image,
                draw_parameters(
                    start_position.0,
                    start_position.1 + small_box_height + padding_v,
                    &texts[1],
                )?,
            );
        }
        5 => {
            draw_4_in_cross(&texts[0], &texts[1], &texts[2], &texts[3], &texts[4])?;
        }
        8 => {
            // the 4 as above
            draw_4_in_cross(&texts[0], &texts[2], &texts[4], &texts[6], "")?;

            // PLUS:
            // top right = NE
            draw_box(
                image,
                draw_parameters(
                    start_position.0 + small_box_length + padding_h,
                    start_position.1,
                    &texts[1],
                )?,
            );
            // bottom right = SE
            draw_box(
                image,
                draw_parameters(
                    start_position.0 + small_box_length + padding_h,
                    start_position.1 + 2 * (small_box_height + padding_v),
                    &texts[3],
                )?,
            );
            // bottom left = SW
            draw_box(
                image,
                draw_parameters(
                    start_position.0 - small_box_length - padding_h,
                    start_position.1 + 2 * (small_box_height + padding_v),
                    &texts[5],
                )?,
            );
            // top left = NW
            draw_box(
                image,
                draw_parameters(
                    start_position.0 - small_box_length - padding_h,
                    start_position.1,
                    &texts[7],
                )?,
            );
        }
        1 => {
            draw_box(
                image,
                draw_parameters(start_position.0, start_position.1, &texts[0])?,
            );
        }
        // 3 horizontal
        3 => {
            // left
            draw_box(
                image,
                draw_parameters(start_position.0, start_position.1, &texts[0])?,
            );
            // center
            draw_box(
                image,
                draw_parameters(
                    start_position.0 + (small_box_length + padding_h),
                    start_position.1,
                    &texts[1],
                )?,
            );
            // right
            draw_box(
                image,
                draw_parameters(
                    start_position.0 + 2 * (small_box_length + padding_h),
                    start_position.1,
                    &texts[2],
                )?,
            );
        }
        _ => {
            // Handle other cases or provide a default behavior
            unimplemented!("draw_boxes: pattern = only 1/2/5/8 are supported");
        }
    };

    Ok(())
}

/// NOTE: this is for one joystick, either L or right
/// (at least for now)
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct TemplateJsonParamaters {
    pub(crate) path_to_full_png: PathBuf,
    pub(crate) path_to_side_png: PathBuf,
    pub(crate) path_to_output_png: PathBuf,
    pub(crate) buttons_params: Vec<TemplateJsonButtonOrStickParameters>,
}

/// This is how/where a button/stick will be drawn in the final composite image
#[derive(Serialize, Deserialize, Debug)]
struct TemplateJsonButtonOrStickParameters {
    /// Based on whate is written on the stick itself: eg "A1", "F1", etc
    /// It MUST either match:
    /// - the "info" field in xml; that would be "(A1)","(F1)" etc for simple buttons
    /// - OR the "desciption" found in bindings/vkb_user_provided_data.csv
    ///   Typically that would be for the 4-ways/8-ways sticks
    /// List b/c for 4-ways/8-ways/encoders etc we group them and draw all-at-once in a box.
    physical_names: Vec<String>,
    /// User-friendly description: eg "Red thumb button top of stick"
    user_desc: String,
    /// Should the 3 points below be relative to the "face"(full) png, or the side one?
    is_using_full_png_center_as_reference: bool,
    desired_box_position_relative_to_center_full_png: (i32, i32),
    /// By convention: `start` is the joystick button
    connector_start_line_position_relative_to_center_full_png: (i32, i32),
    /// By convention: `end` is the box ie near `desired_box_position`
    connector_end_line_position_relative_to_center_full_png: (i32, i32),
}

fn transform_relative_coords_to_absolute(add: (i32, i32), relative_to: (i32, i32)) -> (i32, i32) {
    (add.0 + relative_to.0, add.1 + relative_to.1)
}
