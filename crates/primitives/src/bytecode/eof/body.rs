use super::{Eof, EofDecodeError, EofHeader, TypesSection};
use crate::Bytes;
use std::vec::Vec;

/// EOF container body.
///
/// Contains types, code, container and data sections.
///
/// Can be used to create a new EOF container using the [`into_eof`](EofBody::into_eof) method.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EofBody {
    pub types_section: Vec<TypesSection>,
    pub code_section: Vec<Bytes>,
    pub container_section: Vec<Bytes>,
    pub data_section: Bytes,
    pub is_data_filled: bool,
}

impl EofBody {
    /// Returns the code section at the given index.
    pub fn code(&self, index: usize) -> Option<&Bytes> {
        self.code_section.get(index)
    }

    /// Creates an EOF container from this body.
    pub fn into_eof(self) -> Eof {
        // TODO add bounds checks.
        let header = EofHeader {
            types_size: self.types_section.len() as u16 * 4,
            code_sizes: self.code_section.iter().map(|x| x.len() as u16).collect(),
            container_sizes: self
                .container_section
                .iter()
                .map(|x| x.len() as u16)
                .collect(),
            data_size: self.data_section.len() as u16,
            sum_code_sizes: self.code_section.iter().map(|x| x.len()).sum(),
            sum_container_sizes: self.container_section.iter().map(|x| x.len()).sum(),
        };
        let mut buffer = Vec::new();
        header.encode(&mut buffer);
        self.encode(&mut buffer);
        Eof {
            header,
            body: self,
            raw: buffer.into(),
        }
    }

    /// Encodes this body into the given buffer.
    pub fn encode(&self, buffer: &mut Vec<u8>) {
        for types_section in &self.types_section {
            types_section.encode(buffer);
        }

        for code_section in &self.code_section {
            buffer.extend_from_slice(code_section);
        }

        for container_section in &self.container_section {
            buffer.extend_from_slice(container_section);
        }

        buffer.extend_from_slice(&self.data_section);
    }

    /// Decodes an EOF container body from the given buffer and header.
    pub fn decode(input: &Bytes, header: &EofHeader) -> Result<Self, EofDecodeError> {
        let header_len = header.size();
        let partial_body_len =
            header.sum_code_sizes + header.sum_container_sizes + header.types_size as usize;
        let full_body_len = partial_body_len + header.data_size as usize;

        if input.len() < header_len + partial_body_len {
            return Err(EofDecodeError::MissingBodyWithoutData);
        }

        if input.len() > header_len + full_body_len {
            return Err(EofDecodeError::DanglingData);
        }

        let mut body = EofBody::default();

        let mut types_input = &input[header_len..];
        for _ in 0..header.types_count() {
            let (types_section, local_input) = TypesSection::decode(types_input)?;
            types_input = local_input;
            body.types_section.push(types_section);
        }

        // extract code section
        let mut start = header_len + header.types_size as usize;
        for size in header.code_sizes.iter().map(|x| *x as usize) {
            body.code_section.push(input.slice(start..start + size));
            start += size;
        }

        // extract container section
        for size in header.container_sizes.iter().map(|x| *x as usize) {
            body.container_section
                .push(input.slice(start..start + size));
            start += size;
        }

        body.data_section = input.slice(start..);
        body.is_data_filled = body.data_section.len() == header.data_size as usize;

        Ok(body)
    }
}
