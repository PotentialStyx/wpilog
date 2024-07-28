use anyhow::{format_err, Result};
use kanal::Sender;
use std::{
    io::Write,
    sync::atomic::{AtomicU32, Ordering},
    thread::JoinHandle,
};

use crate::{ControlData, Record, RecordInfo, HEADER_STRING, HEADER_VERSION};

pub(crate) const MAX_ONE_BYTE: u64 = 256u64;
pub(crate) const MAX_TWO_BYTES: u64 = 256u64.pow(2);
pub(crate) const MAX_THREE_BYTES: u64 = 256u64.pow(3);
pub(crate) const MAX_FOUR_BYTES: u64 = 256u64.pow(4);
pub(crate) const MAX_FIVE_BYTES: u64 = 256u64.pow(5);
pub(crate) const MAX_SIX_BYTES: u64 = 256u64.pow(6);
pub(crate) const MAX_SEVEN_BYTES: u64 = 256u64.pow(7);

pub(crate) fn encode_int(num: u64) -> Box<[u8]> {
    if num < MAX_ONE_BYTE {
        Box::new([num as u8])
    } else if num < MAX_TWO_BYTES {
        Box::new((num as u16).to_le_bytes())
    } else if num < MAX_THREE_BYTES {
        let tmp = (num as u32).to_le_bytes();
        Box::new([tmp[0], tmp[1], tmp[2]])
    } else if num < MAX_FOUR_BYTES {
        Box::new((num as u32).to_le_bytes())
    } else if num < MAX_FIVE_BYTES {
        let tmp = num.to_le_bytes();
        Box::new([tmp[0], tmp[1], tmp[2], tmp[3], tmp[4]])
    } else if num < MAX_SIX_BYTES {
        let tmp = num.to_le_bytes();
        Box::new([tmp[0], tmp[1], tmp[2], tmp[3], tmp[4], tmp[5]])
    } else if num < MAX_SEVEN_BYTES {
        let tmp = num.to_le_bytes();
        Box::new([tmp[0], tmp[1], tmp[2], tmp[3], tmp[4], tmp[5], tmp[6]])
    } else {
        Box::new(num.to_le_bytes())
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

    /// Don't use this unless you know what you are doing.
    ///
    /// This method returns a [`RawEntry`], which requires you enforce that data is given the right format.
    ///
    /// Messing up data formatting can result in tools being unable to interpret what your logged values actually mean.
    pub fn make_entry(
        &self,
        name: String,
        r#type: String,
        metadata: String,
    ) -> Result<RawEntry<T>> {
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

        Ok(RawEntry {
            id,
            channel: self.channel.clone(),
            time_provider: self.time_provider.clone(),
        })
    }

    /// Instantly stops new messages from sending, and stops the worker after all previous messages have been written
    ///
    /// ANYTHING SENT AFTER THIS IS CALLED WILL NOT BE RECORDED, AND WILL BE LOST FOREVER!
    pub fn join(self) -> Result<()> {
        self.channel.send(RecvState::Stop)?;

        if let Err(err) = self.handle.join() {
            return Err(format_err!("{err:#?}"));
        }

        Ok(())
    }
}

/// A handle to write raw byte data to the log file. Usually a wrapper type is used.
pub struct RawEntry<T: TimeProvider + Clone + Send + Sync> {
    id: u32,
    channel: Sender<RecvState>,
    pub(super) time_provider: T,
}

impl Record {
    /// Turn the [`Record`] into it's binary representation.
    fn encode(&self) -> Box<[u8]> {
        // TODO: Figure out slice size first
        // This should be possible but might not be that trivial...

        let timestamp_data = encode_int(self.timestamp);

        match &self.info {
            RecordInfo::Control(ctrl) => {
                let mut ret = vec![];

                let mut data = match ctrl {
                    ControlData::Start {
                        name,
                        r#type,
                        metadata,
                    } => {
                        let mut data = vec![0];
                        data.extend_from_slice(&self.id.to_le_bytes());

                        let len: u32 = name.len().try_into().expect("TODO: deal with this");
                        data.extend_from_slice(&len.to_le_bytes());

                        data.extend_from_slice(name.as_bytes());

                        let len: u32 = r#type.len().try_into().expect("TODO: deal with this");
                        data.extend_from_slice(&len.to_le_bytes());

                        data.extend_from_slice(r#type.as_bytes());

                        let len: u32 = metadata.len().try_into().expect("TODO: deal with this");
                        data.extend_from_slice(&len.to_le_bytes());

                        data.extend_from_slice(metadata.as_bytes());

                        data
                    }
                    ControlData::Finish => {
                        let encoded = &self.id.to_le_bytes();
                        vec![1, encoded[0], encoded[1], encoded[2], encoded[3]]
                    }
                    ControlData::SetMetadata(metadata) => {
                        let mut data = vec![2];
                        data.extend_from_slice(&self.id.to_le_bytes());

                        let len: u32 = metadata.len().try_into().expect("TODO: deal with this");
                        data.extend_from_slice(&len.to_le_bytes());

                        data.extend_from_slice(metadata.as_bytes());

                        data
                    }
                };

                let size_data = encode_int(data.len() as u64);

                let mut bitfield = 0;
                // These HAVE to be u8's after the & 0x3/0x7
                bitfield |= (((size_data.len() - 1) & 0x3) as u8) << 2;
                bitfield |= (((timestamp_data.len() - 1) & 0x7) as u8) << 4;

                ret.push(bitfield);

                ret.extend_from_slice(&[0]);
                ret.extend_from_slice(&size_data);
                ret.extend_from_slice(&timestamp_data);

                ret.append(&mut data);

                ret.into_boxed_slice()
            }
            RecordInfo::Data(data) => {
                debug_assert_ne!(
                    self.id, 0,
                    "Data records can't have ID 0 or stuff will go wrong"
                );

                let id_data = encode_int(self.id.into());
                let size_data = encode_int(data.len() as u64);

                let length =
                    id_data.len() + size_data.len() + timestamp_data.len() + data.len() + 1;
                let mut ret = vec![0; length].into_boxed_slice();

                let mut bitfield = 0;

                // These HAVE to be u8's after the & 0x3/0x7
                bitfield |= ((id_data.len() - 1) & 0x3) as u8;
                bitfield |= (((size_data.len() - 1) & 0x3) as u8) << 2;
                bitfield |= (((timestamp_data.len() - 1) & 0x7) as u8) << 4;

                ret[0] = bitfield;

                let mut ptr = 1;
                for data in id_data {
                    ret[ptr] = data;

                    ptr += 1;
                }

                for data in size_data {
                    ret[ptr] = data;

                    ptr += 1;
                }

                for data in timestamp_data {
                    ret[ptr] = data;

                    ptr += 1;
                }

                for data in data {
                    ret[ptr] = *data;

                    ptr += 1;
                }

                ret
            }
        }
    }
}

impl<T: TimeProvider + Clone + Send + Sync> RawEntry<T> {
    /// Logs the data given as-is, without checking if it's the right format for the entry type.
    ///
    /// Automatically fetches timestamp from the `time_provider`
    pub fn log_data(&self, data: Box<[u8]>) -> Result<()> {
        self.log_data_with_timestamp(data, self.time_provider.get_time())
    }

    /// Logs the data given as-is, without checking if it's the right format for the entry type.
    ///
    /// Uses manually set timestamp instead of using the `time_provider`
    pub fn log_data_with_timestamp(&self, data: Box<[u8]>, timestamp: u64) -> Result<()> {
        let record = Record {
            id: self.id,
            timestamp,
            info: RecordInfo::Data(data),
        };

        self.channel.send(RecvState::Msg(record.encode()))?;

        Ok(())
    }

    /// Updates the metadata for the entry, normally this is JSON but it *can* be anything.
    pub fn set_metadata(&self, metadata: Box<str>) -> Result<()> {
        let record = Record {
            id: self.id,
            timestamp: self.time_provider.get_time(),
            info: RecordInfo::Control(ControlData::SetMetadata(metadata)),
        };

        self.channel.send(RecvState::Msg(record.encode()))?;

        Ok(())
    }
}

impl<T: TimeProvider + Clone + Send + Sync> Drop for RawEntry<T> {
    fn drop(&mut self) {
        let record = Record {
            id: self.id,
            timestamp: self.time_provider.get_time(),
            info: RecordInfo::Control(ControlData::Finish),
        };

        // Best attempt at nice cleanup, if it fails oh well...
        let _ = self.channel.send(RecvState::Msg(record.encode()));
    }
}
