use std::{collections::HashMap, io::{Read, Seek}, path::Path};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::ext::{self, Extension};

#[derive(Serialize, Clone, Copy)]
pub struct ModContentSizes {
    meta: [usize; 3],
    classes: [usize; 3],
    assets: [usize; 3],
    data: [usize; 3],
    other: [usize; 3]
}
impl ModContentSizes {
    pub const fn new() -> Self {
        Self { meta: [0, 0, 0], classes: [0, 0, 0], assets: [0, 0, 0], data: [0, 0, 0], other: [0, 0, 0] }
    }
    pub fn extend(&mut self, other: &Self) {
        for i in 0..3 {
            self.meta[i] += other.meta[i];
            self.classes[i] += other.classes[i];
            self.assets[i] += other.assets[i];
            self.data[i] += other.data[i];
            self.other[i] += other.other[i];
        }
    }
}
impl <R> FromIterator<R> for ModContentSizes where R: AsRef<Self> {
    fn from_iter<T: IntoIterator<Item = R>>(iter: T) -> Self {
        iter.into_iter().fold(Self::new(), |mut acc, x| { acc.extend(x.as_ref()); acc })
    }
}

pub fn compute_mod_content_sizes(jar_path: impl AsRef<Path>) -> Result<ModContentSizes> {
    let mut zipfile = ext::zip_open(jar_path)?;
    let mut mcs = ModContentSizes {
        meta: [0, 0, 0],
        classes: [0, 0, 0],
        assets: [0, 0, 0],
        data: [0, 0, 0],
        other: [0, 0, 0]
    };
    ext::zip_each(&mut zipfile, |file| {
        let fname = file.name();
        let op = match fname.split_once('/').map(|x| x.0) {
            Some("assets") => &mut mcs.assets,
            Some("data") => &mut mcs.data,
            Some("META-INF") => match Extension::from_path(fname) {
                Extension::Toml | Extension::Json | Extension::Properties | Extension::Mf => &mut mcs.meta,
                _ => &mut mcs.other
            }
            _ => match Extension::from_path(fname) {
                Extension::Class => &mut mcs.classes,
                Extension::Json => &mut mcs.meta,
                _ => &mut mcs.other
            }
        };
        op[0] += 1;
        op[1] += file.size() as usize;
        op[2] += file.compressed_size() as usize;
        Ok(())
    })?;
    Ok(mcs)
}

type KMap<V> = HashMap<Box<str>, V>;

#[derive(Debug, Serialize)]
pub struct TagsList(KMap<KMap<HashMap<TagItem, usize>>>);
impl TagsList {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn extend(&mut self, other: &Self) {
        for (k, v) in &other.0 {
            if let Some(m1) = self.0.get_mut(k) {
                for (k2, v2) in v {
                    if let Some(m2) = m1.get_mut(k2) {
                        for (k3, v3) in v2 {
                            *m2.entry(k3.clone()).or_default() += *v3;
                        }
                    } else {
                        m1.insert(k2.clone(), v2.clone());
                    }
                }
            } else {
                self.0.insert(k.clone(), v.clone());
            }
        }
    }
}
impl <R> FromIterator<R> for TagsList where R: AsRef<Self> {
    fn from_iter<T: IntoIterator<Item = R>>(iter: T) -> Self {
        iter.into_iter().fold(Self::new(), |mut acc, x| { acc.extend(x.as_ref()); acc })
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum TagItem {
    Tag(Box<str>),
    Item(Box<str>)
}
impl Serialize for TagItem {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        match self {
            Self::Tag(x) => serializer.serialize_str(&format!("#{x}")),
            Self::Item(x) => serializer.serialize_str(x)
        }
    }
}

#[derive(Deserialize)]
pub struct JsonTagsList {
    pub values: Box<[TagEntry]>
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum TagEntry {
    Simple(Box<str>),
    WithReq { id: Box<str>, required: bool }
}
impl TagEntry {
    pub const fn id(&self) -> &str {
        match self {
            Self::WithReq { id, .. } | Self::Simple(id) => id
        }
    }
}

pub fn gather_tags(zipfile: &mut zip::ZipArchive<impl Read + Seek>) -> Result<TagsList> {
    let mut tl = TagsList::new();
    ext::zip_each_by_extension(zipfile, Extension::Json, |mut file| {
        let filename = file.name().to_string();
        if let Some((ns, frest)) = filename.strip_prefix("data/").and_then(|fen| fen.split_once('/')) {
            if let Some((ptyp, prest)) = frest.strip_prefix("tags/").and_then(|fen| fen.split_once('/')) {
                let tname = &prest[..prest.len() - 5];
                let pe = tl.0.entry(ptyp.into()).or_default();
                let nx = format!("{ns}:{tname}").into_boxed_str();
                let tags: JsonTagsList = match serde_json::from_reader(&mut file) {
                    Ok(t) => t,
                    Err(e) => {
                        eprintln!("In {filename}: {e}");
                        return Ok(())
                    }
                };
                let nm = pe.entry(nx).or_default();
                for te in tags.values.as_ref() {
                    let tid = te.id();
                    let tt = tid.strip_prefix('#').map_or_else(|| TagItem::Item(tid.into()), |x| TagItem::Tag(x.into()));
                    *nm.entry(tt).or_default() += 1;
                }
            }
        }
        Ok(())
    })?;
    Ok(tl)
}

pub fn get_img_data(zip: &mut zip::ZipArchive<impl Read + Seek>, name: &str) -> Option<Vec<u8>> {
    let idx = zip.index_for_name(name)?;
    let mut file = zip.by_index(idx).ok()?;
    let mut buf = vec![0; file.size() as usize];
    file.read_exact(&mut buf).ok()?;
    Some(buf)
}
