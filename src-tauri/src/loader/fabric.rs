use std::{collections::HashMap, io::{Read, Seek}};

use crate::jvm;
use super::{lenient_version, DepMap, Extractor, ModData, ParsedVersionReq, VersionData, VersionType};
use anyhow::anyhow;

type VersionReqMap = HashMap<Box<str>, ParsedVersionReq>;

#[derive(serde::Deserialize)]
pub(super) struct FabricMetadata {
    id: Box<str>,
    name: Box<str>,
    version: Box<str>,
    authors: Box<[Authors]>,
    description: Option<Box<str>>,
    license: Option<OneOrMany<Box<str>>>,
    icon: Option<Box<str>>,
    contact: Option<HashMap<Box<str>, Box<str>>>,
    depends: Option<VersionReqMap>,
    suggests: Option<VersionReqMap>,
    entrypoints: Option<HashMap<Box<str>, Box<[Entrypoints]>>>
}

pub struct ExtractFabric(pub(super) FabricMetadata);

impl Extractor for ExtractFabric {
    type Data = Box<[ModData; 1]>;
    fn mod_info(&self) -> Self::Data {
        let fm = &self.0;
        Box::new([ModData {
            name: fm.name.clone(),
            slug: fm.id.clone(),
            version: fm.version.clone(),
            description: fm.description.clone(),
            authors: (!fm.authors.is_empty()).then(|| fm.authors.iter().map(Authors::str).collect::<Vec<_>>().join(", ").into_boxed_str()),
            license: fm.license.as_ref().map(|x| x.join(", ")),
            logo_path: fm.icon.clone(),
            url: fm.contact.as_ref().and_then(|m| m.get("home").cloned())
        }])
    }
    fn deps(&self) -> anyhow::Result<DepMap> {
        let fm = &self.0;
        let mut map = HashMap::new();
        if let Some(d) = &fm.depends {
            for (k, v) in d {
                map.insert(k.clone(), VersionData(v.clone(), VersionType::Required));
            }
        }
        if let Some(d) = &fm.suggests {
            for (k, v) in d {
                map.insert(k.clone(), VersionData(v.clone(), VersionType::Optional));
            }
        }
        Ok(DepMap(vec![(fm.id.clone(), lenient_version(&fm.version), map)]))
    }
    fn entries<RS: Read + Seek>(&self, zipfile: &mut zip::ZipArchive<RS>) -> anyhow::Result<jvm::ModEntries> {
        let Some(entrypoints) = self.0.entrypoints.as_ref() else {
            return Ok(jvm::ModEntries { classes: Box::new([]) })
        };
        let mep = entrypoints.get("main")
            .ok_or_else(|| anyhow!("No entrypoints in fabric.mod.json"))?
            .iter().map(Entrypoints::str).collect::<Box<_>>();
        let mut entries = Vec::with_capacity(mep.len());
        for e in mep.iter() {
            entries.push(jvm::scan_fabric_mod_entry(zipfile, e)?);
        }
        Ok(jvm::ModEntries { classes: entries.into_boxed_slice() })
    }
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum OneOrMany<T> {
    One(T),
    Many(Box<[T]>)
}
impl OneOrMany<Box<str>> {
    pub fn join(&self, sep: &str) -> Box<str> {
        match self {
            Self::One(x) => x.clone(),
            Self::Many(xs) => xs.join(sep).into_boxed_str()
        }
    }
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum Authors {
    String(Box<str>),
    Object{ name: Box<str> }
}
impl Authors {
    const fn str(&self) -> &str {
        match self {
            Self::String(x) => x,
            Self::Object { name } => name
        }
    }
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum Entrypoints {
    String(Box<str>),
    Object{ value: Box<str> }
}
impl Entrypoints {
    const fn str(&self) -> &str {
        match self {
            Self::String(x) => x,
            Self::Object { value } => value
        }
    }
}
