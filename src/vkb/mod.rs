use std::path::PathBuf;

use thiserror::Error;

mod vkb_button;
mod vkb_xml;

#[derive(Debug)]
pub(super) struct VkbBindings {
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
}

pub(crate) fn parse_report(xml_path: PathBuf) -> Result<VkbBindings, VkbError> {
    let vkb_report = vkb_xml::VkbReport::new(xml_path)?;

    Ok(VkbBindings { vkb_report })
}

pub(crate) fn check_report(vkb_binding: VkbBindings) -> vkb_button::ButtonMap {
    let vkb_buttons = vkb_button::ButtonMap::try_from(vkb_binding.vkb_report).unwrap();
    vkb_buttons
}
