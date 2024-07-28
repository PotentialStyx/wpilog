use anyhow::Result;

use crate::writer::{RawEntry, TimeProvider, WPILOGWriter};

pub trait Entry<T> {
    fn update(&self, data: T) -> Result<()>;

    fn update_with_timestamp(&self, data: T, timestamp: u64) -> Result<()>;
}

macro_rules! new_entry_func {
    ($name:ident, $type:ident, $type_name:literal) => {
        #[doc = "Creates a new [`"]
        #[doc = stringify!($type)]
        #[doc = "`]."]
        pub fn $name(&self, name: String, metadata: Option<String>) -> Result<$type<T>> {
            Ok($type(self.make_entry(
                name,
                $type_name.to_string(),
                metadata.unwrap_or_default(),
            )?))
        }
    };
}

impl<T: TimeProvider + Clone + Send + Sync> WPILOGWriter<T> {
    new_entry_func!(new_bytes_entry, BytesEntry, "raw");

    new_entry_func!(new_bool_entry, BooleanEntry, "boolean");
    new_entry_func!(new_i64_entry, I64Entry, "int64");
    new_entry_func!(new_f32_entry, F32Entry, "float");
    new_entry_func!(new_f64_entry, F64Entry, "double");
    new_entry_func!(new_string_entry, StringEntry, "string");

    new_entry_func!(new_bool_array_entry, BooleanArrayEntry, "boolean[]");
    new_entry_func!(new_i64_array_entry, I64ArrayEntry, "int64[]");
    new_entry_func!(new_f32_array_entry, F32ArrayEntry, "float[]");
    new_entry_func!(new_f64_array_entry, F64ArrayEntry, "double[]");
    new_entry_func!(new_string_array_entry, StringArrayEntry, "string[]");
}

macro_rules! make_entry_type {
    ($name:ident) => {
        pub struct $name<T: TimeProvider + Clone + Send + Sync>(RawEntry<T>);
    };
}

macro_rules! update_fn {
    ($type:ty) => {
        fn update(&self, data: $type) -> Result<()> {
            self.update_with_timestamp(data, self.0.time_provider.get_time())
        }
    };
}

macro_rules! full_entry_type {
    ($name:ident, $type:ty) => {
        make_entry_type!($name);

        impl<T: TimeProvider + Clone + Send + Sync> Entry<$type> for $name<T> {
            update_fn!($type);

            fn update_with_timestamp(&self, data: $type, timestamp: u64) -> Result<()> {
                self.0
                    .log_data_with_timestamp(Box::new(data.to_le_bytes()), timestamp)
            }
        }
    };
}

// PRIMITIVES:

full_entry_type!(I64Entry, i64);
full_entry_type!(F32Entry, f32);
full_entry_type!(F64Entry, f64);

make_entry_type!(BooleanEntry);

impl<T: TimeProvider + Clone + Send + Sync> Entry<bool> for BooleanEntry<T> {
    update_fn!(bool);

    fn update_with_timestamp(&self, data: bool, timestamp: u64) -> Result<()> {
        self.0
            .log_data_with_timestamp(Box::new([u8::from(data)]), timestamp)
    }
}

make_entry_type!(BytesEntry);

impl<T: TimeProvider + Clone + Send + Sync> Entry<Box<[u8]>> for BytesEntry<T> {
    update_fn!(Box<[u8]>);

    fn update_with_timestamp(&self, data: Box<[u8]>, timestamp: u64) -> Result<()> {
        self.0.log_data_with_timestamp(data, timestamp)
    }
}

make_entry_type!(StringEntry);

impl<T: TimeProvider + Clone + Send + Sync> Entry<String> for StringEntry<T> {
    update_fn!(String);

    fn update_with_timestamp(&self, data: String, timestamp: u64) -> Result<()> {
        self.0
            .log_data_with_timestamp(data.into_boxed_str().into(), timestamp)
    }
}

// PRIMITIVE ARRAYS:
make_entry_type!(BooleanArrayEntry);

impl<T: TimeProvider + Clone + Send + Sync> Entry<&[bool]> for BooleanArrayEntry<T> {
    update_fn!(&[bool]);

    fn update_with_timestamp(&self, data: &[bool], timestamp: u64) -> Result<()> {
        let mut tmp = vec![0; data.len()].into_boxed_slice();

        // TODO: There has to be a better way to do this
        for (i, item) in data.iter().enumerate() {
            tmp[i] = u8::from(*item);
        }

        self.0.log_data_with_timestamp(tmp, timestamp)
    }
}

make_entry_type!(I64ArrayEntry);

impl<T: TimeProvider + Clone + Send + Sync> Entry<&[i64]> for I64ArrayEntry<T> {
    update_fn!(&[i64]);

    fn update_with_timestamp(&self, data: &[i64], timestamp: u64) -> Result<()> {
        let mut dest = vec![0; data.len() * 4].into_boxed_slice();

        let mut i = 0;
        for item in data {
            let encoded = item.to_le_bytes();
            dest[i] = encoded[0];
            i += 1;
            dest[i] = encoded[1];
            i += 1;
            dest[i] = encoded[2];
            i += 1;
            dest[i] = encoded[3];
            i += 1;
        }

        self.0.log_data_with_timestamp(dest, timestamp)
    }
}

make_entry_type!(F32ArrayEntry);

impl<T: TimeProvider + Clone + Send + Sync> Entry<&[f32]> for F32ArrayEntry<T> {
    update_fn!(&[f32]);

    fn update_with_timestamp(&self, data: &[f32], timestamp: u64) -> Result<()> {
        let mut dest = vec![0; data.len() * 4].into_boxed_slice();

        let mut i = 0;
        for item in data {
            let encoded = item.to_le_bytes();
            dest[i] = encoded[0];
            i += 1;
            dest[i] = encoded[1];
            i += 1;
            dest[i] = encoded[2];
            i += 1;
            dest[i] = encoded[3];
            i += 1;
        }

        self.0.log_data_with_timestamp(dest, timestamp)
    }
}

make_entry_type!(F64ArrayEntry);

impl<T: TimeProvider + Clone + Send + Sync> Entry<&[f64]> for F64ArrayEntry<T> {
    update_fn!(&[f64]);

    fn update_with_timestamp(&self, data: &[f64], timestamp: u64) -> Result<()> {
        let mut dest = vec![0; data.len() * 8].into_boxed_slice();

        let mut i = 0;
        for item in data {
            let encoded = item.to_le_bytes();
            dest[i] = encoded[0];
            i += 1;
            dest[i] = encoded[1];
            i += 1;
            dest[i] = encoded[2];
            i += 1;
            dest[i] = encoded[3];
            i += 1;
            dest[i] = encoded[4];
            i += 1;
            dest[i] = encoded[5];
            i += 1;
            dest[i] = encoded[6];
            i += 1;
            dest[i] = encoded[7];
            i += 1;
        }

        self.0.log_data_with_timestamp(dest, timestamp)
    }
}

make_entry_type!(StringArrayEntry);
impl<T: TimeProvider + Clone + Send + Sync> Entry<&[&str]> for StringArrayEntry<T> {
    update_fn!(&[&str]);

    fn update_with_timestamp(&self, data: &[&str], timestamp: u64) -> Result<()> {
        let length = 4 + 4 * data.len() + data.iter().map(|string| str::len(string)).sum::<usize>();

        let mut dest = vec![0; length].into_boxed_slice();
        let size_encoded = (data.len() as u32).to_le_bytes();
        dest[0] = size_encoded[0];
        dest[1] = size_encoded[1];
        dest[2] = size_encoded[2];
        dest[3] = size_encoded[3];

        let mut i = 4;
        for item in data {
            let size_encoded = (item.len() as u32).to_le_bytes();
            dest[i] = size_encoded[0];
            i += 1;
            dest[i] = size_encoded[1];
            i += 1;
            dest[i] = size_encoded[2];
            i += 1;
            dest[i] = size_encoded[3];
            i += 1;

            let encoded = item.as_bytes();
            for byte in encoded {
                dest[i] = *byte;
                i += 1;
            }
        }

        self.0.log_data_with_timestamp(dest, timestamp)
    }
}
