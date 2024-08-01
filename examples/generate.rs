use anyhow::Result;
use std::{env, fs, io::Write};
use wpilog::{
    entrytypes::Entry,
    writer::{TimeProvider, WPILOGWriter},
};

#[derive(Copy, Clone, Debug)]
struct NoopTimeProvider {}

impl TimeProvider for NoopTimeProvider {
    fn get_time(&self) -> u64 {
        0
    }
}

fn main() -> Result<()> {
    let args = env::args().collect::<Vec<String>>();
    let filename = if let Some(name) = args.get(1) {
        name
    } else {
        "generate-out.wpilog"
    };

    println!("Writing example file with all datatypes to {filename}");

    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(filename)?;

    let writer = WPILOGWriter::new(file, NoopTimeProvider {});

    let raw = writer.new_bytes_entry("NT:Primitives/raw".into(), None)?;
    let boolean = writer.new_bool_entry("NT:Primitives/boolean".into(), None)?;
    let int64 = writer.new_i64_entry("NT:Primitives/int64".into(), None)?;
    let float = writer.new_f32_entry("NT:Primitives/float".into(), None)?;
    let double = writer.new_f64_entry("NT:Primitives/double".into(), None)?;
    let string = writer.new_string_entry("NT:Primitives/string".into(), None)?;

    let boolean_array = writer.new_bool_array_entry("NT:Array/Booleans".into(), None)?;
    let int64_array = writer.new_i64_array_entry("NT:Array/int64".into(), None)?;
    let float_array = writer.new_f32_array_entry("NT:Array/float".into(), None)?;
    let double_array = writer.new_f64_array_entry("NT:Array/double".into(), None)?;
    let string_array = writer.new_string_array_entry("NT:Array/string".into(), None)?;

    let time = 1_000_000;
    raw.update_with_timestamp(Box::new([0, 0]), time)?;
    boolean.update_with_timestamp(false, time)?;
    int64.update_with_timestamp(1, time)?;
    float.update_with_timestamp(0.25, time)?;
    double.update_with_timestamp(0.00000000025, time)?;
    string.update_with_timestamp("Hello".into(), time)?;
    boolean_array.update_with_timestamp(&[false, false], time)?;
    int64_array.update_with_timestamp(&[-2, -1], time)?;
    float_array.update_with_timestamp(&[-1.0, -0.5], time)?;
    double_array.update_with_timestamp(&[-0.0000000001, -0.0000000005], time)?;
    string_array.update_with_timestamp(&["Hello", ", ", "World", "!"], time)?;

    let time = 2_000_000;
    raw.update_with_timestamp(Box::new([0, 1]), time)?;
    int64.update_with_timestamp(2, time)?;
    float.update_with_timestamp(0.50, time)?;
    double.update_with_timestamp(0.00000000050, time)?;
    string.update_with_timestamp(", ".into(), time)?;
    boolean_array.update_with_timestamp(&[false, true], time)?;
    float_array.update_with_timestamp(&[-0.5, -0.0], time)?;
    double_array.update_with_timestamp(&[-0.0000000005, -0.0000000000], time)?;

    let time = 3_000_000;
    raw.update_with_timestamp(Box::new([1, 1]), time)?;
    boolean.update_with_timestamp(true, time)?;
    int64.update_with_timestamp(4, time)?;
    float.update_with_timestamp(0.75, time)?;
    double.update_with_timestamp(0.00000000075, time)?;
    string.update_with_timestamp("World".into(), time)?;
    boolean_array.update_with_timestamp(&[true, false], time)?;
    int64_array.update_with_timestamp(&[0, 1], time)?;
    float_array.update_with_timestamp(&[0.0, 0.5], time)?;
    double_array.update_with_timestamp(&[0.0000000000, 0.0000000005], time)?;
    string_array.update_with_timestamp(&["Goodbye", ", ", "World", "!"], time)?;

    let time = 4_000_000;
    raw.update_with_timestamp(Box::new([1, 0]), time)?;
    int64.update_with_timestamp(8, time)?;
    float.update_with_timestamp(1.0, time)?;
    double.update_with_timestamp(0.00000000010, time)?;
    string.update_with_timestamp("!".into(), time)?;
    boolean_array.update_with_timestamp(&[true, true], time)?;
    int64_array.update_with_timestamp(&[1, 2], time)?;
    float_array.update_with_timestamp(&[0.5, 1.0], time)?;
    double_array.update_with_timestamp(&[0.0000000005, 0.0000000001], time)?;

    let time = 5_000_000;
    int64.update_with_timestamp(8, time)?;

    writer.join()?.flush()?;

    Ok(())
}
