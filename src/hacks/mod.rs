#![allow(clippy::needless_pass_by_value)]

pub mod anti_knockback;

use anti_knockback::anti_knockback;
use azalea::{connection::read_packets, movement::handle_knockback};
use bevy_app::{App, Plugin, PreUpdate};
use bevy_ecs::schedule::IntoScheduleConfigs;

pub struct HacksPlugin;

impl Plugin for HacksPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            anti_knockback.after(read_packets).before(handle_knockback),
        );
    }
}
