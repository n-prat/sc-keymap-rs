use resvg::usvg::TreeParsing;
use resvg::usvg::TreeTextToPath;
use std::path::PathBuf;

///
/// https://github.com/RazrFalcon/resvg/blob/master/examples/minimal.rs
/// TODO see also:
/// https://github.com/RazrFalcon/resvg/blob/master/examples/draw_bboxes.rs
/// https://github.com/RazrFalcon/resvg/blob/master/examples/custom_href_resolver.rs
/// etc
pub(crate) fn svg_parse(input_svg_path: &PathBuf, output_png_path: PathBuf) {
    let mut opt = resvg::usvg::Options::default();
    // Get file's absolute directory.
    opt.resources_dir = std::fs::canonicalize(input_svg_path)
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));

    let mut fontdb = resvg::usvg::fontdb::Database::new();
    fontdb.load_system_fonts();

    let svg_data = std::fs::read(input_svg_path).unwrap();
    let mut tree = resvg::usvg::Tree::from_data(&svg_data, &opt).unwrap();
    tree.convert_text(&fontdb);

    let pixmap_size = tree.size.to_screen_size();
    let mut pixmap =
        resvg::tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
    resvg::render(
        &tree,
        resvg::FitTo::Original,
        resvg::tiny_skia::Transform::default(),
        pixmap.as_mut(),
    )
    .unwrap();
    pixmap.save_png(output_png_path).unwrap();
}
