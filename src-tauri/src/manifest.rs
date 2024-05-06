#![allow(dead_code)]

use std::{collections::HashMap, io::{BufRead, Read, Seek}, path::Path};
use anyhow::anyhow;

use crate::{ext, jvm};

pub enum ModTypeMeta {
    Fabric(serde_json::Map<String, serde_json::Value>),
    Forge(toml::Table)
}
fn determine_mod_type(zip: &mut zip::ZipArchive<impl Read + Seek>) -> anyhow::Result<ModTypeMeta> {
    Ok(if let Some(ix) = zip.index_for_name("fabric.mod.json") {
        let manifest: serde_json::Map<String, serde_json::Value> = serde_json::from_reader(zip.by_index(ix)?)?;
        ModTypeMeta::Fabric(manifest)
    } else if let Some(ix) = zip.index_for_name("META-INF/mods.toml") {
        let mut mf = zip.by_index(ix)?;
        let mut s = String::with_capacity(mf.size() as usize);
        mf.read_to_string(&mut s)?;
        drop(mf);
        let manifest: toml::Table = toml::from_str(&s)?;
        ModTypeMeta::Forge(manifest)
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
    logo_path: Option<Box<str>>
}

fn no_x_in_manifest(key: &'static str) -> anyhow::Error {
    anyhow!("No {key} in manifest")
}

pub fn extract_mod_info(jar_path: impl AsRef<Path>) -> anyhow::Result<ModTypeData> {
    let mut zipfile = ext::zip_open(jar_path)?;
    let md = match determine_mod_type(&mut zipfile)? {
        ModTypeMeta::Fabric(manifest) => {
            fn get_str<'a>(m: &'a serde_json::Map<String, serde_json::Value>, key: &'static str) -> anyhow::Result<&'a str> {
                m.get(key).and_then(|v| v.as_str()).ok_or_else(|| no_x_in_manifest(key))
            }
            fn get_opt_str<'a>(m: &'a serde_json::Map<String, serde_json::Value>, key: &'static str) -> Option<&'a str> {
                m.get(key).and_then(|v| v.as_str())
            }
            let name = get_str(&manifest, "name")?;
            let slug = get_str(&manifest, "id")?;
            let version = get_str(&manifest, "version")?;
            let authors = manifest.get("authors")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join(", ").into_boxed_str());
            ModTypeData::Fabric(Box::new([ModData {
                name: name.into(),
                slug: slug.into(),
                version: version.into(),
                description: get_opt_str(&manifest, "description").map(Box::from),
                authors,
                license: get_opt_str(&manifest, "license").map(Box::from),
                logo_path: get_opt_str(&manifest, "icon").map(Box::from)
            }]))
        }
        ModTypeMeta::Forge(manifest) => {
            fn get_str<'a>(t: &'a toml::Table, key: &'static str) -> Option<&'a str> {
                t.get(key).and_then(|v| v.as_str())
            }
            let Some(toml::Value::Array(mods)) = manifest.get("mods") else { return Err(anyhow!("No mods in manifest")); };
            let x = mods.iter()
                .filter_map(|v| v.as_table())
                .filter_map(|m| {
                    let name = get_str(m, "displayName")?;
                    let slug = get_str(m, "modId")?;
                    let version = get_str(m, "version")?;
                    Some(ModData {
                        name: name.into(),
                        slug: slug.into(),
                        version: version.into(),
                        authors: get_str(m, "authors").map(Box::from),
                        description: get_str(m, "description").map(Box::from),
                        license: get_str(&manifest, "license").map(Box::from),
                        logo_path: get_str(m, "logoFile").map(Box::from)
                    })
                })
                .collect::<Box<_>>();
            if x.is_empty() { return Err(anyhow!("No mods in manifest")); }
            ModTypeData::Forge(x)
        }
    };

    Ok(md)
}

pub struct VersionInfo {
    pub provided: Option<Box<str>>,
    pub required: Vec<Box<str>>,
    pub optional: Vec<Box<str>>
}
pub struct VersionMap(HashMap<Box<str>, VersionInfo>);

fn version_from_mf(zip: &mut zip::ZipArchive<impl Read + Seek>) -> anyhow::Result<Box<str>> {
    let manifest = zip.by_name("META-INF/MANIFEST.MF")?;
    let bufr = std::io::BufReader::new(manifest);
    let Some(fl) = bufr.lines().find_map(|l| l.ok().and_then(|l| l.strip_prefix("Implementation-Version: ").map(|x| x.to_string().into_boxed_str()))) else {
        return Err(anyhow!("No version in manifest"));
    };
    Ok(fl)
}

pub fn extract_versions(jar_path: impl AsRef<Path>) -> anyhow::Result<()> {
    let mut zipfile = ext::zip_open(jar_path)?;
    match determine_mod_type(&mut zipfile)? {
        ModTypeMeta::Fabric(manifest) => {
            let depends = manifest.get("depends");
            let suggests = manifest.get("suggests");
            eprintln!("depends: {depends:?}");
            eprintln!("suggests: {suggests:?}");
        }
        ModTypeMeta::Forge(manifest) => {
            if let Some(dependencies) = manifest.get("dependencies").and_then(|v| v.as_table()) {
                for (dn, dv) in dependencies {
                    eprintln!("{dn}");
                    if let Some(depends) = dv.as_array() {
                        for d in depends {
                            eprintln!(" - {d}");
                        }
                    } else {
                        eprintln!(" - {dv:?}");
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn extract_mod_entries(zipfile: &mut zip::ZipArchive<impl Read + Seek>, mtd: &ModTypeData) -> anyhow::Result<jvm::ModEntries> {
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