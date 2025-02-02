use std::{collections::VecDeque, ffi::OsStr, hash, marker::PhantomData, ops::Deref, path::Path, sync::Arc};
use serde::Serialize;
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
        Self::ext(Path::new(p))
    }
    fn ext(p: &Path) -> Self {
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
        Path::new(s).extension().is_some_and(|ext| ext.eq_ignore_ascii_case(self.str()))
    }
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

pub struct IndexStr<T: ?Sized>(pub Arc<str>, pub usize, PhantomData<fn() -> T>);
impl IndexStr<str> {
    pub fn num(self) -> IndexStr<usize> {
        IndexStr(self.0, self.1, PhantomData)
    }
}

impl <T> PartialEq for IndexStr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ref() == other.0.as_ref()
    }
}
impl <T> Eq for IndexStr<T> {}
impl <T> hash::Hash for IndexStr<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.as_ref().hash(state);
    }
}
impl <T: ?Sized> Clone for IndexStr<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0), self.1, PhantomData)
    }
}

impl serde::Serialize for IndexStr<str> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        serializer.serialize_str(self.0.as_ref())
    }
}
impl serde::Serialize for IndexStr<usize> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        serializer.serialize_u64(self.1 as u64)
    }
}

#[repr(transparent)]
#[derive(Default, serde::Serialize)]
pub struct Indexer(Vec<IndexStr<str>>);
impl Indexer {
    pub fn find_or_insert(&mut self, s: &str) -> IndexStr<str> {
        let i = match self.0.binary_search_by_key(&s, |x| x.0.as_ref()) {
            Ok(i) => i,
            Err(i) => {
                self.0.insert(i, IndexStr(Arc::from(s), self.0.len(), PhantomData));
                i
            }
        };
        self.0[i].clone()
    }
}
impl Deref for Indexer {
    type Target = [IndexStr<str>];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}