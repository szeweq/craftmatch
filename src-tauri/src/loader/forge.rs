use std::io::{Read, Seek};

use crate::jvm;

use super::{no_x_in_manifest, Extractor, ModData};

pub struct ExtractForge(pub toml::Table);

impl Extractor for ExtractForge {
    type Data = Box<[ModData]>;
    fn mod_info(&self) -> anyhow::Result<Self::Data> {
        let manifest = &self.0;
        let Some(toml::Value::Array(mods)) = manifest.get("mods") else { return Err(no_x_in_manifest("mods")); };
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
                    license: get_str(manifest, "license").map(Box::from),
                    logo_path: get_str(m, "logoFile").map(Box::from)
                })
            })
            .collect::<Box<_>>();
        if x.is_empty() { return Err(no_x_in_manifest("mods")); }
        Ok(x)
    }
    fn deps(&self) -> anyhow::Result<()> {
        let manifest = &self.0;
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
        Ok(())
    }
    fn entries<RS: Read + Seek>(&self, zipfile: &mut zip::ZipArchive<RS>) -> anyhow::Result<jvm::ModEntries> {
        let mi = self.mod_info()?;
        let slugs = mi.iter().map(|m| &*m.slug).collect::<Box<_>>();
        let classes = jvm::scan_forge_mod_entries(zipfile, &slugs)?;
        Ok(jvm::ModEntries { classes })
    }
}

fn get_str<'a>(t: &'a toml::Table, key: &'static str) -> Option<&'a str> {
    t.get(key).and_then(|v| v.as_str())
}
