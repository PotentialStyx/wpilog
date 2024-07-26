use anyhow::{format_err, Result};
use core::str;
use std::io::{BufReader, Read};

static HEADER_STRING: &[u8; 6] = b"WPILOG";
static HEADER_VERSION: u16 = 0x0100;

pub struct WPIReader<R: Read> {
    reader: R,
    pub extra_header: Box<[u8]>,
}

impl<R: Read> WPIReader<BufReader<R>> {
    pub fn new_buffered(reader: R) -> Result<Self> {
        WPIReader::new_raw(BufReader::new(reader))
    }
}

impl<R: Read> WPIReader<R> {
    /// Using new_buffered, or passing an already buffered reader is HIGHLY recommended
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

        Ok(WPIReader {
            reader,
            extra_header,
        })
    }

    /// 0 < length <= 8
    fn read_variable_int(&mut self, length: usize) -> Result<u64> {
        let mut final_buf: Box<[u8; 8]> = Box::from([0; 8]);
        self.reader.read_exact(&mut final_buf[0..length])?;

        Ok(u64::from_le_bytes(*final_buf))
    }

    fn internal_next(&mut self) -> Result<PlainRecord> {
        let mut bitfield = [0; 1];
        self.reader.read_exact(&mut bitfield)?;
        let bitfield = bitfield[0];

        let entry_length = (bitfield & 0x3) + 1;
        let size_length = ((bitfield >> 2) & 0x3) + 1;
        let timestamp_length = ((bitfield >> 4) & 0x7) + 1;

        let entry = self.read_variable_int(entry_length.into())? as u32;
        let size = self.read_variable_int(size_length.into())? as usize;
        let timestamp = self.read_variable_int(timestamp_length.into())?;

        let mut data = vec![0; size].into_boxed_slice();

        self.reader.read_exact(&mut data)?;

        Ok(PlainRecord {
            id: entry,
            timestamp,
            data,
        })
    }
}

impl<R: Read> Iterator for WPIReader<R> {
    type Item = PlainRecord;

    fn next(&mut self) -> Option<Self::Item> {
        match self.internal_next() {
            Ok(item) => Some(item),
            Err(err) => {
                eprintln!("{err}");
                None
            }
        }
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
                    /*


                    4-byte (32-bit) length of entry name string

                    entry name UTF-8 string data (arbitrary length)

                    4-byte (32-bit) length of entry type string

                    entry type UTF-8 string data (arbitrary length)

                    4-byte (32-bit) length of entry metadata string

                    entry metadata UTF-8 string data (arbitrary length)
                     */
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

#[derive(Debug, Clone)]
pub struct Record {
    pub id: u32,
    pub timestamp: u64,
    pub info: RecordInfo,
}

#[derive(Debug, Clone)]
pub enum RecordInfo {
    Control(ControlData),
    Data(Box<[u8]>),
}

#[derive(Debug, Clone)]
pub enum ControlData {
    Start {
        name: Box<str>,
        r#type: Box<str>,
        metadata: Box<str>,
    },
    Finish,
    SetMetadata(Box<str>),
}
