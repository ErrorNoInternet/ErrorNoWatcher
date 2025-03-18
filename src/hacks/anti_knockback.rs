use azalea::{
    Vec3,
    movement::{KnockbackEvent, KnockbackType},
    prelude::Component,
};
use bevy_ecs::{event::EventMutator, query::With, system::Query};

#[derive(Component)]
pub struct AntiKnockback;

pub fn anti_knockback(
    mut events: EventMutator<KnockbackEvent>,
    entity_query: Query<(), With<AntiKnockback>>,
) {
    for event in events.read() {
        if entity_query.get(event.entity).is_ok() {
            event.knockback = KnockbackType::Add(Vec3::default());
        }
    }
}
