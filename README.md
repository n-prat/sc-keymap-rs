#

Small util allowing to print the Star Citizen joystick keybinds,
directly updated by reading the exported `layout.xml`.

## Setup

~~- an editable `pdf` (ie a form)~~
  ~~- MUST have all the required text forms with the correct labels~~
  ~~- DO NOT draw buttons etc on this pdf; this will be done programatically~~
  ~~- the pdf(odg, or other image format) MUST ONLY contain forms; each form NAME(NOT "label") MUST be a button/axis eg `button1`~~
- ALTERNATIVE/WIP Prepare a `svg`
  - the joystick image should be in a separate layer, alone; no buttons, labels, text fields, connectors, etc
  - TODO on another layer: draw text field directly on top of the corresponding physical buttons
    - the text field `id` MUST be eg `button123`
    - the text field `label` does not really matter, but it it easier if it is the same as `id`
    - TODO? improve? otehr way? For now the text `stroke` SHOULD be transparent == nothing
    - but the text MUST be visible! Else the bbox computation won't work.
  - NOTE: for now the layers and visibility of the buttons/texts do not matter
- an exported `xml` eg `layout_vkb_custom_v1_exported.xml` from Star Citizen containing the keybinds
- example: if the keybind in `xml` is `js1_button5`, the `pdf` MUST have a label `button5`(or `js1_button5`)

Why is the setup so complicated?

Because apparently neither Draw, Inkscape nor Gimp can handle component reusability.
So we do it programatically.

Yes `svg` has cloning, but a clone is a single element.
ie if a clone a "4 way hats" with contains 4 separate IDs/labels matching the keybinds -> we end up we a single `use`
which means we CAN NOT bind it properly.
Possibly could be done with Illustrator("Dynamic Symbols") but not Open Source nor Free...

### VKB bindings export

Use "save" not "export". You want a `.fp3` file which is machine readable, the `export as pdf` function is not!

**IMPORTANT** you MUST make sure the report is ONE page only; check "Page settings" button, and replace 29,7cm height by eg 300+cm

TODO[page0]? support pagination? But is it really worth to preprocess to merge `b2` split on multiple pages and all the code that comes with it instead of exporting all on one page?

### Run

`RUST_LOG=info cargo run -- --sc-mapping ./bindings/layout_vkb_exported.xml --vkb-report-path ./bindings/vkb_report_R.fp3 --vkb-user-provided-data-path ./data/vkb_user_provided_data.csv --sc-bindings-to-ignore-path ./bindings/sc_duplicates_to_ignore.csv --vkb-template-params-path ./data/vkb_template_params_right.json`

`RUST_LOG=info cargo run -- --sc-mapping ./bindings/layout_vkb_exported.xml --vkb-report-path ./bindings/vkb_report_L.fp3 --vkb-user-provided-data-path ./data/vkb_user_provided_data.csv --sc-bindings-to-ignore-path ./bindings/sc_duplicates_to_ignore.csv --vkb-template-params-path ./data/vkb_template_params_left.json`

### Known Issues

#### Missing text

If no text is rendered:

- download and decompress from eg https://github.com/RazrFalcon/resvg/releases
- `./resvg --list-fonts path_to_a.svg aaa.png`
  - It should display a bunch of errors like `Warning (in usvg_text_layout:658): No match for 'Times New Roman' font-famiy.`
  - It should also display a bunch of info at the start like `/usr/share/fonts/gnu-free/FreeSerifBold.otf: 'FreeSerif (English, United States)', 0, Normal, 700, Normal`
  - use eg `FreeSerif` as font-family option `resvg::usvg::Options.font_family`