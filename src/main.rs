use anyhow::Result;
use std::fs;
use wpilog::WPIReader;

fn main() -> Result<()> {
    let data: &[u8] = &fs::read("test.wpilog")?;

    let reader = WPIReader::new_buffered(data)?;

    let mut records = 0;
    for _ in reader {
        records += 1;
    }

    dbg!(records);
    Ok(())
}
