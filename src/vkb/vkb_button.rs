//! This mod is AFTER parsing the XML.
//! It is doing some "light parsing": ie reading the appropriate fields in VkbReport struct
//! and building various "Button" instances from these.
//!

use std::collections::HashMap;

use scraper::ElementRef;
use scraper::Html;
use scraper::Selector;

use super::vkb_xml::VkbReport;
use super::vkb_xml::VkbXmlButton;
use super::VkbError;
use crate::button::PhysicalButton;
use crate::button::SpecialButtonKind;
use crate::button::VirtualButton;
use crate::button::VirtualButtonKind;
use crate::button::VirtualShiftKind;
use crate::button::VirtualTempoKind;
use crate::button::{PhysicalButtonKind, ShiftKind, TempoKind};

/// Custom `TryFrom<VkbXmlButton>` allowing us to link a parent to a Virtual button
// impl Button {
//     fn try_from(xml_button: VkbXmlButton, parent: &Option<Button>) -> Result<Self, VkbError> {
//         match xml_button {
//             VkbXmlButton::B2(b2) => Button::try_from(b2, parent),
//             VkbXmlButton::B3(b3) => Button::try_from(b3, parent),
//         }
//     }
// }

// /// That is what will be printed on the final png/pdf/svg
// #[derive(PartialEq, Debug, Clone)]
// pub(crate) struct PhysicalButtonWithDesc {
//     pub(crate) id: u8,
//     /// Usually this will be extracted from the "b2.m7" xml field; up to the first "\r\n"
//     pub(crate) info: String,
//     /// cf `extract_button_id_from_inner_html`
//     pub(crate) extended_desc: String,
//     pub(crate) user_desc: String,
// }

/// This is the final mapping, correspondonding to the XML!
///
/// It does NOT contain any game keybind!
#[derive(PartialEq, Debug, Clone)]
pub struct JoystickButtonsMapping {
    /// This set is here to help detect duplicated virtual buttons
    /// Typically when using SHIFT or TEMPO you can have 2 different physical buttons
    /// that end up bound to the same virtual/logical one in-game.
    ///
    /// Result: the same in-game function can be done using two different buttons
    ///
    /// Note that is NOT detected by VkbDevCfg, probably because this NOT (necessarily) a bug;
    /// this is mostly a "waste of space".
    pub(crate) map_virtual_button_id_to_parent_physical_buttons: HashMap<u8, Vec<PhysicalButton>>,
    /// Inverse of `map_virtual_button_id_to_physical_button`
    pub(crate) map_physical_button_id_to_children_virtual_buttons: HashMap<u8, Vec<VirtualButton>>,
    // pub(crate) physical_buttons_with_desc: Vec<PhysicalButtonWithDesc>,
    /// MAP eg "(D1)" -> Shift1
    pub(crate) map_special_buttons: HashMap<String, SpecialButtonKind>,
}

impl JoystickButtonsMapping {
    pub(crate) fn inject_user_provided_desc(
        &mut self,
        vkb_user_provided_data: csv::Reader<std::fs::File>,
    ) {
        let csv_records: Vec<_> = vkb_user_provided_data.into_records().collect();

        // for button_with_desc in &mut self.physical_buttons_with_desc {
        //     let user_desc = csv_records[button_with_desc.id as usize - 1]
        //         .as_ref()
        //         .expect("MISSING CSV RECORD")
        //         .get(1)
        //         .expect("MISSING CSV COLUMN")
        //         .to_string();
        //     button_with_desc.user_desc = user_desc;
        // }

        // ALSO update the other field
        for (_virtual_button_id, physical_buttons_parents) in &mut self
            .map_virtual_button_id_to_parent_physical_buttons
            .iter_mut()
        {
            for physical_button_parent in physical_buttons_parents.iter_mut() {
                let user_desc = csv_records[*physical_button_parent.get_id() as usize - 1]
                    .as_ref()
                    .expect("MISSING CSV RECORD")
                    .get(1)
                    .expect("MISSING CSV COLUMN")
                    .to_string();
                physical_button_parent.set_user_desc(&user_desc);
            }
        }
    }

    /// CHECK/LOG the "free" Virtual buttons
    /// VkbDevCfg "auto" should probably work but it ends up duplicating virtual buttons
    /// This is REALLY useful when trying to add SHIFT1/2/TEMPO to an existing button without messing up existing keybinds
    pub(crate) fn log_free_virtual_buttons(&self) -> Vec<u8> {
        const NB_VIRTUAL_BUTTONS: u8 = 128;

        let mut unused_virtual_buttons = vec![];

        for i in 1..NB_VIRTUAL_BUTTONS {
            if !self
                .map_virtual_button_id_to_parent_physical_buttons
                .contains_key(&i)
            {
                unused_virtual_buttons.push(i);
            }
        }

        log::info!("unused virtual buttons : {unused_virtual_buttons:?}");

        unused_virtual_buttons
    }
}

impl TryFrom<VkbReport> for JoystickButtonsMapping {
    type Error = VkbError;

    // TODO(add-CHECK) this should be 2 maps; one parent->children and one child->parent; that way we can display proper
    // warnings to find where the duplicates originate
    fn try_from(vkb_report: VkbReport) -> Result<Self, Self::Error> {
        let mut map_virtual_button_id_to_parent_physical_buttons: HashMap<u8, Vec<PhysicalButton>> =
            HashMap::new();
        let mut map_physical_button_id_to_children_virtual_buttons: HashMap<
            u8,
            Vec<VirtualButton>,
        > = HashMap::new();
        let mut map_special_buttons: HashMap<String, SpecialButtonKind> = HashMap::new();

        let vkb_buttons = vkb_report.get_all_buttons();

        // We loop on all b2/b3 buttons from the xml
        // IMPORTANT:
        // - b2 are the physical buttons, they are the PARENT
        // - b3 are virtual/logical ones: these are the ones bound in-game
        let mut current_parent = None;
        for vkb_button in vkb_buttons {
            match vkb_button {
                VkbXmlButton::B2(b2_xml) => {
                    // Try to build a Button(Physical) from a B2 xml field
                    let physical_button =
                        parse_b2_button_desc_xml_escaped(&b2_xml.m7.desc_xml_escaped)?;

                    // TODO(re-add CHECK): the "m5" field SHOULD match the parsed button ID
                    // if b2_xml.m5.physical_button_id.parse::<u8>().unwrap() != button.get_id() {
                    //     return Err(VkbError::UnexpectedXmlDesc(format!(
                    //         "m5 field value does not match: {:?}",
                    //         b2_xml
                    //     )));
                    // }

                    // Store the modifier buttons separately
                    // cf `get_virtual_button_ids_from_info_or_user_desc` for how it's used
                    match physical_button.get_kind() {
                        PhysicalButtonKind::Shift1 => {
                            map_special_buttons.insert(
                                physical_button.get_info().to_string(),
                                SpecialButtonKind::Shift1,
                            );
                        }
                        PhysicalButtonKind::Shift2 => {
                            map_special_buttons.insert(
                                physical_button.get_info().to_string(),
                                SpecialButtonKind::Shift2,
                            );
                        }
                        _ => {}
                    };

                    current_parent = Some(physical_button.clone());
                }
                VkbXmlButton::B3(b3_xml) => {
                    // Try to build a Button(Virtual) from a B3 xml field
                    let virtual_button = parse_b3_button_desc_xml_escaped(
                        &b3_xml.m9.desc_xml_escaped,
                        &current_parent
                            .clone()
                            .expect("trying to build a Virtual button without a valid parent!"),
                    )?;

                    // CHECK: the "m8" field SHOULD match the parsed button ID
                    // NO! cf `test_parse_b3_button_desc_xml_escaped`
                    // if b3_xml.m8.virtual_button_id.parse::<u8>().unwrap() != button.get_id() {
                    //     return Err(VkbError::UnexpectedXmlDesc(format!(
                    //         "m8 field value does not match: {:?}",
                    //         b3_xml
                    //     )));
                    // }

                    let virtual_button_id = virtual_button.get_id();

                    map_virtual_button_id_to_parent_physical_buttons
                        .entry(*virtual_button_id)
                        // NOT inserted = the virtual button was already processed!
                        .and_modify(|parents| {
                            parents.push(current_parent.clone().unwrap().clone());
                            log::warn!(
                                "virtual button duplicated : {} from physical : {parents:?}",
                                virtual_button_id
                            );
                        })
                        // inserted = nothing to do
                        .or_insert(vec![current_parent.clone().unwrap().clone()]);

                    map_physical_button_id_to_children_virtual_buttons
                        .entry(*current_parent.clone().unwrap().get_id())
                        // NOT inserted = the virtual button was already processed!
                        .and_modify(|children| {
                            children.push(virtual_button.clone().try_into().unwrap());
                            // is this a warning???
                            log::info!(
                                "physical button duplicated : {} from logical : {children:?}",
                                virtual_button_id
                            );
                        })
                        // inserted = nothing to do
                        .or_insert(vec![virtual_button.clone().try_into().unwrap()]);

                    // NOTE: NOT an error; it can just happen; for example right now for the 8 way ministick switch
                    // they entries have the same physical ID and logical ID
                    if current_parent.clone().unwrap().get_id() == virtual_button_id {
                        log::info!(
                            "Virtual button and parent (physical) button have the same ID??? {}",
                            virtual_button_id
                        );
                    }
                }
            }
        }

        Ok(Self {
            map_virtual_button_id_to_parent_physical_buttons,
            map_physical_button_id_to_children_virtual_buttons,
            map_special_buttons,
        })
    }
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

fn parse_b2_button_desc_xml_escaped(desc_xml_escaped: &str) -> Result<PhysicalButton, VkbError> {
    // let lines: Vec<&str> = desc_xml_escaped.split("\r\n").collect();
    // let first_line = lines[0];

    let fragment = Html::parse_fragment(desc_xml_escaped);

    // log::debug!("parse_desc_xml fragment : {:#?}", fragment.tree);
    // for node in fragment.tree.nodes() {
    //     log::debug!("node : {:#?}", node);
    // }

    // the selected inner_html should contain something like:
    // "#1 (E1) ", "#2  - Encoder 2/4", etc
    let b_selector = Selector::parse("b").unwrap();
    let b_nodes: Vec<_> = fragment.select(&b_selector).collect();

    // log::debug!("b_nodes [{}] : {:?}", b_nodes.len(), b_nodes);
    // for b_node in b_nodes.iter() {
    //     log::debug!("b_node : inner_html : {:#?}", b_node.inner_html());
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

    let button = PhysicalButton::new(
        button_id_info.id,
        kind,
        button_id_info
            .info
            .expect(format!("MISSING INFO FOR BUTTON {}", button_id_info.id).as_str()),
        remaining_b_node_inner_html,
        "".to_string(),
    );

    Ok(button)
}

fn parse_b3_button_desc_xml_escaped(
    desc_xml_escaped: &str,
    parent_button: &PhysicalButton,
) -> Result<VirtualButton, VkbError> {
    // eg with "<b>#5 </b> Joystick button : #11" ->
    // physical_button_id_str: 5
    // virtual_button_id_str: 11
    let splitted = desc_xml_escaped
        .split("Joystick button : #")
        .collect::<Vec<_>>();
    let physical_button_id_str = splitted[0].split("<b>#").collect::<Vec<_>>()[1]
        .split(" </b>")
        .collect::<Vec<_>>()[0]
        .trim();
    let virtual_button_id_str = splitted[1].trim();

    let physical_button_id: u8 = physical_button_id_str
        .parse()
        .map_err(|err| VkbError::ParseIntError { err })?;
    let virtual_button_id: u8 = virtual_button_id_str
        .parse()
        .map_err(|err| VkbError::ParseIntError { err })?;

    let button = VirtualButton {
        id: virtual_button_id,
        kind: match parent_button.get_kind() {
            PhysicalButtonKind::Momentary { shift } => match shift {
                Some(shift_kind) => match shift_kind {
                    ShiftKind::Shift1 { button_id_shift1 } => {
                        if &virtual_button_id == button_id_shift1 {
                            VirtualButtonKind::Momentary(Some(VirtualShiftKind::Shift1))
                        } else if &virtual_button_id == parent_button.get_id() {
                            VirtualButtonKind::Momentary(None)
                        } else {
                            unimplemented!(
                                "Physical parent button DOES NOT match {virtual_button_id}"
                            );
                        }
                    }
                    ShiftKind::Shift2 { button_id_shift2 } => {
                        if &virtual_button_id == button_id_shift2 {
                            VirtualButtonKind::Momentary(Some(VirtualShiftKind::Shift2))
                        } else if &virtual_button_id == parent_button.get_id() {
                            VirtualButtonKind::Momentary(None)
                        } else {
                            unimplemented!(
                                "Physical parent button DOES NOT match {virtual_button_id}"
                            );
                        }
                    }
                    ShiftKind::Shift12 {
                        button_id_shift1,
                        button_id_shift2,
                    } => {
                        if &virtual_button_id == button_id_shift1 {
                            VirtualButtonKind::Momentary(Some(VirtualShiftKind::Shift1))
                        } else if &virtual_button_id == button_id_shift2 {
                            VirtualButtonKind::Momentary(Some(VirtualShiftKind::Shift2))
                        } else if &virtual_button_id == parent_button.get_id() {
                            VirtualButtonKind::Momentary(None)
                        } else if &physical_button_id == parent_button.get_id() {
                            VirtualButtonKind::Momentary(None)
                        } else if &physical_button_id == button_id_shift1 {
                            VirtualButtonKind::Momentary(Some(VirtualShiftKind::Shift1))
                        } else if &physical_button_id == button_id_shift2 {
                            VirtualButtonKind::Momentary(Some(VirtualShiftKind::Shift2))
                        } else {
                            unimplemented!(
                                "Physical parent button DOES NOT match {virtual_button_id}"
                            );
                        }
                    }
                },
                None => VirtualButtonKind::Momentary(None),
            },
            PhysicalButtonKind::Encoder => VirtualButtonKind::Momentary(None),
            PhysicalButtonKind::Tempo(tempo) => match tempo {
                TempoKind::_Tempo1 => {
                    unimplemented!("Physical parent button SHOULD NOT be _Tempo1")
                }
                TempoKind::Tempo2 {
                    button_id_short,
                    button_id_long,
                } => {
                    if &virtual_button_id == button_id_short {
                        VirtualButtonKind::Tempo(VirtualTempoKind::Short)
                    } else if &virtual_button_id == button_id_long {
                        VirtualButtonKind::Tempo(VirtualTempoKind::Long)
                    } else if &virtual_button_id == parent_button.get_id() {
                        VirtualButtonKind::Momentary(None)
                    } else {
                        unimplemented!("Physical parent button DOES NOT match {virtual_button_id}");
                    }
                }
                TempoKind::Tempo3 {
                    button_id_short,
                    button_id_long,
                    button_id_double,
                } => {
                    if &virtual_button_id == button_id_short {
                        VirtualButtonKind::Tempo(VirtualTempoKind::Short)
                    } else if &virtual_button_id == button_id_long {
                        VirtualButtonKind::Tempo(VirtualTempoKind::Long)
                    } else if &virtual_button_id == button_id_double {
                        VirtualButtonKind::Tempo(VirtualTempoKind::Double)
                    } else if &virtual_button_id == parent_button.get_id() {
                        VirtualButtonKind::Momentary(None)
                    } else if &physical_button_id == button_id_short {
                        VirtualButtonKind::Tempo(VirtualTempoKind::Short)
                    } else {
                        unimplemented!("Physical parent button DOES NOT match {virtual_button_id}");
                    }
                }
            },
            PhysicalButtonKind::Shift1 => {
                unimplemented!("Physical parent button SHOULD NOT be Shift1")
            }
            PhysicalButtonKind::Shift2 => {
                unimplemented!("Physical parent button SHOULD NOT be Shift2")
            }
            PhysicalButtonKind::Pov { direction: _ } => {
                unimplemented!("Physical parent button SHOULD NOT be Pov")
            }
            PhysicalButtonKind::Undefined => {
                unimplemented!("Physical parent button SHOULD NOT be Undefined")
            }
            PhysicalButtonKind::MicrostickModeSwitch => {
                unimplemented!("Physical parent button SHOULD NOT be MicrostickModeSwitch")
            }
        },
    };

    Ok(button)
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
                        kind: PhysicalButtonKind::Encoder
                        , info: "(E1)".to_string(), extended_desc: "#2  - Encoder 2/4".to_string(), user_desc: "".to_string(),
                    }
                },
            ),
            (
                "<b>#3 (E2) </b><b>- Button with momentary action</b>",
                Button {
                    kind: ButtonKind::Physical {
                        id: 3,
                        kind: PhysicalButtonKind::Momentary{ shift: None },
                        info: "(E2)".to_string(), extended_desc: "- Button with momentary action".to_string(), user_desc: "".to_string(),
                    }
                },
            ),
            (
                "<b>#4 </b><b>- Button with momentary action</b>",
                Button {
                    kind: ButtonKind::Physical {
                        id: 4,
                        kind: PhysicalButtonKind::Momentary{ shift: None }, info: "".to_string(), extended_desc: "- Button with momentary action".to_string(), user_desc: "".to_string(),
                    }
                },
            ),
            (
                "<b>#5 (F3) </b><b>TEMPO </b>\r\nVirtual button Short #5\r\nVirtual button Long #94",
                Button {
                    kind: ButtonKind::Physical {
                        id: 5,
                        kind: PhysicalButtonKind::Tempo(TempoKind::Tempo2 { button_id_short: 5, button_id_long: 94 }), info: "(F3)".to_string(), extended_desc: "TEMPO ".to_string(), user_desc: "".to_string(),
                    }
                },
            ),
            (
                "<b>#5 (F3) </b><b>TEMPO </b>\r\nVirtual button Short #5\r\nVirtual button Long #94\r\nVirtual button Double Short #95",
                Button {
                    kind: ButtonKind::Physical {
                        id: 5,
                        kind: PhysicalButtonKind::Tempo(TempoKind::Tempo3 { button_id_short: 5, button_id_long: 94, button_id_double: 95 } ), info: "(F3)".to_string(), extended_desc: "TEMPO ".to_string(), user_desc: "".to_string(),
                    }
                },
            ),
            (
                "<b>#9 (Fire 2-nd stage) </b><b>- Button with momentary action</b>",
                Button {
                    kind: ButtonKind::Physical {
                        id: 9,
                        kind: PhysicalButtonKind::Momentary { shift: None }, info: "(Fire 2-nd stage)".to_string(), extended_desc: "- Button with momentary action".to_string(), user_desc: "".to_string(),
                    }
                },
            ),
            (
                "<b>#10 (Fire 1-st stage) </b><b>- Button with momentary action</b>\r\nVirtual button with SHIFT1 = 64\r\nVirtual button with SHIFT2 = 91",
                Button {
                    kind: ButtonKind::Physical {
                        id: 10,
                        kind: PhysicalButtonKind::Momentary { shift: Some(ShiftKind::Shift12 { button_id_shift1: 64, button_id_shift2: 91 }) }, info: "(Fire 1-st stage)".to_string(), extended_desc: "- Button with momentary action".to_string(), user_desc: "".to_string(),
                    }
                },
            ),
            (
                "<b>#11 (D1) </b><b> SHIFT1 </b>",
                Button {
                    kind: ButtonKind::Physical {
                        id: 11,
                        kind: PhysicalButtonKind::Shift1, info: "(D1)".to_string(), extended_desc: " SHIFT1 ".to_string(), user_desc: "".to_string(),
                    }
                },
            ),
            (
                "<b>#12 (A2) </b><b>- Button with momentary action</b>\r\nVirtual button with SHIFT1 = 13\r\nVirtual button with SHIFT2 = 90",
                Button {
                    kind: ButtonKind::Physical {
                        id: 12,
                        kind: PhysicalButtonKind::Momentary { shift: Some(ShiftKind::Shift12 { button_id_shift1: 13, button_id_shift2: 90 }) }, info: "(A2)".to_string(), extended_desc: "- Button with momentary action".to_string(), user_desc: "".to_string()
                    }
                },
            ),
            (
                "<b>#18 (A1 down) </b> <b>Point of view Switch</b> POV1  Down",
                Button {
                    kind: ButtonKind::Physical {
                        id: 18,
                        kind: PhysicalButtonKind::Pov { direction: "Down".to_string() }, info: "(A1 down)".to_string(), extended_desc: "Point of view Switch".to_string(), user_desc: "".to_string(),
                    }
                },
            ),
            (
                "<b>#35 (Rapid fire forward) </b><b>- Button with momentary action</b>\r\nVirtual button with SHIFT1 = 37",
                Button {
                    kind: ButtonKind::Physical {
                        id: 35,
                        kind: PhysicalButtonKind::Momentary { shift: Some(ShiftKind::Shift1 { button_id_shift1: 37 }) }, info: "(Rapid fire forward)".to_string(), extended_desc: "- Button with momentary action".to_string(), user_desc: "".to_string(),
                    },
                },
            ),
            (
                "<b>#37 </b><b> No defined function</b>",
                Button {
                    kind: ButtonKind::Physical {
                        id: 37,
                        kind: PhysicalButtonKind::Undefined
                        , info: "".to_string(), extended_desc: " No defined function".to_string(), user_desc: "".to_string(),
                    }
                },
            ),
            (
                "<b>#9 (Fire 2-nd stage) </b><b>- Button with momentary action</b>\r\nVirtual button with SHIFT1 = 63\r\nVirtual button with SHIFT2 = 92",
                Button {
                    kind: ButtonKind::Physical {
                        id: 9,
                        kind: PhysicalButtonKind::Momentary { shift: Some(ShiftKind::Shift12 { button_id_shift1: 63, button_id_shift2: 92 }) }, info: "(Fire 2-nd stage)".to_string(), extended_desc: "- Button with momentary action".to_string(), user_desc: "".to_string(),
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
        let test_inputs_vs_expected_results = vec![
            (
                r#"<b>#61 </b> Joystick button : #61"#,
                Button {
                    kind: ButtonKind::Virtual { id: 61 },
                },
            ),
            (
                // REALLY IMPORTANT to have proper duplication detection:
                // in this case the game WILL see "button 53" NOT "button 7"
                r#"<b>#7 </b> Joystick button : #53"#,
                Button {
                    kind: ButtonKind::Virtual { id: 53 },
                },
            ),
        ];

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

        assert!(JoystickButtonsMapping::try_from(vkb_report).is_ok());
    }

    #[test]
    fn test_button_map_vkb_report_r() {
        let vkb_report = VkbReport::new(
            concat!(env!("CARGO_MANIFEST_DIR"), "/bindings/vkb_report_R.fp3").into(),
        )
        .unwrap();

        assert!(JoystickButtonsMapping::try_from(vkb_report).is_ok());
    }

    #[test]
    fn test_button_map_vkb_report_l() {
        let vkb_report = VkbReport::new(
            concat!(env!("CARGO_MANIFEST_DIR"), "/bindings/vkb_report_L.fp3").into(),
        )
        .unwrap();

        assert!(JoystickButtonsMapping::try_from(vkb_report).is_ok());
    }
}
