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

///
/// https://github.com/pdf-rs/pdf/blob/a6e2abc96b23b64aa1051966bb000aabf1275d9f/pdf/examples/metadata.rs#L7
pub(crate) fn list_forms(pdf_path: &PathBuf) -> Result<(), PdfFormError> {
    let file = FileOptions::cached().open(&pdf_path).unwrap();

    if let Some(ref info) = file.trailer.info_dict {
        info.iter()
            .filter(|(_, primitive)| primitive.to_string_lossy().is_ok())
            .for_each(|(key, value)| {
                eprintln!("{:>15}: {}", key, value.to_string_lossy().unwrap());
            });
    }

    if let Some(ref forms) = file.get_root().forms {
        for field in forms.fields.iter() {
            print_field(field, &file);
        }
    }

    Ok(())
}

fn print_field(field: &FieldDictionary, resolve: &impl Resolve) {
    print!("print_field : {:?} = ", field.name);
    match field.value {
        Primitive::String(ref s) => println!("{}", s.to_string_lossy()),
        Primitive::Integer(i) => println!("{}", i),
        Primitive::Name(ref s) => println!("{}", s),
        ref p => println!("{:?}", p),
    }

    if field.typ == Some(FieldType::Signature) {
        println!("{:?}", field);
    }
    for &kid in field.kids.iter() {
        let child = resolve.get(kid).unwrap();
        print_field(&child, resolve);
    }
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
