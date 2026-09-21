use crate::error::AppError;
use serde::{Deserialize, Deserializer};

// #[serde(default)] maps absence to Missing; the deserializer handles explicit null.
#[derive(Debug, Default)]
pub enum Patch<T> {
    #[default]
    Missing,
    Null,
    Value(T),
}
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Patch<T> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Option::<T>::deserialize(d).map(|v| match v {
            Some(v) => Self::Value(v),
            None => Self::Null,
        })
    }
}
impl<T> Patch<T> {
    pub fn required(self, field: &str) -> Result<Option<T>, AppError> {
        match self {
            Self::Missing => Ok(None),
            Self::Value(v) => Ok(Some(v)),
            Self::Null => Err(AppError::bad(format!("{field} no admite null"))),
        }
    }
    pub fn optional(self) -> Option<Option<T>> {
        match self {
            Self::Missing => None,
            Self::Null => Some(None),
            Self::Value(v) => Some(Some(v)),
        }
    }
}
