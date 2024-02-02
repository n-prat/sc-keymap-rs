//! Try to map as-is the xml(.fp3) file generated by `VKBDevCfg`'s `FastReport`
//! There is NO processing/cleaning up/checks done here!
//! These are done by the next step see `vkb_button.rs`
//!
// TODO see also https://github.com/tafia/quick-xml/blob/master/examples/read_nodes_serde.rs

use std::path::PathBuf;

use serde::Deserialize;

use crate::Error;

/// Maps eg
/// <m7 t="0" h="32,12105"
/// u="&#60;font color=&#34;#000000&#34;&#62;Virtual button with SHIFT1 = 63&#13;&#10;Virtual button with SHIFT2 = 92" />
///
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub(super) struct M7 {
    /// This is the description; xml escaped!
    #[serde(rename = "@u")]
    pub(super) desc_xml_escaped: String,
}

/// Maps the M5 child struct of "b2"
/// <m5 u="29" />
/// and is "Page0.LineN" which would seem to indicate this is only for ordering
/// BUT it maps nicely to the "physical button ID"???
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub(super) struct M5 {
    /// This is the description; xml escaped!
    #[serde(rename = "@u")]
    pub(super) physical_button_id: String,
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
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub(super) struct B2 {
    #[serde(rename = "@t")]
    t: String,
    #[serde(rename = "@h")]
    h: String,
    /// The fields below SHOULD always be there, but sometimes a struct is split
    /// onto two pages
    /// grep for "<b2 t="1006,65661" h="8,11023377999998">" and "<b2 t="990,09462" h="34,15801">"
    pub(super) m5: M5,
    pub(super) m7: M7,
}

// impl B2 {
//     /// Preprocessing step: merge the B2 fields on two different pages `page0`
//     fn merge_with(&self, next_b2: &Self) {
//         todo!()
//     }
// }

/// Maps the M3 child struct Virtual Button ID ("VBN" in VKB terminology)
/// <m8 u="95" />
#[derive(Deserialize, Debug, Clone)]
pub(super) struct M8 {
    /// This is the description; xml escaped!
    #[serde(rename = "@u")]
    pub(super) _virtual_button_id: String,
}

/// Maps the description for the M3 child struct
/// <m9 u="&#60;b&#62;#95 &#60;/b&#62; Joystick button : #95" />
#[derive(Deserialize, Debug, Clone)]
pub(super) struct M9 {
    /// This is the description; xml escaped!
    #[serde(rename = "@u")]
    pub(super) desc_xml_escaped: String,
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
#[derive(Deserialize, Debug, Clone)]
pub(super) struct B3 {
    #[serde(rename = "@t")]
    _t: String,
    #[serde(rename = "@h")]
    _h: String,
    #[serde(rename = "m8")]
    pub(super) _m8: M8,
    pub(super) m9: M9,
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
pub(super) enum Page0Item {
    #[serde(rename = "b1")]
    B1(B1),
    TfrxNullBand(TfrxNullBand),
    #[serde(rename = "b2")]
    B2(B2),
    #[serde(rename = "b3")]
    B3(B3),
    #[serde(rename = "b4")]
    B4(B4),
    #[serde(rename = "b5")]
    B5(B5),
    #[serde(rename = "b6")]
    B6(B6),
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
    b: Vec<Page0Item>,
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
pub(super) struct VkbReport {
    previewpages: PreviewPages,
}

/// A XML parsed button, matching a "b2" or "b3" xml field
/// this is mapped directly from the xml field, no logic, no checks, etc
///
/// The next step is done by `src/vkb/vkb_button.rs`
#[derive(Debug, Clone)]
pub(super) enum VkbXmlButton {
    B2(B2),
    B3(B3),
}

impl VkbReport {
    /// Parse a VKB .fp3 report (== .xml)
    // TODO remove feature "overlapped-lists" and add a wrapper for B2 + Optional<Vec<B3>>
    pub(super) fn new(xml_path: PathBuf) -> Result<Self, Error> {
        let xml_str = std::fs::read_to_string(xml_path).map_err(|_| Error::Unknown)?;

        // let xml_str = Self::preprocess_fix_pages(&xml_str);

        let vkb_report: VkbReport = quick_xml::de::from_str(&xml_str).map_err(|err| {
            log::error!("report error: {:?}", err);
            Error::Unknown
        })?;

        Ok(vkb_report)
    }

    // /// Preprocessing step
    // // TODO this is really ugly: it is parsing the whole report first; then doing again with Serde...
    // fn preprocess_fix_pages(xml_str: &str) -> String {
    //     use quick_xml::Reader;

    //     let mut reader = Reader::from_str(xml_str);
    //     reader.trim_text(true).expand_empty_elements(true);

    //     let mut buf = Vec::new();
    //     let mut buf_nested = Vec::new();

    //     // let mut previewpages = PreviewPages {
    //     //     page0: Page0 { b: vec![] },
    //     // };

    //     loop {
    //         buf.clear();
    //         match reader.read_event_into(&mut buf) {
    //             Ok(Event::Start(ref element)) => match element.name().as_ref() {
    //                 b"b2" => {
    //                     // let buf_clone = buf.clone();
    //                     // log::debug!("parsing b2: {:?}", buf.clone());
    //                     let attrs = element
    //                         .attributes()
    //                         .map(|a| a.unwrap().value)
    //                         .collect::<Vec<_>>();
    //                     log::debug!("b2 attrs: {:?}", attrs);

    //                     // reader.read_to_end(element.name());

    //                     // buf_nested.clear();
    //                     loop {
    //                         buf_nested.clear(); // NO! MUST be outside the loop! else always empty after exiting
    //                         match reader.read_event_into(&mut buf_nested) {
    //                             Ok(Event::Start(element)) => match element.name().as_ref() {
    //                                 b"m7" => {
    //                                     // stats.rows.push(vec![]);
    //                                     // row_index = stats.rows.len() - 1;
    //                                     let attrs_value = element
    //                                         .attributes()
    //                                         .map(|a| {
    //                                             let val = a.unwrap().value.clone().to_vec();
    //                                             std::str::from_utf8(&val).unwrap().to_string()
    //                                         })
    //                                         .collect::<Vec<_>>();
    //                                     let attrs_keys = element
    //                                         .attributes()
    //                                         .map(|a| {
    //                                             let val = a.unwrap().key.0.to_vec();
    //                                             std::str::from_utf8(&val).unwrap().to_string()
    //                                         })
    //                                         .collect::<Vec<_>>();
    //                                     log::debug!("b2 nested attrs_value: {:?}", attrs_value);
    //                                     log::debug!("b2 nested attrs_keys: {:?}", attrs_keys);

    //                                     let attrs =
    //                                         element.try_get_attribute("m7").unwrap().unwrap();
    //                                     log::debug!("b2 attrs: {:?}", attrs);
    //                                 }
    //                                 b"w:tc" => {
    //                                     // stats.rows[row_index].push(
    //                                     //     String::from_utf8(element.name().as_ref().to_vec())
    //                                     //         .unwrap(),
    //                                     // );
    //                                 }
    //                                 _ => {}
    //                             },
    //                             Ok(Event::End(_element)) => {
    //                                 // if element.name().as_ref() == b"m7" {
    //                                 //     // found_tables.push(stats);
    //                                 //     break;
    //                                 // }
    //                             }
    //                             Ok(Event::Eof) => break,
    //                             _ => {}
    //                         }
    //                     }

    //                     log::debug!(
    //                         "b2: {:?}, nested: {:?}",
    //                         std::str::from_utf8(&buf).unwrap(),
    //                         std::str::from_utf8(&buf_nested).unwrap()
    //                     );

    //                     // let b2: Result<B2, _> = quick_xml::de::from_reader::<_, B2>(buf.as_slice());
    //                     // match b2 {
    //                     //     Ok(b2) => {
    //                     //         // Add the deserialized B2 instance to the PreviewPages.
    //                     //         // previewpages.page0.b.push(Page0Item::b2(b2));
    //                     //         log::debug!("b2 ok: {:?}", b2);
    //                     //     }
    //                     //     Err(err) => {
    //                     //         // Handle the special case: peek into the next "b2" element and merge the instances.
    //                     //         log::debug!(
    //                     //             "b2 error: {}, data: {:?}",
    //                     //             err,
    //                     //             std::str::from_utf8(&buf).unwrap()
    //                     //         );
    //                     //         // todo!()
    //                     //     }
    //                     // }
    //                 }
    //                 _ => {}
    //             },
    //             Ok(Event::Eof) => break,
    //             Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
    //             _ => {}
    //         }
    //     }

    //     xml_str.to_owned()

    //     // let mut settings: HashMap<String, String>;

    //     // loop {
    //     //     let event = reader.read_event_into(&mut buf).map_err(|err| todo!())?;

    //     //     match event {
    //     //         Event::Start(element) => match element.name().as_ref() {
    //     //
    //     //                 // Note: real app would handle errors with good defaults or halt program with nice message
    //     //                 // This illustrates decoding an attribute's key and value with error handling
    //     //                 settings = element
    //     //                     .attributes()
    //     //                     .map(|attr_result| {
    //     //                         match attr_result {
    //     //                             Ok(a) => {
    //     //                                 let key = reader.decoder().decode(a.key.local_name().as_ref())
    //     //                                     .or_else(|err| {
    //     //                                         dbg!("unable to read key in DefaultSettings attribute {:?}, utf8 error {:?}", &a, err);
    //     //                                         Ok::<Cow<'_, str>, Infallible>(std::borrow::Cow::from(""))
    //     //                                     })
    //     //                                     .unwrap().to_string();
    //     //                                 let value = a.decode_and_unescape_value(&reader).or_else(|err| {
    //     //                                         dbg!("unable to read key in DefaultSettings attribute {:?}, utf8 error {:?}", &a, err);
    //     //                                         Ok::<Cow<'_, str>, Infallible>(std::borrow::Cow::from(""))
    //     //                                 }).unwrap().to_string();
    //     //                                 (key, value)
    //     //                             },
    //     //                             Err(err) => {
    //     //                                  dbg!("unable to read key in DefaultSettings, err = {:?}", err);
    //     //                                 (String::new(), String::new())
    //     //                             }
    //     //                         }
    //     //                     })
    //     //                     .collect();
    //     //                 // assert_eq!(settings["Language"], "es");
    //     //                 // assert_eq!(settings["Greeting"], "HELLO");
    //     //                 reader.read_to_end(element.name()).map_err(|err| todo!())?;
    //     //             }
    //     //             b"b3" => {
    //     //                 // translations.push(B3::new_from_element(&mut reader, element)?);
    //     //                 // B3::from(element);
    //     //                 // let b3: B3 = from_str("aaa").map_err(|err| todo!())?;
    //     //                 // let b3: B3 = from_reader(reader.clone().into_inner()).map_err(|err| {
    //     //                 //     error!("error: {}", err);
    //     //                 //     // todo!()
    //     //                 //     Error::Unknown
    //     //                 // })?;
    //     //                 // let span = reader.read_to_end(element.name()).map_err(|err| todo!())?;
    //     //                 // let text = reader.decoder().decode(&reader.into_inner()[span]);

    //     //                 // FAIL: this contains only the inner part
    //     //                 // <s1 />
    //     //                 // <p3 w="113" h="22" ImageIndex="24" Transparent="1" />
    //     //                 // <m8 u="9" />
    //     //                 // <s2 />
    //     //                 // <m9 u="&#60;b&#62;#9 &#60;/b&#62; Joystick button : #2" />
    //     //                 // <g2 Left="0" Top="4,22046999999998" Width="718,1107" Height="1,13385826771654"
    //     //                 //     ShowHint="false" BeginColor="12632256" Style="gsHorizontal" Color="10526880" />")
    //     //                 // let text = reader.read_text(element.name()).unwrap();
    //     //                 // let end = element.name().to_end().into_owned();
    //     //                 // let text = reader.read_to_end(element.name()).unwrap();

    //     //                 // let b3: B3 = from_str(&text).map_err(|err| todo!())?;

    //     //                 // serde deserialization
    //     //                 let mut m8_buf = Vec::new();
    //     //                 reader
    //     //                     .read_event_into(&mut m8_buf)
    //     //                     .map_err(|err| todo!())
    //     //                     .unwrap();
    //     //                 let m8_str = String::from_utf8_lossy(&m8_buf);
    //     //                 let m8: B3 = from_str(&m8_str).unwrap();
    //     //                 log::debug!("{:?}", m8);

    //     //                 // todo!()
    //     //             }
    //     //             _ => (),
    //     //         },

    //     //         Event::Eof => break, // exits the loop when reaching end of file
    //     //         _ => (),             // There are `Event` types not considered here
    //     //     }
    //     // }

    //     // Ok(VkbReport { previewpages })
    // }

    /// Return only the b2/b3 list of fields from the VKB report
    pub(super) fn get_all_buttons(&self) -> Vec<VkbXmlButton> {
        let mut vkb_buttons = vec![];

        for page in &self.previewpages.page0 {
            for page_item in &page.b {
                match page_item {
                    Page0Item::B2(b2) => vkb_buttons.push(VkbXmlButton::B2(b2.clone())),
                    Page0Item::B3(b3) => vkb_buttons.push(VkbXmlButton::B3(b3.clone())),
                    _ => {}
                }
            }
        }

        vkb_buttons
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_m7() {
        let xml_str = r#"
        <m7 h="20,59895" u="&#60;b&#62;#9 (Fire 2-nd stage) &#60;/b&#62;&#60;b&#62;- Button with momentary action&#60;/b&#62;" />
        "#;

        quick_xml::de::from_str::<M7>(xml_str).unwrap();
    }

    #[test]
    fn test_parse_b2() {
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
    fn test_parse_m8() {
        let xml_str = r#"
        <m8 u="95" />
        "#;

        quick_xml::de::from_str::<M8>(xml_str).unwrap();
    }

    #[test]
    fn test_parse_m9() {
        let xml_str = r#"
        <m9 u="&#60;b&#62;#95 &#60;/b&#62; Joystick button : #95" />
        "#;

        quick_xml::de::from_str::<M9>(xml_str).unwrap();
    }

    #[test]
    fn test_parse_b3() {
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
    fn test_parse_page0() {
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
    fn test_parse_report_full_simplified() {
        assert!(VkbReport::new(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/data/vkb_report_simplified.fp3"
            )
            .into(),
        )
        .is_ok());
    }

    #[test]
    fn test_parse_report_full_full_r() {
        assert!(VkbReport::new(
            concat!(env!("CARGO_MANIFEST_DIR"), "/bindings/vkb_report_R.fp3").into(),
        )
        .is_ok());
    }

    #[test]
    fn test_parse_report_full_full_l() {
        assert!(VkbReport::new(
            concat!(env!("CARGO_MANIFEST_DIR"), "/bindings/vkb_report_L.fp3").into(),
        )
        .is_ok());
    }

    #[test]
    #[ignore = "VKB report merging not supported anymore, for now"]
    fn test_parse_report_full_b2_merging() {
        assert!(VkbReport::new(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/data/vkb_report_test_b2_merging.fp3"
            )
            .into(),
        )
        .is_ok());
    }
}
