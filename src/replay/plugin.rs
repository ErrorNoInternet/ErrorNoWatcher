#![allow(clippy::needless_pass_by_value)]

use std::sync::Arc;

use azalea::{
    ecs::{event::EventReader, system::Query},
    packet::{
        config::ReceiveConfigPacketEvent,
        game::emit_receive_packet_events,
        login::{LoginPacketEvent, process_packet_events},
    },
    protocol::packets::login::ClientboundLoginPacket,
    raw_connection::RawConnection,
};
use bevy_app::{App, First, Plugin};
use bevy_ecs::{schedule::IntoSystemConfigs, system::ResMut};
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
                .add_systems(First, record_login_packets.before(process_packet_events))
                .add_systems(First, record_configuration_packets)
                .add_systems(
                    First,
                    record_game_packets.before(emit_receive_packet_events),
                );
        }
    }
}

fn record_login_packets(
    recorder: Option<ResMut<Recorder>>,
    mut events: EventReader<LoginPacketEvent>,
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
            if let Err(error) = recorder.save_packet(&event.packet) {
                error!("failed to record configuration packet: {error:?}");
            }
        }
    }
}

fn record_game_packets(recorder: Option<ResMut<Recorder>>, query: Query<&RawConnection>) {
    if let Some(mut recorder) = recorder
        && let Ok(raw_conn) = query.get_single()
    {
        let queue = raw_conn.incoming_packet_queue();
        for raw_packet in queue.lock().iter() {
            if let Err(error) = recorder.save_raw_packet(raw_packet) {
                error!("failed to record game packet: {error:?}");
            }
        }
    }
}
