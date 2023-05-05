//TODO see also https://github.com/tafia/quick-xml/blob/master/examples/read_nodes_serde.rs

use std::path::PathBuf;

use quick_xml::events::Event;
use scraper::Html;
use scraper::Selector;
use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VkbReportError {
    #[error("the data for key `{0}` is not available")]
    Redaction(String),
    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader { expected: String, found: String },
    #[error("unknown xml error")]
    Unknown,
    #[error("the xml desc `{0}` is not handled")]
    UnexpectedXmlDesc(String),
}

/// Maps eg
/// <m7 t="0" h="32,12105"
/// u="&#60;font color=&#34;#000000&#34;&#62;Virtual button with SHIFT1 = 63&#13;&#10;Virtual button with SHIFT2 = 92" />
///
#[derive(Deserialize, Debug)]
struct M7 {
    /// This is the description; xml escaped!
    #[serde(rename = "@u")]
    desc_xml_escaped: String,
}

/// Maps eg
/// <b2 t="22,67718" h="32,12105">
///     <p2 t="0" w="113" h="22" ImageIndex="23" Transparent="1" />
///     <m7 t="0" h="32,12105"
///         u="&#60;font color=&#34;#000000&#34;&#62;Virtual button with SHIFT1 = 63&#13;&#10;Virtual button with SHIFT2 = 92" />
/// </b2>
///
/// NOTE: only care about "m7": <m7 name="Page0.Description1" />
/// "p2" is an image: <p2 name="Page0.Image1" />
#[derive(Deserialize, Debug)]
struct B2 {
    #[serde(rename = "@t")]
    t: String,
    #[serde(rename = "@h")]
    h: String,
    /// NOTE: there is ONE instance of "b2" where this field is not there
    /// in which case there is a additional field "g1"
    /// Not sure what that is, so we ignore it for now.
    m7: Option<M7>,
}

/// Maps the M3 child struct Virtual Button ID ("VBN" in VKB terminology)
/// <m8 u="95" />
#[derive(Deserialize, Debug)]
struct M8 {
    /// This is the description; xml escaped!
    #[serde(rename = "@u")]
    virtual_button_id: String,
}

/// Maps the description for the M3 child struct
/// <m9 u="&#60;b&#62;#95 &#60;/b&#62; Joystick button : #95" />
#[derive(Deserialize, Debug)]
struct M9 {
    /// This is the description; xml escaped!
    #[serde(rename = "@u")]
    desc_xml_escaped: String,
}

/// Maps eg
/// <b3 t="560,72449" h="33,77953">
///     <s1 />
///     <p3 w="113" h="22" ImageIndex="12" Transparent="1" />
///     <m8 u="95" />
///     <s2 />
///     <m9 u="&#60;b&#62;#95 &#60;/b&#62; Joystick button : #95" />
///     <g2 Left="0" Top="4,22046999999998" Width="718,1107" Height="1,13385826771654"
///         ShowHint="false" BeginColor="12632256" Style="gsHorizontal" Color="10526880" />
// </b3>
///
/// NOTE: only care about
/// - "m8": <m8 name="Page0.VBN" /> -> the Virtual Button number?
/// - "m9": <m9 name="Page0.Decsription2" /> -> description field, same as "m7" for "b2" struct
#[derive(Deserialize, Debug)]
struct B3 {
    #[serde(rename = "@t")]
    t: String,
    #[serde(rename = "@h")]
    h: String,
    m8: M8,
    m9: M9,
}

/// Intro segment, only on the first page0?
type B1 = ();
/// also only on the first page0; not sure what that is: "<TfrxNullBand Height="1046,92981" Left="0" Top="0" Width="718,1107" l="0" t="0" />"
type TfrxNullBand = ();
/// summary; only on the last page
type B4 = ();
type B5 = ();
type B6 = ();

/// To preserve the relative order fo b2/b3 we need an wrapper struct
/// thanks phind.com
#[derive(Deserialize, Debug)]
enum Page0Item {
    b1(B1),
    TfrxNullBand(TfrxNullBand),
    b2(B2),
    b3(B3),
    b4(B4),
    b5(B5),
    b6(B6),
}

/// Maps the per-page structure of B2/B3
/// <page0>
///     <b2...>
///     <b3 ...>
///     <b3 ...>
///     <b2...>
///     ...
/// </page0>
#[derive(Deserialize, Debug)]
struct Page0 {
    /// Option b/c the last page only has b6,b4,b5
    #[serde(rename = "$value")]
    b: Option<Vec<Page0Item>>,
}

/// <previewpages>Page0,Page0,...</previewpages>
#[derive(Deserialize, Debug)]
struct PreviewPages {
    page0: Vec<Page0>,
}

/// Maps the full report eg
/// <?xml version="1.0" encoding="utf-8" standalone="no"?>
/// <preparedreport>
/// <previewpages>
///     <page0>
///         <b2 ...>
///         <b3 ...>
///         <b3 ...>
///         <b2 ...>
///     ...
///     </page0>
///     <page0>
///         <b2 ...>
///         <b3 ...>
///         <b3 ...>
///         <b2 ...>
///     ...
///     </page0>
///     ...
/// </previewpages>
/// ...
/// </preparedreport>
///
#[derive(Deserialize, Debug)]
pub(crate) struct VkbReport {
    previewpages: PreviewPages,
}

///
// TODO remove feature "overlapped-lists" and add a wrapper for B2 + Optional<Vec<B3>>
pub(crate) fn parse_report(xml_path: PathBuf) -> Result<VkbReport, VkbReportError> {
    let xml_str = std::fs::read_to_string(xml_path).map_err(|_| VkbReportError::Unknown)?;

    // TODO
    let vkb_report: VkbReport = quick_xml::de::from_str(&xml_str).map_err(|err| {
        println!("report error: {:?}", err);
        VkbReportError::Unknown
    })?;

    println!("report: {:#?}", vkb_report);

    Ok(vkb_report)
}

/// This is NOT from the xml, this is the end result.
/// We construct the final "buttons" by parsing the "xml_desc" field eg:
///
/// <b>#6 </b> Joystick button : #52
/// <b>#6 (F1) </b><b>TEMPO </b>\r\nVirtual button Short #6\r\nVirtual button Long #96
/// etc
#[derive(PartialEq, Debug)]
struct Button {
    kind: ButtonKind,
}

#[derive(PartialEq, Debug)]
enum ButtonKind {
    /// This matches a "b2" field in xml
    /// To get the ID we need to parse the desc...
    Physical { physical_button_id: u8 },
    /// Virtual/Logical
    /// This matches a "b3" field in xml
    /// In this case the "m8" field directly contains the ID, no parsing needed.
    /// The "m9" SHOULD also contain the same ID in the desc.
    Virtual { virtual_button_id: u8 },
}

/// Parse "desc_xml_escaped" FIRST LINE eg:
/// <b>#4 </b>
/// <b>#5 (F3) </b><b>TEMPO </b>
/// <b>#7 (F2) </b><b>TEMPO </b>
/// <font color=\"#000000\">Virtual button with SHIFT1 = 63
/// <b>#10 (Fire 1-st stage) </b><b>- Button with momentary action</b>
/// <b>#12 (A2) </b><b>- Button with momentary action</b>
/// <b>#13 (Ministick push) </b><b>Microstick Mode Switch </b>
/// etc
fn parse_desc_xml_first_line(first_line: &str) {}

/// Parse eg "#1 (E1) ", "#2  - Encoder 2/4", etc
/// Return:
/// - ALWAYS a "Button ID" eg 1,2,etc
/// - if applicable: "additional into" eg "(E1)", "Encoder 2/4", etc
fn parse_inner_html_desc(inner_html_desc: &str) -> ButtonIdAndInfo {
    assert!(inner_html_desc.starts_with("#"));
    let (button_id_str, info_str) = inner_html_desc[1..].split_once(" ").unwrap();

    ButtonIdAndInfo {
        id: button_id_str.parse().unwrap(),
        additional_info: Some(info_str.trim().to_string()),
    }
}

#[derive(Debug, PartialEq)]
struct ButtonIdAndInfo {
    id: u8,
    additional_info: Option<String>,
}

fn parse_desc_xml(desc_xml_escaped: &str) {
    let fragment = Html::parse_fragment(desc_xml_escaped);
    println!("parse_desc_xml fragment : {:#?}", fragment.tree);

    for node in fragment.tree.nodes() {
        println!("node : {:#?}", node);
    }

    // the selected inner_html should contain something like:
    // "#1 (E1) ", "#2  - Encoder 2/4", etc
    let b_selector = Selector::parse("b").unwrap();
    let b_nodes: Vec<_> = fragment.select(&b_selector).collect();
    println!("b_nodes [{}] : {:?}", b_nodes.len(), b_nodes);
    for b_node in b_nodes {
        println!("b_node : inner_html : {:#?}", b_node.inner_html());
    }
}

fn parse_b2_button_desc_xml_escaped(desc_xml_escaped: &str) -> Result<Button, VkbReportError> {
    // let lines: Vec<&str> = desc_xml_escaped.split("\r\n").collect();
    // let first_line = lines[0];
    parse_desc_xml(desc_xml_escaped);

    let button = Button {
        kind: ButtonKind::Physical {
            physical_button_id: todo!(),
        },
    };

    Ok(button)
}

fn parse_b3_button_desc_xml_escaped(desc_xml_escaped: &str) -> Result<Button, VkbReportError> {
    let button = Button {
        kind: ButtonKind::Virtual {
            virtual_button_id: todo!(),
        },
    };

    Ok(button)
}

fn construct_button_from_xml(page_item: Page0Item) -> Result<Button, VkbReportError> {
    match page_item {
        Page0Item::b2(b2_xml) => match b2_xml.m7 {
            Some(m7) => parse_b2_button_desc_xml_escaped(&m7.desc_xml_escaped),
            None => Err(VkbReportError::UnexpectedXmlDesc(format!("{:?}", b2_xml))),
        },
        Page0Item::b3(b3_xml) => {
            let button = parse_b3_button_desc_xml_escaped(&b3_xml.m9.desc_xml_escaped)?;
            // TODO assert_eq!(button.id, b3_xml.m8.virtual_button_id);
            Ok(button)
        }
        _ => unimplemented!("parse_button_xml SHOULD only be called with b2 or b3 field"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_M7() {
        let xml_str = r#"
        <m7 h="20,59895" u="&#60;b&#62;#9 (Fire 2-nd stage) &#60;/b&#62;&#60;b&#62;- Button with momentary action&#60;/b&#62;" />
        "#;

        quick_xml::de::from_str::<M7>(xml_str).unwrap();
    }

    #[test]
    fn test_parse_B2() {
        let xml_str = r#"
        <b2 t="991,09462" h="33,15801">
            <m4 u="7" />
            <m5 u="9" />
            <m6 u="[R2.1]" />
            <m7 h="20,59895"
                u="&#60;b&#62;#9 (Fire 2-nd stage) &#60;/b&#62;&#60;b&#62;- Button with momentary action&#60;/b&#62;" />
            <g1 Left="0" Top="6,22046999999998" Width="718,1107" Height="1,88976378"
                ShowHint="false" BeginColor="12632256" Style="gsHorizontal" Color="10526880" />
        </b2>
        "#;

        quick_xml::de::from_str::<B2>(xml_str).unwrap();
    }

    #[test]
    fn test_parse_M8() {
        let xml_str = r#"
        <m8 u="95" />
        "#;

        quick_xml::de::from_str::<M8>(xml_str).unwrap();
    }

    #[test]
    fn test_parse_M9() {
        let xml_str = r#"
        <m9 u="&#60;b&#62;#95 &#60;/b&#62; Joystick button : #95" />
        "#;

        quick_xml::de::from_str::<M9>(xml_str).unwrap();
    }

    #[test]
    fn test_parse_B3() {
        let xml_str = r#"
        <b3 t="560,72449" h="33,77953">
                <s1 />
                <p3 w="113" h="22" ImageIndex="12" Transparent="1" />
                <m8 u="95" />
                <s2 />
                <m9 u="&#60;b&#62;#95 &#60;/b&#62; Joystick button : #95" />
                <g2 Left="0" Top="4,22046999999998" Width="718,1107" Height="1,13385826771654"
                    ShowHint="false" BeginColor="12632256" Style="gsHorizontal" Color="10526880" />
            </b3>
        "#;

        quick_xml::de::from_str::<B3>(xml_str).unwrap();
    }

    #[test]
    fn test_parse_Page0() {
        let xml_str = r#"
        <page0>
            <b1 t="0" h="173,89765">
                <m1 u="www.vkb-sim.pro" />
                <p1 ImageIndex="1" Transparent="1" TransparentColor="-16777208" />
                <m2 h="155"
                    u="Report generated by VKB Device Configurator v0.92.51  01/05/2023   11:25:59&#13;&#10; &#13;&#10;Controller : VKB NJoy32 XT PRO  v2.122&#13;&#10; &#13;&#10;Number of logical buttons : 128&#13;&#10;Number of HATs : 1&#13;&#10;MOUSE - Not used&#13;&#10;Virtual Keyboard : Not used&#13;&#10;Multimedia Controls : Not used&#13;&#10;Windows system Controls : Not used" />
                <m3 u="" />
            </b1>
            <TfrxNullBand Height="1046,92981" Left="0" Top="0" Width="718,1107" l="0" t="0" />
            <b2 t="173,89765" h="45,55906">
                <p2 w="228" h="22" ImageIndex="2" Transparent="1" />
                <m4 u="Line" />
                <m5 u="1" />
                <m6 u="[R1.1]" />
                <m7 h="33"
                    u="&#60;b&#62;#1 (E1) &#60;/b&#62; / &#60;b&#62;#2  - Encoder 2/4&#60;/b&#62;&#13;&#10;Virtual buttons : #61 / #62" />
                <g1 Left="0" Top="6,22046999999998" Width="718,1107" Height="1,88976378"
                    ShowHint="false" BeginColor="12632256" Style="gsHorizontal" Color="10526880" />
            </b2>
            <b3 t="219,45671" h="33,77953">
                <s1 />
                <p3 w="113" h="22" ImageIndex="3" Transparent="1" />
                <m8 u="61" />
                <s2 />
                <m9 u="&#60;b&#62;#61 &#60;/b&#62; Joystick button : #61" />
                <g2 Left="0" Top="4,22046999999998" Width="718,1107" Height="1,13385826771654"
                    ShowHint="false" BeginColor="12632256" Style="gsHorizontal" Color="10526880" />
            </b3>
            <b5 t="1024,25263" />
        </page0>
        "#;

        quick_xml::de::from_str::<Page0>(xml_str).unwrap();
    }

    #[test]
    fn test_parse_ReportFull_sample() {
        let xml_str = include_str!("../tests/data/vkb_report_simplified.fp3");

        quick_xml::de::from_str::<VkbReport>(xml_str).unwrap();
    }

    #[test]
    fn test_parse_ReportFull_sample2() {
        let xml_str =
            include_str!("/home/pratn/workspace/sc-keymap-rs/sc-keymap-rs/data/report_R.fp3");

        quick_xml::de::from_str::<VkbReport>(xml_str).unwrap();
    }

    #[test]
    fn test_parse_inner_html_desc() {
        let test_inputs_vs_expected_results = vec![
            (
                "#1 (E1) ",
                ButtonIdAndInfo {
                    id: 1,
                    additional_info: Some("(E1)".to_string()),
                },
            ),
            (
                "#2  - Encoder 2/4",
                ButtonIdAndInfo {
                    id: 2,
                    additional_info: Some("- Encoder 2/4".to_string()),
                },
            ),
        ];

        for (input, expected_result) in test_inputs_vs_expected_results {
            let button = parse_inner_html_desc(input);
            assert_eq!(button, expected_result);
        }
    }

    #[test]
    fn test_construct_button_b2() {
        let test_inputs_vs_expected_results = vec![
            (
                "<b>#1 (E1) </b> / <b>#2  - Encoder 2/4</b>\r\nVirtual buttons : #61 / #62",
                Button {
                    kind: ButtonKind::Physical {
                        physical_button_id: 1,
                    },
                },
            ),
            (
                "<b>#3 (E2) </b><b>- Button with momentary action</b>",
                Button {
                    kind: ButtonKind::Physical {
                        physical_button_id: 3,
                    },
                },
            ),
        ];

        for (input, expected_result) in test_inputs_vs_expected_results {
            let button = parse_b2_button_desc_xml_escaped(input).unwrap();
            assert_eq!(button, expected_result);
        }
    }
}
