//! This is how we represent both Physical and logical buttons
//! They are usually obtained after parsing a joystick configuration directly; NOT from exported game mapping.
//!

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ButtonError {
    #[error("could not find info_or_user_desc : `{info_or_user_desc}`")]
    ButtonNotFound { info_or_user_desc: String },
}

#[derive(PartialEq, Clone)]
pub(crate) enum TempoKind {
    /// Short+Long press
    /// "second line pulse length is equal to T_Tgl value in no matter to real depressing time"
    _Tempo1,
    /// Short+Long press
    /// "second line pulse length is equal to button depressing time"
    Tempo2 {
        button_id_short: u8,
        button_id_long: u8,
    },
    /// Short+Long press+Double press
    Tempo3 {
        button_id_short: u8,
        button_id_long: u8,
        button_id_double: u8,
    },
}

impl core::fmt::Debug for TempoKind {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            TempoKind::_Tempo1 => write!(f, "Tempo1[]"),
            TempoKind::Tempo2 {
                button_id_short,
                button_id_long,
            } => write!(f, "Tempo2[{}/{}]", button_id_short, button_id_long),
            TempoKind::Tempo3 {
                button_id_short,
                button_id_long,
                button_id_double,
            } => write!(
                f,
                "Tempo3[{}/{}/{}]",
                button_id_short, button_id_long, button_id_double
            ),
        }
    }
}

#[derive(PartialEq, Clone)]
pub(crate) enum ShiftKind {
    Shift1 {
        button_id_shift1: u8,
    },
    Shift2 {
        button_id_shift2: u8,
    },
    Shift12 {
        button_id_shift1: u8,
        button_id_shift2: u8,
    },
}

impl core::fmt::Debug for ShiftKind {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            ShiftKind::Shift1 { button_id_shift1 } => write!(f, "Shift1[{}]", button_id_shift1),
            ShiftKind::Shift2 { button_id_shift2 } => {
                write!(f, "Shift2[{}]", button_id_shift2)
            }
            ShiftKind::Shift12 {
                button_id_shift1,
                button_id_shift2,
            } => write!(f, "Shift12[{}/{}]", button_id_shift1, button_id_shift2),
        }
    }
}

/// For now this is based on VKB buttons, but it should be applicable to all joysticks
#[derive(PartialEq, Debug, Clone)]
pub(crate) enum PhysicalButtonKind {
    /// The standard, basic button with no SHIT, or anything particular
    /// VKB = "Button with momentary action"
    Momentary {
        shift: Option<ShiftKind>,
    },
    /// This is the wheel on the bottom right of the stick (one per stick)
    Encoder,
    Tempo(TempoKind),
    /// The SHIFT1 = ALT button 1
    Shift1,
    /// The SHIFT2 = ALT button 2
    Shift2,
    /// "Point of view Switch"
    /// eg "POV1  Up", "POV1  Left", etc
    Pov {
        direction: String,
    },
    /// "No defined function"
    Undefined,
    /// "(Ministick push) Microstick Mode Switch"
    MicrostickModeSwitch,
}

#[derive(PartialEq, Clone, Debug)]
pub(crate) enum VirtualTempoKind {
    Short,
    Long,
    Double,
}

#[derive(PartialEq, Clone, Debug)]
pub(crate) enum VirtualShiftKind {
    Shift1,
    Shift2,
}

/// To properly print the final keybinds we need to know how a button is "reached".
/// eg is it a standard press, a long press, a double press, is SHIFT1 or SHIFT2 required ?
///
#[derive(PartialEq, Clone, Debug)]
pub(crate) enum VirtualButtonKind {
    /// From `PhysicalButtonKind::Momentary`
    Momentary(Option<VirtualShiftKind>),
    /// From `PhysicalButtonKind::Tempo`
    Tempo(VirtualTempoKind),
}

/// Intermediate struct only needed because that way we can have eg `Vec<VirtualButton>`
/// which is better than having `Vec<ButtonKind>` when we known they are all `Virtual` variants
///
#[derive(PartialEq, Clone)]
pub(crate) struct VirtualButton {
    pub(crate) id: u8,
    pub(crate) kind: VirtualButtonKind,
}

impl VirtualButton {
    pub(crate) fn get_id(&self) -> &u8 {
        &self.id
    }
}

impl core::fmt::Debug for VirtualButton {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match &self.kind {
            VirtualButtonKind::Momentary(shift) => {
                write!(f, "VirtualButton [{} ({:?})]", self.id, shift)
            }
            VirtualButtonKind::Tempo(tempo_kind) => {
                write!(f, "VirtualButton [{} ({:?})]", self.id, tempo_kind,)
            }
        }
    }
}

// impl TryFrom<Button> for VirtualButton {
//     type Error = ButtonError;

//     fn try_from(value: Button) -> Result<Self, Self::Error> {
//         match value {}
//     }
// }

/// Intermediate struct only needed because that way we can have eg `Vec<VirtualButton>`
/// which is better than having `Vec<ButtonKind>` when we known they are all `Virtual` variants
///
#[derive(PartialEq, Clone)]
pub(crate) struct PhysicalButton {
    id: u8,
    kind: PhysicalButtonKind,
    info: String,
    extended_desc: String,
    user_desc: String,
}

impl PhysicalButton {
    pub(crate) fn get_id(&self) -> &u8 {
        &self.id
    }

    pub(crate) fn get_info(&self) -> &String {
        &self.info
    }

    pub(crate) fn get_user_desc(&self) -> &String {
        &self.user_desc
    }

    pub(crate) fn get_kind(&self) -> &PhysicalButtonKind {
        &self.kind
    }
}

/// This SHOULD(is) pretty generic; ie not really related to VKB
#[derive(PartialEq, Clone)]
pub(crate) enum ButtonKind {
    /// This matches a "b2" field in xml
    /// To get the ID we need to parse the desc...
    Physical(PhysicalButton),
    /// Virtual/Logical
    /// This matches a "b3" field in xml
    /// In this case the "m8" field directly contains the ID, no parsing needed.
    /// The "m9" SHOULD also contain the same ID in the desc.
    Virtual(VirtualButton),
}

impl core::fmt::Debug for ButtonKind {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            ButtonKind::Physical(button) => write!(
                f,
                "PhysicalButton [{} {} {} {} ({:?})]",
                button.id, button.info, button.extended_desc, button.user_desc, button.kind
            ),
            ButtonKind::Virtual(button) => {
                write!(f, "VirtualButton [{:?}]", button)
            }
        }
    }
}

/// This is NOT from the xml, this is the end result.
/// We construct the final "buttons" by parsing the "xml_desc" field eg:
///
/// <b>#6 </b> Joystick button : #52
/// <b>#6 (F1) </b><b>TEMPO </b>\r\nVirtual button Short #6\r\nVirtual button Long #96
/// etc
#[derive(PartialEq, Clone)]
pub(crate) struct Button {
    kind: ButtonKind,
}

impl core::fmt::Debug for Button {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        // Skip self, and forward directly to `ButtonKind`
        write!(f, "{:?}", self.kind)
    }
}

impl Button {
    pub(super) fn new_physical(
        id: u8,
        kind: PhysicalButtonKind,
        info: String,
        extended_desc: String,
        user_desc: String,
    ) -> Button {
        Button {
            kind: ButtonKind::Physical(PhysicalButton {
                id,
                kind,
                info,
                extended_desc,
                user_desc,
            }),
        }
    }

    pub(super) fn get_id(&self) -> u8 {
        match &self.kind {
            ButtonKind::Physical(button) => *button.get_id(),
            ButtonKind::Virtual(button) => *button.get_id(),
        }
    }

    pub(super) fn set_user_desc(&mut self, new_user_desc: &str) {
        match &mut self.kind {
            ButtonKind::Physical(button) => button.user_desc = new_user_desc.to_string(),
            ButtonKind::Virtual(_) => {}
        }
    }

    pub(crate) fn get_kind(&self) -> &ButtonKind {
        &self.kind
    }
}
