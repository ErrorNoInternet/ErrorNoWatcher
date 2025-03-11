use super::recorder::Recorder;
use azalea::{
    ecs::{event::EventReader, system::Query},
    packet_handling::{
        configuration::ConfigurationEvent,
        game::send_packet_events,
        login::{LoginPacketEvent, process_packet_events},
    },
    protocol::packets::login::ClientboundLoginPacket,
    raw_connection::RawConnection,
};
use bevy_app::{First, Plugin};
use bevy_ecs::{schedule::IntoSystemConfigs, system::ResMut};
use log::error;
use parking_lot::Mutex;
use std::sync::Arc;

pub struct RecordPlugin {
    pub recorder: Arc<Mutex<Option<Recorder>>>,
}

impl Plugin for RecordPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        if let Some(recorder) = self.recorder.lock().take() {
            app.insert_resource(recorder);
        }
        app.add_systems(First, record_login_packets.before(process_packet_events))
            .add_systems(First, record_configuration_packets)
            .add_systems(First, record_game_packets.before(send_packet_events));
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
    mut events: EventReader<ConfigurationEvent>,
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
