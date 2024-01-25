use std::{num::ParseIntError, path::PathBuf};

use thiserror::Error;

pub mod vkb_button;
mod vkb_xml;

#[derive(Debug)]
pub struct VkbBindings {
    vkb_report: vkb_xml::VkbReport,
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
}

pub fn parse_report(xml_path: PathBuf) -> Result<VkbBindings, VkbError> {
    let vkb_report = vkb_xml::VkbReport::new(xml_path)?;

    Ok(VkbBindings { vkb_report })
}

pub fn check_report(
    vkb_binding: VkbBindings,
    vkb_user_provided_data: Option<csv::Reader<std::fs::File>>,
) -> vkb_button::JoystickButtonsMapping {
    let mut vkb_buttons =
        vkb_button::JoystickButtonsMapping::try_from(vkb_binding.vkb_report).unwrap();

    match vkb_user_provided_data {
        Some(vkb_user_provided_data) => {
            vkb_buttons.inject_user_provided_desc(vkb_user_provided_data)
        }
        None => {}
    }

    vkb_buttons
}
