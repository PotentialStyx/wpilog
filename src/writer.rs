use anyhow::{format_err, Result};
use kanal::{Receiver, Sender};
use std::{
    io::Write,
    sync::atomic::{AtomicU32, Ordering},
    thread::JoinHandle,
};

const MAX_ONE_BYTE: u64 = 256u64;
const MAX_TWO_BYTES: u64 = 256u64.pow(2);
const MAX_THREE_BYTES: u64 = 256u64.pow(3);
const MAX_FOUR_BYTES: u64 = 256u64.pow(4);
const MAX_FIVE_BYTES: u64 = 256u64.pow(5);
const MAX_SIX_BYTES: u64 = 256u64.pow(6);
const MAX_SEVEN_BYTES: u64 = 256u64.pow(7);

use crate::Record;

pub trait TimeProvider {
    fn get_time(&self) -> u64;
}

pub struct WPILOGWriter<T: TimeProvider + Clone + Send + Sync> {
    id: AtomicU32,
    channel: Sender<Box<[u8]>>,
    handle: JoinHandle<()>,
    time_provider: T,
}

impl<T: TimeProvider + Clone + Send + Sync> WPILOGWriter<T> {
    /// # Panics
    ///
    /// Can panic is writer fails `write_all()`
    pub fn new(mut writer: impl Write + Send + 'static, time_provider: T) -> WPILOGWriter<T> {
        let (sender, recv) = kanal::unbounded::<Box<[u8]>>();

        let handle = std::thread::spawn(move || {
            for item in recv {
                writer.write_all(&item).unwrap();
            }
        });

        WPILOGWriter {
            id: AtomicU32::new(0),
            channel: sender,
            handle,
            time_provider,
        }
    }

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

    pub fn make_entry(&self) -> WPILOGEntry<T> {
        WPILOGEntry {
            id: self.id.fetch_add(1, Ordering::Relaxed),
            channel: self.channel.clone(),
            time_provider: self.time_provider.clone(),
        }
    }

    pub fn join(self) -> Result<()> {
        self.channel.close();

        if let Err(err) = self.handle.join() {
            return Err(format_err!("{err:#?}"));
        }

        Ok(())
    }
}

pub struct WPILOGEntry<T: TimeProvider + Clone + Send + Sync> {
    id: u32,
    channel: Sender<Box<[u8]>>,
    time_provider: T,
}

impl<T: TimeProvider + Clone + Send + Sync> WPILOGEntry<T> {
    pub fn log_data(&self, data: Box<[u8]>) -> Result<()> {
        let record = Record {
            id: self.id,
            timestamp: self.time_provider.get_time(),
            info: crate::RecordInfo::Data(data),
        };

        self.channel.send(Box::default())?;

        Ok(())
    }
}
