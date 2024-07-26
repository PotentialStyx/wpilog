use anyhow::Result;
use std::{fs, hint::black_box};
use wpilog::{PlainRecord, Record, WPIReader};

fn main() -> Result<()> {
    let data: &[u8] = &fs::read("test.wpilog")?;

    let reader = WPIReader::new_raw(data)?;

    let mut records = 0;
    for record in reader.map(|item: PlainRecord| -> Record { item.try_into().unwrap() }) {
        if records < 5 {
            black_box(dbg!(record));
        }

        records += 1;
    }

    black_box(dbg!(records));
    Ok(())
}
