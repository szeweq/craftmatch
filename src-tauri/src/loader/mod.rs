use std::{collections::HashMap, io::{BufRead, BufReader, Read, Seek}};
use anyhow::anyhow;

use crate::{iter_extend, jvm, slice::ExtendSelf};

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
        Ld::Fabric(fabric::ExtractFabric(serde_json::from_reader(zip.by_index(ix)?)?))
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
pub struct VersionData(semver::VersionReq, VersionType);

#[derive(Clone, Copy, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum VersionType {
    Provided,
    Required,
    Optional
}

#[derive(Default, serde::Serialize)]
pub struct DepMap(Vec<(Box<str>, HashMap<Box<str>, VersionData>)>);

impl ExtendSelf for DepMap {
    fn extend(&mut self, other: &Self) {
        self.0.extend(other.0.iter().cloned());
    }
}
iter_extend!(DepMap);

fn version_from_mf<RS: Read + Seek>(zip: &mut zip::ZipArchive<RS>) -> Option<Box<str>> {
    let manifest = zip.by_name("META-INF/MANIFEST.MF").ok()?;
    let bufr = BufReader::new(manifest);
    bufr.lines().find_map(|l| l.ok().and_then(|l| l.strip_prefix("Implementation-Version: ").map(|x| x.to_string().into_boxed_str())))
}

pub fn extract_mod_entries<RS: Read + Seek>(zipfile: &mut zip::ZipArchive<RS>, mtd: &ModTypeData) -> anyhow::Result<jvm::ModEntries> {
    match mtd {
        ModTypeData::Fabric(_) => {
            if let Some(ix) = zipfile.index_for_name("fabric.mod.json") {
                let manifest: serde_json::Map<String, serde_json::Value> = serde_json::from_reader(zipfile.by_index(ix)?)?;
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