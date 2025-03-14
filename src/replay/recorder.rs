use crate::build_info;
use anyhow::Result;
use azalea::{
    buf::AzaleaWriteVar,
    prelude::Resource,
    protocol::packets::{PROTOCOL_VERSION, ProtocolPacket, VERSION_NAME},
};
use log::debug;
use serde_json::json;
use std::{
    fs::File,
    io::{BufWriter, Write},
    time::{Instant, SystemTime, UNIX_EPOCH},
};
use zip::{ZipWriter, write::SimpleFileOptions};

#[derive(Resource)]
pub struct Recorder {
    zip_writer: BufWriter<ZipWriter<File>>,
    start: Instant,
    server: String,
    pub should_ignore_compression: bool,
}

impl Recorder {
    pub fn new(path: String, server: String, should_ignore_compression: bool) -> Result<Self> {
        let mut zip_writer = ZipWriter::new(
            File::options()
                .write(true)
                .create(true)
                .truncate(true)
                .open(path)?,
        );
        zip_writer.start_file("recording.tmcpr", SimpleFileOptions::default())?;
        Ok(Self {
            zip_writer: BufWriter::new(zip_writer),
            start: Instant::now(),
            server,
            should_ignore_compression,
        })
    }

    pub fn finish(self) -> Result<()> {
        debug!("finishing replay recording");

        let elapsed = self.start.elapsed().as_millis();
        let mut zip_writer = self.zip_writer.into_inner()?;
        zip_writer.start_file("metaData.json", SimpleFileOptions::default())?;
        zip_writer.write_all(
            json!({
                "singleplayer": false,
                "serverName": self.server,
                "duration": elapsed,
                "date": SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() - elapsed,
                "mcversion": VERSION_NAME,
                "fileFormat": "MCPR",
                "fileFormatVersion": 14,
                "protocol": PROTOCOL_VERSION,
                "generator": format!("errornowatcher {}", build_info::version_formatted()),
            })
            .to_string()
            .as_bytes(),
        )?;
        zip_writer.finish()?;

        debug!("finished replay recording");
        Ok(())
    }

    pub fn save_raw_packet(&mut self, raw_packet: &[u8]) -> Result<()> {
        self.zip_writer.write_all(
            &TryInto::<u32>::try_into(self.start.elapsed().as_millis())?.to_be_bytes(),
        )?;
        self.zip_writer
            .write_all(&TryInto::<u32>::try_into(raw_packet.len())?.to_be_bytes())?;
        self.zip_writer.write_all(raw_packet)?;
        Ok(())
    }

    pub fn save_packet<T: ProtocolPacket>(&mut self, packet: &T) -> Result<()> {
        let mut raw_packet = Vec::with_capacity(64);
        packet.id().azalea_write_var(&mut raw_packet)?;
        packet.write(&mut raw_packet)?;
        self.save_raw_packet(&raw_packet)
    }
}
