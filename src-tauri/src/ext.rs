use std::{collections::VecDeque, ffi::OsStr, fs::File, io::{BufReader, Read, Seek}, path::Path};

use serde::Serialize;
use anyhow::Result;
use zip::ZipArchive;


pub enum Extension {
    Empty,
    Class,
    Json,
    Png,
    Ogg,
    Toml,
    Properties,
    Mf,
    Jar,
    Other(Box<str>)
}

impl Extension {
    pub fn from_path<P: AsRef<OsStr> + ?Sized>(p: &P) -> Self {
        let p = Path::new(p);
        let Some(x) = p.extension() else {
            return Self::Empty;
        };

        if x.eq_ignore_ascii_case("class") { Self::Class }
        else if x.eq_ignore_ascii_case("json") { Self::Json }
        else if x.eq_ignore_ascii_case("png") { Self::Png }
        else if x.eq_ignore_ascii_case("ogg") { Self::Ogg }
        else if x.eq_ignore_ascii_case("toml") { Self::Toml }
        else if x.eq_ignore_ascii_case("properties") { Self::Properties }
        else if x.eq_ignore_ascii_case("mf") { Self::Mf }
        else if x.eq_ignore_ascii_case("jar") { Self::Jar }
        else { Self::Other(x.to_string_lossy().into_owned().to_ascii_lowercase().into_boxed_str()) }
    }
    const fn str(&self) -> &str {
        match self {
            Self::Empty => "",
            Self::Class => "class",
            Self::Json => "json",
            Self::Png => "png",
            Self::Ogg => "ogg",
            Self::Toml => "toml",
            Self::Properties => "properties",
            Self::Mf => "mf",
            Self::Jar => "jar",
            Self::Other(x) => x
        }
    }
    #[inline]
    pub fn matches(&self, s: &(impl AsRef<OsStr> + ?Sized)) -> bool {
        Path::new(s).extension().map_or(false, |ext| ext.eq_ignore_ascii_case(self.str()))
    }
}

pub fn zip_open(p: impl AsRef<Path>) -> anyhow::Result<ZipArchive<BufReader<File>>> {
    Ok(ZipArchive::new(BufReader::new(File::open(p)?))?)
}

#[inline]
pub fn zip_each(zip: &mut ZipArchive<impl Read + Seek>, mut f: impl FnMut(zip::read::ZipFile) -> Result<()>) -> Result<()> {
    for i in 0..zip.len() {
        let zf = zip.by_index(i)?;
        if zf.is_dir() { continue; }
        f(zf)?;
    }
    Ok(())
}

#[inline]
pub fn zip_each_by_extension(zip: &mut ZipArchive<impl Read + Seek>, ext: Extension, mut f: impl FnMut(zip::read::ZipFile) -> Result<()>) -> Result<()> {
    for i in 0..zip.len() {
        let Some(name) = zip.name_for_index(i) else { continue; };
        if !ext.matches(name) { continue; }
        let zf = zip.by_index(i)?;
        if zf.is_dir() { continue; }
        f(zf)?;
    }
    Ok(())
}

#[inline]
pub fn zip_find_by_extension<T>(zip: &mut ZipArchive<impl Read + Seek>, ext: Extension, mut f: impl FnMut(zip::read::ZipFile) -> Result<Option<T>>) -> Result<Option<T>> {
    for i in 0..zip.len() {
        let Some(name) = zip.name_for_index(i) else { continue; };
        if !ext.matches(name) { continue; }
        let zf = zip.by_index(i)?;
        if zf.is_dir() { continue; }
        if let Some(x) = f(zf)? { return Ok(Some(x)); }
    }
    Ok(None)
}

#[derive(Serialize, Debug)]
pub struct Inheritance {
    pub indices: Vec<(Box<str>, usize)>,
    pub inherits: Vec<Vec<usize>>,
}
#[allow(dead_code)]
impl Inheritance {
    pub fn new() -> Self {
        Self {
            indices: vec![],
            inherits: vec![],
        }
    }
    pub fn name_by_index(&self, index: usize) -> Box<str> {
        self.indices.iter().find(|(_, i)| *i == index).map_or_else(|| Box::from(format!("#{index}")), |(n, _)| Box::from(n.as_ref()))
        //self.indices.binary_search_by_key(&&index, |(_, i)| i).map_or_else(|_| Cow::from(format!("#{index}")), |i| Cow::from(&self.indices[i].0))
    }
    pub fn iter_inherits(&self, index: usize) -> impl Iterator<Item = Box<str>> + '_ {
        self.inherits[index].iter().map(|&i| self.name_by_index(i))
    }
    pub fn find(&mut self, name: &str) -> usize {
        match self.indices.binary_search_by_key(&name, |(n, _)| n) {
            Ok(i) => self.indices[i].1,
            Err(i) => {
                let ni = self.inherits.len();
                self.indices.insert(i, (name.to_string().into_boxed_str(), ni));
                self.inherits.push(vec![]);
                ni
            }
        }
    }
    pub fn add_inherit(&mut self, index: usize, name: &str) -> usize {
        let ni = self.find(name);
        self.inherits[index].push(ni);
        ni
    }
    pub fn iter_tree(&self, index: usize) -> impl Iterator<Item = Box<str>> + '_ {
        let mut q = VecDeque::new();
        q.push_back(index);
        std::iter::from_fn(move || {
            let i = q.pop_front()?;
            Some(self.name_by_index(i))
        })
    }
    pub fn inherits(&self, index: usize, name: &str) -> bool {
        let mut q = VecDeque::new();
        q.push_back(index);
        while let Some(i) = q.pop_front() {
            if name == self.indices[i].0.as_ref() {
                return true;
            }
            q.extend(self.inherits[i].iter());
        }
        false
    }
    pub fn extend(&mut self, other: &Self) {
        for (n, oi) in other.indices.iter() {
            let i = self.find(n);
            let v = &mut self.inherits[i];
            v.extend_from_slice(&other.inherits[*oi]);
            v.dedup();
        }
    }
}

impl <R> FromIterator<R> for Inheritance where R: AsRef<Self> {
    fn from_iter<T: IntoIterator<Item = R>>(iter: T) -> Self {
        iter.into_iter().fold(Self::new(), |mut acc, x| { acc.extend(x.as_ref()); acc })
    }
}
