use crate::build_info;
use anyhow::Result;
use azalea::{
    buf::AzaleaWriteVar,
    prelude::Resource,
    protocol::packets::{PROTOCOL_VERSION, ProtocolPacket, VERSION_NAME},
};
use serde_json::json;
use smallvec::SmallVec;
use std::{
    fs::File,
    io::Write,
    time::{Instant, SystemTime, UNIX_EPOCH},
};
use zip::{ZipWriter, write::SimpleFileOptions};

#[derive(Resource)]
pub struct Recorder {
    zip_writer: ZipWriter<File>,
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
            zip_writer,
            start: Instant::now(),
            server,
            should_ignore_compression,
        })
    }

    pub fn finish(mut self) -> Result<()> {
        let elapsed = self.start.elapsed().as_millis();

        self.zip_writer
            .start_file("metaData.json", SimpleFileOptions::default())?;
        self.zip_writer.write_all(
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
        self.zip_writer.finish()?;

        Ok(())
    }

    pub fn save_raw_packet(&mut self, raw_packet: &[u8]) -> Result<()> {
        println!("{}", raw_packet.len());
        let mut data = Vec::with_capacity(raw_packet.len() + 8);
        data.extend(&TryInto::<u32>::try_into(self.start.elapsed().as_millis())?.to_be_bytes());
        data.extend(&TryInto::<u32>::try_into(raw_packet.len())?.to_be_bytes());
        data.extend(raw_packet);
        self.zip_writer.write_all(&data)?;
        Ok(())
    }

    pub fn save_packet<T: ProtocolPacket>(&mut self, packet: &T) -> Result<()> {
        let mut raw_packet = SmallVec::<[u8; 256]>::new();
        packet.id().azalea_write_var(&mut raw_packet)?;
        packet.write(&mut raw_packet)?;
        self.save_raw_packet(&raw_packet)
    }
}
