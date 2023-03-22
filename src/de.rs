use std::io::BufRead;

use super::marker::{
    MARKER_SINGLE_BOOL, MARKER_SINGLE_F64, MARKER_SINGLE_I16, MARKER_SINGLE_I32, MARKER_SINGLE_I64,
    MARKER_SINGLE_I8, MARKER_SINGLE_STRING, MARKER_SINGLE_STRUCT, MARKER_SINGLE_U16,
    MARKER_SINGLE_U32, MARKER_SINGLE_U64, MARKER_SINGLE_U8, MARKER_U8,
};
use super::read_ext::ReadExt;
use super::MAX_STRING_LEN_POSSIBLE;
use super::{varint, Error, Marker, Result};

use serde::de::{self, Visitor};

pub struct Deserializer<'de> {
    // This string starts with the input data and characters are truncated off
    // the beginning as data is parsed.
    buffer: &'de mut dyn BufRead,
    header_read: bool,
}

impl<'de> Deserializer<'de> {
    pub fn from_bytes(buffer: &'de mut dyn BufRead) -> Self {
        Deserializer {
            buffer,
            header_read: false,
        }
    }
}

impl<'de> Deserializer<'de> {
    fn read_bytes(&mut self, length: usize) -> Result<Vec<u8>> {
        let mut buffer = vec![0u8; length];
        self.buffer.read_exact(&mut buffer)?;

        Ok(buffer)
    }

    fn read_varint(&mut self) -> Result<usize> {
        let varint = varint::decode(&mut self.buffer)?;

        Ok(varint)
    }

    fn read_marked_string(&mut self, length: usize) -> Result<Vec<u8>> {
        if length > MAX_STRING_LEN_POSSIBLE {
            return Err(Error::MarkedStringExceededMaxLength);
        }
        self.read_bytes(length)
    }

    fn read_varint_marked_string(&mut self) -> Result<Vec<u8>> {
        let length = self.read_varint()?;
        self.read_marked_string(length)
    }

    fn read_string(&mut self, length: usize) -> Result<String> {
        let potential_str = self.read_marked_string(length)?;
        String::from_utf8(potential_str).map_err(|_| Error::InvalidString)
    }

    fn read_marker(&mut self) -> Result<Marker> {
        let marker_value = self.buffer.read_u8()?;

        Ok(Marker::from_byte(marker_value))
    }

    fn read_expected_marker(&mut self, expected_marker: Marker) -> Result<()> {
        let actual_marker = self.read_marker()?;

        if expected_marker != actual_marker {
            return Err(Error::UnexpectedMarker(format!(
                "expected: {expected_marker}, got: {actual_marker}"
            )));
        }

        Ok(())
    }

    fn dispatch_based_on_marker<V>(&mut self, marker: Marker, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match marker {
            Marker::Sequence {
                element: element_marker,
            } => visitor.visit_seq(SeqAccess::with_varint_encoded_length(self, element_marker)?),
            MARKER_SINGLE_I64 => visitor.visit_i64(self.buffer.read_i64()?),
            MARKER_SINGLE_I32 => visitor.visit_i32(self.buffer.read_i32()?),
            MARKER_SINGLE_I16 => visitor.visit_i16(self.buffer.read_i16()?),
            MARKER_SINGLE_I8 => visitor.visit_i8(self.buffer.read_i8()?),
            MARKER_SINGLE_U64 => visitor.visit_u64(self.buffer.read_u64()?),
            MARKER_SINGLE_U32 => visitor.visit_u32(self.buffer.read_u32()?),
            MARKER_SINGLE_U16 => visitor.visit_u16(self.buffer.read_u16()?),
            MARKER_SINGLE_U8 => visitor.visit_u8(self.buffer.read_u8()?),
            MARKER_SINGLE_F64 => visitor.visit_f64(self.buffer.read_f64()?),
            MARKER_SINGLE_STRING => visitor.visit_byte_buf(self.read_varint_marked_string()?),
            MARKER_SINGLE_BOOL => visitor.visit_bool(self.buffer.read_bool()?),
            MARKER_SINGLE_STRUCT => visitor.visit_map(MapAccess::with_varint_encoded_fields(self)?),
            _ => Err(Error::UnknownMarker(marker)),
        }
    }
}

pub struct MapAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    number_of_fields: usize,
    fields_read: usize,
}

impl<'a, 'de> MapAccess<'a, 'de> {
    /// Creates a new instance of [`MapAccess`] that initializes itself by
    /// reading a varint from the reader within [`Deserializer`] for the
    /// expected number of fields.
    fn with_varint_encoded_fields(de: &'a mut Deserializer<'de>) -> Result<Self> {
        let number_of_fields = varint::decode(&mut de.buffer)?;

        Ok(MapAccess {
            de,
            number_of_fields,
            fields_read: 0,
        })
    }
}

impl<'a, 'de> serde::de::MapAccess<'de> for MapAccess<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        if self.fields_read == self.number_of_fields {
            return Ok(None);
        }

        seed.deserialize(SectionFieldNameDeserializer { de: self.de })
            .map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let value = seed.deserialize(&mut *self.de)?;
        self.fields_read += 1;

        Ok(value)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.number_of_fields)
    }
}

struct SectionFieldNameDeserializer<'a, 'de> {
    de: &'a mut Deserializer<'de>,
}

impl<'de, 'a> serde::de::Deserializer<'de> for SectionFieldNameDeserializer<'a, 'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        let field_name_length = self.de.buffer.read_u8()? as usize;
        let field_name = self.de.read_string(field_name_length)?;

        visitor.visit_string(field_name)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

pub struct SeqAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    /// How long we expect the sequence to be.
    length: usize,
    /// What kind of item we are expecting.
    element_marker: u8,
    /// How many items we already emitted.
    emitted_items: usize,
}

impl<'a, 'de> SeqAccess<'a, 'de> {
    fn with_varint_encoded_length(
        de: &'a mut Deserializer<'de>,
        element_marker: u8,
    ) -> Result<Self> {
        let length = de.read_varint()?;

        Ok(Self::with_length(de, element_marker, length))
    }

    fn with_length(de: &'a mut Deserializer<'de>, element_marker: u8, length: usize) -> Self {
        Self {
            de,
            length,
            element_marker,
            emitted_items: 0,
        }
    }
}

impl<'de, 'a> serde::de::SeqAccess<'de> for SeqAccess<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        if self.emitted_items == self.length {
            return Ok(None);
        }

        let element = seed.deserialize(SeqElementDeserializer {
            de: self.de,
            marker: self.element_marker,
        })?;
        self.emitted_items += 1;

        Ok(Some(element))
    }
}

struct SeqElementDeserializer<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    marker: u8,
}

impl<'de, 'a> serde::de::Deserializer<'de> for SeqElementDeserializer<'a, 'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        self.de
            .dispatch_based_on_marker(Marker::Single { value: self.marker }, visitor)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if !self.header_read {
            self.header_read = true;
            visitor.visit_map(MapAccess::with_varint_encoded_fields(self)?)
        } else {
            let marker = self.read_marker()?;
            self.dispatch_based_on_marker(marker, visitor)
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64
        bytes byte_buf option unit unit_struct newtype_struct seq
        tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.read_expected_marker(MARKER_SINGLE_U8)?;
        visitor.visit_char(self.buffer.read_u8()? as char)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.read_expected_marker(MARKER_SINGLE_STRING)?;
        let potential_str = self.read_varint_marked_string()?;
        visitor.visit_string(String::from_utf8(potential_str).map_err(|_| Error::InvalidString)?)
    }

    fn deserialize_tuple<V>(
        self,
        expected_length: usize,
        v: V,
    ) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        // special case tuples.
        // byte arrays and sequences are serialized as "strings" in epee-bin
        // hence, if we are told to deserialize a tuple, we check if the marker is a string, if that is the case, tell the deserializer to deserialize it as individual bytes
        match self.read_marker()? {
            MARKER_SINGLE_STRING => {
                let got_length = self.read_varint()?;

                if expected_length != got_length {
                    return Err(Error::LengthMismatch(format!(
                        "expected: {expected_length}, got: {got_length}"
                    )));
                }

                v.visit_seq(SeqAccess::with_length(self, MARKER_U8, got_length))
            }
            marker => Err(Error::TuplesOfTypeNotSupported(marker)),
        }
    }
}
