use std::path::PathBuf;

use maud::DOCTYPE;
use maud::{html, Markup, PreEscaped};
use serde::Deserialize;
use serde::Serialize;

use crate::sc::parse_keybind_xml;
use crate::template_gen::read_json_template;
use crate::vkb::vkb_button::JoystickButtonsMapping;
use crate::Error;

pub fn generate_html(
    game_buttons_mapping: &parse_keybind_xml::GameButtonsMapping,
    joysticks_mappings: &JoystickButtonsMapping,
    json_template_path: &PathBuf,
    game_device_id: u8,
) -> Result<(), Error> {
    let mut json_params = read_json_template(json_template_path)?;

    // let buttons: Vec<Markup> = params.buttons_params.iter().map(|button| {
    //     html! {
    //         div style=(format!("position: absolute; left: {}px; top: {}px;", button.position.0, button.position.1)) title=(button.user_desc.clone()) {
    //             (button.physical_names.join(", "))
    //         }
    //     }
    // }).collect();

    let buttons: Vec<Markup> = vec![];

    let output = html! {
        (DOCTYPE)  // <!DOCTYPE html>
        html {
            head {
                title { "Joystick Template" }
                style {
                    (PreEscaped("
                    body {
                        display: flex;
                        justify-content: center;
                        align-items: center;
                        height: 1500px;
                    }
                    #searchBox {
                        position: absolute;
                        top: 100px;
                        left: 100px;
                        z-index: 3;
                    }
                    #image-joystick-main {
                        // position: absolute;
                        // top: 200;
                        // left: 500;
                        width: 1200px;
                        height: 1200px;
                        position: absolute;
                        top: 400px;
                        left: 800px;
                        z-index: 1;
                    }
                    #image-joystick-small  {
                        // position: absolute;
                        // top: 0;
                        // left: 0;
                        width: 800px;
                        height: 800px;
                        position: absolute;
                        z-index: 2;
                        top: 800px;
                        left: 50px;
                    }
                    "))
                }
            }
            body {
                input type="text" id="searchBox" onkeyup="searchFunction()" placeholder="Search for buttons..";

                div class="image-container" {
                    img id="image-joystick-small" src="./data/R_side_view.jpg";
                    img id="image-joystick-main" src="./data/EVO_R_official.jpg";
                }

                @for button in buttons {
                    (button)
                }

                script {
                    (PreEscaped("
                    function searchFunction() {
                      var input, filter, divs, i, txtValue;
                      input = document.getElementById('searchBox');
                      filter = input.value.toUpperCase();
                      divs = document.getElementsByTagName('div');
                      for (i = 0; i < divs.length; i++) {
                        txtValue = divs[i].textContent || divs[i].innerText;
                        if (txtValue.toUpperCase().indexOf(filter) > -1) {
                          divs[i].style.display = \"\";
                        } else {
                          divs[i].style.display = \"none\";
                        }
                      }
                    }
                    "))
                }
            }
        }
    };

    json_params.path_to_output_png.set_extension("html");
    println!("Writing to {}", json_params.path_to_output_png.display());
    std::fs::write(json_params.path_to_output_png, output.into_string())
        .map_err(|err| Error::Other("std::fs::write {err}".to_string()))?;

    Ok(())
}
