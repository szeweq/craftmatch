use std::{borrow::Cow, collections::HashMap, io::{Read, Seek}, sync::Mutex, time};

use cafebabe::{attributes::{AnnotationElement, AnnotationElementValue, AttributeData}, descriptor::{FieldType, Ty}};
use once_cell::sync::Lazy;
use serde::Serialize;
use zip::ZipArchive;

use crate::ext;
use jclass::{self, pool::PoolIter};

pub static PARSE_TIMES: Lazy<Mutex<HashMap<Box<str>, time::Duration>>> = Lazy::new(|| Mutex::new(HashMap::new()));

fn parse_class_safe(b: &[u8], bytecode: bool) -> Result<cafebabe::ClassFile<'_>, cafebabe::ParseError> {
    let now = time::Instant::now();
    match cafebabe::parse_class_with_options(b, cafebabe::ParseOptions::default().parse_bytecode(bytecode)) {
        Ok(x) => {
            PARSE_TIMES.lock().unwrap().insert(x.this_class.clone().into(), now.elapsed());
            Ok(x)
        }
        e => e
    }
}

pub struct CFOwned(Box<cafebabe::ClassFile<'static>>, #[allow(unused)] Box<[u8]>);
impl CFOwned {
    #[inline]
    pub fn new(x: cafebabe::ClassFile<'static>, buf: Box<[u8]>) -> Self {
        Self(Box::new(x), buf)
    }
}
impl std::ops::Deref for CFOwned {
    type Target = cafebabe::ClassFile<'static>;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[inline]
fn zip_classes<RS: Read + Seek>(zar: &mut ZipArchive<RS>, bytecode: bool) -> impl Iterator<Item = anyhow::Result<CFOwned>> + '_ {
    ext::zip_file_ext_iter(zar, ext::Extension::Class).map(move |z| match z {
        Ok(mut zf) => {
            let mut buf = Vec::new();
            zf.read_to_end(&mut buf)?;
            let bb = {unsafe {std::mem::transmute::<&[u8], &[u8]>(buf.as_slice())}};
            Ok(CFOwned::new(parse_class_safe(bb, bytecode)?, buf.into_boxed_slice()))
        }
        Err(e) => Err(e.into())
    })
}

#[inline]
fn zip_jclass_readers<RS: Read + Seek>(zar: &mut ZipArchive<RS>) -> impl Iterator<Item = anyhow::Result<jclass::JClassReader<zip::read::ZipFile<'_>, jclass::read::AtInterfaces>>> + '_ {
    ext::zip_file_ext_iter(zar, ext::Extension::Class).map(|z| match z {
        Ok(zf) => Ok(jclass::JClassReader::new(zf)?),
        Err(e) => Err(e.into())
    })
}

pub fn gather_inheritance_v2<RS: Read + Seek>(mut zar: ZipArchive<RS>) -> anyhow::Result<ext::Inheritance> {
    use std::str::from_utf8;
    let mut inh = ext::Inheritance::default();
    for jcr in zip_jclass_readers(&mut zar) {
        let jcr = jcr?;
        let ajcn = jcr.class_name()?;
        let cname = from_utf8(ajcn)?;
        let ci = inh.find(cname);
        if let Some(ajcn) = jcr.super_class()? {
            let s = from_utf8(ajcn)?;
            if s != "java/lang/Object" {
                inh.add_inherit(ci, s);
            }
        }
        let (_, av) = jcr.interfaces()?;
        for ajcn in av {
            let ajcn = ajcn?;
            let s = from_utf8(&ajcn)?;
            inh.add_inherit(ci, s);
        }
    }
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
    fn fill_from(&mut self, cf: &cafebabe::ClassFile) {
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

pub fn gather_complexity<RS: Read + Seek>(mut zar: ZipArchive<RS>) -> anyhow::Result<Complexity> {
    let mut cmplx = Complexity(HashMap::new());
    for cf in zip_classes(&mut zar, true) {
        let cf = cf?;
        cmplx.fill_from(&cf);
    }
    Ok(cmplx)
}

fn find_annotation<'a>(cf: &'a CFOwned, name: &str) -> Option<&'a cafebabe::attributes::Annotation<'a>> {
    let typedesc = FieldType::Ty(Ty::Object(Cow::Borrowed(name)));
    cf.attributes.iter().find_map(|a| {
        match &a.data {
            AttributeData::RuntimeVisibleAnnotations(van) |
            AttributeData::RuntimeInvisibleAnnotations(van) => van,
            _ => { return None }
        }.iter().find(|an| an.type_descriptor == typedesc)
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

pub fn gather_str_index_v2<RS: Read + Seek>(mut zar: ZipArchive<RS>) -> anyhow::Result<StrIndexMapped> {
    let mut sidx = StrIndex { classes: vec![], strings: HashMap::new() };
    for jcr in zip_jclass_readers(&mut zar) {
        let jcr = jcr?;
        let name = jcr.class_name()?.to_string().into_boxed_str();
        let sz = sidx.classes.len();
        sidx.classes.push(name);
        for x in jcr.iter_pool().by_type::<jclass::idx::Utf8>() {
            sidx.strings.entry(x.to_string().into_boxed_str()).or_default().push(sz);
        }
    }
    Ok(sidx.into())
}

#[derive(Serialize)]
pub struct ModEntries {
    pub classes: Box<[Box<str>]>
}

pub fn scan_forge_mod_entries<RS: Read + Seek>(zar: &mut ZipArchive<RS>, names: &[&str]) -> anyhow::Result<Box<[Box<str>]>> {
    let mut found = vec![None; names.len()];
    for cf in zip_classes(zar, false) {
        let cf = cf?;
        let Some(a) = find_annotation(&cf, "Lnet/minecraftforge/fml/common/Mod;") else { continue; };
        for e in &a.elements {
            if let AnnotationElement{name: x, value: AnnotationElementValue::StringConstant(s)} = e {
                if x != "value" { continue; }
                if let Some(i) = names.iter().position(|n| *n == *s) {
                    found[i] = Some(cf.this_class.to_string().into_boxed_str());
                    break;
                }
            }
        }
    }
    Ok(found.into_iter().flatten().collect())
}

pub fn scan_fabric_mod_entry<RS: Read + Seek>(zipfile: &mut zip::ZipArchive<RS>, classpath: &str) -> anyhow::Result<Box<str>> {
    let mut classfile = classpath.replace('.', "/");
    classfile.push_str(".class");
    let mut zf = zipfile.by_name(&classfile)?;
    let mut buf = Vec::new();
    zf.read_to_end(&mut buf)?;
    let jcr = jclass::JClassReader::new(buf.as_slice())?;
    jcr.class_name().map(|x| x.to_string().into_boxed_str())
}