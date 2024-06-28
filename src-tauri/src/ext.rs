use std::{collections::VecDeque, ffi::OsStr, io::{Read, Seek}, path::Path};

use serde::Serialize;
use zip::{read::ZipFile, result::{ZipError, ZipResult}, ZipArchive};

use crate::{iter_extend, slice::ExtendSelf};


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
        else { Self::Other(x.to_ascii_lowercase().to_string_lossy().into_owned().into_boxed_str()) }
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
    pub fn matches<P: AsRef<OsStr> + ?Sized>(&self, s: &P) -> bool {
        Path::new(s).extension().map_or(false, |ext| ext.eq_ignore_ascii_case(self.str()))
    }
    #[inline]
    pub fn names_iter<'a, RS: Read + Seek>(&'a self, zar: &'a ZipArchive<RS>) -> impl Iterator<Item = &str> + '_ {
        zar.file_names().filter(|&x| self.matches(x))
    }
}

#[inline]
pub fn zip_file_iter<RS: Read + Seek>(z: &mut ZipArchive<RS>) -> ZipFileIter<RS, fn(&str) -> bool> {
    ZipFileIter(0, z, |_| true)
}

#[inline]
pub fn zip_file_ext_iter<RS: Read + Seek>(z: &mut ZipArchive<RS>, ext: Extension) -> ZipFileIter<RS, impl Fn(&str) -> bool> {
    ZipFileIter(0, z, move |name| ext.matches(name))
}
// pub fn zip_file_match_iter<RS: Read + Seek, F: Fn(&str) -> bool>(z: &mut ZipArchive<RS>, f: F) -> ZipFileIter<RS, F> {
//     ZipFileIter(0, z, f)
// }

pub struct ZipFileIter<'a, RS: Read + Seek, F: Fn(&str) -> bool>(usize, &'a mut ZipArchive<RS>, F);
impl <'a, RS: Read + Seek, F: Fn(&str) -> bool> ZipFileIter<'a, RS, F> {
    fn next_index(&mut self) -> Option<usize> {
        Some(loop {
            let i = self.0;
            let name = self.1.name_for_index(i)?;
            self.0 += 1;
            if !is_zip_dir(name) && self.2(name) {
                break i
            }
        })
    }
}
impl <'a, RS: Read + Seek, F: Fn(&str) -> bool> Iterator for ZipFileIter<'a, RS, F> {
    type Item = ZipResult<ZipFile<'a>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let i = self.next_index()?;
        let azip: &'a mut ZipArchive<RS> = unsafe { std::mem::transmute(&mut *self.1) };
        let x = azip.by_index(i);
        if matches!(x, Err(ZipError::FileNotFound)) { None } else { Some(x) }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let rest = self.1.len() - self.0;
        (rest, Some(rest))
    }
}

#[inline]
fn is_zip_dir(p: &str) -> bool {
    p.ends_with(&['/', '\\'])
}

#[derive(Serialize, Default)]
pub struct Inheritance {
    pub indices: Vec<(Box<str>, usize)>,
    pub inherits: Vec<Vec<usize>>,
}
#[allow(dead_code)]
impl Inheritance {
    pub fn name_by_index(&self, index: usize) -> Option<&str> {
        self.indices.iter().find(|(_, i)| *i == index).map(|(n, _)| n.as_ref())
        //self.indices.binary_search_by_key(&&index, |(_, i)| i).map_or_else(|_| Cow::from(format!("#{index}")), |i| Cow::from(&self.indices[i].0))
    }
    pub fn iter_inherits(&self, index: usize) -> impl Iterator<Item = &str> + '_ {
        self.inherits[index].iter().filter_map(|&i| self.name_by_index(i))
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
}
impl ExtendSelf for Inheritance {
    fn extend(&mut self, other: &Self) {
        for (n, oi) in &other.indices {
            let i = self.find(n);
            let v: &mut Vec<usize> = unsafe {
                std::mem::transmute(&mut self.inherits[i])
            };
            for x in other.inherits[*oi].iter().filter_map(|&k| other.name_by_index(k)) {
                let nk = self.find(x);
                v.push(nk);
            }
            v.dedup();
        }
    }
}
iter_extend!(Inheritance);
