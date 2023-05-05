use std::collections::HashMap;
use std::env::args;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use pdf::enc::StreamFilter;
use pdf::error::PdfError;
use pdf::file::FileOptions;
use pdf::object::*;
use pdf::primitive::Primitive;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PdfFormError {
    #[error("the data for key `{0}` is not available")]
    Redaction(String),
    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader { expected: String, found: String },
    #[error("unknown pdf form error")]
    Unknown,
}

pub(crate) fn list_forms(pdf_path: &PathBuf) -> Result<(), PdfFormError> {
    let file = FileOptions::cached().open(&pdf_path).unwrap();

    if let Some(ref forms) = file.get_root().forms {
        println!("pdf_path = {}, Forms:", pdf_path.display());
        for field in forms.fields.iter() {
            print!("  {:?} = ", field.name);
            match field.value {
                Primitive::String(ref s) => println!("{}", s.to_string_lossy()),
                Primitive::Integer(i) => println!("{}", i),
                Primitive::Name(ref s) => println!("{}", s),
                ref p => println!("{:?}", p),
            }
        }
    }

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// ARCHIVE: try with "lopdf+pdf_form"
// use std::path::PathBuf;

// use pdf_form::{FieldType, Form};

// pub(crate) fn list_forms(pdf_path: &PathBuf) {
//     // Load the pdf into a form from a path
//     let form = Form::load(pdf_path).unwrap();
//     // Get all types of the form fields (e.g. Text, Radio, etc) in a Vector
//     let field_types = form.get_all_types();
//     // Print the types
//     for field_type in field_types {
//         println!("{:?}", field_type);
//     }
// }
