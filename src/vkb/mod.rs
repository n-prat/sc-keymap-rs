use std::path::PathBuf;

use crate::Error;

use self::vkb_button::JoystickButtonsMapping;

pub(crate) mod vkb_button;
mod vkb_xml;

fn parse_report(xml_path: PathBuf) -> Result<vkb_xml::VkbReport, Error> {
    let vkb_report = vkb_xml::VkbReport::new(xml_path)?;

    Ok(vkb_report)
}

fn check_report(
    vkb_report: vkb_xml::VkbReport,
    vkb_user_provided_data: Option<csv::Reader<std::fs::File>>,
) -> Result<vkb_button::JoystickButtonsMapping, Error> {
    let mut vkb_buttons = vkb_button::JoystickButtonsMapping::try_from(vkb_report)?;

    if let Some(vkb_user_provided_data) = vkb_user_provided_data {
        vkb_buttons.inject_user_provided_desc(vkb_user_provided_data)?;
    }

    vkb_buttons.log_free_virtual_buttons();

    Ok(vkb_buttons)
}

/// Parse and process both the L and R sticks
///
/// # Errors
/// - `Error::Csv` if the csv path could not be read
/// - if `stick_fp3_report_path` could not be parsed by `parse_report`
/// - if `stick_fp3_report_path` failed at `check_report`
///   NOTE: for now we are only logging the errors/duplicated buttons etc but that MAY change
///
pub fn parse_and_check_vkb_both_sticks(
    stick_fp3_report_path: PathBuf,
    vkb_user_provided_data_path: &Option<PathBuf>,
) -> Result<JoystickButtonsMapping, Error> {
    let vkb_user_provided_data = match vkb_user_provided_data_path {
        Some(ref vkb_user_provided_data_path) => {
            let rdr = csv::Reader::from_path(vkb_user_provided_data_path).map_err(Error::Csv)?;
            Some(rdr)
        }
        None => None,
    };

    let vkb_report = parse_report(stick_fp3_report_path)?;
    log::info!("vkb_report : {:#?}", vkb_report);

    let vkb_mappings = check_report(vkb_report, vkb_user_provided_data)?;
    log::info!("vkb_buttons : {:#?}", vkb_mappings);

    Ok(vkb_mappings)
}

#[cfg(test)]
mod tests {
    use std::collections::hash_map;

    use crate::button::{
        PhysicalButton, PhysicalButtonKind, ShiftKind, VirtualButton, VirtualButtonKind,
        VirtualButtonOrSpecial,
    };

    use super::*;

    fn get_sample_mappings() -> JoystickButtonsMapping {
        // for simplicity both L and R sticks use the same config
        let config = JoystickButtonsMapping {
            map_virtual_button_id_to_parent_physical_buttons: hash_map::HashMap::from([
                // Most basic case: a standard button, no user-injected data (so user_desc is empty)
                (
                    44,
                    vec![PhysicalButton::new(
                        27,
                        PhysicalButtonKind::Momentary {
                            shift: Some(ShiftKind::Shift12 {
                                button_id_shift1: 44,
                                button_id_shift2: 82,
                            }),
                        },
                        "(A4 left)".to_string(),
                        "- Button with momentary action".to_string(),
                        "".to_string(),
                    )],
                ),
                // More advanced case: the key (108) is NOT found in PhysicalButtonKind
                // Here we have both "info" and "user_desc" -> "user_desc" is ignored
                (
                    108,
                    vec![PhysicalButton::new(
                        12,
                        PhysicalButtonKind::Momentary {
                            shift: Some(ShiftKind::Shift12 {
                                button_id_shift1: 100,
                                button_id_shift2: 101,
                            }),
                        },
                        "(A2)".to_string(),
                        "- Button with momentary action".to_string(),
                        "Thumb red button".to_string(),
                    )],
                ),
                // Special case: 8-ways POV stick
                // It is a toggle in VKB config so here we have NO link from physical in VkbDevCfg to physical button
                // eg Physical button 72 parent's == Physical button 72
                // Here we SKIP "info" and only map using "user_desc"
                (
                    72,
                    vec![PhysicalButton::new(
                        72,
                        PhysicalButtonKind::Momentary {
                            shift: Some(ShiftKind::Shift2 {
                                button_id_shift2: 80,
                            }),
                        },
                        "".to_string(),
                        "- Button with momentary action".to_string(),
                        "A1 8-way ministick NW".to_string(),
                    )],
                ),
            ]),
            map_physical_button_id_to_children_virtual_buttons: hash_map::HashMap::from([
                (
                    27,
                    vec![VirtualButton {
                        id: 44,
                        kind: VirtualButtonKind::Momentary(None),
                    }],
                ),
                (
                    12,
                    vec![VirtualButton {
                        id: 108,
                        kind: VirtualButtonKind::Momentary(None),
                    }],
                ),
                (
                    72,
                    vec![VirtualButton {
                        id: 72,
                        kind: VirtualButtonKind::Momentary(None),
                    }],
                ),
            ]),
            map_special_buttons: hash_map::HashMap::new(),
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

        config
    }

    #[test]
    fn test_get_virtual_button_id_from_info_or_user_desc_simple() {
        let sample_mappings = get_sample_mappings();

        assert_eq!(
            sample_mappings
                .get_virtual_button_ids_from_info_or_user_desc("(A4 left)")
                .unwrap(),
            vec![VirtualButtonOrSpecial::Virtual(VirtualButton {
                id: 44,
                kind: VirtualButtonKind::Momentary(None)
            })]
        );
    }

    #[test]
    fn test_get_virtual_button_id_from_info_or_user_desc_advanced() {
        let sample_mappings = get_sample_mappings();

        assert_eq!(
            sample_mappings
                .get_virtual_button_ids_from_info_or_user_desc("(A2)")
                .unwrap(),
            vec![VirtualButtonOrSpecial::Virtual(VirtualButton {
                id: 108,
                kind: VirtualButtonKind::Momentary(None)
            })]
        );
    }

    #[test]
    fn test_get_virtual_button_id_from_info_or_user_desc_special() {
        let sample_mappings = get_sample_mappings();

        assert_eq!(
            sample_mappings
                .get_virtual_button_ids_from_info_or_user_desc("A1 8-way ministick NW")
                .unwrap(),
            vec![VirtualButtonOrSpecial::Virtual(VirtualButton {
                id: 72,
                kind: VirtualButtonKind::Momentary(None)
            })]
        );
    }
}
