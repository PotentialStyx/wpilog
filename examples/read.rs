use anyhow::Result;
use std::{env, fs};
use wpilog::{
    reader::{PlainRecord, WPILOGReader},
    Record,
};

fn main() -> Result<()> {
    let args = env::args().collect::<Vec<String>>();
    let filename = args.get(1).expect("Need a file to read from");

    let count = if let Some(count) = args.get(2) {
        count.parse().unwrap_or(10)
    } else {
        10
    };

    println!("Reading first {count} record(s) from {filename}");

    let data: &[u8] = &fs::read(filename)?;

    let reader = WPILOGReader::new_raw(data)?;

    let mut records = 0;
    for record in reader.map(|item: PlainRecord| -> Record { item.try_into().unwrap() }) {
        if records < count {
            dbg!(record);
        }

        records += 1;
    }

    println!("The file had {records} record(s) in total.");

    Ok(())
}
