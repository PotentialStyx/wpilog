use anyhow::{format_err, Result};
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

    fn internal_next(&mut self) -> Result<Record> {
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

        Ok(Record {
            id: entry,
            timestamp,
            data,
        })
    }
}

impl<R: Read> Iterator for WPIReader<R> {
    type Item = Record;

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
pub struct Record {
    pub id: u32,
    pub timestamp: u64,
    pub data: Box<[u8]>,
}
