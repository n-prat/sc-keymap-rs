//! This is how we represent both Physical and logical buttons
//! They are usually obtained after parsing a joystick configuration directly; NOT from exported game mapping.
//!

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

/// This SHOULD(is) pretty generic; ie not really related to VKB
#[derive(PartialEq, Clone)]
pub(crate) enum ButtonKind {
    /// This matches a "b2" field in xml
    /// To get the ID we need to parse the desc...
    Physical {
        id: u8,
        kind: PhysicalButtonKind,
        info: String,
        extended_desc: String,
    },
    /// Virtual/Logical
    /// This matches a "b3" field in xml
    /// In this case the "m8" field directly contains the ID, no parsing needed.
    /// The "m9" SHOULD also contain the same ID in the desc.
    Virtual { id: u8 },
}

impl core::fmt::Debug for ButtonKind {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            ButtonKind::Physical {
                id,
                kind,
                info,
                extended_desc,
            } => write!(
                f,
                "PhysicalButton [{} {} {} ({kind:?})]",
                id, info, extended_desc
            ),
            ButtonKind::Virtual { id } => {
                write!(f, "VirtualButton [{}]", id)
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
    pub(crate) kind: ButtonKind,
}

impl core::fmt::Debug for Button {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        // Skip self, and forward directly to `ButtonKind`
        write!(f, "{:?}", self.kind)
    }
}

impl Button {
    pub(super) fn get_id(&self) -> u8 {
        match &self.kind {
            ButtonKind::Physical {
                id,
                kind: _,
                info: _,
                extended_desc: _,
            } => *id,
            ButtonKind::Virtual { id } => *id,
        }
    }
}
