use std::{collections::HashMap, io::{Read, Seek}};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{ext::{self, Extension}, slice::{iter_extend, ExtendSelf}};

#[derive(Serialize, Default)]
pub struct ModFileTypeSizes(HashMap<Box<str>, [usize; 3]>);
impl ExtendSelf for ModFileTypeSizes {
    fn extend(&mut self, other: &Self) {
        for (k, v) in &other.0 {
            let av = self.0.entry(k.clone()).or_default();
            for i in 0..3 {
                av[i] += v[i];
            }
        }
    }
}
iter_extend!(ModFileTypeSizes);

#[derive(Serialize, Clone, Copy, Default)]
pub struct ModContentSizes {
    meta: [usize; 3],
    classes: [usize; 3],
    assets: [usize; 3],
    data: [usize; 3],
    other: [usize; 3]
}
impl ExtendSelf for ModContentSizes {
    fn extend(&mut self, other: &Self) {
        for i in 0..3 {
            self.meta[i] += other.meta[i];
            self.classes[i] += other.classes[i];
            self.assets[i] += other.assets[i];
            self.data[i] += other.data[i];
            self.other[i] += other.other[i];
        }
    }
}
iter_extend!(ModContentSizes);

pub fn compute_file_type_sizes<RS: Read + Seek>(zar: &mut zip::ZipArchive<RS>) -> Result<ModFileTypeSizes> {
    ext::zip_file_iter(zar).try_fold(ModFileTypeSizes::default(), |mut mfts, file| {
        let file = file?;
        let fname = file.name();
        let ext = match fname.rsplit_once('.') {
            None | Some(("", _) | (_, "")) => "".into(),
            Some((_, x)) => x.to_lowercase().into_boxed_str()
        };
        let op = mfts.0.entry(ext).or_default();
        op[0] += 1;
        op[1] += file.size() as usize;
        op[2] += file.compressed_size() as usize;
        Ok(mfts)
    })
}

pub fn compute_mod_content_sizes<RS: Read + Seek>(zar: &mut zip::ZipArchive<RS>) -> Result<ModContentSizes> {
    ext::zip_file_iter(zar).try_fold(ModContentSizes::default(), |mut mcs, file| {
        let file = file?;
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
        Ok(mcs)
    })
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

pub fn gather_tags<RS: Read + Seek>(mut zar: zip::ZipArchive<RS>) -> Result<TagsList> {
    ext::zip_file_ext_iter(&mut zar, Extension::Json).try_fold(TagsList::new(), |mut tl, file| {
        let mut file = file?;
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
                        return Ok(tl)
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
        Ok(tl)
    })
}

pub fn get_raw_data(zip: &mut zip::ZipArchive<impl Read + Seek>, name: &str) -> Option<Vec<u8>> {
    let idx = zip.index_for_name(name)?;
    let mut file = zip.by_index(idx).ok()?;
    let mut buf = vec![0; file.size() as usize];
    file.read_exact(&mut buf).ok()?;
    Some(buf)
}

#[derive(Deserialize)]
pub struct RecipeData {
    #[serde(rename = "type")]
    pub typ: Box<str>,
}

#[derive(Serialize)]
pub struct RecipeTypeMap(HashMap<Box<str>, Vec<Box<str>>>);
impl RecipeTypeMap {
    pub fn extend(&mut self, other: &Self) {
        for (k, v) in &other.0 {
            self.0.entry(k.clone()).or_default().extend_from_slice(v.as_slice());
        }
    }
}
impl <R> FromIterator<R> for RecipeTypeMap where R: AsRef<Self> {
    fn from_iter<T: IntoIterator<Item = R>>(iter: T) -> Self {
        iter.into_iter().fold(Self(HashMap::new()), |mut acc, x| { acc.extend(x.as_ref()); acc })
    }
}

pub fn gather_recipes(zipfile: &mut zip::ZipArchive<impl Read + Seek>) -> Result<RecipeTypeMap> {
    let recipes = ext::zip_file_ext_iter(zipfile, Extension::Json).try_fold(HashMap::<Box<str>, Vec<Box<str>>>::new(), |mut recipes, file| {
        let mut file = file?;
        let filename = file.name().to_string();
        if let Some((ns, frest)) = filename.strip_prefix("data/").and_then(|fen| fen.split_once('/')) {
            if let Some(pname) = frest.strip_prefix("recipes/") {
                let tname = &pname[..pname.len() - 5];
                let nx = format!("{ns}:{tname}").into_boxed_str();
                let recipe: RecipeData = match serde_json::from_reader(&mut file) {
                    Ok(t) => t,
                    Err(e) => {
                        eprintln!("In {filename}: {e}");
                        return anyhow::Ok(recipes)
                    }
                };
                recipes.entry(recipe.typ).or_default().push(nx);
            }
        }
        Ok(recipes)
    })?;
    Ok(RecipeTypeMap(recipes))
}

#[derive(Serialize)]
pub struct PlayableFiles(Box<[Box<str>]>);

pub fn gather_playable_files(zipfile: &zip::ZipArchive<impl Read + Seek>) -> PlayableFiles {
    let mut files = zipfile.file_names()
        .filter(|&x| Extension::Ogg.matches(x)).map(Box::from)
        .collect::<Vec<_>>();
    files.sort();
    PlayableFiles(files.into_boxed_slice())
}