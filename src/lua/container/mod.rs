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
                    .map(|i| ItemStack { inner: i.clone() })
                    .collect::<Vec<_>>()
            }))
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        m.add_method("click", |_, this, operation: Table| {
            this.inner.click(click_operation_from_table(operation)?);
            Ok(())
        });
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
                    .map(|i| ItemStack { inner: i.clone() })
                    .collect::<Vec<_>>()
            }))
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        m.add_method("close", |_, this, (): ()| {
            this.inner.close();
            Ok(())
        });

        m.add_method("click", |_, this, operation: Table| {
            this.inner.click(click_operation_from_table(operation)?);
            Ok(())
        });
    }
}

fn click_operation_from_table(o: Table) -> Result<ClickOperation> {
    Ok(match o.get("type")? {
        0 => ClickOperation::Pickup(PickupClick::Left {
            slot: o.get("slot")?,
        }),
        1 => ClickOperation::Pickup(PickupClick::Right {
            slot: o.get("slot")?,
        }),
        2 => ClickOperation::Pickup(PickupClick::LeftOutside),
        3 => ClickOperation::Pickup(PickupClick::RightOutside),
        5 => ClickOperation::QuickMove(QuickMoveClick::Right {
            slot: o.get("slot")?,
        }),
        6 => ClickOperation::Swap(SwapClick {
            source_slot: o.get("source_slot")?,
            target_slot: o.get("target_slot")?,
        }),
        7 => ClickOperation::Clone(CloneClick {
            slot: o.get("slot")?,
        }),
        8 => ClickOperation::Throw(ThrowClick::Single {
            slot: o.get("slot")?,
        }),
        9 => ClickOperation::Throw(ThrowClick::All {
            slot: o.get("slot")?,
        }),
        10 => ClickOperation::QuickCraft(QuickCraftClick {
            kind: match o.get("kind").unwrap_or_default() {
                1 => QuickCraftKind::Right,
                2 => QuickCraftKind::Middle,
                _ => QuickCraftKind::Left,
            },
            status: match o.get("status").unwrap_or_default() {
                1 => QuickCraftStatus::Add {
                    slot: o.get("slot")?,
                },
                2 => QuickCraftStatus::End,
                _ => QuickCraftStatus::Start,
            },
        }),
        11 => ClickOperation::PickupAll(PickupAllClick {
            slot: o.get("slot")?,
            reversed: o.get("reversed").unwrap_or_default(),
        }),
        _ => ClickOperation::QuickMove(QuickMoveClick::Left {
            slot: o.get("slot")?,
        }),
    })
}
