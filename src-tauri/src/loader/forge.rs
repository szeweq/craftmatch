use std::{collections::HashMap, io::{Read, Seek}};

use crate::jvm;

use super::{Extractor, ModData};

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ForgeMetadata {
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
        mods.iter().map(|fmi| ModData {
            name: fmi.display_name.clone(),
            slug: fmi.mod_id.clone(),
            version: fmi.version.clone(),
            authors: fmi.authors.clone(),
            description: fmi.description.clone(),
            license: license.clone(),
            logo_path: fmi.logo_file.clone(),
            url: fmi.display_url.clone()
        })
        .collect::<Box<_>>()
    }
    fn deps(&self) -> anyhow::Result<()> {
        let depm = &self.0.dependencies;
        for (dn, dv) in depm {
            eprintln!("{dn}");
            for d in dv {
                eprintln!(" - {}", d.mod_id);
            }
        }
        Ok(())
    }
    fn entries<RS: Read + Seek>(&self, zipfile: &mut zip::ZipArchive<RS>) -> anyhow::Result<jvm::ModEntries> {
        let mi = self.mod_info();
        let slugs = mi.iter().map(|m| &*m.slug).collect::<Box<_>>();
        let classes = jvm::scan_forge_mod_entries(zipfile, &slugs)?;
        Ok(jvm::ModEntries { classes })
    }
}
