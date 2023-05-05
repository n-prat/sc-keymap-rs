use std::path::PathBuf;
use std::rc::Rc;

use resvg::usvg::NodeExt;
use resvg::usvg::Rect;
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
    tree.convert_text(&fontdb);

    // let mut bboxes = Vec::new();
    // let mut text_bboxes = Vec::new();

    // https://github.com/RazrFalcon/resvg/blob/master/examples/draw_bboxes.rs
    //
    // let mut current_group_bboxes = vec![];
    let mut all_group_bboxes = vec![];
    for node in tree.root.descendants() {
        if let Some(bbox) = node.calculate_bbox().and_then(|r| r.to_rect()) {
            println!("NodeKind calculate_bbox : {}", bbox);
            // bboxes.push(bbox);

            // NOTE: careful NodeKind::Group means both:
            // - a group, in which case group.id is empty
            // - a text field, in which case eg: `group.id == "button5"`
            if let resvg::usvg::NodeKind::Group(ref group) = *node.borrow() {
                println!("NodeKind::Group : {}", group.id);
                // first case: new group
                if group.id.is_empty() || group.id.starts_with("layer") {
                    // // when we have a "current group", store it!
                    // if !current_group_bboxes.is_empty() {
                    //     all_group_bboxes.push(current_group_bboxes.clone());
                    // }
                    // current_group_bboxes.clear();

                    // nothing to do!
                } else {
                    // current_group_bboxes.push(bbox);
                    all_group_bboxes.push(bbox);
                }
            }
        }

        // Text bboxes are different from path bboxes.
        if let resvg::usvg::NodeKind::Path(ref path) = *node.borrow() {
            // println!("NodeKind::Path : {}", path.id);
            if let Some(ref bbox) = path.text_bbox {
                println!("NodeKind::Path : {}, {:?}", path.id, path.text_bbox);
                // text_bboxes.push(*bbox);
            }
        }

        if let resvg::usvg::NodeKind::Text(ref text) = *node.borrow() {
            println!("NodeKind::Text : {}, {:?}", text.id, text.positions);
            todo!("NodeKind::Text");
        }

        // // NOTE: careful NodeKind::Group means both:
        // // - a group, in which case group.id is empty
        // // - a text field, in which case eg: `group.id == "button5"`
        // if let resvg::usvg::NodeKind::Group(ref group) = *node.borrow() {
        //     println!("NodeKind::Group : {}", group.id);
        //     // first case: new group
        //     if group.id.is_empty() {
        //         current_group_bboxes.clear();
        //     } else {
        //     }
        // }
    }

    // compute the centers of each "group of bboxes"
    // let mut bboxes = Vec::new();
    // for group_bboxes in all_group_bboxes {
    //     // NOTE: bottom == "self.y + self.height"
    //     // means the top is down! Sowe need to invert the min/max logic for top/bottom
    //     let mut min_top = f64::MAX;
    //     let mut max_bottom = f64::MIN;
    //     let mut max_right = f64::MIN;
    //     let mut min_left = f64::MAX;

    //     for bbox in group_bboxes {
    //         min_top = min_top.min(bbox.top());
    //         max_bottom = max_bottom.max(bbox.bottom());
    //         max_right = max_right.max(bbox.right());
    //         min_left = min_left.min(bbox.left());
    //     }

    //     let bounding_bbox = Rect::new(
    //         min_left,
    //         min_top,
    //         max_right - min_left,
    //         max_bottom - min_top,
    //     )
    //     .unwrap();

    //     println!("group_bboxes : {}", bounding_bbox);
    //     bboxes.push(bounding_bbox);
    // }

    let bboxes = all_group_bboxes;

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

    // for bbox in text_bboxes {
    //     tree.root.append_kind(usvg::NodeKind::Path(usvg::Path {
    //         stroke: stroke2.clone(),
    //         data: Rc::new(usvg::PathData::from_rect(bbox)),
    //         ..usvg::Path::default()
    //     }));
    // }

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
