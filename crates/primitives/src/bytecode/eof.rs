mod body;
mod decode_helpers;
mod header;
mod types_section;

use alloy_primitives::Bytes;
pub use body::EofBody;
pub use header::EofHeader;
pub use types_section::TypesSection;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Eof {
    pub header: EofHeader,
    pub body: EofBody,
    pub raw: Option<Bytes>,
}

impl Eof {
    /// Returns len of the header and body in bytes.
    pub fn len(&self) -> usize {
        self.header.len() + self.header.body_len()
    }

    pub fn raw(&self) -> Option<Bytes> {
        self.raw.clone()
    }

    pub fn decode(raw: Bytes) -> Result<Self, ()> {
        let (header, _) = EofHeader::decode(&raw)?;
        let body = EofBody::decode(&raw, &header)?;
        Ok(Self {
            header,
            body,
            raw: Some(raw),
        })
    }

    /// TODO implement it.
    pub fn push_aux_data(&mut self, _aux_data: Bytes) {
        // Need to modify/replace raw Bytes, and recalculate body sections.
        // We can be little wasteful here and just replace the raw Bytes and
        // data section in the body. Other sections would still pin old raw Bytes.
    }
}
