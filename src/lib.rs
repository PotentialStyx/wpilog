#![feature(test)]
#![warn(clippy::pedantic, clippy::all)]
#![allow(
    clippy::module_name_repetitions,

    // TODO: Remove these exceptions
    clippy::missing_errors_doc,
    clippy::cast_possible_truncation,
    clippy::too_many_lines,
)]

#[cfg(test)]
mod tests;

static HEADER_STRING: &[u8; 6] = b"WPILOG";
static HEADER_VERSION: u16 = 0x0100;

pub mod reader;
pub mod writer;

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
