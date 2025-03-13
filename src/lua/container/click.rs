use azalea::inventory::operations::{
    ClickOperation, CloneClick, PickupAllClick, PickupClick, QuickCraftClick, QuickCraftKind,
    QuickCraftStatus, QuickMoveClick, SwapClick, ThrowClick,
};
use mlua::{Result, Table};

pub fn operation_from_table(op: &Table, op_type: Option<u8>) -> Result<ClickOperation> {
    Ok(match op_type.unwrap_or_default() {
        0 => ClickOperation::Pickup(PickupClick::Left {
            slot: op.get("slot")?,
        }),
        1 => ClickOperation::Pickup(PickupClick::Right {
            slot: op.get("slot")?,
        }),
        2 => ClickOperation::Pickup(PickupClick::LeftOutside),
        3 => ClickOperation::Pickup(PickupClick::RightOutside),
        5 => ClickOperation::QuickMove(QuickMoveClick::Right {
            slot: op.get("slot")?,
        }),
        6 => ClickOperation::Swap(SwapClick {
            source_slot: op.get("source_slot")?,
            target_slot: op.get("target_slot")?,
        }),
        7 => ClickOperation::Clone(CloneClick {
            slot: op.get("slot")?,
        }),
        8 => ClickOperation::Throw(ThrowClick::Single {
            slot: op.get("slot")?,
        }),
        9 => ClickOperation::Throw(ThrowClick::All {
            slot: op.get("slot")?,
        }),
        10 => ClickOperation::QuickCraft(QuickCraftClick {
            kind: match op.get("kind").unwrap_or_default() {
                1 => QuickCraftKind::Right,
                2 => QuickCraftKind::Middle,
                _ => QuickCraftKind::Left,
            },
            status: match op.get("status").unwrap_or_default() {
                1 => QuickCraftStatus::Add {
                    slot: op.get("slot")?,
                },
                2 => QuickCraftStatus::End,
                _ => QuickCraftStatus::Start,
            },
        }),
        11 => ClickOperation::PickupAll(PickupAllClick {
            slot: op.get("slot")?,
            reversed: op.get("reversed").unwrap_or_default(),
        }),
        _ => ClickOperation::QuickMove(QuickMoveClick::Left {
            slot: op.get("slot")?,
        }),
    })
}
