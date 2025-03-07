pub mod click;
pub mod item_stack;

use azalea::container::{ContainerHandle, ContainerHandleRef};
use click::operation_from_table;
use item_stack::ItemStack;
use mlua::{Table, UserData, UserDataFields, UserDataMethods};

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
                    .click(operation_from_table(operation, operation_type)?);
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
                    .click(operation_from_table(operation, operation_type)?);
                Ok(())
            },
        );
    }
}
