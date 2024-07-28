use anyhow::Result;
use std::{
    fs,
    hint::black_box,
    thread,
    time::{Duration, Instant},
};
use wpilog::{
    entrytypes::Entry,
    reader::{PlainRecord, WPILOGReader},
    writer::{TimeProvider, WPILOGWriter},
    Record,
};

#[derive(Clone, Debug)]
struct SimpleTimeProvider {
    start: Instant,
}

impl TimeProvider for SimpleTimeProvider {
    fn get_time(&self) -> u64 {
        Instant::now().duration_since(self.start).as_micros() as u64
    }
}

fn main() -> Result<()> {
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open("test2.wpilog")?;

    let writer = WPILOGWriter::new(
        file,
        SimpleTimeProvider {
            start: Instant::now(),
        },
    );

    let entry = writer.new_i64_entry("NT:Test/Key".into(), None)?;
    entry.update(0)?;

    thread::sleep(Duration::from_secs(1));
    entry.update(5)?;

    thread::sleep(Duration::from_secs(1));
    entry.update(10)?;

    thread::sleep(Duration::from_secs(1));
    entry.update(15)?;

    thread::sleep(Duration::from_secs(1));
    entry.update(65)?;

    writer.join()?;

    let data: &[u8] = &fs::read("test2.wpilog")?;

    let reader = WPILOGReader::new_raw(data)?;

    let mut records = 0;
    for record in reader.map(|item: PlainRecord| -> Record { item.try_into().unwrap() }) {
        if records < 50 {
            black_box(dbg!(record));
        }

        records += 1;
    }

    black_box(dbg!(records));
    Ok(())
}
