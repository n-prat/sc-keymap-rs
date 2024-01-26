use image::imageops;
use rusttype::Font;
use rusttype::Scale;
use std::path::PathBuf;

use crate::sc::parse_keybind_xml;
use crate::vkb::vkb_button;

pub fn generate_sc_template(
    game_buttons_mapping: parse_keybind_xml::GameButtonsMapping,
    joysticks_mappings: Vec<vkb_button::JoystickButtonsMapping>,
) {
    let path1: PathBuf = concat!(env!("CARGO_MANIFEST_DIR"), "/bindings/EVO_L_official.jpg").into();
    let image_full_front = image::open(path1).expect("Failed to open image1");

    let path2: PathBuf = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/bindings/EVO_stick_L_official.jpg"
    )
    .into();
    let image_back = image::open(path2).expect("Failed to open image2");

    // Create a new RgbaImage with dimensions based on the larger of the two images
    const WIDTH: u32 = 2000;
    const HEIGHT: u32 = 1200;
    // let width = image_full_front.width().max(image_back.width());
    // let height = image_full_front.height().max(image_back.height());
    let mut final_image = image::RgbaImage::new(WIDTH, HEIGHT);

    // Draw the first image onto the final image
    image::imageops::overlay(
        &mut final_image,
        &image_full_front,
        (WIDTH as f32 * 0.3) as i64,
        (HEIGHT as f32 * 0.2) as i64,
    );

    // Draw the second image onto the final image with an offset
    let image_back = imageops::resize(&image_back, 400, 400, imageops::FilterType::Nearest);
    image::imageops::overlay(
        &mut final_image,
        &image_back,
        (WIDTH as f32 * 0.05) as i64,
        (HEIGHT as f32 * 0.2) as i64,
    );

    // Add line connectors (example: draw a green line)
    let line_color = image::Rgba([0, 255, 0, 255]);
    // imageproc::drawing::draw_line_segment_mut(
    //     &mut final_image,
    //     (50.0, 50.0),
    //     (150.0, 150.0),
    //     line_color,
    // );
    draw_thicker_line_mut(&mut final_image, (50, 50), (150, 150), 4, line_color);

    // Load a system font (replace with the path to your TTF or OTF font file)
    let font_data = include_bytes!("../bindings/BF_Modernista-Regular.ttf");
    let font = rusttype::Font::try_from_bytes(font_data).expect("Failed to load font");

    // Draw boxes in a 4-way pattern with customizable color and stroke thickness
    const BOX_LENGTH: i32 = 150;
    const BOX_HEIGHT: i32 = 20;
    const PADDING_H: i32 = 10;
    const PADDING_V: i32 = 10;
    draw_boxes(
        &mut final_image,
        4,
        image::Rgba([120, 0, 80, 180]),
        2,
        (200, 300),
        BOX_LENGTH,
        BOX_HEIGHT,
        PADDING_H,
        PADDING_V,
        &font,
        image::Rgba([10, 10, 10, 255]),
        24,
    );
    draw_boxes(
        &mut final_image,
        2,
        image::Rgba([120, 0, 80, 180]),
        2,
        (1500, 300),
        BOX_LENGTH,
        BOX_HEIGHT,
        PADDING_H,
        PADDING_V,
        &font,
        image::Rgba([10, 10, 10, 255]),
        24,
    );
    draw_boxes(
        &mut final_image,
        8,
        image::Rgba([0, 150, 80, 180]),
        2,
        (1000, 800),
        BOX_LENGTH,
        BOX_HEIGHT,
        PADDING_H,
        PADDING_V,
        &font,
        image::Rgba([10, 10, 10, 255]),
        24,
    );

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
            let text_size =
                imageproc::drawing::text_size(scale, text_params.font, &text_params.text);

            // Center the text, both horizontally and vertically
            imageproc::drawing::draw_text_mut(
                image,
                text_params.text_color,
                (parameters.position.0 as u32 + parameters.size.0 / 2 - text_size.0 as u32 / 2)
                    .try_into()
                    .unwrap(),
                // text_size.1 / 4 b/c 2 would make the bottom of the text on the bottom of the box
                (parameters.position.1 as u32 + parameters.size.1 / 4 - text_size.1 as u32 / 2)
                    .try_into()
                    .unwrap(),
                scale,
                text_params.font,
                &text_params.text,
            );
        }
        None => {}
    }
}

/// https://chat.openai.com
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
) {
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

    let mut draw_4_in_cross = || {
        // top center
        draw_box(
            image,
            draw_parameters(start_position.0, start_position.1, "000"),
        );
        // right, vertically in between "top center" and "bottom center"
        draw_box(
            image,
            draw_parameters(
                start_position.0 + small_box_length + padding_h,
                start_position.1 + small_box_height + padding_v,
                "111",
            ),
        );
        // bottom center
        draw_box(
            image,
            draw_parameters(
                start_position.0,
                start_position.1 + 2 * (small_box_height + padding_v),
                "222",
            ),
        );
        // left, vertically in between "top center" and "bottom center"
        draw_box(
            image,
            draw_parameters(
                start_position.0 - small_box_length - padding_h,
                start_position.1 + small_box_height + padding_v,
                "333",
            ),
        );
    };

    match pattern {
        2 => {
            draw_box(
                image,
                draw_parameters(start_position.0, start_position.1, "aaa"),
            );
            draw_box(
                image,
                draw_parameters(
                    start_position.0,
                    start_position.1 + small_box_height + padding_v,
                    "bbb",
                ),
            );
        }
        4 => {
            draw_4_in_cross();
        }
        8 => {
            // the 4 as above
            draw_4_in_cross();

            // PLUS:
            // top right
            draw_box(
                image,
                draw_parameters(
                    start_position.0 + small_box_length + padding_h,
                    start_position.1,
                    "aaa",
                ),
            );
            // top left
            draw_box(
                image,
                draw_parameters(
                    start_position.0 - small_box_length - padding_h,
                    start_position.1,
                    "bbb",
                ),
            );
            // bottom right
            draw_box(
                image,
                draw_parameters(
                    start_position.0 + small_box_length + padding_h,
                    start_position.1 + 2 * (small_box_height + padding_v),
                    "ccc",
                ),
            );
            // bottom left
            draw_box(
                image,
                draw_parameters(
                    start_position.0 - small_box_length - padding_h,
                    start_position.1 + 2 * (small_box_height + padding_v),
                    "ddd",
                ),
            );
        }
        _ => {
            // Handle other cases or provide a default behavior
        }
    }
}
