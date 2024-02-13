/// EOF Header containing
pub struct Header {
    /// Size of EOF types section.
    /// types section includes num of input and outputs and max stack size.
    pub types_size: u16,
    /// Sizes of EOF code section.
    /// Code size can't be zero.
    pub code_sizes: Vec<u16>,
    /// EOF Container size.
    /// Container size can be zero.
    pub container_sizes: Vec<u16>,
    /// EOF data size.
    pub data_size: u16,
}

impl Header {
    /// Create new EOF Header.
    pub fn new(
        types_size: u16,
        code_sizes: Vec<u16>,
        container_sizes: Vec<u16>,
        data_size: u16,
    ) -> Self {
        Self {
            types_size,
            code_sizes,
            container_sizes,
            data_size,
        }
    }
}
