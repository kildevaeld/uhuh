use alloc::{format, string::ToString};
use vaerdi::{merge, Map, Value};

#[derive(Debug, Default, Clone)]
pub struct Config {
    pub(crate) inner: Map,
}

impl Config {
    pub fn get(&self, name: impl AsRef<str>) -> Option<&Value> {
        self.inner.get(name.as_ref())
    }

    pub fn get_mut<K>(&mut self, name: impl AsRef<str>) -> Option<&mut Value> {
        self.inner.get_mut(name.as_ref())
    }

    pub fn try_get<'a, S: serde::Deserialize<'a>>(
        &self,
        name: &str,
    ) -> Result<S, vaerdi::de::DeserializerError> {
        if let Some(v) = self.inner.get(name).cloned() {
            S::deserialize(v)
        } else {
            Err(vaerdi::de::DeserializerError::Custom(format!(
                "field not found: {}",
                name
            )))
        }
    }

    pub fn try_set<S: serde::Serialize>(
        &mut self,
        name: &str,
        value: S,
    ) -> Result<Option<Value>, vaerdi::ser::SerializerError> {
        Ok(self.inner.insert(name, vaerdi::ser::to_value(value)?))
    }

    pub fn set(&mut self, name: impl ToString, value: impl Into<Value>) -> Option<Value> {
        self.inner.insert(name.to_string(), value.into())
    }

    pub fn contains(&self, name: impl AsRef<str>) -> bool {
        self.inner.contains(name.as_ref())
    }

    pub fn extend(&mut self, config: Config) {
        for (key, value) in config.inner.into_iter() {
            if !self.inner.contains(&key) {
                self.inner.insert(key, value);
            } else {
                let prev = self.inner.get_mut(&key).unwrap();
                merge(prev, value);
            }
        }
    }

    pub fn try_into<'de, T: serde::Deserialize<'de>>(
        self,
    ) -> Result<T, vaerdi::de::DeserializerError> {
        T::deserialize(Value::Map(self.inner))
    }
}

impl<S: AsRef<str>> core::ops::Index<S> for Config {
    type Output = Value;
    fn index(&self, idx: S) -> &Self::Output {
        static NULL: Value = Value::Null;
        self.inner.get(idx.as_ref()).unwrap_or(&NULL)
    }
}

impl<S: AsRef<str>> core::ops::IndexMut<S> for Config {
    fn index_mut(&mut self, idx: S) -> &mut Self::Output {
        if !self.inner.contains(idx.as_ref()) {
            self.inner.insert(idx.as_ref().to_string(), Value::Null);
        }

        self.inner.get_mut(idx.as_ref()).unwrap()
    }
}

impl serde::Serialize for Config {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Config {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        Ok(Config {
            inner: Map::deserialize(deserializer)?,
        })
    }
}
