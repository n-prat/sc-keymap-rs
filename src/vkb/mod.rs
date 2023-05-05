use std::path::PathBuf;

use thiserror::Error;

mod vkb_button;
mod vkb_xml;

pub(super) struct VkbBindings {}

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
    let vkb_report = vkb_xml::parse_report_xml(xml_path)?;

    Ok(VkbBindings {})
}
