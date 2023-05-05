use std::path::PathBuf;
use std::rc::Rc;

use resvg::usvg::NodeExt;
use resvg::usvg::TreeParsing;
use resvg::usvg::TreeTextToPath;

use resvg::usvg;

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
    // "Will be used when no font-family attribute is set in the SVG. Default: Times New Roman"
    // But apparently on Arch, there is no TNR, so the texts are missing in the rendered .png
    opt.font_family = "FreeSerif".to_string();

    let mut fontdb = resvg::usvg::fontdb::Database::new();
    fontdb.load_system_fonts();

    let svg_data = std::fs::read(input_svg_path).unwrap();
    let mut tree = resvg::usvg::Tree::from_data(&svg_data, &opt).unwrap();

    // TODO? But this display the text in teh final .png, which is NOT what we want
    // ideally we should play with layer and visibility
    // tree.convert_text(&fontdb);

    ////////////////////////////////////////////////////////////////////////////
    /// https://github.com/RazrFalcon/resvg/blob/master/examples/draw_bboxes.rs
    //
    let mut bboxes = Vec::new();
    let mut text_bboxes = Vec::new();
    for node in tree.root.descendants() {
        if let Some(bbox) = node.calculate_bbox().and_then(|r| r.to_rect()) {
            bboxes.push(bbox);
        }

        // Text bboxes are different from path bboxes.
        if let resvg::usvg::NodeKind::Path(ref path) = *node.borrow() {
            println!("NodeKind::Path : {}", path.id);
            if let Some(ref bbox) = path.text_bbox {
                text_bboxes.push(*bbox);
            }
        }

        if let resvg::usvg::NodeKind::Text(ref text) = *node.borrow() {
            println!("NodeKind::Text : {}, {:?}", text.id, text.positions);
        }

        if let resvg::usvg::NodeKind::Group(ref group) = *node.borrow() {
            println!("NodeKind::Group : {}", group.id);
        }
    }

    let stroke = Some(usvg::Stroke {
        paint: usvg::Paint::Color(usvg::Color::new_rgb(255, 0, 0)),
        opacity: usvg::Opacity::new_clamped(0.5),
        ..usvg::Stroke::default()
    });

    let stroke2 = Some(usvg::Stroke {
        paint: usvg::Paint::Color(usvg::Color::new_rgb(0, 0, 200)),
        opacity: usvg::Opacity::new_clamped(0.5),
        ..usvg::Stroke::default()
    });

    for bbox in bboxes {
        tree.root.append_kind(usvg::NodeKind::Path(usvg::Path {
            stroke: stroke.clone(),
            data: Rc::new(usvg::PathData::from_rect(bbox)),
            ..usvg::Path::default()
        }));
    }

    for bbox in text_bboxes {
        tree.root.append_kind(usvg::NodeKind::Path(usvg::Path {
            stroke: stroke2.clone(),
            data: Rc::new(usvg::PathData::from_rect(bbox)),
            ..usvg::Path::default()
        }));
    }

    ////////////////////////////////////////////////////////////////////////////

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
