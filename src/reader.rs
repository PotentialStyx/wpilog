use anyhow::{format_err, Result};
use core::str;
use std::io::{BufReader, Read};

use crate::{ControlData, Record, RecordInfo, HEADER_STRING, HEADER_VERSION};

pub struct WPILOGReader<R: Read> {
    reader: R,
    pub extra_header: Box<[u8]>,
}

impl<R: Read> WPILOGReader<BufReader<R>> {
    /// Takes a reader and wraps it in a [`BufReader`] before makings the [`WPIReader`]
    /// This is way more efficient since the wpilog implementation makes a lot of small reads
    pub fn new_buffered(reader: R) -> Result<Self> {
        WPILOGReader::new_raw(BufReader::new(reader))
    }
}

impl<R: Read> WPILOGReader<R> {
    /// Using [`WPIReader::new_buffered()`], or passing an already buffered reader is HIGHLY recommended
    pub fn new_raw(mut reader: R) -> Result<Self> {
        // Read and check header
        let mut header = [0; 6];
        reader.read_exact(&mut header)?;

        if header != *HEADER_STRING {
            return Err(format_err!("Invalid Header"));
        }

        // Read and check version number
        let mut version = [0; 2];
        reader.read_exact(&mut version)?;
        let version = u16::from_le_bytes(version);

        if version != HEADER_VERSION {
            return Err(format_err!("Invalid Version"));
        }

        // Read and save extra header
        let mut length = [0; 4];
        reader.read_exact(&mut length)?;
        let length = u32::from_le_bytes(length).try_into()?;

        let mut extra_header = vec![0; length].into_boxed_slice();
        reader.read_exact(&mut extra_header)?;

        Ok(WPILOGReader {
            reader,
            extra_header,
        })
    }

    /// Preconditions: `length <= 8`
    fn read_variable_int(&mut self, length: usize) -> Result<u64> {
        debug_assert!(length <= 8, "Invalid variable int length {length}");

        let mut final_buf: Box<[u8; 8]> = Box::from([0; 8]);
        self.reader.read_exact(&mut final_buf[0..length])?;

        Ok(u64::from_le_bytes(*final_buf))
    }

    /// Preconditions: `length <= 8`
    fn read_variable_int_option(&mut self, length: usize) -> Option<u64> {
        match self.read_variable_int(length) {
            Ok(value) => Some(value),
            // TODO: actually check what the error is
            Err(_err) => None,
        }
    }
}

impl<R: Read> Iterator for WPILOGReader<R> {
    type Item = PlainRecord;

    fn next(&mut self) -> Option<Self::Item> {
        let mut bitfield = [0; 1];

        // TODO: actually check what the error is
        if let Err(_err) = self.reader.read_exact(&mut bitfield) {
            return None;
        }

        let bitfield = bitfield[0];

        let entry_length = (bitfield & 0x3) + 1;
        let size_length = ((bitfield >> 2) & 0x3) + 1;
        let timestamp_length = ((bitfield >> 4) & 0x7) + 1;

        // Entry has to be a u32 or smaller since the bitfield can only represent byte lengths of 1-4
        #[allow(clippy::cast_possible_truncation)]
        let entry = self.read_variable_int_option(entry_length.into())? as u32;
        // Entry has to be a u32 or smaller since the bitfield can only represent byte lengths of 1-4
        // This code doesn't target lower than 32 bit systems so this cast will always be safe
        #[allow(clippy::cast_possible_truncation)]
        let size = self.read_variable_int_option(size_length.into())? as usize;

        let timestamp = self.read_variable_int_option(timestamp_length.into())?;

        let mut data = vec![0; size].into_boxed_slice();

        // TODO: actually check what the error is
        if let Err(_err) = self.reader.read_exact(&mut data) {
            return None;
        }

        Some(PlainRecord {
            id: entry,
            timestamp,
            data,
        })
    }
}

#[derive(Debug)]
pub struct PlainRecord {
    pub id: u32,
    pub timestamp: u64,
    pub data: Box<[u8]>,
}

impl TryFrom<PlainRecord> for Record {
    type Error = anyhow::Error;

    fn try_from(record: PlainRecord) -> std::result::Result<Self, Self::Error> {
        if record.id == 0 {
            let mut ptr = 0;

            if record.data.is_empty() {
                return Err(format_err!("Not enough data"));
            }

            let rtype = record.data[ptr];

            ptr += 1;

            if record.data.len() < ptr + 4 {
                return Err(format_err!("Not enough data for entry id"));
            }

            let id = u32::from_le_bytes([
                record.data[ptr],
                record.data[ptr + 1],
                record.data[ptr + 2],
                record.data[ptr + 3],
            ]);
            ptr += 4;

            let info = match rtype {
                0 => {
                    let name = {
                        if record.data.len() < ptr + 4 {
                            return Err(format_err!("Not enough data for length of entry name"));
                        }

                        let length = u32::from_le_bytes([
                            record.data[ptr],
                            record.data[ptr + 1],
                            record.data[ptr + 2],
                            record.data[ptr + 3],
                        ]) as usize;
                        ptr += 4;

                        if record.data.len() < ptr + length {
                            return Err(format_err!("Not enough data for entry name"));
                        }

                        let res = str::from_utf8(&record.data[ptr..ptr + length])?
                            .to_string()
                            .into_boxed_str();
                        ptr += length;

                        res
                    };

                    let etype = {
                        if record.data.len() < ptr + 4 {
                            return Err(format_err!("Not enough data for length of entry type"));
                        }

                        let length = u32::from_le_bytes([
                            record.data[ptr],
                            record.data[ptr + 1],
                            record.data[ptr + 2],
                            record.data[ptr + 3],
                        ]) as usize;
                        ptr += 4;

                        if record.data.len() < ptr + length {
                            return Err(format_err!("Not enough data for entry type"));
                        }

                        let res = str::from_utf8(&record.data[ptr..ptr + length])?
                            .to_string()
                            .into_boxed_str();
                        ptr += length;

                        res
                    };

                    let metadata = {
                        if record.data.len() < ptr + 4 {
                            return Err(format_err!(
                                "Not enough data for length of entry metadata"
                            ));
                        }

                        let length = u32::from_le_bytes([
                            record.data[ptr],
                            record.data[ptr + 1],
                            record.data[ptr + 2],
                            record.data[ptr + 3],
                        ]) as usize;
                        ptr += 4;

                        if record.data.len() < ptr + length {
                            return Err(format_err!("Not enough data for entry metadata"));
                        }

                        str::from_utf8(&record.data[ptr..ptr + length])?
                            .to_string()
                            .into_boxed_str()
                    };

                    ControlData::Start {
                        name,
                        r#type: etype,
                        metadata,
                    }
                }
                1 => ControlData::Finish,
                2 => {
                    let metadata = {
                        if record.data.len() < ptr + 4 {
                            return Err(format_err!(
                                "Not enough data for length of entry metadata"
                            ));
                        }

                        let length = u32::from_le_bytes([
                            record.data[ptr],
                            record.data[ptr + 1],
                            record.data[ptr + 2],
                            record.data[ptr + 3],
                        ]) as usize;
                        ptr += 4;

                        if record.data.len() < ptr + length {
                            return Err(format_err!("Not enough data for entry metadata"));
                        }

                        str::from_utf8(&record.data[ptr..ptr + length])?
                            .to_string()
                            .into_boxed_str()
                    };

                    ControlData::SetMetadata(metadata)
                }
                _ => return Err(format_err!("Invalid Control Record Type: {rtype}")),
            };

            Ok(Record {
                id,
                timestamp: record.timestamp,
                info: RecordInfo::Control(info),
            })
        } else {
            Ok(Record {
                id: record.id,
                timestamp: record.timestamp,
                info: RecordInfo::Data(record.data),
            })
        }
    }
}
