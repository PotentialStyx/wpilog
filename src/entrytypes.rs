use anyhow::Result;

use crate::writer::{RawEntry, TimeProvider, WPILOGWriter};

pub trait Entry<T> {
    fn update(&self, data: T) -> Result<()>;
}

impl<T: TimeProvider + Clone + Send + Sync> WPILOGWriter<T> {
    pub fn new_bool_entry(
        &self,
        name: String,
        metadata: Option<String>,
    ) -> Result<BooleanEntry<T>> {
        Ok(BooleanEntry {
            inner: self.make_entry(name, "boolean".to_string(), metadata.unwrap_or_default())?,
        })
    }

    pub fn new_i64_entry(&self, name: String, metadata: Option<String>) -> Result<I64Entry<T>> {
        Ok(I64Entry {
            inner: self.make_entry(name, "int64".to_string(), metadata.unwrap_or_default())?,
        })
    }

    pub fn new_f32_entry(&self, name: String, metadata: Option<String>) -> Result<F32Entry<T>> {
        Ok(F32Entry {
            inner: self.make_entry(name, "float".to_string(), metadata.unwrap_or_default())?,
        })
    }

    pub fn new_f64_entry(&self, name: String, metadata: Option<String>) -> Result<F64Entry<T>> {
        Ok(F64Entry {
            inner: self.make_entry(name, "double".to_string(), metadata.unwrap_or_default())?,
        })
    }

    pub fn new_string_entry(
        &self,
        name: String,
        metadata: Option<String>,
    ) -> Result<StringEntry<T>> {
        Ok(StringEntry {
            inner: self.make_entry(name, "string".to_string(), metadata.unwrap_or_default())?,
        })
    }
}

pub struct BooleanEntry<T: TimeProvider + Clone + Send + Sync> {
    inner: RawEntry<T>,
}

impl<T: TimeProvider + Clone + Send + Sync> Entry<bool> for BooleanEntry<T> {
    fn update(&self, data: bool) -> Result<()> {
        self.inner.log_data(Box::new([u8::from(data)]))
    }
}

pub struct I64Entry<T: TimeProvider + Clone + Send + Sync> {
    inner: RawEntry<T>,
}

impl<T: TimeProvider + Clone + Send + Sync> Entry<i64> for I64Entry<T> {
    fn update(&self, data: i64) -> Result<()> {
        self.inner.log_data(Box::new(data.to_le_bytes()))
    }
}

pub struct F32Entry<T: TimeProvider + Clone + Send + Sync> {
    inner: RawEntry<T>,
}

impl<T: TimeProvider + Clone + Send + Sync> Entry<f32> for F32Entry<T> {
    fn update(&self, data: f32) -> Result<()> {
        self.inner.log_data(Box::new(data.to_le_bytes()))
    }
}

pub struct F64Entry<T: TimeProvider + Clone + Send + Sync> {
    inner: RawEntry<T>,
}

impl<T: TimeProvider + Clone + Send + Sync> Entry<f64> for F64Entry<T> {
    fn update(&self, data: f64) -> Result<()> {
        self.inner.log_data(Box::new(data.to_le_bytes()))
    }
}

pub struct StringEntry<T: TimeProvider + Clone + Send + Sync> {
    inner: RawEntry<T>,
}

impl<T: TimeProvider + Clone + Send + Sync> Entry<String> for StringEntry<T> {
    fn update(&self, data: String) -> Result<()> {
        self.inner.log_data(data.into_boxed_str().into())
    }
}
