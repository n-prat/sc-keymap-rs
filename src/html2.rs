/// From https://github.com/Stebalien/horrorshow-rs/blob/081ee9a70543362d8b9984d254798a3498eb0d5a/examples/subtemplate.rs
use std::error::Error;
use std::io;
use std::io::BufWriter;

use horrorshow::owned_html;
use horrorshow::prelude::*;

// page_title and content can be anything that can be rendered. A string, a
// template, a number, etc.
fn layout(page_title: impl Render, content: impl Render) -> impl Render {
    // owned_html _moves_ the arguments into the template. Useful for returning
    // owned (movable) templates.
    owned_html! {
        head {
            title : &page_title;
        }
        body {
            :&content
        }
    }
}

fn home_content() -> impl Render {
    owned_html! {
        h1 { :"Home Page" }
    }
}

fn about_content() -> impl Render {
    owned_html! {
        h1 { :"About Us" }
    }
}

fn contact_content() -> impl Render {
    owned_html! {
        h1 { :"Contact Us" }
    }
}

pub fn render_html() -> Result<(), Box<dyn Error>> {
    let mut f = std::fs::File::create("out2.html").expect("Unable to create file ./out2.html");
    let mut f = BufWriter::new(f);

    layout("Home", home_content()).write_to_io(&mut f)?;
    layout("About", about_content()).write_to_io(&mut f)?;
    layout("Contact", contact_content()).write_to_io(&mut f)?;
    Ok(())
}
