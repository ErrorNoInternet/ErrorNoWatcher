pub mod item_stack;

use azalea::{
    container::{ContainerHandle, ContainerHandleRef},
    inventory::operations::{
        ClickOperation, CloneClick, PickupAllClick, PickupClick, QuickCraftClick, QuickCraftKind,
        QuickCraftStatus, QuickMoveClick, SwapClick, ThrowClick,
    },
};
use item_stack::ItemStack;
use mlua::{Result, Table, UserData, UserDataFields, UserDataMethods};

pub struct Container {
    pub inner: ContainerHandle,
}

impl UserData for Container {
    fn add_fields<F: UserDataFields<Self>>(f: &mut F) {
        f.add_field_method_get("id", |_, this| Ok(this.inner.id()));

        f.add_field_method_get("menu", |_, this| {
            Ok(this.inner.menu().map(|m| format!("{m:?}")))
        });

        f.add_field_method_get("contents", |_, this| {
            Ok(this.inner.contents().map(|v| {
                v.iter()
                    .map(|i| ItemStack::from(i.to_owned()))
                    .collect::<Vec<_>>()
            }))
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        m.add_method(
            "click",
            |_, this, (operation, operation_type): (Table, Option<u8>)| {
                this.inner
                    .click(click_operation_from_table(operation, operation_type)?);
                Ok(())
            },
        );
    }
}

pub struct ContainerRef {
    pub inner: ContainerHandleRef,
}

impl UserData for ContainerRef {
    fn add_fields<F: UserDataFields<Self>>(f: &mut F) {
        f.add_field_method_get("id", |_, this| Ok(this.inner.id()));

        f.add_field_method_get("menu", |_, this| {
            Ok(this.inner.menu().map(|m| format!("{m:?}")))
        });

        f.add_field_method_get("contents", |_, this| {
            Ok(this.inner.contents().map(|v| {
                v.iter()
                    .map(|i| ItemStack::from(i.to_owned()))
                    .collect::<Vec<_>>()
            }))
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        m.add_method("close", |_, this, (): ()| {
            this.inner.close();
            Ok(())
        });

        m.add_method(
            "click",
            |_, this, (operation, operation_type): (Table, Option<u8>)| {
                this.inner
                    .click(click_operation_from_table(operation, operation_type)?);
                Ok(())
            },
        );
    }
}

fn click_operation_from_table(op: Table, op_type: Option<u8>) -> Result<ClickOperation> {
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
