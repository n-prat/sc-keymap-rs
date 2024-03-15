// #![cfg_attr(not(feature = "std"), no_std)]
#![deny(elided_lifetimes_in_paths)]
#![warn(clippy::suspicious)]
#![warn(clippy::complexity)]
#![warn(clippy::perf)]
#![warn(clippy::style)]
#![warn(clippy::pedantic)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::unwrap_used)]

use quick_xml::DeError;
use std::num::{ParseIntError, TryFromIntError};
use thiserror::Error;

mod button;
mod html_gen;
mod sc;
mod template_gen;
mod vkb;

#[derive(Error, Debug)]
pub enum Error {
    #[error("error : `{0}`")]
    Other(String),
    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader { expected: String, found: String },
    #[error("unknown xml error")]
    Unknown,
    #[error("the xml desc `{0}` is not handled")]
    UnexpectedXmlDesc(String),
    #[error("MISSING INFO FOR BUTTON `{0}`")]
    MissingXmlInfo(u8),
    #[error("generic xml parsing error : `{0}`")]
    OtherXmlParsingError(String),
    #[error("could not parse to integer `{0}`")]
    ParseIntError(ParseIntError),
    #[error("could not convert to integer `{0}`")]
    TryFromIntError(TryFromIntError),
    #[error("could not find info_or_user_desc : `{info_or_user_desc}`")]
    ButtonNotFound { info_or_user_desc: String },
    #[error("could not read csv file : `{0}`")]
    Csv(csv::Error),
    #[error("read error")]
    ReadError { err: std::io::Error },
    #[error("deserialization error")]
    DeError { err: DeError },
}

/// Re-export
pub use html_gen::generate_html;
pub use sc::parse_keybind_xml::parse_keybind as sc_parse_keybind;
pub use template_gen::generate_template;
pub use vkb::parse_and_check_vkb_both_sticks as vkb_parse_and_check_both_sticks;
