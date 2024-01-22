//! This mod is AFTER parsing the XML.
//! It is doing some "light parsing": ie reading the appropriate fields in VkbReport struct
//! and building various "Button" instances from these.
//!

use std::collections::HashSet;

use log::warn;
use scraper::ElementRef;
use scraper::Html;
use scraper::Selector;
use serde::Deserialize;

use super::vkb_xml::Page0Item;
use super::vkb_xml::VkbReport;
use super::vkb_xml::VkbXmlButton;
use super::vkb_xml::B2;
use super::vkb_xml::B3;
use super::VkbError;

/// This is NOT from the xml, this is the end result.
/// We construct the final "buttons" by parsing the "xml_desc" field eg:
///
/// <b>#6 </b> Joystick button : #52
/// <b>#6 (F1) </b><b>TEMPO </b>\r\nVirtual button Short #6\r\nVirtual button Long #96
/// etc
#[derive(PartialEq, Debug, Clone)]
struct Button {
    kind: ButtonKind,
}

impl Button {
    pub(super) fn get_id(&self) -> u8 {
        match &self.kind {
            ButtonKind::Physical { id, kind } => *id,
            ButtonKind::Virtual { id } => *id,
        }
    }
}

impl TryFrom<VkbXmlButton> for Button {
    type Error = VkbError;

    fn try_from(xml_button: VkbXmlButton) -> Result<Self, Self::Error> {
        match xml_button {
            VkbXmlButton::B2(b2) => Button::try_from(b2),
            VkbXmlButton::B3(b3) => Button::try_from(b3),
        }
    }
}

/// This is the final mapping, correspondonding to the XML!
///
/// It does NOT contain any game keybind!
#[derive(PartialEq, Debug)]
pub(crate) struct ButtonMap {
    /// This set is here to help detect duplicated virtual buttons
    /// Typically when using SHIFT or TEMPO you can have 2 different physical buttons
    /// that end up bound to the same virtual/logical one in-game.
    ///
    /// Result: the same in-game function can be done using two different buttons
    ///
    /// Note that is NOT detect by VkbDevCfg, probably because this NOT (necessarily) a bug;
    /// this is mostly a "waste of space".
    already_seen_virtual_buttons: HashSet<u8>,
}

impl TryFrom<VkbReport> for ButtonMap {
    type Error = VkbError;

    fn try_from(vkb_report: VkbReport) -> Result<Self, Self::Error> {
        let mut already_seen_virtual_buttons = HashSet::new();

        let vkb_buttons = vkb_report.get_all_buttons();

        // We loop on all b2/b3 buttons from the xml
        // IMPORTANT:
        // - b2 are the physical buttons, they are the PARENT
        // - b3 are virtual/logical ones: these are the ones bound in-game
        let mut current_parent = None;
        for vkb_button in vkb_buttons {
            match Button::try_from(vkb_button) {
                Ok(button) => {
                    match &button.kind {
                        ButtonKind::Physical { id, kind } => {
                            current_parent = Some(&button);
                        }
                        ButtonKind::Virtual { id } => {
                            match already_seen_virtual_buttons.insert(id.clone()) {
                                true => {
                                    // inserted = nothing to do
                                }
                                false => {
                                    // NOT inserted = the virtual button was already processed!
                                    warn!("virtual button duplicated : {}", id);
                                }
                            };
                        }
                    };
                }
                Err(err) => todo!(),
            }
        }

        Ok(Self {
            already_seen_virtual_buttons,
        })
    }
}

#[derive(PartialEq, Debug, Clone)]
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

#[derive(PartialEq, Debug, Clone)]
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
    /// "(Ministick push) Microstick Mode Switch"
    MicrostickModeSwitch,
}

#[derive(PartialEq, Debug, Clone)]
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
    Tempo3 {
        button_id_short: u8,
        button_id_long: u8,
        button_id_double: u8,
    },
}

#[derive(PartialEq, Debug, Clone)]
enum ShiftKind {
    Shift1 {
        button_id_shift1: u8,
    },
    Shift2 {
        button_id_shift2: u8,
    },
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
        if text.contains("Virtual button Short #")
            && text.contains("Virtual button Long #")
            && !text.contains("Virtual button Double Short #")
        {
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
        } else if text.contains("Virtual button Short #")
            && text.contains("Virtual button Long #")
            && text.contains("Virtual button Double Short #")
        {
            let short_id = text
                .split("Virtual button Short #")
                .last()
                .unwrap()
                .split("Virtual button Long #")
                .next()
                .unwrap();
            let long_id = text
                .split("Virtual button Long #")
                .last()
                .unwrap()
                .split("Virtual button Double Short #")
                .next()
                .unwrap()
                .replace("\n", "");
            let double_id = text.split("Virtual button Double Short #").last().unwrap();
            PhysicalButtonKind::Tempo(TempoKind::Tempo3 {
                button_id_short: short_id.parse().unwrap(),
                button_id_long: long_id.parse().unwrap(),
                button_id_double: double_id.parse().unwrap(),
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
                } else if !text.contains("Virtual button with SHIFT1 =")
                    && text.contains("Virtual button with SHIFT2 =")
                {
                    let shift1_id = text.split("Virtual button with SHIFT2 = ").last().unwrap();
                    PhysicalButtonKind::Momentary {
                        shift: Some(ShiftKind::Shift2 {
                            button_id_shift2: shift1_id.parse().unwrap(),
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
    } else if remaining_b_node_inner_html.contains(" SHIFT2 alternate action") {
        PhysicalButtonKind::Shift2
    } else if remaining_b_node_inner_html.contains("Point of view Switch") {
        let direction = texts.1.unwrap().split(" ").last().unwrap().to_string();
        PhysicalButtonKind::Pov { direction }
    } else if remaining_b_node_inner_html.contains("No defined function") {
        PhysicalButtonKind::Undefined
    } else if remaining_b_node_inner_html.contains("Microstick Mode Switch") {
        PhysicalButtonKind::MicrostickModeSwitch
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
    let fragment = Html::parse_fragment(desc_xml_escaped);

    // the selected inner_html should contain something like:
    // "#1 (E1) ", "#2  - Encoder 2/4", etc
    let b_selector = Selector::parse("b").unwrap();
    let b_nodes: Vec<_> = fragment.select(&b_selector).collect();
    assert_eq!(
        b_nodes.len(),
        1,
        "b3 buttons SHOULD have only one <b> elem!"
    );

    // Only extract the ID from the FIRST "b" node
    let button_id_info = extract_button_id_from_inner_html(&b_nodes[0].inner_html());
    assert!(
        button_id_info.info.is_none() || button_id_info.info.unwrap_or_default().is_empty(),
        "b3 SHOULD NOT have info!"
    );

    let button = Button {
        kind: ButtonKind::Virtual {
            id: button_id_info.id,
        },
    };

    Ok(button)
}

/// Try to build a Button(Physical) from a B2 xml field
impl TryFrom<B2> for Button {
    type Error = VkbError;

    fn try_from(b2_xml: B2) -> Result<Self, Self::Error> {
        let button = parse_b2_button_desc_xml_escaped(&b2_xml.m7.desc_xml_escaped)?;

        // TODO(re-add CHECK): the "m5" field SHOULD match the parsed button ID
        // if b2_xml.m5.physical_button_id.parse::<u8>().unwrap() != button.get_id() {
        //     return Err(VkbError::UnexpectedXmlDesc(format!(
        //         "m5 field value does not match: {:?}",
        //         b2_xml
        //     )));
        // }

        Ok(button)
    }
}

/// Try to build a Button(Virtual) from a B3 xml field
impl TryFrom<B3> for Button {
    type Error = VkbError;

    fn try_from(b3_xml: B3) -> Result<Self, Self::Error> {
        let button = parse_b3_button_desc_xml_escaped(&b3_xml.m9.desc_xml_escaped)?;

        // CHECK: the "m8" field SHOULD match the parsed button ID
        if b3_xml.m8.virtual_button_id.parse::<u8>().unwrap() != button.get_id() {
            return Err(VkbError::UnexpectedXmlDesc(format!(
                "m8 field value does not match: {:?}",
                b3_xml
            )));
        }

        Ok(button)
    }
}

#[cfg(test)]
mod tests {
    use crate::vkb::vkb_xml::VkbReport;

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
    fn test_parse_b2_button_desc_xml_escaped() {
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
                "<b>#5 (F3) </b><b>TEMPO </b>\r\nVirtual button Short #5\r\nVirtual button Long #94\r\nVirtual button Double Short #95",
                Button {
                    kind: ButtonKind::Physical {
                        id: 5,
                        kind: PhysicalButtonKind::Tempo(TempoKind::Tempo3 { button_id_short: 5, button_id_long: 94, button_id_double: 95 } ),
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
            (
                "<b>#9 (Fire 2-nd stage) </b><b>- Button with momentary action</b>\r\nVirtual button with SHIFT1 = 63\r\nVirtual button with SHIFT2 = 92",
                Button {
                    kind: ButtonKind::Physical {
                        id: 9,
                        kind: PhysicalButtonKind::Momentary { shift: Some(ShiftKind::Shift12 { button_id_shift1: 63, button_id_shift2: 92 }) },
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

    #[test]
    fn test_parse_b3_button_desc_xml_escaped() {
        let test_inputs_vs_expected_results = vec![(
            r#"<b>#61 </b> Joystick button : #61"#,
            Button {
                kind: ButtonKind::Virtual { id: 61 },
            },
        )];

        for (input, expected_result) in test_inputs_vs_expected_results {
            let button = parse_b3_button_desc_xml_escaped(input).unwrap();
            assert_eq!(button, expected_result);
        }
    }

    #[test]
    fn test_buttons_try_from_vkb_report_simplified() {
        let vkb_report = VkbReport::new(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/data/vkb_report_simplified.fp3"
            )
            .into(),
        )
        .unwrap();

        let vkb_buttons = vkb_report.get_all_buttons();
        assert!(vkb_buttons.len() > 10);

        for vkb_button in vkb_buttons {
            assert!(
                Button::try_from(vkb_button.clone()).is_ok(),
                "FAIL: could not parse: {:?}",
                vkb_button
            );
        }
    }

    #[test]
    fn test_button_map_simplified() {
        let vkb_report = VkbReport::new(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/data/vkb_report_simplified.fp3"
            )
            .into(),
        )
        .unwrap();

        assert!(ButtonMap::try_from(vkb_report).is_ok());
    }

    #[test]
    fn test_button_map_vkb_report_R() {
        let vkb_report = VkbReport::new(
            concat!(env!("CARGO_MANIFEST_DIR"), "/bindings/vkb_report_R.fp3").into(),
        )
        .unwrap();

        assert!(ButtonMap::try_from(vkb_report).is_ok());
    }

    #[test]
    fn test_button_map_vkb_report_L() {
        let vkb_report = VkbReport::new(
            concat!(env!("CARGO_MANIFEST_DIR"), "/bindings/vkb_report_L.fp3").into(),
        )
        .unwrap();

        assert!(ButtonMap::try_from(vkb_report).is_ok());
    }
}
