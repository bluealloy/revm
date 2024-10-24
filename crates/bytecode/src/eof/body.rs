use super::{Eof, EofDecodeError, EofHeader, TypesSection};
use primitives::Bytes;
use std::vec::Vec;

/// EOF container body.
///
/// Contains types, code, container and data sections.
///
/// Can be used to create a new EOF container using the [`into_eof`](EofBody::into_eof) method.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EofBody {
    /// Code information
    pub types_section: Vec<TypesSection>,
    /// Index of the last byte of each code section.
    pub code_section: Vec<usize>,
    pub code: Bytes,
    pub container_section: Vec<Bytes>,
    pub data_section: Bytes,
    pub is_data_filled: bool,
}

impl EofBody {
    /// Returns the code section at the given index.
    pub fn code(&self, index: usize) -> Option<Bytes> {
        if index == 0 {
            // There should be at least one code section.
            return Some(self.code.slice(..self.code_section[0]));
        }
        self.code_section
            .get(index)
            .map(|end| self.code.slice(self.code_section[index - 1]..*end))
    }

    /// Creates an EOF container from this body.
    pub fn into_eof(self) -> Eof {
        // TODO add bounds checks.
        let mut prev_value = 0;
        let header = EofHeader {
            types_size: self.types_section.len() as u16 * 4,
            code_sizes: self
                .code_section
                .iter()
                .map(|x| {
                    let ret = (x - prev_value) as u16;
                    prev_value = *x;
                    ret
                })
                .collect(),
            container_sizes: self
                .container_section
                .iter()
                .map(|x| x.len() as u16)
                .collect(),
            data_size: self.data_section.len() as u16,
            sum_code_sizes: self.code.len(),
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

    /// Returns offset of the start of indexed code section.
    ///
    /// First code section starts at 0.
    pub fn eof_code_section_start(&self, idx: usize) -> Option<usize> {
        // starting code section start with 0.
        if idx == 0 {
            return Some(0);
        }
        self.code_section.get(idx - 1).cloned()
    }

    /// Encodes this body into the given buffer.
    pub fn encode(&self, buffer: &mut Vec<u8>) {
        for types_section in &self.types_section {
            types_section.encode(buffer);
        }

        buffer.extend_from_slice(&self.code);

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
        let mut code_end = 0;
        for size in header.code_sizes.iter().map(|x| *x as usize) {
            code_end += size;
            body.code_section.push(code_end);
        }
        body.code = input.slice(start..start + header.sum_code_sizes);

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
