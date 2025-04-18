#![allow(clippy::needless_pass_by_value)]

use std::sync::Arc;

use azalea::{
    ecs::event::EventReader,
    packet::{
        config::ReceiveConfigPacketEvent, game::ReceiveGamePacketEvent,
        login::ReceiveLoginPacketEvent,
    },
    protocol::packets::login::ClientboundLoginPacket,
};
use bevy_app::{App, First, Plugin};
use bevy_ecs::system::ResMut;
use log::error;
use parking_lot::Mutex;

use super::recorder::Recorder;

pub struct RecordPlugin {
    pub recorder: Arc<Mutex<Option<Recorder>>>,
}

impl Plugin for RecordPlugin {
    fn build(&self, app: &mut App) {
        let recorder = self.recorder.lock().take();
        if let Some(recorder) = recorder {
            app.insert_resource(recorder)
                .add_systems(First, record_login_packets)
                .add_systems(First, record_configuration_packets)
                .add_systems(First, record_game_packets);
        }
    }
}

fn record_login_packets(
    recorder: Option<ResMut<Recorder>>,
    mut events: EventReader<ReceiveLoginPacketEvent>,
) {
    if let Some(mut recorder) = recorder {
        for event in events.read() {
            if recorder.should_ignore_compression
                && let ClientboundLoginPacket::LoginCompression(_) = *event.packet
            {
                continue;
            }

            if let Err(error) = recorder.save_packet(event.packet.as_ref()) {
                error!("failed to record login packet: {error:?}");
            }
        }
    }
}

fn record_configuration_packets(
    recorder: Option<ResMut<Recorder>>,
    mut events: EventReader<ReceiveConfigPacketEvent>,
) {
    if let Some(mut recorder) = recorder {
        for event in events.read() {
            if let Err(error) = recorder.save_packet(event.packet.as_ref()) {
                error!("failed to record configuration packet: {error:?}");
            }
        }
    }
}

fn record_game_packets(
    recorder: Option<ResMut<Recorder>>,
    mut events: EventReader<ReceiveGamePacketEvent>,
) {
    if let Some(mut recorder) = recorder {
        for event in events.read() {
            if let Err(error) = recorder.save_packet(event.packet.as_ref()) {
                error!("failed to record game packet: {error:?}");
            }
        }
    }
}
