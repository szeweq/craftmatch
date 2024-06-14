use std::{fmt::Display, str::FromStr, time::Duration};

use base64::Engine;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[repr(transparent)]
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Id(Uuid);
impl Id {
    pub fn new(d: Duration) -> Self {
        let (secs, nanos) = (d.as_secs(), d.subsec_nanos());
        Self(Uuid::new_v7(uuid::Timestamp::from_unix(uuid::timestamp::context::NoContext, secs, nanos)))
    }
    pub fn b64(&self) -> Result<Box<[u8; 22]>, base64::EncodeSliceError> {
        let mut buf = [0u8; 22];
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode_slice(self.0.as_bytes(), &mut buf)?;
        Ok(buf.into())
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let buf = self.b64().map_err(|_| std::fmt::Error)?;
        let s = unsafe { std::str::from_utf8_unchecked(&*buf) };
        f.write_str(s)
    }
}
impl FromStr for Id {
    type Err = base64::DecodeSliceError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut buf = [0u8; 16];
        base64::engine::general_purpose::URL_SAFE_NO_PAD.decode_slice(s, &mut buf)?;
        Ok(Self(Uuid::from_bytes(buf)))
    }
}
impl Serialize for Id {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
impl<'de> Deserialize<'de> for Id {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}