use azalea::inventory::components::{CustomName, Damage, Food, MaxDamage};
use mlua::{UserData, UserDataFields, UserDataMethods};

pub struct ItemStack {
    pub inner: azalea::inventory::ItemStack,
}

impl From<azalea::inventory::ItemStack> for ItemStack {
    fn from(inner: azalea::inventory::ItemStack) -> Self {
        Self { inner }
    }
}

impl UserData for ItemStack {
    fn add_fields<F: UserDataFields<Self>>(f: &mut F) {
        f.add_field_method_get("is_empty", |_, this| Ok(this.inner.is_empty()));
        f.add_field_method_get("is_present", |_, this| Ok(this.inner.is_present()));
        f.add_field_method_get("count", |_, this| Ok(this.inner.count()));
        f.add_field_method_get("kind", |_, this| Ok(this.inner.kind().to_string()));
        f.add_field_method_get("custom_name", |_, this| {
            Ok(this.inner.as_present().map(|data| {
                data.components
                    .get::<CustomName>()
                    .map(|c| c.name.to_string())
            }))
        });
        f.add_field_method_get("damage", |_, this| {
            Ok(this
                .inner
                .as_present()
                .map(|data| data.components.get::<Damage>().map(|d| d.amount)))
        });
        f.add_field_method_get("max_damage", |_, this| {
            Ok(this
                .inner
                .as_present()
                .map(|data| data.components.get::<MaxDamage>().map(|d| d.amount)))
        });

        f.add_field_method_get("food", |lua, this| {
            Ok(
                if let Some(food) = this
                    .inner
                    .as_present()
                    .and_then(|data| data.components.get::<Food>())
                {
                    let table = lua.create_table()?;
                    table.set("nutrition", food.nutrition)?;
                    table.set("saturation", food.saturation)?;
                    table.set("can_always_eat", food.can_always_eat)?;
                    table.set("eat_seconds", food.eat_seconds)?;
                    Some(table)
                } else {
                    None
                },
            )
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(m: &mut M) {
        m.add_method_mut("split", |_, this, count: u32| {
            Ok(ItemStack::from(this.inner.split(count)))
        });
        m.add_method_mut("update_empty", |_, this, (): ()| {
            this.inner.update_empty();
            Ok(())
        });
    }
}
