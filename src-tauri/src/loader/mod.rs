use std::{collections::HashMap, io::{BufRead, BufReader, Read, Seek}};
use anyhow::anyhow;

use crate::{ext::{IndexStr, Indexer}, iter_extend, jvm, slice::ExtendSelf};

pub mod fabric;
pub mod forge;

pub trait Extractor {
    type Data;
    fn mod_info(&self) -> Self::Data;
    fn deps(&self) -> anyhow::Result<DepMap>;
    fn entries<RS: Read + Seek>(&self, zipfile: &mut zip::ZipArchive<RS>) -> anyhow::Result<jvm::ModEntries>;
}

pub enum Ld<ForFabric: Sized, ForForge: Sized> {
    Fabric(ForFabric),
    Forge(ForForge)
}

type ExtractLoader = Ld<fabric::ExtractFabric, forge::ExtractForge>;
impl Extractor for ExtractLoader {
    type Data = Ld<Box<[ModData; 1]>, Box<[ModData]>>;
    fn mod_info(&self) -> Self::Data {
        match self {
            Self::Fabric(x) => Ld::Fabric(x.mod_info()),
            Self::Forge(x) => Ld::Forge(x.mod_info()),
        }
    }
    fn deps(&self) -> anyhow::Result<DepMap> {
        match self {
            Self::Fabric(x) => x.deps(),
            Self::Forge(x) => x.deps(),
        }
    }
    fn entries<RS: Read + Seek>(&self, zipfile: &mut zip::ZipArchive<RS>) -> anyhow::Result<jvm::ModEntries> {
        match self {
            Self::Fabric(x) => x.entries(zipfile),
            Self::Forge(x) => x.entries(zipfile),
        }
    }
}

fn get_extractor<RS: Read + Seek>(zip: &mut zip::ZipArchive<RS>) -> anyhow::Result<ExtractLoader> {
    Ok(if let Some(ix) = zip.index_for_name("fabric.mod.json") {
        Ld::Fabric(fabric::ExtractFabric(json_safe_parse(zip.by_index(ix)?)?))
    } else if let Some(ix) = zip.index_for_name("META-INF/mods.toml") {
        let mut mf = zip.by_index(ix)?;
        let mut s = String::with_capacity(mf.size() as usize);
        mf.read_to_string(&mut s)?;
        drop(mf);
        let mut fmd: forge::ForgeMetadata = toml::from_str(&s)?;
        let impl_version = version_from_mf(zip);
        fmd.impl_version = impl_version;
        Ld::Forge(forge::ExtractForge(fmd))
    } else {
        return Err(anyhow!("No manifest in jar"));
    })
}

#[derive(serde::Serialize)]
#[serde(tag = "type", content = "mods")]
pub enum ModTypeData {
    Fabric(Box<[ModData; 1]>),
    Forge(Box<[ModData]>)
}

#[derive(serde::Serialize)]
pub struct ModData {
    name: Box<str>,
    slug: Box<str>,
    version: Box<str>,
    description: Option<Box<str>>,
    authors: Option<Box<str>>,
    license: Option<Box<str>>,
    logo_path: Option<Box<str>>,
    url: Option<Box<str>>,
}
impl ModData {
    pub const fn slug(&self) -> &str { &self.slug }
}

pub fn extract_mod_info<RS: Read + Seek>(zar: &mut zip::ZipArchive<RS>) -> anyhow::Result<ModTypeData> {
    Ok(match get_extractor(zar)?.mod_info() {
        Ld::Fabric(md) => ModTypeData::Fabric(md),
        Ld::Forge(md) => ModTypeData::Forge(md)
    })
}
pub fn extract_dep_map<RS: Read + Seek>(zar: &mut zip::ZipArchive<RS>) -> anyhow::Result<DepMap> {
    get_extractor(zar)?.deps()
}

#[derive(Clone, serde::Serialize)]
pub struct VersionData(ParsedVersionReq, VersionType);

#[derive(Clone, Copy, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum VersionType {
    Provided,
    Required,
    Optional
}

#[derive(Default, serde::Serialize)]
pub struct DepMap(Vec<(Box<str>, Option<semver::Version>, HashMap<Box<str>, VersionData>)>);

impl ExtendSelf for DepMap {
    fn extend(&mut self, other: &Self) {
        self.0.extend(other.0.iter().cloned());
    }
}
iter_extend!(DepMap);

#[derive(serde::Serialize)]
pub struct DepMapIndexed(Indexer, Vec<Option<(Option<semver::Version>, HashMap<usize, VersionData>)>>);
impl From<&DepMap> for DepMapIndexed {
    fn from(x: &DepMap) -> Self {
        let mut idxr = Indexer::default();
        let mut v = Vec::with_capacity(x.0.len());
        v.fill(None);
        for (n, o, d) in &x.0 {
            let i = idxr.find_or_insert(n);
            if i.1 >= v.len() { v.resize(i.1 + 1, None); }
            v[i.1] = Some((o.clone(), d.iter().map(|(n, v)| (idxr.find_or_insert(n).num(), v.clone())).collect::<HashMap<_, _>>()));
        }
        let mut iiv = idxr.iter().enumerate().map(|(i, x)| (i, x.1)).collect::<Vec<_>>();
        iiv.sort_by_key(|x| x.1);
        let ilv = idxr.iter()
            .map(|x| {
                let (o, d) = v.get_mut(x.1).map(std::mem::take).unwrap_or_default()?;
                let d = d.into_iter().map(|(k, v)| (iiv[k.1].0, v)).collect();
                Some((o, d))
            }).collect();
        Self(idxr, ilv)
    }
}

#[derive(Clone)]
pub enum ParsedVersionReq {
    Correct(semver::VersionReq),
    Invalid(Box<str>)
}
impl ParsedVersionReq {
    pub fn parse(v: &str) -> Self {
        semver::VersionReq::parse(v).map_or_else(|_| Self::Invalid(v.into()), Self::Correct)
    }
}
impl <'de> serde::Deserialize<'de> for ParsedVersionReq {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        let s = Box::<str>::deserialize(deserializer)?;
        Ok(semver::VersionReq::parse(&s).map_or_else(|_| Self::Invalid(s), Self::Correct))
    }
}
impl serde::Serialize for ParsedVersionReq {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        match self {
            Self::Correct(v) => v.serialize(serializer),
            Self::Invalid(s) => s.serialize(serializer)
        }
    }
}

fn version_from_mf<RS: Read + Seek>(zip: &mut zip::ZipArchive<RS>) -> Option<Box<str>> {
    let manifest = zip.by_name("META-INF/MANIFEST.MF").ok()?;
    let bufr = BufReader::new(manifest);
    bufr.lines().find_map(|l| l.ok().and_then(|l| l.strip_prefix("Implementation-Version: ").map(|x| x.to_string().into_boxed_str())))
}

pub fn extract_mod_entries<RS: Read + Seek>(zipfile: &mut zip::ZipArchive<RS>, mtd: &ModTypeData) -> anyhow::Result<jvm::ModEntries> {
    match mtd {
        ModTypeData::Fabric(_) => {
            if let Some(ix) = zipfile.index_for_name("fabric.mod.json") {
                let manifest: serde_json::Map<String, serde_json::Value> = json_safe_parse(zipfile.by_index(ix)?)?;
                let entrypoints = manifest.get("entrypoints")
                    .and_then(|v| v.as_object()?.get("main")?.as_array())
                    .ok_or_else(|| anyhow!("No entrypoints in fabric.mod.json"))?
                    .iter().filter_map(|v| v.as_str()).collect::<Box<_>>();
                let mut entries = Vec::with_capacity(entrypoints.len());
                for &e in entrypoints.iter() {
                    entries.push(jvm::scan_fabric_mod_entry(zipfile, e)?);
                }
                return Ok(jvm::ModEntries { classes: entries.into_boxed_slice() });
            }
            Err(anyhow!("No fabric.mod.json"))
        }
        ModTypeData::Forge(md) => {
            let slugs = md.iter().map(|m| &*m.slug).collect::<Box<_>>();
            let classes = jvm::scan_forge_mod_entries(zipfile, &slugs)?;
            Ok(jvm::ModEntries { classes })
        }
    }
}

fn json_safe_parse<R: Read, T: serde::de::DeserializeOwned>(r: R) -> serde_json::Result<T> {
    serde_json::from_reader(FlatReader(r))
}

#[repr(transparent)]
struct FlatReader<R>(R);
impl <R: Read> Read for FlatReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let len = self.0.read(buf)?;
        buf[..len].iter_mut().for_each(|b| match *b {
            b'\r' | b'\n' => *b = b' ',
            _ => {}
        });
        Ok(len)
    }
}

pub fn lenient_version(s: &str) -> Option<semver::Version> {
    semver::Version::parse(s).map_or_else(|_| {
        if let Ok(c) = semver::Comparator::parse(s) {
            Some(semver::Version {
                major: c.major,
                minor: c.minor.unwrap_or_default(),
                patch: c.patch.unwrap_or_default(),
                pre: c.pre,
                build: semver::BuildMetadata::EMPTY
            })
        } else {
            None
        }
    }, Some)
}