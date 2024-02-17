//TODO see also https://github.com/tafia/quick-xml/blob/master/examples/read_nodes_serde.rs

use std::{collections::HashMap, path::PathBuf};

use serde::Deserialize;

use crate::Error;

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
/// using the above "`XmlRebindInput`"
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
    /// - "`js1_button{virtual_button_id`}"
    /// - "`js2_button{virtual_button_id`}"
    ///
    /// `joystick_id` SHOULD be either "1" or "2"
    /// or more exactly it MUST match the number/ID of "<options type="joystick" instance=" defined in
    /// "`layout_AAA_exported.xml`"
    ///
    pub fn get_action_from_virtual_button_id(
        &self,
        virtual_button_id: u8,
        joystick_id: u8,
    ) -> Option<&Vec<String>> {
        match self
            .map_virtual_button_to_actions
            .get(&format!("js{joystick_id}_button{virtual_button_id}"))
        {
            Some(actions_names) => Some(actions_names),
            None => None,
        }
    }
}

/// Each key,value in the csv will be in the result Vec:
/// - (key,value)
/// That way when checking with `contains` the order does not matter.
///
/// (Yes, the proper way would be to use a map/set)
///
fn parse_csv_binding_pairs_to_ignore(
    sc_bindings_to_ignore: Option<csv::Reader<std::fs::File>>,
) -> Result<Vec<(String, String)>, Error> {
    let mut binding_pairs_to_ignore = vec![];

    if let Some(sc_bindings_to_ignore) = sc_bindings_to_ignore {
        for record in sc_bindings_to_ignore.into_records() {
            let record = record.map_err(|_err| Error::Other("record missing?".to_string()))?;
            let left = record
                .get(0)
                .ok_or_else(|| Error::Other("record: could not get column 0".to_string()))?;
            let right = record
                .get(1)
                .ok_or_else(|| Error::Other("record: could not get column 1".to_string()))?;

            binding_pairs_to_ignore.push((left.to_string(), right.to_string()));
        }
    };

    Ok(binding_pairs_to_ignore)
}

/// Parse a Star Citizen keybinds, and optionally ignore warnings related to user-given keybinds pairs
///
/// # Errors
///
pub fn parse_keybind(
    xml_path: PathBuf,
    sc_bindings_to_ignore: Option<csv::Reader<std::fs::File>>,
) -> Result<GameButtonsMapping, Error> {
    let binding_pairs_to_ignore = parse_csv_binding_pairs_to_ignore(sc_bindings_to_ignore)?;

    let xml_str = std::fs::read_to_string(xml_path).map_err(|err| Error::ReadError { err })?;

    let xml_data: XmlFull =
        quick_xml::de::from_str(&xml_str).map_err(|err| Error::DeError { err })?;

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
            if all_joystick_keybinds.is_empty() {
                log::info!("[sc] parse_keybind: NO key for \"{action_name}\"");
                continue;
            }

            let logical_button_name = &all_joystick_keybinds[0].input;

            // Finally; sometimes the bind is just empty
            // <rebind input="js2_ " />
            // -> skip
            if logical_button_name
                .split('_')
                .last()
                .ok_or_else(|| {
                    Error::Other("logical_button_name unexpected number of _".to_string())
                })?
                .trim()
                .is_empty()
            {
                log::info!("[sc] parse_keybind: empty key for \"{action_name}\" = \"{logical_button_name}\"");
                continue;
            }

            // insert a new vec if needed
            map_virtual_button_to_actions
                .entry(logical_button_name.clone())
                .or_insert(vec![]);

            if let Some(actions) = map_virtual_button_to_actions.get_mut(logical_button_name) {
                // update the bindings EVEN if duplicated
                // we still WANT to print them in the final template!
                actions.push(action_name.clone());

                // SHORTCUT if there is still only one action: we can stop now
                if actions.len() < 2 {
                    continue;
                }

                // first pair: (0, 1)
                let new_pair1 = (
                    actions
                        .first()
                        .ok_or_else(|| Error::Other("actions is empty".to_string()))?
                        .to_string(),
                    action_name.to_string(),
                );
                // same pair but inverted (1, 0)
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
            }
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
    //                 .map_err(|_| Error::Unknown)?;
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
            <CustomisationUIHeader label="vkb" description="" image="">
            </CustomisationUIHeader>
            <deviceoptions name=" VKBsim Gladiator EVO  L    {0201231D-0000-0000-0000-504944564944}">
                <option input="x" deadzone="0.0198" />
            </deviceoptions>
            <options type="keyboard" instance="1" Product="Keyboard  {6F1D2B61-D5A0-11CF-BFC7-444553540000}" />
            <modifiers />
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
