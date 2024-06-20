use std::{collections::HashMap, io::{Read, Seek}};

use crate::jvm;

use super::{Extractor, ModData};
use anyhow::anyhow;

#[derive(serde::Deserialize)]
pub(super) struct FabricMetadata {
    id: Box<str>,
    name: Box<str>,
    version: Box<str>,
    authors: Box<[Box<str>]>,
    description: Option<Box<str>>,
    license: Option<Box<str>>,
    icon: Option<Box<str>>,
    contact: HashMap<Box<str>, Box<str>>,
    depends: HashMap<Box<str>, Box<str>>,
    suggests: HashMap<Box<str>, Box<str>>,
    entrypoints: HashMap<Box<str>, Box<[Box<str>]>> 
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
            authors: (!fm.authors.is_empty()).then(|| fm.authors.join(", ").into_boxed_str()),
            license: fm.license.clone(),
            logo_path: fm.icon.clone(),
            url: fm.contact.get("home").cloned()
        }])
    }
    fn deps(&self) -> anyhow::Result<()> {
        let fm = &self.0;
        eprintln!("depends: {:?}", fm.depends);
        eprintln!("suggests: {:?}", fm.suggests);
        Ok(())
    }
    fn entries<RS: Read + Seek>(&self, zipfile: &mut zip::ZipArchive<RS>) -> anyhow::Result<jvm::ModEntries> {
        let entrypoints = self.0.entrypoints.get("main")
            .ok_or_else(|| anyhow!("No entrypoints in fabric.mod.json"))?
            .iter().cloned().collect::<Box<_>>();
        let mut entries = Vec::with_capacity(entrypoints.len());
        for e in entrypoints.iter() {
            entries.push(jvm::scan_fabric_mod_entry(zipfile, e)?);
        }
        Ok(jvm::ModEntries { classes: entries.into_boxed_slice() })
    }
}
