pub mod anti_knockback;

use azalea::movement::KnockbackEvent;
use bevy_app::{App, Plugin, PostStartup};
use bevy_ecs::world::World;

pub struct HacksPlugin;

impl Plugin for HacksPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, init_hacks);
    }
}

fn init_hacks(ecs: &mut World) {
    let observers = ecs
        .observers()
        .try_get_observers(ecs.event_key::<KnockbackEvent>().unwrap());
    let mut to_despawn = Vec::new();
    for (observer_entity, _) in observers.unwrap().global_observers() {
        to_despawn.push(*observer_entity);
    }
    for observer_entity in to_despawn {
        ecs.despawn(observer_entity);
    }

    ecs.add_observer(anti_knockback::anti_knockback);
}
