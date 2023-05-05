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
  - on another layer: draw text field directly on top of the corresponding physical buttons
- an exported `xml` eg `layout_vkb_custom_v1_exported.xml` from Star Citizen containing the keybinds
- example: if the keybind in `xml` is `js1_button5`, the `pdf` MUST have a label `button5`(or `js1_button5`)

Why is the setup so complicated?

Because apparently neither Draw, Inkscape nor Gimp can handle component reusability.
So we do it programatically.

Yes `svg` has cloning, but a clone is a single element.
ie if a clone a "4 way hats" with contains 4 separate IDs/labels matching the keybinds -> we end up we a single `use`
which means we CAN NOT bind it properly.
Possibly could be done with Illustrator("Dynamic Symbols") but not Open Source nor Free...