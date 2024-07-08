use std::{collections::HashMap, io::{Read, Seek}};

use crate::{jvm, loader::{VersionData, VersionType}};

use super::{lenient_version, DepMap, Extractor, ModData, ParsedVersionReq};

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ForgeMetadata {
    #[serde(skip)]
    pub(super) impl_version: Option<Box<str>>,
    license: Option<Box<str>>,
    //#[serde(rename = "issueTrackerURL")]
    //issue_tracker_url: Option<Box<str>>,
    mods: Box<[ForgeModInfo]>,
    dependencies: HashMap<Box<str>, Vec<ForgeDependency>>
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ForgeModInfo {
    mod_id: Box<str>,
    display_name: Box<str>,
    version: Box<str>,
    authors: Option<Box<str>>,
    description: Option<Box<str>>,
    logo_file: Option<Box<str>>,
    #[serde(rename = "displayURL")]
    display_url: Option<Box<str>>,
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ForgeDependency {
    mod_id: Box<str>,
    mandatory: bool,
    version_range: Box<str>,
    ordering: Option<Box<str>>,
    side: Option<Box<str>>
}

pub struct ExtractForge(pub(super) ForgeMetadata);

impl Extractor for ExtractForge {
    type Data = Box<[ModData]>;
    fn mod_info(&self) -> Self::Data {
        let license = &self.0.license;
        let mods = &self.0.mods;
        let impl_version = &self.0.impl_version;
        mods.iter().map(|fmi| ModData {
            name: fmi.display_name.clone(),
            slug: fmi.mod_id.clone(),
            version: if fmi.version.trim_start().starts_with('$') {
                impl_version.as_ref().unwrap_or(&fmi.version)
            } else {
                &fmi.version
            }.clone(),
            authors: fmi.authors.clone(),
            description: fmi.description.clone(),
            license: license.clone(),
            logo_path: fmi.logo_file.clone(),
            url: fmi.display_url.clone()
        })
        .collect::<Box<_>>()
    }
    fn deps(&self) -> anyhow::Result<DepMap> {
        let mut v = Vec::new();
        let impl_version = &self.0.impl_version;
        let depm = &self.0.dependencies;
        for fmi in self.0.mods.iter() {
            let Some(dv) = depm.get(&fmi.mod_id) else { continue; };
            let dver = if fmi.version.trim_start().starts_with('$') {
                impl_version.as_ref().unwrap_or(&fmi.version)
            } else {
                &fmi.version
            };
            let mut map = HashMap::new();
            for d in dv {
                let vd = VersionData(
                    translate_version(&d.version_range)?,
                    if d.mandatory { VersionType::Required } else { VersionType::Optional }
                );
                map.insert(d.mod_id.clone(), vd);
            }
            v.push((fmi.mod_id.clone(), lenient_version(dver), map));
        }
        Ok(DepMap(v))
    }
    fn entries<RS: Read + Seek>(&self, zipfile: &mut zip::ZipArchive<RS>) -> anyhow::Result<jvm::ModEntries> {
        let mi = self.mod_info();
        let slugs = mi.iter().map(|m| &*m.slug).collect::<Box<_>>();
        let classes = jvm::scan_forge_mod_entries(zipfile, &slugs)?;
        Ok(jvm::ModEntries { classes })
    }
}

fn translate_version(mut ver: &str) -> anyhow::Result<ParsedVersionReq> {
    ver = ver.trim();
    let start = if ver.starts_with('[') {
        Some(true)
    } else if ver.starts_with('(') {
        Some(false)
    } else {
        None
    };
    let end = if ver.ends_with(']') {
        Some(true)
    } else if ver.ends_with(')') {
        Some(false)
    } else {
        None
    };
    if let (Some(b), Some(e)) = (start, end) {
        ver = &ver[1..ver.len()-1];
        let (mut vfrom, mut vto) = ver.split_once(',').ok_or_else(|| anyhow::anyhow!("Invalid version range"))?;
        vfrom = vfrom.trim();
        vto = vto.trim();
        let mut cv = Vec::new();
        if !vfrom.is_empty() {
            let mut cc = semver::Comparator::parse(vfrom)?;
            cc.op = if b { semver::Op::GreaterEq } else { semver::Op::Greater };
            cv.push(cc);
        }
        if !vto.is_empty() {
            let mut cc = semver::Comparator::parse(vto)?;
            cc.op = if e { semver::Op::LessEq } else { semver::Op::Less };
            cv.push(cc);
        }
        Ok(ParsedVersionReq::Correct(semver::VersionReq { comparators: cv }))
    } else {
        Ok(ParsedVersionReq::parse(ver))
    }
}