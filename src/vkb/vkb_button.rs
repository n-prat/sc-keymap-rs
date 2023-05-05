//! This mod is AFTER parsing the XML.
//! It is doing some "light parsing": ie reading the appropriate fields in VkbReport struct
//! and building various "Button" instances from these.
//!

use scraper::ElementRef;
use scraper::Html;
use scraper::Selector;
use serde::Deserialize;

use super::vkb_xml::Page0Item;
use super::VkbError;

/// This is NOT from the xml, this is the end result.
/// We construct the final "buttons" by parsing the "xml_desc" field eg:
///
/// <b>#6 </b> Joystick button : #52
/// <b>#6 (F1) </b><b>TEMPO </b>\r\nVirtual button Short #6\r\nVirtual button Long #96
/// etc
#[derive(PartialEq, Debug)]
struct Button {
    kind: ButtonKind,
}

#[derive(PartialEq, Debug)]
enum ButtonKind {
    /// This matches a "b2" field in xml
    /// To get the ID we need to parse the desc...
    Physical { id: u8, kind: PhysicalButtonKind },
    /// Virtual/Logical
    /// This matches a "b3" field in xml
    /// In this case the "m8" field directly contains the ID, no parsing needed.
    /// The "m9" SHOULD also contain the same ID in the desc.
    Virtual { id: u8 },
}

#[derive(PartialEq, Debug)]
enum PhysicalButtonKind {
    /// The standard, basic button with no SHIT, or anything particular
    /// VKB = "Button with momentary action"
    Momentary {
        shift: Option<ShiftKind>,
    },
    /// This is the wheel on the bottom right of the stick (one per stick)
    Encoder,
    Tempo(TempoKind),
    /// The SHIFT1 = ALT button 1
    Shift1,
    /// The SHIFT2 = ALT button 2
    Shift2,
    /// "Point of view Switch"
    /// eg "POV1  Up", "POV1  Left", etc
    Pov {
        direction: String,
    },
    /// "No defined function"
    Undefined,
}

#[derive(PartialEq, Debug)]
enum TempoKind {
    /// Short+Long press
    /// "second line pulse length is equal to T_Tgl value in no matter to real depressing time"
    Tempo1,
    /// Short+Long press
    /// "second line pulse length is equal to button depressing time"
    Tempo2 {
        button_id_short: u8,
        button_id_long: u8,
    },
    /// Short+Long press+Double press
    Tempo3,
}

#[derive(PartialEq, Debug)]
enum ShiftKind {
    Shift1 {
        button_id_shift1: u8,
    },
    Shift2,
    Shift12 {
        button_id_shift1: u8,
        button_id_shift2: u8,
    },
}

/// Parse eg "#1 (E1) ", "#2  - Encoder 2/4", etc
/// SHOULD be called with the FIRST "b" node of the desc!
/// Return:
/// - ALWAYS a "Button ID" eg 1,2,etc
/// - if applicable: "additional into" eg "(E1)", "Encoder 2/4", etc
fn extract_button_id_from_inner_html(inner_html_desc: &str) -> ButtonIdAndInfo {
    assert!(inner_html_desc.starts_with("#"));
    let (button_id_str, info_str) = inner_html_desc[1..].split_once(" ").unwrap();

    ButtonIdAndInfo {
        id: button_id_str.parse().unwrap(),
        info: Some(info_str.trim().to_string()),
    }
}

#[derive(Debug, PartialEq)]
struct ButtonIdAndInfo {
    id: u8,
    info: Option<String>,
}

/// Get all the Text matching a given element sibling
/// and apply some cleanup (remove new lines, blanks, etc)
/// Return either:
/// - only one string if there is only one (useful) line in the siblings
/// - or 2 strings if there are two useful lines
/// - or none at all
///
/// NOTE: the given "element" SHOULD be the first element in tree order b/c we only iterate forward("next_sibling")
fn extract_text_next_siblings(element: &ElementRef) -> (Option<String>, Option<String>) {
    let mut text_siblings = Vec::new();

    // Iterate over the next siblings
    let mut current = element.next_sibling();
    while let Some(sibling) = current {
        // if let NodeRef::Text(text) = sibling.value() {
        if let Some(text) = sibling.value().as_text() {
            let text = text.trim();
            let text = text.replacen("\n", "", 1);
            text_siblings.push(text);
        }
        current = sibling.next_sibling();
    }

    match &text_siblings[..] {
        [] => (None, None),
        [first] => (Some(first.to_string()), None),
        [first, second] => (Some(first.to_string()), Some(second.to_string())),
        _ => todo!(),
    }
}

fn parse_b2_button_desc_xml_escaped(desc_xml_escaped: &str) -> Result<Button, VkbError> {
    // let lines: Vec<&str> = desc_xml_escaped.split("\r\n").collect();
    // let first_line = lines[0];

    let fragment = Html::parse_fragment(desc_xml_escaped);

    // println!("parse_desc_xml fragment : {:#?}", fragment.tree);
    // for node in fragment.tree.nodes() {
    //     println!("node : {:#?}", node);
    // }

    // the selected inner_html should contain something like:
    // "#1 (E1) ", "#2  - Encoder 2/4", etc
    let b_selector = Selector::parse("b").unwrap();
    let b_nodes: Vec<_> = fragment.select(&b_selector).collect();

    // println!("b_nodes [{}] : {:?}", b_nodes.len(), b_nodes);
    // for b_node in b_nodes.iter() {
    //     println!("b_node : inner_html : {:#?}", b_node.inner_html());
    // }

    // Only extract the ID from the FIRST "b" node
    let button_id_info = extract_button_id_from_inner_html(&b_nodes[0].inner_html());
    // IF there are more "b" nodes, they contains only additional info like:
    // "<b>- Button with momentary action</b>"
    // "<b>TEMPO </b>"
    // etc
    assert!(b_nodes.len() <= 2, "more b nodes than expected!");
    let remaining_b_node = &b_nodes.get(1);
    let texts = extract_text_next_siblings(&b_nodes[0]);

    // Now we have various cases:
    //
    // - NO "b" node, only text: eg
    //      "Joystick button : #3"
    // - "b" node AND text: eg
    //      "<b>Microstick Mode Switch </b>\r\n Switch Mode:"
    //      "<b>- Button with momentary action</b>\r\nVirtual button with SHIFT1 = 50"
    //      "<b>Point of view Switch</b> POV1  Down"
    //      " / <b>#2  - Encoder 2/4</b>\r\nVirtual buttons : #61 / #62"
    // - "b" node, NO text:
    //      "<b>#39 </b><b> No defined function</b>"
    //      "<b>#66 </b><b>- Button with momentary action</b>"
    let remaining_b_node = match remaining_b_node {
        Some(remaining_b_node) => remaining_b_node,
        None => unimplemented!("SHOULD NOT be here, this is only for b3 field???"),
    };

    let remaining_b_node_inner_html = remaining_b_node.inner_html();
    let kind = if remaining_b_node_inner_html.contains("TEMPO") {
        let text = match texts {
            (Some(text), None) => text,
            _ => unimplemented!("TEMPO: SHOULD NOT be here!"),
        };
        if text.contains("Virtual button Short #") && text.contains("Virtual button Long #") {
            let short_id = text
                .split("Virtual button Short #")
                .last()
                .unwrap()
                .split("Virtual button Long #")
                .next()
                .unwrap();
            let long_id = text.split("Virtual button Long #").last().unwrap();
            PhysicalButtonKind::Tempo(TempoKind::Tempo2 {
                button_id_short: short_id.parse().unwrap(),
                button_id_long: long_id.parse().unwrap(),
            })
        } else {
            todo!()
        }
    } else if remaining_b_node_inner_html.contains("Encoder") {
        PhysicalButtonKind::Encoder
    } else if remaining_b_node_inner_html.contains("Button with momentary action") {
        match texts {
            (None, None) => PhysicalButtonKind::Momentary { shift: None },
            (None, Some(_)) => unimplemented!(
                "SHOULD NOT be here, SHOULD NOT be able to get a second txt without a first!"
            ),
            (Some(text), None) => {
                if text.contains("Virtual button with SHIFT1 =")
                    && text.contains("Virtual button with SHIFT2 =")
                {
                    let shift1_id = text
                        .split("Virtual button with SHIFT1 = ")
                        .last()
                        .unwrap()
                        .split("Virtual button with SHIFT2 = ")
                        .next()
                        .unwrap();
                    let shift2_id = text.split("Virtual button with SHIFT2 = ").last().unwrap();
                    PhysicalButtonKind::Momentary {
                        shift: Some(ShiftKind::Shift12 {
                            button_id_shift1: shift1_id.parse().unwrap(),
                            button_id_shift2: shift2_id.parse().unwrap(),
                        }),
                    }
                } else if text.contains("Virtual button with SHIFT1 =")
                    && !text.contains("Virtual button with SHIFT2 =")
                {
                    let shift1_id = text.split("Virtual button with SHIFT1 = ").last().unwrap();
                    PhysicalButtonKind::Momentary {
                        shift: Some(ShiftKind::Shift1 {
                            button_id_shift1: shift1_id.parse().unwrap(),
                        }),
                    }
                } else {
                    todo!()
                }
            }
            (Some(_), Some(_)) => todo!(),
        }
    } else if remaining_b_node_inner_html.contains(" SHIFT1 ") {
        PhysicalButtonKind::Shift1
    } else if remaining_b_node_inner_html.contains("Point of view Switch") {
        let direction = texts.1.unwrap().split(" ").last().unwrap().to_string();
        PhysicalButtonKind::Pov { direction }
    } else if remaining_b_node_inner_html.contains("No defined function") {
        PhysicalButtonKind::Undefined
    } else {
        todo!("not TEMPO");
    };

    // else if remaining_b_node.is_some() && remaining_b_node.unwrap().parent().unwrap().s

    let button = Button {
        kind: ButtonKind::Physical {
            id: button_id_info.id,
            kind,
        },
    };

    Ok(button)
}

fn parse_b3_button_desc_xml_escaped(desc_xml_escaped: &str) -> Result<Button, VkbError> {
    let button = Button {
        kind: ButtonKind::Virtual { id: todo!() },
    };

    Ok(button)
}

fn construct_button_from_xml(page_item: Page0Item) -> Result<Button, VkbError> {
    match page_item {
        Page0Item::b2(b2_xml) => match b2_xml.m7 {
            Some(m7) => parse_b2_button_desc_xml_escaped(&m7.desc_xml_escaped),
            None => Err(VkbError::UnexpectedXmlDesc(format!("{:?}", b2_xml))),
        },
        Page0Item::b3(b3_xml) => {
            let button = parse_b3_button_desc_xml_escaped(&b3_xml.m9.desc_xml_escaped)?;
            // TODO assert_eq!(button.id, b3_xml.m8.virtual_button_id);
            Ok(button)
        }
        _ => unimplemented!("parse_button_xml SHOULD only be called with b2 or b3 field"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_button_id_from_inner_html() {
        let test_inputs_vs_expected_results = vec![
            (
                "#1 (E1) ",
                ButtonIdAndInfo {
                    id: 1,
                    info: Some("(E1)".to_string()),
                },
            ),
            (
                "#2  - Encoder 2/4",
                ButtonIdAndInfo {
                    id: 2,
                    info: Some("- Encoder 2/4".to_string()),
                },
            ),
        ];

        for (input, expected_result) in test_inputs_vs_expected_results {
            let button = extract_button_id_from_inner_html(input);
            assert_eq!(button, expected_result);
        }
    }

    #[test]
    fn test_construct_button_b2() {
        // Here are all (?) the possible cases for a "b2" desc field:
        // "<b>#1 (E1) </b> / <b>#2  - Encoder 2/4</b>\r\nVirtual buttons : #61 / #62"
        // "<b>#3 (E2) </b><b>- Button with momentary action</b>"
        // "<b>#4 </b><b>- Button with momentary action</b>"
        // "<b>#5 (F3) </b><b>TEMPO </b>\r\nVirtual button Short #5\r\nVirtual button Long #94"
        // "<b>#9 (Fire 2-nd stage) </b><b>- Button with momentary action</b>"
        // "<font color=\"#000000\">Virtual button with SHIFT1 = 63\r\nVirtual button with SHIFT2 = 92"
        // "<b>#10 (Fire 1-st stage) </b><b>- Button with momentary action</b>\r\nVirtual button with SHIFT1 = 64\r\nVirtual button with SHIFT2 = 91"
        // "<b>#11 (D1) </b><b> SHIFT1 </b>"
        // "<b>#12 (A2) </b><b>- Button with momentary action</b>\r\nVirtual button with SHIFT1 = 13\r\nVirtual button with SHIFT2 = 90"
        // "<b>#18 (A1 down) </b> <b>Point of view Switch</b> POV1  Down"
        // "<b>#35 (Rapid fire forward) </b><b>- Button with momentary action</b>\r\nVirtual button with SHIFT1 = 37"
        // "<b>#37 </b><b> No defined function</b>"
        // ""
        let test_inputs_vs_expected_results = vec![
            (
                "<b>#1 (E1) </b> / <b>#2  - Encoder 2/4</b>\r\nVirtual buttons : #61 / #62",
                Button {
                    kind: ButtonKind::Physical {
                        id: 1,
                        kind: PhysicalButtonKind::Encoder,
                    },
                },
            ),
            (
                "<b>#3 (E2) </b><b>- Button with momentary action</b>",
                Button {
                    kind: ButtonKind::Physical {
                        id: 3,
                        kind: PhysicalButtonKind::Momentary{ shift: None },
                    },
                },
            ),
            (
                "<b>#4 </b><b>- Button with momentary action</b>",
                Button {
                    kind: ButtonKind::Physical {
                        id: 4,
                        kind: PhysicalButtonKind::Momentary{ shift: None },
                    },
                },
            ),
            (
                "<b>#5 (F3) </b><b>TEMPO </b>\r\nVirtual button Short #5\r\nVirtual button Long #94",
                Button {
                    kind: ButtonKind::Physical {
                        id: 5,
                        kind: PhysicalButtonKind::Tempo(TempoKind::Tempo2 { button_id_short: 5, button_id_long: 94 }),
                    },
                },
            ),
            (
                "<b>#9 (Fire 2-nd stage) </b><b>- Button with momentary action</b>",
                Button {
                    kind: ButtonKind::Physical {
                        id: 9,
                        kind: PhysicalButtonKind::Momentary { shift: None },
                    },
                },
            ),
            (
                "<b>#10 (Fire 1-st stage) </b><b>- Button with momentary action</b>\r\nVirtual button with SHIFT1 = 64\r\nVirtual button with SHIFT2 = 91",
                Button {
                    kind: ButtonKind::Physical {
                        id: 10,
                        kind: PhysicalButtonKind::Momentary { shift: Some(ShiftKind::Shift12 { button_id_shift1: 64, button_id_shift2: 91 }) },
                    },
                },
            ),
            (
                "<b>#11 (D1) </b><b> SHIFT1 </b>",
                Button {
                    kind: ButtonKind::Physical {
                        id: 11,
                        kind: PhysicalButtonKind::Shift1,
                    },
                },
            ),
            (
                "<b>#12 (A2) </b><b>- Button with momentary action</b>\r\nVirtual button with SHIFT1 = 13\r\nVirtual button with SHIFT2 = 90",
                Button {
                    kind: ButtonKind::Physical {
                        id: 12,
                        kind: PhysicalButtonKind::Momentary { shift: Some(ShiftKind::Shift12 { button_id_shift1: 13, button_id_shift2: 90 }) },
                    },
                },
            ),
            (
                "<b>#18 (A1 down) </b> <b>Point of view Switch</b> POV1  Down",
                Button {
                    kind: ButtonKind::Physical {
                        id: 18,
                        kind: PhysicalButtonKind::Pov { direction: "Down".to_string() },
                    },
                },
            ),
            (
                "<b>#35 (Rapid fire forward) </b><b>- Button with momentary action</b>\r\nVirtual button with SHIFT1 = 37",
                Button {
                    kind: ButtonKind::Physical {
                        id: 35,
                        kind: PhysicalButtonKind::Momentary { shift: Some(ShiftKind::Shift1 { button_id_shift1: 37 }) },
                    },
                },
            ),
            (
                "<b>#37 </b><b> No defined function</b>",
                Button {
                    kind: ButtonKind::Physical {
                        id: 37,
                        kind: PhysicalButtonKind::Undefined,
                    },
                },
            ),
        ];

        // TODO handle // "<font color=\"#000000\">Virtual button with SHIFT1 = 63\r\nVirtual button with SHIFT2 = 92"
        // This seem to mean we need to combine this b2 with the one from the previous page0?

        for (input, expected_result) in test_inputs_vs_expected_results {
            let button = parse_b2_button_desc_xml_escaped(input).unwrap();
            assert_eq!(button, expected_result);
        }
    }
}
