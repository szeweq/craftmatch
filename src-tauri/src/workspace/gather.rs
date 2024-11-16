use std::sync::Arc;

use indexmap::IndexMap;
use parking_lot::RwLockReadGuard;
use rayon::iter::ParallelIterator;

use crate::{ext, extract, id::Id, jvm, loader};

use super::{FileInfo, LockMap};


pub trait AllGather {
    fn gather_with<T: Send + Sync + 'static>(&self, force: bool, gfn: Gatherer<T>) -> RwLockReadGuard<'_, IndexMap<Id, FileInfo>>;
    fn gather_by_id<T: Send + Sync + 'static>(&self, id: Id, gfn: Gatherer<T>) -> anyhow::Result<Arc<T>>;
}
impl AllGather for LockMap<FileInfo> {
    fn gather_with<T: Send + Sync + 'static>(&self, force: bool, gfn: Gatherer<T>) -> RwLockReadGuard<'_, IndexMap<Id, FileInfo>> {
        self.write().par_values_mut().for_each(|file_entry| {
            if let Err(e) = file_entry.gather(gfn, force) {
                eprintln!("{}: {}", file_entry.path.display(), e);
            }
        });
        self.read()
    }
    fn gather_by_id<T: Send + Sync + 'static>(&self, id: Id, gfn: Gatherer<T>) -> anyhow::Result<Arc<T>> {
        let fe = &mut *self.write();
        let Some(fi) = fe.get_mut(&id) else { anyhow::bail!("file not found") };
        fi.get_or_gather(gfn)
    }
}

pub type Gatherer<T> = fn(&FileInfo) -> anyhow::Result<T>;

fn get_file_map(fi: &FileInfo) -> anyhow::Result<Arc<cm_zipext::FileMap>> {
    fi.filemap.upgrade().ok_or_else(|| anyhow::anyhow!("No file map"))
}

pub fn gather_mod_data(fi: &FileInfo) -> anyhow::Result<loader::ModTypeData> {
    let fm = get_file_map(fi)?;
    loader::extract_mod_info(&fm, &mut fi.file_buf()?)
}
pub fn gather_dep_map(fi: &FileInfo) -> anyhow::Result<loader::DepMap> {
    let fm = get_file_map(fi)?;
    loader::extract_dep_map(&fm, &mut fi.file_buf()?)
}
pub fn gather_file_type_sizes(fi: &FileInfo) -> anyhow::Result<extract::ModFileTypeSizes> {
    let fm = get_file_map(fi)?;
    extract::compute_file_type_sizes(&fm)
}
pub fn gather_content_sizes(fi: &FileInfo) -> anyhow::Result<extract::ModContentSizes> {
    let fm = get_file_map(fi)?;
    extract::compute_mod_content_sizes(&fm)
}
pub fn gather_inheritance(fi: &FileInfo) -> anyhow::Result<ext::Inheritance> {
    let fm = get_file_map(fi)?;
    jvm::gather_inheritance_v2(&fm, &mut fi.file_mem()?)
}
pub fn gather_complexity(fi: &FileInfo) -> anyhow::Result<jvm::Complexity> {
    let fm = get_file_map(fi)?;
    jvm::gather_complexity(&fm, &mut fi.file_mem()?)
}
pub fn gather_tags(fi: &FileInfo) -> anyhow::Result<extract::TagsList> {
    let fm = get_file_map(fi)?;
    extract::gather_tags(&fm, &mut fi.file_mem()?)
}
pub fn gather_str_index(fi: &FileInfo) -> anyhow::Result<jvm::StrIndexMapped> {
    let fm = get_file_map(fi)?;
    jvm::gather_str_index_v2(&fm, &mut fi.file_mem()?)
}
pub fn gather_mod_entries(fi: &FileInfo) -> anyhow::Result<jvm::ModEntries> {
    let fm = get_file_map(fi)?;
    let Some(moddata) = fi.get::<loader::ModTypeData>() else { return Err(anyhow::anyhow!("No moddata")) };
    loader::extract_mod_entries(&fm, moddata.as_ref(), &mut fi.file_mem()?)
}
pub fn gather_recipes(fi: &FileInfo) -> anyhow::Result<extract::RecipeTypeMap> {
    let fm = get_file_map(fi)?;
    extract::gather_recipes(&fm, &mut fi.file_mem()?)
}
pub fn gather_playable(fi: &FileInfo) -> anyhow::Result<extract::PlayableFiles> {
    let fm = get_file_map(fi)?;
    Ok(extract::gather_playable_files(&fm))
}