use crate::Bytes;

use super::{EofHeader, TypesSection};

#[derive(Default, Clone, Debug)]
pub struct EofBody {
    types_section: Vec<TypesSection>,
    code_section: Vec<Bytes>,
    container_section: Vec<Bytes>,
    data_section: Bytes,
    is_data_filled: bool,
}

impl EofBody {
    pub fn decode(input: &Bytes, header: &EofHeader) -> Result<Self, ()> {
        let header_len = header.len();
        let partial_body_len = header.sum_code_sizes + header.sum_container_sizes;
        let full_body_len = partial_body_len + header.data_size as usize;

        if input.len() < header_len + partial_body_len {
            return Err(());
        }

        if input.len() > header_len + full_body_len {
            return Err(());
        }

        let mut body = EofBody::default();

        let mut types_input = &input[header_len..];
        for _ in 0..header.types_items() {
            let (types_section, local_input) = TypesSection::decode(types_input)?;
            types_input = local_input;
            body.types_section.push(types_section);
        }

        // extract code section
        let mut start = header_len + header.types_size as usize;
        for size in header.code_sizes.iter().map(|x| *x as usize) {
            body.code_section.push(input.slice(start..size));
            start += size;
        }

        // extract container section
        for size in header.container_sizes.iter().map(|x| *x as usize) {
            body.container_section.push(input.slice(start..size));
            start += size;
        }

        body.data_section = input.slice(start..);
        body.is_data_filled = body.data_section.len() == header.data_size as usize;

        Ok(body)
    }
}
