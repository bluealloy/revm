use crate::evm::FrameTr;

/// Represents either an item or a result.
#[derive(Clone, Debug)]
pub enum ItemOrResult<ITEM, RES> {
    /// Contains an item that needs further processing.
    Item(ITEM),
    /// Contains a final result.
    Result(RES),
}

impl<ITEM, RES> ItemOrResult<ITEM, RES> {
    /// Maps the item variant using the provided function, leaving results unchanged.
    pub fn map_frame<OITEM>(self, f: impl FnOnce(ITEM) -> OITEM) -> ItemOrResult<OITEM, RES> {
        match self {
            ItemOrResult::Item(item) => ItemOrResult::Item(f(item)),
            ItemOrResult::Result(result) => ItemOrResult::Result(result),
        }
    }

    /// Maps the result variant using the provided function, leaving items unchanged.
    pub fn map_result<ORES>(self, f: impl FnOnce(RES) -> ORES) -> ItemOrResult<ITEM, ORES> {
        match self {
            ItemOrResult::Item(item) => ItemOrResult::Item(item),
            ItemOrResult::Result(result) => ItemOrResult::Result(f(result)),
        }
    }
}

impl<ITEM, RES> ItemOrResult<ITEM, RES> {
    /// Returns true if this is a result variant.
    pub fn is_result(&self) -> bool {
        matches!(self, ItemOrResult::Result(_))
    }

    /// Returns true if this is an item variant.
    pub fn is_item(&self) -> bool {
        matches!(self, ItemOrResult::Item(_))
    }
}

/// Type alias for frame initialization or result.
pub type FrameInitOrResult<FRAME> =
    ItemOrResult<<FRAME as FrameTr>::FrameInit, <FRAME as FrameTr>::FrameResult>;
