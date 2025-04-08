use crate::Frame;

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

pub type FrameOrResult<FRAME> = ItemOrResult<FRAME, <FRAME as Frame>::FrameResult>;
pub type FrameInitOrResult<FRAME> =
    ItemOrResult<<FRAME as Frame>::FrameInit, <FRAME as Frame>::FrameResult>;
