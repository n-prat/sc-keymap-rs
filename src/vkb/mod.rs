use std::{num::ParseIntError, path::PathBuf};

use thiserror::Error;

use self::vkb_button::JoystickButtonsMapping;

pub mod vkb_button;
mod vkb_xml;

#[derive(Debug)]
pub struct VkbBothSticksMappings {
    vkb_mappings1: JoystickButtonsMapping,
    vkb_mappings2: JoystickButtonsMapping,
}

impl VkbBothSticksMappings {
    pub fn get_first(&self) -> &JoystickButtonsMapping {
        &self.vkb_mappings1
    }

    pub fn get_second(&self) -> &JoystickButtonsMapping {
        &self.vkb_mappings2
    }

    /// Let's say info_or_user_desc = "A1 8-way ministick N" or "(A2)"
    /// We want to return the corresponding VIRTUAL BUTTON IDS (plural!)
    /// That way when a loop in the game binding, we can easily get the corresponding label from it eg "deploy landing gear" etc
    ///
    /// We are looking for a VIRTUAL BUTTON (ID) whose PARENT (PHYSICAL) BUTTON
    /// has "info" == `info_or_user_desc` or "user_desc" == `info_or_user_desc`
    ///
    // TODO is this OK? should it return a Vec? Add more tests with STD+SHIFT1+SHIFT2 from real bininds
    // and check
    pub fn get_virtual_button_ids_from_info_or_user_desc(
        &self,
        info_or_user_desc: &str,
        use_second_stick: bool,
    ) -> Result<Vec<u8>, VkbError> {
        let stick = match use_second_stick {
            true => &self.vkb_mappings1,
            false => &self.vkb_mappings2,
        };

        let mut found_physical_button_id: Option<u8> = None;

        // First: loop for the target PHYSICAL button; cf docstring
        for (virtual_button_id, parent_physical_buttons) in stick
            .map_virtual_button_id_to_parent_physical_buttons
            .iter()
        {
            for parent_physical_button in parent_physical_buttons {
                match &parent_physical_button.kind {
                    crate::button::ButtonKind::Physical {
                        id: _,
                        kind: _,
                        info,
                        extended_desc: _,
                        user_desc,
                    } => {
                        if info_or_user_desc == info || info_or_user_desc == user_desc {
                            found_physical_button_id = Some(parent_physical_button.get_id());
                            break;
                        }
                    }
                    crate::button::ButtonKind::Virtual { id } => {
                        unimplemented!("parent SHOULD be physical NOT virtual button!")
                    }
                }
            }
        }

        // Next we MUST get ALL the children VIRTUAL buttons
        match found_physical_button_id {
            Some(found_physical_button_id) => Ok(stick
                .map_physical_button_id_to_children_virtual_button_ids
                .get(&found_physical_button_id).expect("physical button ID not found in map_physical_button_id_to_children_virtual_button_ids!").clone()),
            None => Err(VkbError::ButtonNotFound {
                info_or_user_desc: info_or_user_desc.to_string(),
            }),
        }
    }
}

#[derive(Error, Debug)]
pub enum VkbError {
    #[error("the data for key `{0}` is not available")]
    Redaction(String),
    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader { expected: String, found: String },
    #[error("unknown xml error")]
    Unknown,
    #[error("the xml desc `{0}` is not handled")]
    UnexpectedXmlDesc(String),
    #[error("could not parse to integer `{err}`")]
    ParseIntError { err: ParseIntError },
    #[error("could not find info_or_user_desc : `{info_or_user_desc}`")]
    ButtonNotFound { info_or_user_desc: String },
}

fn parse_report(xml_path: PathBuf) -> Result<vkb_xml::VkbReport, VkbError> {
    let vkb_report = vkb_xml::VkbReport::new(xml_path)?;

    Ok(vkb_report)
}

fn check_report(
    vkb_report: vkb_xml::VkbReport,
    vkb_user_provided_data: Option<csv::Reader<std::fs::File>>,
) -> vkb_button::JoystickButtonsMapping {
    let mut vkb_buttons = vkb_button::JoystickButtonsMapping::try_from(vkb_report).unwrap();

    match vkb_user_provided_data {
        Some(vkb_user_provided_data) => {
            vkb_buttons.inject_user_provided_desc(vkb_user_provided_data)
        }
        None => {}
    }

    vkb_buttons
}

/// Parse and process both the L and R sticks
pub fn parse_and_check_vkb_both_sticks(
    stick1_fp3_report_path: PathBuf,
    stick2_fp3_report_path: PathBuf,
    vkb_user_provided_data_path: Option<PathBuf>,
) -> Result<VkbBothSticksMappings, VkbError> {
    let vkb_user_provided_data1 = match vkb_user_provided_data_path {
        Some(ref vkb_user_provided_data_path) => {
            let rdr = csv::Reader::from_path(vkb_user_provided_data_path).unwrap();
            Some(rdr)
        }
        None => None,
    };
    let vkb_user_provided_data2 = match vkb_user_provided_data_path {
        Some(ref vkb_user_provided_data_path) => {
            let rdr = csv::Reader::from_path(vkb_user_provided_data_path).unwrap();
            Some(rdr)
        }
        None => None,
    };

    let vkb_report1 = parse_report(stick1_fp3_report_path).unwrap();
    log::info!("vkb_report : {:#?}", vkb_report1);

    let vkb_mappings1 = check_report(vkb_report1, vkb_user_provided_data1);
    log::info!("vkb_buttons : {:#?}", vkb_mappings1);

    let vkb_report2 = parse_report(stick2_fp3_report_path).unwrap();
    log::info!("vkb_report : {:#?}", vkb_report2);

    let vkb_mappings2 = check_report(vkb_report2, vkb_user_provided_data2);
    log::info!("vkb_buttons : {:#?}", vkb_mappings2);

    Ok(VkbBothSticksMappings {
        vkb_mappings1,
        vkb_mappings2,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::hash_map;

    use crate::button::{Button, PhysicalButtonKind, ShiftKind};

    use super::*;

    fn get_sample_mappings() -> VkbBothSticksMappings {
        // for simplicity both L and R sticks use the same config
        let config = JoystickButtonsMapping {
            map_virtual_button_id_to_parent_physical_buttons: hash_map::HashMap::from([
                // Most basic case: a standard button, no user-injected data (so user_desc is empty)
                (
                    44,
                    vec![Button {
                        kind: crate::button::ButtonKind::Physical {
                            id: 27,
                            kind: PhysicalButtonKind::Momentary {
                                shift: Some(ShiftKind::Shift12 {
                                    button_id_shift1: 44,
                                    button_id_shift2: 82,
                                }),
                            },
                            info: "(A4 left)".to_string(),
                            extended_desc: "- Button with momentary action".to_string(),
                            user_desc: "".to_string(),
                        },
                    }],
                ),
                // More advanced case: the key (108) is NOT found in PhysicalButtonKind
                // Here we have both "info" and "user_desc" -> "user_desc" is ignored
                (
                    108,
                    vec![Button {
                        kind: crate::button::ButtonKind::Physical {
                            id: 12,
                            kind: PhysicalButtonKind::Momentary {
                                shift: Some(ShiftKind::Shift12 {
                                    button_id_shift1: 100,
                                    button_id_shift2: 101,
                                }),
                            },
                            info: "(A2)".to_string(),
                            extended_desc: "- Button with momentary action".to_string(),
                            user_desc: "Thumb red button".to_string(),
                        },
                    }],
                ),
                // Special case: 8-ways POV stick
                // It is a toggle in VKB config so here we have NO link from physical in VkbDevCfg to physical button
                // eg Physical button 72 parent's == Physical button 72
                // Here we SKIP "info" and only map using "user_desc"
                (
                    72,
                    vec![Button {
                        kind: crate::button::ButtonKind::Physical {
                            id: 72,
                            kind: PhysicalButtonKind::Momentary {
                                shift: Some(ShiftKind::Shift2 {
                                    button_id_shift2: 80,
                                }),
                            },
                            info: "".to_string(),
                            extended_desc: "- Button with momentary action".to_string(),
                            user_desc: "A1 8-way ministick NW".to_string(),
                        },
                    }],
                ),
            ]),
            map_physical_button_id_to_children_virtual_button_ids: hash_map::HashMap::from([
                (27, vec![44]),
                (12, vec![108]),
                (72, vec![72]),
            ]),
            // physical_buttons_with_desc: vec![
            //     PhysicalButtonWithDesc {
            //         id: 27,
            //         info: "(A4 left)".to_string(),
            //         extended_desc: "- Button with momentary action".to_string(),
            //         user_desc: "".to_string(),
            //     },
            //     PhysicalButtonWithDesc {
            //         id: 12,
            //         info: "(A2)".to_string(),
            //         extended_desc: "- Button with momentary action".to_string(),
            //         user_desc: "Thumb red button".to_string(),
            //     },
            //     PhysicalButtonWithDesc {
            //         id: 72,
            //         info: "".to_string(),
            //         extended_desc: "- Button with momentary action".to_string(),
            //         user_desc: "A1 8-way ministick NW".to_string(),
            //     },
            // ],
        };

        VkbBothSticksMappings {
            vkb_mappings1: config.clone(),
            vkb_mappings2: config,
        }
    }

    #[test]
    fn test_get_virtual_button_id_from_info_or_user_desc_simple() {
        let sample_mappings = get_sample_mappings();

        assert_eq!(
            sample_mappings
                .get_virtual_button_ids_from_info_or_user_desc("(A4 left)", false)
                .unwrap(),
            44
        );
    }

    #[test]
    fn test_get_virtual_button_id_from_info_or_user_desc_advanced() {
        let sample_mappings = get_sample_mappings();

        assert_eq!(
            sample_mappings
                .get_virtual_button_ids_from_info_or_user_desc("(A2)", false)
                .unwrap(),
            108
        );
    }

    #[test]
    fn test_get_virtual_button_id_from_info_or_user_desc_special() {
        let sample_mappings = get_sample_mappings();

        assert_eq!(
            sample_mappings
                .get_virtual_button_ids_from_info_or_user_desc("A1 8-way ministick NW", false)
                .unwrap(),
            72
        );
    }
}
