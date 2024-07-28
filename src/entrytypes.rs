use anyhow::Result;

use crate::writer::{RawEntry, TimeProvider, WPILOGWriter};

pub trait Entry<T> {
    fn update(&self, data: T) -> Result<()>;
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
    new_entry_func!(new_bool_entry, BooleanEntry, "boolean");
    new_entry_func!(new_i64_entry, I64Entry, "int64");
    new_entry_func!(new_f32_entry, F32Entry, "float");
    new_entry_func!(new_f64_entry, F64Entry, "double");
    new_entry_func!(new_string_entry, StringEntry, "string");
    new_entry_func!(new_bytes_entry, BytesEntry, "raw");

    new_entry_func!(new_bool_array_entry, BooleanArrayEntry, "boolean[]");
}

macro_rules! make_entry_type {
    ($name:ident) => {
        pub struct $name<T: TimeProvider + Clone + Send + Sync>(RawEntry<T>);
    };
}

/* Doesn't really work that well:
macro_rules! impl_entry_type {
    {
        impl $name:ident {
            fn update(&self, $var:ident: $type:ty) $code:block

        }
    } => {
        impl<T: TimeProvider + Clone + Send + Sync> Entry<$type> for $name<T> {
            fn update(&self, $var: $type) -> Result<()> {
                self.inner.log_data(Box::new($code))
            }
        }
    };
}

impl_entry_type! {
    impl BooleanEntry {
        fn update(&self, data: bool) {
            [u8::from(data)]
        }
    }
}
*/

macro_rules! full_entry_type {
    ($name:ident, $type:ty) => {
        make_entry_type!($name);

        impl<T: TimeProvider + Clone + Send + Sync> Entry<$type> for $name<T> {
            fn update(&self, data: $type) -> Result<()> {
                self.0.log_data(Box::new(data.to_le_bytes()))
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
    fn update(&self, data: bool) -> Result<()> {
        self.0.log_data(Box::new([u8::from(data)]))
    }
}

make_entry_type!(BytesEntry);

impl<T: TimeProvider + Clone + Send + Sync> Entry<Box<[u8]>> for BytesEntry<T> {
    fn update(&self, data: Box<[u8]>) -> Result<()> {
        self.0.log_data(data)
    }
}

make_entry_type!(StringEntry);

impl<T: TimeProvider + Clone + Send + Sync> Entry<String> for StringEntry<T> {
    fn update(&self, data: String) -> Result<()> {
        self.0.log_data(data.into_boxed_str().into())
    }
}

// PRIMITIVE ARRAYS:
make_entry_type!(BooleanArrayEntry);

impl<T: TimeProvider + Clone + Send + Sync> Entry<&[bool]> for BooleanArrayEntry<T> {
    fn update(&self, data: &[bool]) -> Result<()> {
        let mut tmp = vec![0; data.len()].into_boxed_slice();

        // TODO: There has to be a better way to do this
        for (i, item) in data.iter().enumerate() {
            tmp[i] = u8::from(*item);
        }

        self.0.log_data(tmp)
    }
}
