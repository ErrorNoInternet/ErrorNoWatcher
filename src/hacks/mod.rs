#![allow(clippy::needless_pass_by_value)]

pub mod anti_knockback;

use anti_knockback::anti_knockback;
use azalea::{movement::handle_knockback, packet::game::process_packet_events};
use bevy_app::{App, Plugin, PreUpdate};
use bevy_ecs::schedule::IntoSystemConfigs;

pub struct HacksPlugin;

impl Plugin for HacksPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            anti_knockback
                .after(process_packet_events)
                .before(handle_knockback),
        );
    }
}
