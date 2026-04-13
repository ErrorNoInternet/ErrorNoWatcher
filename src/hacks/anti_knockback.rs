use azalea::{
    entity::Physics,
    movement::{KnockbackEvent, handle_knockback},
    prelude::*,
};
use bevy_ecs::{observer::On, query::With, system::Query};

#[derive(Component)]
pub struct AntiKnockback;

pub fn anti_knockback(
    knockback: On<KnockbackEvent>,
    entity_query: Query<(), With<AntiKnockback>>,
    handle_knockback_query: Query<&mut Physics>,
) {
    if entity_query.get(knockback.entity).is_err() {
        handle_knockback(knockback, handle_knockback_query);
    }
}
