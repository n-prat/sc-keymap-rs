use image::imageops;
use rusttype::Font;
use rusttype::Scale;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::sc::parse_keybind_xml;
use crate::vkb::VkbBothSticksMappings;

pub fn generate_sc_template(
    game_buttons_mapping: parse_keybind_xml::GameButtonsMapping,
    joysticks_mappings: VkbBothSticksMappings,
    json_template_params_path: PathBuf,
) {
    ////////////////////////////////////////////////////////////////////////////
    // Parse the "vkb_template_params.json"
    // and check eveything is OK: paths, etc
    let json_params: TemplateJsonParamaters = serde_json::from_reader(std::io::BufReader::new(
        std::fs::File::open(json_template_params_path).unwrap(),
    ))
    .unwrap();
    println!("json_params : {json_params:?}");

    ////////////////////////////////////////////////////////////////////////////
    let image_full_front =
        image::open(json_params.path_to_full_png).expect("Failed to open image1");

    let image_back = image::open(json_params.path_to_side_png).expect("Failed to open image2");

    // Create a new RgbaImage with dimensions based on the larger of the two images
    const WIDTH: u32 = 3000;
    const HEIGHT: u32 = 1800;
    // let width = image_full_front.width().max(image_back.width());
    // let height = image_full_front.height().max(image_back.height());
    let mut final_image = image::RgbaImage::new(WIDTH, HEIGHT);

    ////////////////////////////////////////////////////////////////////////////

    // Draw the main image; usually this is the "front" or "3/4 front" view
    let image_full_front =
        imageops::resize(&image_full_front, 1800, 1800, imageops::FilterType::Nearest);
    let full_png_top_left_position: (i32, i32) =
        ((WIDTH as f32 * 0.15) as i32, (HEIGHT as f32 * 0.0) as i32);
    image::imageops::overlay(
        &mut final_image,
        &image_full_front,
        full_png_top_left_position.0.into(),
        full_png_top_left_position.1.into(),
    );
    let full_png_center_position =
        transform_relative_coords_to_absolute((1800 / 2, 1800 / 2), full_png_top_left_position);

    // Draw the side/back
    let image_back = imageops::resize(&image_back, 400, 400, imageops::FilterType::Nearest);
    let side_png_position: (i32, i32) =
        ((WIDTH as f32 * 0.05) as i32, (HEIGHT as f32 * 0.2) as i32);
    image::imageops::overlay(
        &mut final_image,
        &image_back,
        side_png_position.0.into(),
        side_png_position.1.into(),
    );

    ////////////////////////////////////////////////////////////////////////////
    // main drawing code

    let line_color = image::Rgba([0, 255, 0, 255]);

    // Load a system font (replace with the path to your TTF or OTF font file)
    let font_data = include_bytes!("../bindings/BF_Modernista-Regular.ttf");
    let font = rusttype::Font::try_from_bytes(font_data).expect("Failed to load font");

    // Draw boxes in a 4-way pattern with customizable color and stroke thickness
    const BOX_LENGTH: i32 = 350;
    const BOX_HEIGHT: i32 = 85;
    const PADDING_H: i32 = 10;
    const PADDING_V: i32 = 10;

    for button_param in &json_params.buttons_params {
        let keybinds: Vec<String> = button_param
            .physical_names
            .iter()
            .map(|physical_name| {
                // First: get the corresponding VIRTUAL button ID from "physical_name" in json
                // TODO(2-sticks) handle two sticks
                let virtual_buttons = joysticks_mappings
                    .get_virtual_button_ids_from_info_or_user_desc(physical_name, false)
                    .unwrap();

                // Next: get the game binding from this virtual_button_id
                let mut actions_names: String = "".to_string();
                for virtual_button in virtual_buttons {
                    let modifier: String = match &virtual_button.kind {
                        crate::button::VirtualButtonKind::Momentary(shift) => match shift {
                            Some(shift_kind) => match shift_kind {
                                crate::button::VirtualShiftKind::Shift1 => "[SHIFT1] ".to_string(),
                                crate::button::VirtualShiftKind::Shift2 => "[SHIFT2] ".to_string(),
                            },
                            None => "".to_string(),
                        },
                        crate::button::VirtualButtonKind::Tempo(tempo) => match tempo {
                            crate::button::VirtualTempoKind::Short => "[SHORT] ".to_string(),
                            crate::button::VirtualTempoKind::Long => "[LONG] ".to_string(),
                            crate::button::VirtualTempoKind::Double => "[DOUBLE] ".to_string(),
                        },
                    };

                    let mut action_name_with_modifier = modifier;

                    match game_buttons_mapping
                        .get_action_from_virtual_button_id(virtual_button.get_id())
                    {
                        Some(act_names) => {
                            action_name_with_modifier.push_str(&act_names.join("\n"));
                        }
                        None => action_name_with_modifier.push_str("NO BINDING"),
                    }

                    actions_names.push_str(&action_name_with_modifier);

                    actions_names.push_str("\n");
                }

                actions_names
            })
            .collect();

        match button_param.physical_names.len() {
            1 | 2 | 4 | 8 | 3 => {
                draw_boxes(
                    &mut final_image,
                    button_param.physical_names.len(),
                    image::Rgba([0, 150, 80, 180]),
                    2,
                    transform_relative_coords_to_absolute(
                        full_png_center_position,
                        button_param.desired_box_position_relative_to_center_full_png,
                    ),
                    BOX_LENGTH,
                    BOX_HEIGHT,
                    PADDING_H,
                    PADDING_V,
                    &font,
                    image::Rgba([200, 200, 10, 255]),
                    24,
                    keybinds,
                );

                draw_thicker_line_mut(
                    &mut final_image,
                    transform_relative_coords_to_absolute(
                        full_png_center_position,
                        button_param.connector_start_line_position_relative_to_center_full_png,
                    ),
                    transform_relative_coords_to_absolute(
                        full_png_center_position,
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
        .save("output.png")
        .expect("Failed to save the final image");
}

#[derive(Debug, Clone)]
struct TextParameters<'a> {
    text: String,
    text_size: u32,
    text_color: image::Rgba<u8>,
    font: &'a Font<'static>,
}

/// https://chat.openai.com
#[derive(Debug, Clone)]
struct BoxParameters<'a> {
    position: (i32, i32),
    size: (u32, u32),
    color: image::Rgba<u8>,
    stroke_thickness: i32,
    text_params: Option<TextParameters<'a>>,
}

/// Draw a thick line
/// https://chat.openai.com
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

/// https://chat.openai.com
fn draw_box(image: &mut image::RgbaImage, parameters: BoxParameters) {
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

    match parameters.text_params {
        Some(text_params) => {
            let scale = Scale::uniform(text_params.text_size as f32);
            // height: use the max height
            let text_height =
                imageproc::drawing::text_size(scale, text_params.font, &text_params.text).1;
            // TODO width: use the longest b/w every lines
            let max_text_width = imageproc::drawing::text_size(
                scale,
                text_params.font,
                &text_params.text.split("\n").collect::<Vec<_>>()[0],
            )
            .0;

            // Center the text, both horizontally and vertically
            for (line_no, line) in text_params.text.split("\n").enumerate() {
                imageproc::drawing::draw_text_mut(
                    image,
                    text_params.text_color,
                    (parameters.position.0 + parameters.size.0 as i32 / 2
                        - max_text_width as i32 / 2)
                        .try_into()
                        .unwrap(),
                    // text_size.1 / 4 b/c 2 would make the bottom of the text on the bottom of the box
                    (parameters.position.1 + parameters.size.1 as i32 / 4 - text_height as i32 / 2
                        + line_no as i32 * text_height as i32)
                        .try_into()
                        .unwrap(),
                    scale,
                    text_params.font,
                    &line,
                );
            }
        }
        None => {}
    }
}

/// https://chat.openai.com
///
/// `texts`: order is important; it MUST match with how "vkb_template_params.json" is handled
/// - 2 boxes: vertical, top -> bottom
/// - 3 boxes: horizontal, left -> right
/// - 4 boxes: clockwise, starts from NORTH: N -> E -> S -> W
/// - 8 boxes: clockwise, starts from NORTH = N -> NE -> E -> SE -> S -> SW -> W -> NW
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
    texts: Vec<String>,
) {
    assert_eq!(texts.len(), pattern);

    let draw_parameters = |x, y, txt: &str| BoxParameters {
        position: (x, y),
        size: (
            small_box_length.try_into().unwrap(),
            small_box_height.try_into().unwrap(),
        ),
        color,
        stroke_thickness,
        text_params: Some(TextParameters {
            text: txt.to_string(),
            text_size,
            text_color,
            font,
        }),
    };

    let mut draw_4_in_cross = |text_a, text_b, text_c, text_d| {
        // top center
        draw_box(
            image,
            draw_parameters(start_position.0, start_position.1, text_a),
        );
        // right, vertically in between "top center" and "bottom center"
        draw_box(
            image,
            draw_parameters(
                start_position.0 + small_box_length + padding_h,
                start_position.1 + small_box_height + padding_v,
                text_b,
            ),
        );
        // bottom center
        draw_box(
            image,
            draw_parameters(
                start_position.0,
                start_position.1 + 2 * (small_box_height + padding_v),
                text_c,
            ),
        );
        // left, vertically in between "top center" and "bottom center"
        draw_box(
            image,
            draw_parameters(
                start_position.0 - small_box_length - padding_h,
                start_position.1 + small_box_height + padding_v,
                text_d,
            ),
        );
    };

    match pattern {
        2 => {
            draw_box(
                image,
                draw_parameters(start_position.0, start_position.1, &texts[0]),
            );
            draw_box(
                image,
                draw_parameters(
                    start_position.0,
                    start_position.1 + small_box_height + padding_v,
                    &texts[1],
                ),
            );
        }
        4 => {
            draw_4_in_cross(&texts[0], &texts[1], &texts[2], &texts[3]);
        }
        8 => {
            // the 4 as above
            draw_4_in_cross(&texts[0], &texts[2], &texts[4], &texts[6]);

            // PLUS:
            // top right = NE
            draw_box(
                image,
                draw_parameters(
                    start_position.0 + small_box_length + padding_h,
                    start_position.1,
                    &texts[1],
                ),
            );
            // bottom right = SE
            draw_box(
                image,
                draw_parameters(
                    start_position.0 + small_box_length + padding_h,
                    start_position.1 + 2 * (small_box_height + padding_v),
                    &texts[3],
                ),
            );
            // bottom left = SW
            draw_box(
                image,
                draw_parameters(
                    start_position.0 - small_box_length - padding_h,
                    start_position.1 + 2 * (small_box_height + padding_v),
                    &texts[5],
                ),
            );
            // top left = NW
            draw_box(
                image,
                draw_parameters(
                    start_position.0 - small_box_length - padding_h,
                    start_position.1,
                    &texts[7],
                ),
            );
        }
        1 => {
            draw_box(
                image,
                draw_parameters(start_position.0, start_position.1, &texts[0]),
            );
        }
        // 3 horizontal
        3 => {
            // left
            draw_box(
                image,
                draw_parameters(start_position.0, start_position.1, &texts[0]),
            );
            // center
            draw_box(
                image,
                draw_parameters(
                    start_position.0 + 1 * (small_box_length + padding_h),
                    start_position.1,
                    &texts[1],
                ),
            );
            // right
            draw_box(
                image,
                draw_parameters(
                    start_position.0 + 2 * (small_box_length + padding_h),
                    start_position.1,
                    &texts[2],
                ),
            );
        }
        _ => {
            // Handle other cases or provide a default behavior
            unimplemented!("draw_boxes: pattern = only 1/2/4/8 are supported");
        }
    }
}

/// NOTE: this is for one joystick, either L or right
/// (at least for now)
#[derive(Serialize, Deserialize, Debug)]
struct TemplateJsonParamaters {
    path_to_full_png: PathBuf,
    path_to_side_png: PathBuf,
    buttons_params: Vec<TemplateJsonButtonOrStickParameters>,
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
    desired_box_position_relative_to_center_full_png: (i32, i32),
    connector_start_line_position_relative_to_center_full_png: (i32, i32),
    connector_end_line_position_relative_to_center_full_png: (i32, i32),
}

fn transform_relative_coords_to_absolute(add: (i32, i32), relative_to: (i32, i32)) -> (i32, i32) {
    (add.0 + relative_to.0, add.1 + relative_to.1)
}
