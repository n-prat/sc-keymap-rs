use std::io::BufWriter;

use askama::Template; // bring trait in scope

#[derive(Template)] // this will generate the code...
#[template(path = "hello.html")] // using the template in this path, relative
                                 // to the `templates` dir in the crate root
struct HelloTemplate<'a> {
    // the name of the struct can be anything
    pub name: &'a str, // the field name should match the variable name
                       // in your template
}

pub fn render_template() {
    let hello = HelloTemplate { name: "world" }; // instantiate your struct
    println!("{}", hello.render().unwrap()); // then render it.

    let mut f = std::fs::File::create("out.html").expect("Unable to create file ./out.html");
    let mut f = BufWriter::new(f);
    // hello.render_into(&mut f).unwrap();
    hello.write_into(&mut f).unwrap();
}
