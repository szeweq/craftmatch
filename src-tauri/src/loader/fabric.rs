use std::io::{Read, Seek};

use crate::jvm;

use super::{no_x_in_manifest, Extractor, ModData};
use anyhow::anyhow;

pub struct ExtractFabric(pub serde_json::Map<String, serde_json::Value>);

impl Extractor for ExtractFabric {
    type Data = Box<[ModData; 1]>;
    fn mod_info(&self) -> anyhow::Result<Self::Data> {
        let manifest = &self.0;
        let name = get_str(manifest, "name")?;
        let slug = get_str(manifest, "id")?;
        let version = get_str(manifest, "version")?;
        let authors = manifest.get("authors")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join(", ").into_boxed_str());
        Ok(Box::new([ModData {
            name: name.into(),
            slug: slug.into(),
            version: version.into(),
            description: get_opt_str(manifest, "description").map(Box::from),
            authors,
            license: get_opt_str(manifest, "license").map(Box::from),
            logo_path: get_opt_str(manifest, "icon").map(Box::from)
        }]))
    }
    fn deps(&self) -> anyhow::Result<()> {
        let manifest = &self.0;
        let depends = manifest.get("depends");
        let suggests = manifest.get("suggests");
        eprintln!("depends: {depends:?}");
        eprintln!("suggests: {suggests:?}");
        Ok(())
    }
    fn entries<RS: Read + Seek>(&self, zipfile: &mut zip::ZipArchive<RS>) -> anyhow::Result<jvm::ModEntries> {
        let manifest = &self.0;
        let entrypoints = manifest.get("entrypoints")
            .and_then(|v| v.as_object()?.get("main")?.as_array())
            .ok_or_else(|| anyhow!("No entrypoints in fabric.mod.json"))?
            .iter().filter_map(|v| v.as_str()).collect::<Box<_>>();
        let mut entries = Vec::with_capacity(entrypoints.len());
        for &e in entrypoints.iter() {
            entries.push(jvm::scan_fabric_mod_entry(zipfile, e)?);
        }
        Ok(jvm::ModEntries { classes: entries.into_boxed_slice() })
    }
}

fn get_str<'a>(m: &'a serde_json::Map<String, serde_json::Value>, key: &'static str) -> anyhow::Result<&'a str> {
    m.get(key).and_then(|v| v.as_str()).ok_or_else(|| no_x_in_manifest(key))
}
fn get_opt_str<'a>(m: &'a serde_json::Map<String, serde_json::Value>, key: &'static str) -> Option<&'a str> {
    m.get(key).and_then(|v| v.as_str())
}
