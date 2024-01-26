//TODO see also https://github.com/tafia/quick-xml/blob/master/examples/read_nodes_serde.rs

use std::{collections::HashMap, path::PathBuf};

use quick_xml::DeError;
use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KeybindError {
    #[error("the data for key `{0}` is not available")]
    Redaction(String),
    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader { expected: String, found: String },
    #[error("read error")]
    ReadError { err: std::io::Error },
    #[error("deserialization error")]
    DeError { err: DeError },
}

/// Maps eg "<rebind input="js1_button2"/>"
#[derive(Deserialize, Debug)]
struct XmlRebindInput {
    #[serde(rename = "@input")]
    input: String,
}

/// Maps eg
/// <action name="v_weapon_toggle_launch_missile">
///     <rebind input="js1_button2"/>
/// </action>
///
/// using the above "XmlRebindInput"
///
/// NOTE apparently sometimes we can have two rebinds???
/// ```xml
///    <action name="v_capacitor_assignment_engine_combined_increase_max">
///        <rebind input="kb1_ " />
///        <rebind input="js2_ " />
///    </action>
/// ```
#[derive(Deserialize, Debug)]
struct XmlActionName {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "rebind")]
    rebind: Vec<XmlRebindInput>,
}

/// Maps eg
///
/// <actionmap name="spaceship_missiles">
///     <action name="v_weapon_toggle_launch_missile">
///         <rebind input="js1_button2"/>
///     </action>
/// </actionmap>
///
/// using all the above Xml*
#[derive(Deserialize, Debug)]
struct XmlActionMap {
    #[serde(rename = "@name")]
    _name: String,
    #[serde(rename = "$value")]
    action: Vec<XmlActionName>,
}

#[derive(Deserialize, Debug)]
struct XmlCustomisationUIHeader {}

#[derive(Deserialize, Debug)]
struct XmlDeviceOptions {
    #[serde(rename = "@name")]
    _name: String,
}

#[derive(Deserialize, Debug)]
struct XmlOptions {
    #[serde(rename = "@type")]
    _option_type: String,
    #[serde(rename = "@instance")]
    _instance: String,
    #[serde(rename = "@Product")]
    _product: String,
}

#[derive(Deserialize, Debug)]
struct XmlModifiers {}

#[derive(Deserialize, Debug)]
#[serde(rename = "@ActionMaps")]
struct XmlFull {
    #[serde(rename = "CustomisationUIHeader")]
    _customisation_uiheader: XmlCustomisationUIHeader,
    #[serde(rename = "deviceoptions")]
    _device_options: Vec<XmlDeviceOptions>,
    #[serde(rename = "options")]
    _options: Vec<XmlOptions>,
    #[serde(rename = "modifiers")]
    _modifiers: XmlModifiers,
    #[serde(rename = "actionmap")]
    actionmap: Vec<XmlActionMap>,
}

/// The "game version" of `JoystickButtonsMapping`
///
/// It contains ONLY game keybinds!
#[derive(PartialEq, Debug)]
pub struct GameButtonsMapping {
    /// This set is here to help detect duplicated keybinds
    /// NOTE: this is NOT necessarily en error; sometimes we WANT to have a given action from 2 different buttons
    /// (eg same from left stick and right stick) or maybe the same button X is used for two different things
    /// in "flight mode" vs "driving mode", etc
    /// It could also do two different functions in game based on long/short/double press but we can't see it from
    /// the exported keybinds; eg "v_toggle_quantum_mode" + "v_toggle_qdrive_engagement" are using the same key
    map_virtual_button_to_actions: HashMap<String, Vec<String>>,
}

impl GameButtonsMapping {
    /// [Star Citizen] specific:
    /// CHECK for:
    /// - "js1_button{virtual_button_id}"
    /// - "js2_button{virtual_button_id}"
    pub fn get_action_from_virtual_button_id(&self, virtual_button_id: u8) -> Option<&Vec<String>> {
        match self
            .map_virtual_button_to_actions
            .get(&format!("js1_button{}", virtual_button_id))
        {
            Some(actions_names) => Some(actions_names),
            None => match self
                .map_virtual_button_to_actions
                .get(&format!("js2_button{}", virtual_button_id))
            {
                Some(actions_names) => Some(actions_names),
                None => None,
            },
        }
    }
}

///
pub fn parse_keybind(
    xml_path: PathBuf,
    sc_bindings_to_ignore: Option<csv::Reader<std::fs::File>>,
) -> Result<GameButtonsMapping, KeybindError> {
    let binding_pairs_to_ignore: Vec<(String, String)> = match sc_bindings_to_ignore {
        Some(sc_bindings_to_ignore) => {
            let csv_records = sc_bindings_to_ignore
                .into_records()
                .into_iter()
                .map(|record| {
                    let record = record.unwrap();
                    let left = record.get(0).unwrap();
                    let right = record.get(1).unwrap();

                    (left.to_string(), right.to_string())
                })
                .collect();

            csv_records
        }
        None => {
            vec![]
        }
    };

    let xml_str =
        std::fs::read_to_string(xml_path).map_err(|err| KeybindError::ReadError { err })?;

    let xml_data: XmlFull =
        quick_xml::de::from_str(&xml_str).map_err(|err| KeybindError::DeError { err })?;

    log::debug!("keybinds: {:?}", xml_data);

    let mut map_virtual_button_to_actions = HashMap::new();

    for actionmap in &xml_data.actionmap {
        for action in &actionmap.action {
            let action_name = &action.name;
            // IMPORTANT sometimes even with the JOYSTICK exported keybinds we find eg "<rebind input="kb1_ " />"
            // so just ignore these
            let all_joystick_keybinds: Vec<_> = action
                .rebind
                .iter()
                .filter(|rebind| rebind.input.starts_with("js"))
                .collect();

            if all_joystick_keybinds.len() > 1 {
                log::info!("[sc] parse_keybind: more than one key for \"{action_name}\" : {all_joystick_keybinds:?} ");
            }

            // IMPORTANT sometimes there is ONLY a mouse or keyboard here for some reason...
            // <action name="selectUnarmedCombat">
            //     <rebind input="kb1_o" />
            // </action>
            // -> skip
            if all_joystick_keybinds.len() == 0 {
                log::info!("[sc] parse_keybind: NO key for \"{action_name}\"");
                continue;
            }

            let logical_button_name = &all_joystick_keybinds[0].input;

            // Finally; sometimes the bind is just empty
            // <rebind input="js2_ " />
            // -> skip
            if logical_button_name
                .split("_")
                .last()
                .unwrap()
                .trim()
                .is_empty()
            {
                log::info!("[sc] parse_keybind: empty key for \"{action_name}\" = \"{logical_button_name}\"");
                continue;
            }

            map_virtual_button_to_actions
                .entry(logical_button_name.clone())
                // NOT inserted = the virtual button was already processed!
                .and_modify(|actions: &mut Vec<String>| {
                    // update the bindings EVEN if duplicated
                    // we still WANT to print them in the final template!
                    actions.push(action_name.clone());

                    let new_pair1 = (
                        actions.first().unwrap().to_string(),
                        action_name.to_string(),
                    );
                    let new_pair2 = (new_pair1.1.to_string(), new_pair1.0.to_string());

                    if binding_pairs_to_ignore.contains(&new_pair1)
                        || binding_pairs_to_ignore.contains(&new_pair2)
                    {
                        log::info!("skipping {new_pair1:?}");
                    } else {
                        log::warn!(
                            "keybind duplicated : {logical_button_name} used for : \"{actions:?}\""
                        );
                    }
                })
                // inserted = nothing to do
                .or_insert(vec![action_name.clone()]);
        }
    }

    Ok(GameButtonsMapping {
        map_virtual_button_to_actions,
    })

    //TODO? https://github.com/tafia/quick-xml/blob/9fb797e921d83467c89e78de7de6511801f335b1/examples/read_buffered.rs#L10
    // let mut buf = Vec::new();

    // let mut count = 0;

    // loop {
    //     match reader.read_event_into(&mut buf) {
    //         Ok(Event::Start(ref e)) => {
    //             let name = e.name();
    //             let name = reader
    //                 .decoder()
    //                 .decode(name.as_ref())
    //                 .map_err(|_| KeybindError::Unknown)?;
    //             log::debug!("read start event {:?}", name.as_ref());
    //             count += 1;
    //         }
    //         Ok(Event::Eof) => break, // exits the loop when reaching end of file
    //         Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
    //         _ => (), // There are several other `Event`s we do not consider here
    //     }
    // }

    // log::debug!("read {} start events in total", count);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_xml_rebind_input() {
        let xml_str = r#"<rebind input="js1_button2"/>"#;

        quick_xml::de::from_str::<XmlRebindInput>(xml_str).unwrap();
    }

    #[test]
    fn test_parse_xml_action_name() {
        let xml_str = r#"
        <action name="v_weapon_toggle_launch_missile">
            <rebind input="js1_button2"/>
        </action>
        "#;

        quick_xml::de::from_str::<XmlActionName>(xml_str).unwrap();
    }

    #[test]
    fn test_parse_xml_action_map() {
        let xml_str = r#"
        <actionmap name="spaceship_missiles">
        <action name="v_weapon_toggle_launch_missile">
         <rebind input="js1_button2"/>
        </action>
       </actionmap>
       "#;

        quick_xml::de::from_str::<XmlActionMap>(xml_str).unwrap();
    }

    #[test]
    fn test_parse_xml_action_map_multiple() {
        let xml_str = r#"
        <actionmap name="spaceship_missiles">
        <action name="v_weapon_toggle_launch_missile">
         <rebind input="js1_button2"/>
        </action>
        <action name="foip_pushtotalk_proximity">
         <rebind input="js1_button3"/>
        </action>
       </actionmap>
       "#;

        quick_xml::de::from_str::<XmlActionMap>(xml_str).unwrap();
    }

    #[test]
    fn test_parse_xml_full() {
        let xml_str = r#"
        <ActionMaps version="1" optionsVersion="2" rebindVersion="2" profileName="vkb_custom_v1">
        <actionmap name="player_input_optical_tracking">
            <action name="foip_pushtotalk_proximity">
                <rebind input="kb1_lalt+capslock" />
            </action>
        </actionmap>
    </ActionMaps>
    "#;

        quick_xml::de::from_str::<XmlFull>(xml_str).unwrap();
    }

    #[test]
    fn test_parse_full_sample() {
        let xml_str = include_str!("../../tests/data/layout_exported_simplified.xml");

        quick_xml::de::from_str::<XmlFull>(xml_str).unwrap();
    }

    #[test]
    fn test_parse_keybind_full_exported() {
        parse_keybind(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/bindings/layout_vkb_exported.xml"
            )
            .into(),
            None,
        )
        .unwrap();
    }
}
