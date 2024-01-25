use std::path::PathBuf;
use std::rc::Rc;

use usvg::{TreeParsing, TreePostProc};

use resvg::usvg;

///
/// https://github.com/RazrFalcon/resvg/blob/master/examples/minimal.rs
/// TODO see also:
/// https://github.com/RazrFalcon/resvg/blob/master/examples/draw_bboxes.rs
/// https://github.com/RazrFalcon/resvg/blob/master/examples/custom_href_resolver.rs
/// etc
///
/// You can get templates .svg from: https://github.com/Rexeh/joystick-diagrams/tree/master/templates
/// NOTE: this is a good start; but does NOT support advanced VKB features like SHIFT,TEMPO,etc
///
/// WARNING apparently there is no easy way to override the font used in the .svg so you MUST edit
/// (find+replace) to one present in your system eg find/replace Tahoma->FreeSerif and Helvetica->FreeSerif...
/// TODO do this with a svg editor? or possible with resvg?
///     MAYBE related https://github.com/RazrFalcon/resvg/issues/555 ?
///
pub fn svg_parse(input_svg_path: PathBuf, output_png_path: PathBuf) {
    let mut opt = resvg::usvg::Options::default();
    // Get file's absolute directory.
    opt.resources_dir = std::fs::canonicalize(&input_svg_path)
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));
    // "Will be used when no font-family attribute is set in the SVG. Default: Times New Roman"
    // But apparently on Arch, there is no TNR, so the texts are missing in the rendered .png
    opt.font_family = "FreeSerif".to_string();

    let mut fontdb = resvg::usvg::fontdb::Database::new();
    fontdb.load_system_fonts();
    // TODO? https://github.com/RazrFalcon/resvg/blob/c6fd7f486aaef81704546f298541513f50f4f868/crates/usvg/src/main.rs#L361
    // fontdb.set_serif_family("Times New Roman");
    // fontdb.set_sans_serif_family("Arial");
    // fontdb.set_cursive_family("Comic Sans MS");
    // fontdb.set_fantasy_family("Impact");
    // fontdb.set_monospace_family("Courier New");

    let svg_data = std::fs::read(input_svg_path).unwrap();
    let mut tree = usvg::Tree::from_data(&svg_data, &opt).unwrap();

    // TODO? But this display the text in teh final .png, which is NOT what we want
    // ideally we should play with layer and visibility
    // https://github.com/RazrFalcon/resvg/blob/c6fd7f486aaef81704546f298541513f50f4f868/crates/resvg/examples/draw_bboxes.rs#L33C5-L33C69
    tree.postprocess(
        usvg::PostProcessingSteps {
            // `resvg` cannot render text as is. We have to convert it into paths first.
            convert_text_into_paths: true,
        },
        &fontdb,
    );

    // let mut bboxes = Vec::new();
    // let mut text_bboxes = Vec::new();

    // https://github.com/RazrFalcon/resvg/blob/master/examples/draw_bboxes.rs
    //
    // let mut current_group_bboxes = vec![];
    // let mut all_group_bboxes = vec![];

    // We want to keep a node if it is:
    // - a leaf(ie it has no children), but IFF its parent was not already selected
    //   Usually this is a basic button, not part of a group.
    // - a group, if its children are in the previous case
    //   Usually this is a standard group, whose children are final leaves.
    //   In this case we DO NOT want the children, but only their parent!
    // let leaf_nodes: Vec<_> = tree
    //     .root
    //     .children
    //     .iter()
    //     .filter(|n| !*n.())
    //     .collect();
    // let parent_leaf_nodes: Vec<_> = tree
    //     .root
    //     .children
    //     .iter()
    //     // .filter(|n| n.has_children() && n.descendants().all(|child| leaf_nodes.contains(&child)))
    //     .filter(|n| n.children.iter().all(|child| leaf_nodes.contains(&child)))
    //     .collect();
    // let parent_nodes: Vec<_> = tree
    //     .root
    //     .children
    //     .iter()
    //     .filter(|n| n.has_children())
    //     .collect();
    // let final_leaf_nodes: Vec<_> = tree
    //     .root
    //     .children
    //     .iter()
    //     .filter(|n| !n.has_children() && n.parent().unwrap().children().count() == 1)
    //     .collect();
    // let group_nodes: Vec<_> = tree
    //     .root
    //     .descendants()
    //     .filter(|n| n.has_children() && n.parent().unwrap().children().count() > 1)
    //     .collect();
    // let parent_nodes2: Vec<_> = leaf_nodes
    //     .iter()
    //     .filter(|n| n.parent().unwrap().children().count() == 1)
    //     .collect();

    // TODO(re-add?) .filter(|n| n.has_children())
    // for node in &tree.root.children {
    //     if let Some(bbox) = node.abs_bounding_box().and_then(|r: Rect| Some(r.clone())) {
    //         log::debug!("NodeKind calculate_bbox : {:?}", bbox);
    //         // bboxes.push(bbox);

    //         // NOTE: careful NodeKind::Group means both:
    //         // - a group, in which case group.id is empty
    //         // - a text field, in which case eg: `group.id == "button5"`
    //         if let resvg::usvg::Node::Group(ref group) = node {
    //             log::debug!("NodeKind::Group : {}", group.id);
    //             // first case: new group
    //             if group.id.is_empty() || group.id.starts_with("layer") {
    //                 // // when we have a "current group", store it!
    //                 // if !current_group_bboxes.is_empty() {
    //                 //     all_group_bboxes.push(current_group_bboxes.clone());
    //                 // }
    //                 // current_group_bboxes.clear();

    //                 // nothing to do!
    //             } else {
    //                 // current_group_bboxes.push(bbox);
    //                 all_group_bboxes.push(bbox);
    //             }
    //         }
    //     }

    //     if let resvg::usvg::Node::Group(ref group) = node {
    //         log::debug!("NodeKind::Group : {}", group.id);
    //     }

    //     // "Text bboxes are different from path bboxes."
    //     // https://github.com/RazrFalcon/resvg/blob/1dfe9e506c2f90b55e662b1803d27f0b4e4ace77/crates/resvg/examples/draw_bboxes.rs#L43C9-L48C10
    //     if let usvg::Node::Text(ref text) = node {
    //         if let Some(ref bbox) = text.bounding_box {
    //             log::debug!("NodeKind::Text : {:?}, {:?}", text, bbox);
    //             // text_bboxes.push(bbox.to_rect());
    //         }
    //     }

    //     if let resvg::usvg::Node::Path(ref path) = *node.borrow() {
    //         log::debug!("NodeKind::Text : {:?}", path);
    //         todo!("NodeKind::Path");
    //     }

    //     // // NOTE: careful NodeKind::Group means both:
    //     // // - a group, in which case group.id is empty
    //     // // - a text field, in which case eg: `group.id == "button5"`
    //     // if let resvg::usvg::Node::Group(ref group) = *node.borrow() {
    //     //     log::debug!("NodeKind::Group : {}", group.id);
    //     //     // first case: new group
    //     //     if group.id.is_empty() {
    //     //         current_group_bboxes.clear();
    //     //     } else {
    //     //     }
    //     // }
    // }

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

    //     log::debug!("group_bboxes : {}", bounding_bbox);
    //     bboxes.push(bounding_bbox);
    // }

    // https://github.com/RazrFalcon/resvg/blob/c6fd7f486aaef81704546f298541513f50f4f868/crates/resvg/examples/draw_bboxes.rs#L35C5-L37C65
    let mut bboxes = Vec::new();
    let mut stroke_bboxes = Vec::new();
    collect_bboxes(&tree.root, &mut bboxes, &mut stroke_bboxes);

    // https://github.com/RazrFalcon/resvg/blob/c6fd7f486aaef81704546f298541513f50f4f868/crates/resvg/examples/draw_bboxes.rs#L39C5-L49C8
    let stroke1 = Some(usvg::Stroke {
        paint: usvg::Paint::Color(usvg::Color::new_rgb(255, 0, 0)),
        opacity: usvg::Opacity::new_clamped(0.5),
        ..usvg::Stroke::default()
    });

    let _stroke2 = Some(usvg::Stroke {
        paint: usvg::Paint::Color(usvg::Color::new_rgb(0, 200, 0)),
        opacity: usvg::Opacity::new_clamped(0.5),
        ..usvg::Stroke::default()
    });

    // https://github.com/RazrFalcon/resvg/blob/c6fd7f486aaef81704546f298541513f50f4f868/crates/resvg/examples/draw_bboxes.rs#L51
    for bbox in bboxes {
        let mut path = usvg::Path::new(Rc::new(resvg::tiny_skia::PathBuilder::from_rect(bbox)));
        path.stroke = stroke1.clone();
        tree.root.children.push(usvg::Node::Path(Box::new(path)));
    }

    // for bbox in text_bboxes {
    //     tree.root.append_kind(usvg::Node::Path(usvg::Path {
    //         stroke: stroke2.clone(),
    //         data: Rc::new(usvg::PathData::from_rect(bbox)),
    //         ..usvg::Path::default()
    //     }));
    // }

    // https://github.com/RazrFalcon/resvg/blob/c6fd7f486aaef81704546f298541513f50f4f868/crates/resvg/examples/draw_bboxes.rs#L63C5-L64C37
    // "Calculate bboxes of newly added path."
    tree.calculate_bounding_boxes();

    ////////////////////////////////////////////////////////////////////////////

    // https://github.com/RazrFalcon/resvg/blob/c6fd7f486aaef81704546f298541513f50f4f868/crates/resvg/examples/custom_usvg_tree.rs#L56
    const ZOOM: f32 = 1.0;
    let pixmap_size = tree.size.to_int_size().scale_by(ZOOM).unwrap();
    let mut pixmap =
        resvg::tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );
    pixmap.save_png(output_png_path).unwrap();
}

/// https://github.com/RazrFalcon/resvg/blob/c6fd7f486aaef81704546f298541513f50f4f868/crates/resvg/examples/draw_bboxes.rs#L73C1-L93C2
///
fn collect_bboxes(
    parent: &usvg::Group,
    bboxes: &mut Vec<usvg::Rect>,
    stroke_bboxes: &mut Vec<usvg::Rect>,
) {
    for node in &parent.children {
        if let usvg::Node::Group(ref group) = node {
            log::debug!("Node::Group");
            collect_bboxes(group, bboxes, stroke_bboxes);
        }

        // "Text bboxes are different from path bboxes."
        // https://github.com/RazrFalcon/resvg/blob/1dfe9e506c2f90b55e662b1803d27f0b4e4ace77/crates/resvg/examples/draw_bboxes.rs#L43C9-L48C10
        if let usvg::Node::Text(ref text) = node {
            if let Some(ref bbox) = text.bounding_box {
                log::debug!("Node::Text : {:#?}, {:?}", text.chunks[0].text, bbox);
                // text_bboxes.push(bbox.to_rect());
            }

            if let Some(bbox) = node.abs_bounding_box() {
                bboxes.push(bbox);

                if let Some(stroke_bbox) = node.abs_stroke_bounding_box() {
                    if bbox != stroke_bbox.to_rect() {
                        stroke_bboxes.push(stroke_bbox.to_rect());
                    }
                }
            }
        }

        if let resvg::usvg::Node::Path(ref _path) = *node {
            log::debug!("Node::Path");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_map_vkb_report_l() {
        svg_parse(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/bindings/VKB-Sim Gladiator NXT L.svg"
            )
            .into(),
            concat!(env!("CARGO_MANIFEST_DIR"), "/out.png").into(),
        );
    }
}
