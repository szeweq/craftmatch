use std::{collections::HashMap, io::{Read, Seek}, path::Path, sync::Mutex};

use cafebabe::attributes::{AnnotationElement, AnnotationElementValue, AttributeData};
use once_cell::sync::Lazy;
use serde::Serialize;
use zip::ZipArchive;

use crate::{ext, jclass::{self, pool::PoolIter}};

pub static PARSE_TIMES: Lazy<Mutex<HashMap<Box<str>, std::time::Duration>>> = Lazy::new(|| Mutex::new(HashMap::new()));

fn parse_class_safe(b: &[u8], bytecode: bool) -> Result<cafebabe::ClassFile<'_>, cafebabe::ParseError> {
    let now = std::time::Instant::now();
    match cafebabe::parse_class_with_options(b, cafebabe::ParseOptions::default().parse_bytecode(bytecode)) {
        Ok(x) => {
            PARSE_TIMES.lock().unwrap().insert(x.this_class.clone().into(), now.elapsed());
            Ok(x)
        }
        e => e
    }
}

#[inline]
fn zip_each_class<F>(zip: &mut zip::ZipArchive<impl Read + Seek>, bytecode: bool, mut f: F) -> anyhow::Result<()>
where for<'a> F: FnMut(&'a cafebabe::ClassFile<'a>) -> anyhow::Result<()> {
    ext::zip_file_ext_iter(zip, ext::Extension::Class).try_for_each(|zf| {
        let mut zf = zf?;
        let mut buf = Vec::new();
        zf.read_to_end(&mut buf)?;
        f(&parse_class_safe(&buf, bytecode)?)
    })
}

#[inline]
fn zip_each_jclass<F>(zip: &mut zip::ZipArchive<impl Read + Seek>, mut f: F) -> anyhow::Result<()>
where F: FnMut(jclass::JClassReader<&mut zip::read::ZipFile, jclass::AtInterfaces>) -> anyhow::Result<()> {
    ext::zip_file_ext_iter(zip, ext::Extension::Class).try_for_each(|zf| {
        let mut zf = zf?;
        let jcr = match jclass::JClassReader::new(&mut zf) {
            Ok(x) => x,
            Err(e) => {
                return Err(anyhow::anyhow!("Parsing failed (in {}): {}", zf.name(), e));
            }
        };
        f(jcr)
    })
}

pub fn gather_inheritance_v2<RS: Read + Seek>(rs: RS) -> anyhow::Result<ext::Inheritance> {
    let mut zip = ZipArchive::new(rs)?;
    let mut inh = ext::Inheritance::default();
    zip_each_jclass(&mut zip, |jcr| {
        let ajcn = jcr.class_name()?;
        let cname = std::str::from_utf8(ajcn)?;
        let ci = inh.find(cname);
        if let Some(ajcn) = jcr.super_class()? {
            let s = std::str::from_utf8(ajcn)?;
            if s != "java/lang/Object" {
                inh.add_inherit(ci, s);
            }
        }
        jcr.interfaces(|av| {
            for ajcn in av {
                let ajcn = ajcn?;
                let s = std::str::from_utf8(&ajcn)?;
                inh.add_inherit(ci, s);
            }
            Ok(())
        })?;
        Ok(())
    })?;
    Ok(inh)
}

#[derive(Serialize)]
pub struct Complexity(HashMap<Box<str>, ClassCounting>);

#[derive(Clone, Serialize)]
pub struct ClassCounting {
    total: usize,
    fields: usize,
    methods: usize,
    code: Vec<(Box<str>, usize)>
}
impl Complexity {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    fn fill_from<'a>(&mut self, cf: &'a cafebabe::ClassFile<'a>) {
        let s = &cf.this_class;
        if !self.0.contains_key(s.as_ref()) {
            let mut total = cf.fields.len() + cf.methods.len();
            let mut v = vec![];
            for m in &cf.methods {
                let Some(mcode) = m.attributes.iter().find_map(|a| {
                    if let cafebabe::attributes::AttributeData::Code(code) = &a.data {
                        Some(code)
                    } else {
                        None
                    }
                }) else { continue; };
                let Some(bc) = &mcode.bytecode else { continue; };
                let mn = m.name.replacen("lambda$", "λ ", 1);
                let x = &m.descriptor;
                total += bc.opcodes.len();
                v.push(((format!("{mn} {x}")).into_boxed_str(), bc.opcodes.len()));
            }
            if (s.ends_with("package-info") || s.ends_with("module-info")) && v.is_empty() { return; }
            self.0.insert(s.to_string().into_boxed_str(), ClassCounting { total, fields: cf.fields.len(), methods: cf.methods.len(), code: v });
        }
    }
    pub fn extend(&mut self, other: &Self) {
        self.0.extend(other.0.iter().map(|(k, v)| (k.to_string().into_boxed_str(), v.clone())));
    }
}
impl <R> FromIterator<R> for Complexity where R: AsRef<Self> {
    fn from_iter<T: IntoIterator<Item = R>>(iter: T) -> Self {
        iter.into_iter().fold(Self::new(), |mut acc, x| { acc.extend(x.as_ref()); acc })
    }
}

pub fn gather_complexity(p: impl AsRef<Path>) -> anyhow::Result<Complexity> {
    let mut zip = ext::zip_open(p)?;
    let mut cmplx = Complexity(HashMap::new());
    zip_each_class(&mut zip, true, |cf| {
        cmplx.fill_from(cf);
        Ok(())
    })?;
    Ok(cmplx)
}

fn find_annotation<'a>(cf: &'a cafebabe::ClassFile<'a>, name: &'a str) -> Option<&'a cafebabe::attributes::Annotation<'a>> {
    cf.attributes.iter().find_map(|a| {
        match &a.data {
            AttributeData::RuntimeVisibleAnnotations(an) |
            AttributeData::RuntimeInvisibleAnnotations(an) => an,
            _ => { return None }
        }.iter().find(|a| a.type_descriptor == name)
    })
}

pub struct StrIndex {
    pub classes: Vec<Box<str>>,
    pub strings: HashMap<Box<str>, Vec<usize>>
}

type Slice<T> = Box<[T]>;

#[derive(Serialize, Clone)]
pub struct StrIndexMapped {
    pub classes: Slice<Box<str>>,
    pub strings: Slice<(Box<str>, Slice<usize>)>
}
impl From<StrIndex> for StrIndexMapped {
    fn from(x: StrIndex) -> Self {
        let classes = x.classes.into_boxed_slice();
        let mut strings = x.strings.into_iter().map(|(k, v)| (k, v.into_boxed_slice())).collect::<Box<_>>();
        strings.sort();
        Self { classes, strings }
    }
}

pub fn gather_str_index_v2<RS: Read + Seek>(rs: RS) -> anyhow::Result<StrIndexMapped> {
    let mut zip = ZipArchive::new(rs)?;
    let mut sidx = StrIndex { classes: vec![], strings: HashMap::new() };
    zip_each_jclass(&mut zip, |jcr| {
        let name = jcr.class_name()?.to_string().into_boxed_str();
        let sz = sidx.classes.len();
        sidx.classes.push(name);
        for x in jcr.iter_pool().by_type::<jclass::idx::Utf8>() {
            sidx.strings.entry(x.to_string().into_boxed_str()).or_default().push(sz);
        }
        Ok(())
    })?;
    Ok(sidx.into())
}

#[derive(Serialize)]
pub struct ModEntries {
    pub classes: Box<[Box<str>]>
}

pub fn scan_forge_mod_entries(zipfile: &mut zip::ZipArchive<impl Read + Seek>, names: &[&str]) -> anyhow::Result<Box<[Box<str>]>> {
    let mut found = vec![None; names.len()];
    zip_each_class(zipfile, false, |cf| {
        let Some(a) = find_annotation(cf, "Lnet/minecraftforge/fml/common/Mod;") else { return Ok(()) };
        for e in &a.elements {
            if let AnnotationElement{name: x, value: AnnotationElementValue::StringConstant(s)} = e {
                if x != "value" { continue; }
                if let Some(i) = names.iter().position(|n| *n == *s) {
                    found[i] = Some(cf.this_class.to_string().into_boxed_str());
                    break;
                }
            }
        }
        Ok(())
    })?;
    Ok(found.into_iter().flatten().collect())
}

pub fn scan_fabric_mod_entry(zipfile: &mut zip::ZipArchive<impl Read + Seek>, classpath: &str) -> anyhow::Result<Box<str>> {
    let classfile = classpath.replace('.', "/") + ".class";
    let mut zf = zipfile.by_name(&classfile)?;
    let mut buf = Vec::new();
    zf.read_to_end(&mut buf)?;
    let jcr = jclass::JClassReader::new(buf.as_slice())?;
    let cn = jcr.class_name()?;
    Ok(cn.to_string().into_boxed_str())
}