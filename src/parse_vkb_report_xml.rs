//TODO see also https://github.com/tafia/quick-xml/blob/master/examples/read_nodes_serde.rs

use std::path::PathBuf;

use quick_xml::events::Event;
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
    m7: M7,
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
    m8: M8,
    m9: M8,
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
    b2: Vec<B2>,
    b3: Vec<B3>,
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
struct ReportFull {
    previewpages: PreviewPages,
}

///
// TODO remove feature "overlapped-lists" and add a wrapper for B2 + Optional<Vec<B3>>
pub(crate) fn parse_report(xml_path: PathBuf) -> Result<(), VkbReportError> {
    let xml_str = std::fs::read_to_string(xml_path).map_err(|_| VkbReportError::Unknown)?;

    // TODO
    let xml_data: ReportFull =
        quick_xml::de::from_str(&xml_str).map_err(|_| VkbReportError::Unknown)?;

    println!("report: {:?}", xml_data);

    Ok(())
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

        quick_xml::de::from_str::<ReportFull>(xml_str).unwrap();
    }
}
