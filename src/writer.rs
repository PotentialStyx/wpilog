use anyhow::{format_err, Result};
use kanal::Sender;
use std::{
    io::Write,
    sync::atomic::{AtomicU32, Ordering},
    thread::JoinHandle,
};

use crate::{ControlData, Record, RecordInfo, HEADER_STRING, HEADER_VERSION};

const MAX_ONE_BYTE: u64 = 256u64;
const MAX_TWO_BYTES: u64 = 256u64.pow(2);
const MAX_THREE_BYTES: u64 = 256u64.pow(3);
const MAX_FOUR_BYTES: u64 = 256u64.pow(4);
const MAX_FIVE_BYTES: u64 = 256u64.pow(5);
const MAX_SIX_BYTES: u64 = 256u64.pow(6);
const MAX_SEVEN_BYTES: u64 = 256u64.pow(7);

fn encode_int(num: u64) -> Box<[u8]> {
    match num {
        0..MAX_ONE_BYTE => Box::new([num as u8]),
        MAX_ONE_BYTE..MAX_TWO_BYTES => Box::new((num as u16).to_le_bytes()),
        MAX_TWO_BYTES..MAX_THREE_BYTES => {
            let tmp = (num as u32).to_le_bytes();
            Box::new([tmp[0], tmp[1], tmp[2]])
        }
        MAX_THREE_BYTES..MAX_FOUR_BYTES => Box::new((num as u32).to_le_bytes()),
        MAX_FOUR_BYTES..MAX_FIVE_BYTES => {
            let tmp = num.to_le_bytes();
            Box::new([tmp[0], tmp[1], tmp[2], tmp[3], tmp[4]])
        }
        MAX_FIVE_BYTES..MAX_SIX_BYTES => {
            let tmp = num.to_le_bytes();
            Box::new([tmp[0], tmp[1], tmp[2], tmp[3], tmp[4], tmp[5]])
        }
        MAX_SIX_BYTES..MAX_SEVEN_BYTES => {
            let tmp = num.to_le_bytes();
            Box::new([tmp[0], tmp[1], tmp[2], tmp[3], tmp[4], tmp[5], tmp[6]])
        }
        _ => Box::new(num.to_le_bytes()),
    }
}

pub trait TimeProvider {
    fn get_time(&self) -> u64;
}

enum RecvState {
    Msg(Box<[u8]>),
    Stop,
}

pub struct WPILOGWriter<T: TimeProvider + Clone + Send + Sync> {
    id: AtomicU32,
    channel: Sender<RecvState>,
    handle: JoinHandle<()>,
    time_provider: T,
}

impl<T: TimeProvider + Clone + Send + Sync> WPILOGWriter<T> {
    /// # Panics
    ///
    /// Can panic is writer fails `write_all()`
    pub fn new(mut writer: impl Write + Send + 'static, time_provider: T) -> WPILOGWriter<T> {
        let (sender, recv) = kanal::unbounded();

        writer.write_all(HEADER_STRING).unwrap();
        writer.write_all(&HEADER_VERSION.to_le_bytes()).unwrap();
        writer.write_all(&[0, 0, 0, 0]).unwrap();

        let handle = std::thread::spawn(move || {
            for item in recv {
                match item {
                    RecvState::Msg(data) => {
                        dbg!(&data);

                        writer.write_all(&data).unwrap();
                    }
                    RecvState::Stop => {
                        writer.flush().unwrap();
                        break;
                    }
                }
            }
        });

        WPILOGWriter {
            id: AtomicU32::new(1),
            channel: sender,
            handle,
            time_provider,
        }
    }

    pub fn make_entry(
        &self,
        name: String,
        r#type: String,
        metadata: String,
    ) -> Result<WPILOGEntry<T>> {
        let id = self.id.fetch_add(1, Ordering::Relaxed);
        let record = Record {
            id,
            timestamp: self.time_provider.get_time(),
            info: RecordInfo::Control(ControlData::Start {
                name: name.into_boxed_str(),
                r#type: r#type.into_boxed_str(),
                metadata: metadata.into_boxed_str(),
            }),
        };
        self.channel.send(RecvState::Msg(record.encode()))?;

        Ok(WPILOGEntry {
            id,
            channel: self.channel.clone(),
            time_provider: self.time_provider.clone(),
        })
    }

    /// Instantly stops new messages from sending, and stops the worker after all previous messages have been written
    /// Anything sent after will NOT BE RECORDED
    pub fn join(self) -> Result<()> {
        self.channel.send(RecvState::Stop)?;

        if let Err(err) = self.handle.join() {
            return Err(format_err!("{err:#?}"));
        }

        Ok(())
    }
}

pub struct WPILOGEntry<T: TimeProvider + Clone + Send + Sync> {
    id: u32,
    channel: Sender<RecvState>,
    time_provider: T,
}

impl Record {
    fn encode(&self) -> Box<[u8]> {
        // TODO: Figure out slice size first
        // This should be possible but might not be that trivial...
        let mut tmp = vec![];

        let timestamp_data = encode_int(self.timestamp);

        match &self.info {
            RecordInfo::Control(ctrl) => {
                let mut data = match ctrl {
                    ControlData::Start {
                        name,
                        r#type,
                        metadata,
                    } => {
                        let mut data = vec![0];
                        data.write_all(&self.id.to_le_bytes())
                            .expect("TODO: check if this can fail");

                        let len: u32 = name.len().try_into().expect("TODO: deal with this");
                        data.write_all(&len.to_le_bytes())
                            .expect("TODO: check if this can fail");

                        data.write_all(name.as_bytes())
                            .expect("TODO: check if this can fail");

                        let len: u32 = r#type.len().try_into().expect("TODO: deal with this");
                        data.write_all(&len.to_le_bytes())
                            .expect("TODO: check if this can fail");

                        data.write_all(r#type.as_bytes())
                            .expect("TODO: check if this can fail");

                        let len: u32 = metadata.len().try_into().expect("TODO: deal with this");
                        data.write_all(&len.to_le_bytes())
                            .expect("TODO: check if this can fail");

                        data.write_all(metadata.as_bytes())
                            .expect("TODO: check if this can fail");

                        data
                    }
                    ControlData::Finish => {
                        let mut data = vec![1];
                        data.write_all(&self.id.to_le_bytes())
                            .expect("TODO: check if this can fail");
                        data
                    }
                    ControlData::SetMetadata(metadata) => {
                        let mut data = vec![2];
                        data.write_all(&self.id.to_le_bytes())
                            .expect("TODO: check if this can fail");

                        let len: u32 = metadata.len().try_into().expect("TODO: deal with this");
                        data.write_all(&len.to_le_bytes())
                            .expect("TODO: check if this can fail");

                        data.write_all(metadata.as_bytes())
                            .expect("TODO: check if this can fail");

                        data
                    }
                };

                let size_data = encode_int(data.len() as u64);

                let mut bitfield = 0;
                // These HAVE to be u8's after the & 0x3/0x7
                bitfield |= (((size_data.len() - 1) & 0x3) as u8) << 2;
                bitfield |= (((timestamp_data.len() - 1) & 0x7) as u8) << 4;

                tmp.push(bitfield);

                tmp.write_all(&[0]).expect("TODO: check if this can fail");
                tmp.write_all(&size_data)
                    .expect("TODO: check if this can fail");
                tmp.write_all(&timestamp_data)
                    .expect("TODO: check if this can fail");

                tmp.append(&mut data);
            }
            RecordInfo::Data(data) => {
                let id_data = encode_int(self.id.into());
                let size_data = encode_int(data.len() as u64);

                tmp.reserve(
                    id_data.len() + size_data.len() + timestamp_data.len() + data.len() + 1,
                );

                let mut bitfield = 0;

                // These HAVE to be u8's after the & 0x3/0x7
                bitfield |= ((id_data.len() - 1) & 0x3) as u8;
                bitfield |= (((size_data.len() - 1) & 0x3) as u8) << 2;
                bitfield |= (((timestamp_data.len() - 1) & 0x7) as u8) << 4;

                tmp.push(bitfield);

                tmp.write_all(&id_data)
                    .expect("TODO: check if this can fail");
                tmp.write_all(&size_data)
                    .expect("TODO: check if this can fail");
                tmp.write_all(&timestamp_data)
                    .expect("TODO: check if this can fail");

                tmp.write_all(data).expect("TODO: check if this can fail");
            }
        }

        tmp.into_boxed_slice()
    }
}

impl<T: TimeProvider + Clone + Send + Sync> WPILOGEntry<T> {
    pub fn log_data(&self, data: Box<[u8]>) -> Result<()> {
        let record = Record {
            id: self.id,
            timestamp: self.time_provider.get_time(),
            info: crate::RecordInfo::Data(data),
        };

        self.channel.send(RecvState::Msg(record.encode()))?;

        Ok(())
    }
}
