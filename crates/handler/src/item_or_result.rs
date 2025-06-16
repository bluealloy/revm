use crate::evm::NewFrameTr;

#[derive(Clone, Debug)]
pub enum ItemOrResult<ITEM, RES> {
    Item(ITEM),
    Result(RES),
}

impl<ITEM, RES> ItemOrResult<ITEM, RES> {
    pub fn map_frame<OITEM>(self, f: impl FnOnce(ITEM) -> OITEM) -> ItemOrResult<OITEM, RES> {
        match self {
            ItemOrResult::Item(item) => ItemOrResult::Item(f(item)),
            ItemOrResult::Result(result) => ItemOrResult::Result(result),
        }
    }

    pub fn map_result<ORES>(self, f: impl FnOnce(RES) -> ORES) -> ItemOrResult<ITEM, ORES> {
        match self {
            ItemOrResult::Item(item) => ItemOrResult::Item(item),
            ItemOrResult::Result(result) => ItemOrResult::Result(f(result)),
        }
    }
}

impl<ITEM, RES> ItemOrResult<ITEM, RES> {
    pub fn is_result(&self) -> bool {
        matches!(self, ItemOrResult::Result(_))
    }

    pub fn is_item(&self) -> bool {
        matches!(self, ItemOrResult::Item(_))
    }
}

pub type NewFrameTrInitOrResult<FRAME> =
    ItemOrResult<<FRAME as NewFrameTr>::FrameInit, <FRAME as NewFrameTr>::FrameResult>;
