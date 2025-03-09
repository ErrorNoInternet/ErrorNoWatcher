pub mod plugin;

use crate::build_info;
use anyhow::Result;
use azalea::{
    buf::AzaleaWriteVar,
    prelude::Resource,
    protocol::packets::{PROTOCOL_VERSION, ProtocolPacket, VERSION_NAME},
};
use serde_json::json;
use std::{
    fs::File,
    io::Write,
    time::{SystemTime, UNIX_EPOCH},
};
use zip::{ZipWriter, write::SimpleFileOptions};

#[derive(Resource)]
pub struct Recorder {
    zip_writer: ZipWriter<File>,
    start_time: u128,
    server: String,
    ignore_compression: bool,
}

impl Recorder {
    pub fn new(path: String, server: String, ignore_compression: bool) -> Result<Self> {
        let mut zip_writer = ZipWriter::new(
            File::options()
                .write(true)
                .create(true)
                .truncate(true)
                .open(path)?,
        );
        zip_writer.start_file("recording.tmcpr", SimpleFileOptions::default())?;
        Ok(Self {
            zip_writer,
            start_time: SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis(),
            server,
            ignore_compression,
        })
    }

    pub fn finish(mut self) -> Result<()> {
        self.zip_writer
            .start_file("metaData.json", SimpleFileOptions::default())?;
        self.zip_writer.write_all(
            json!({
                "singleplayer": false,
                "serverName": self.server,
                "duration": SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() - self.start_time,
                "date": self.start_time,
                "mcversion": VERSION_NAME,
                "fileFormat": "MCPR",
                "fileFormatVersion": 14,
                "protocol": PROTOCOL_VERSION,
                "generator": build_info::version_formatted(),
            })
            .to_string()
            .as_bytes(),
        )?;
        self.zip_writer.finish()?;

        Ok(())
    }

    fn get_timestamp(&self) -> Result<[u8; 4]> {
        Ok(TryInto::<u32>::try_into(
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() - self.start_time,
        )?
        .to_be_bytes())
    }

    fn save_raw_packet(&mut self, raw_packet: &[u8]) -> Result<()> {
        let mut data = Vec::from(self.get_timestamp()?);
        data.extend(TryInto::<u32>::try_into(raw_packet.len())?.to_be_bytes());
        data.extend(raw_packet);
        self.zip_writer.write_all(&data)?;
        Ok(())
    }

    fn save_packet<T: ProtocolPacket>(&mut self, packet: &T) -> Result<()> {
        let mut raw_packet = Vec::new();
        packet.id().azalea_write_var(&mut raw_packet)?;
        packet.write(&mut raw_packet)?;
        self.save_raw_packet(&raw_packet)
    }
}
