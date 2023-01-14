use std::fmt;

pub const MARKER_SINGLE_I64: Marker = Marker::Single { value: 1 };
pub const MARKER_SINGLE_I32: Marker = Marker::Single { value: 2 };
pub const MARKER_SINGLE_I16: Marker = Marker::Single { value: 3 };
pub const MARKER_SINGLE_I8: Marker = Marker::Single { value: 4 };
pub const MARKER_SINGLE_U64: Marker = Marker::Single { value: 5 };
pub const MARKER_SINGLE_U32: Marker = Marker::Single { value: 6 };
pub const MARKER_SINGLE_U16: Marker = Marker::Single { value: 7 };
pub const MARKER_U8: u8 = 8;
pub const MARKER_SINGLE_U8: Marker = Marker::Single { value: MARKER_U8 };
pub const MARKER_SINGLE_F64: Marker = Marker::Single { value: 9 };
pub const MARKER_SINGLE_STRING: Marker = Marker::Single { value: 10 };
pub const MARKER_SINGLE_BOOL: Marker = Marker::Single { value: 11 };
pub const MARKER_SINGLE_STRUCT: Marker = Marker::Single { value: 12 };
pub const MARKER_ARRAY_ELEMENT: u8 = 0x80;

#[derive(Debug, PartialEq, Eq)]
pub enum Marker {
    Single { value: u8 },
    Sequence { element: u8 },
}

impl Marker {
    pub fn from_byte(value: u8) -> Self {
        let is_sequence = value & MARKER_ARRAY_ELEMENT > 0;

        if is_sequence {
            return Self::Sequence {
                element: value ^ MARKER_ARRAY_ELEMENT,
            };
        }

        Self::Single { value }
    }

    pub fn to_sequence(&self) -> Self {
        match *self {
            Marker::Single { value } => Marker::Sequence { element: value },
            Marker::Sequence { element } => Marker::Sequence { element },
        }
    }

    pub fn to_byte(&self) -> u8 {
        match *self {
            Marker::Single { value } => value,
            Marker::Sequence { element } => element | MARKER_ARRAY_ELEMENT,
        }
    }
}

impl fmt::Display for Marker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Single { value } => write!(f, "Single({:x})", value),
            Self::Sequence { element } => write!(f, "Sequence({:x})", element),
        }
    }
}
