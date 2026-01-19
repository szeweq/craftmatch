use std::{
    collections::HashMap,
    io::{Read, Seek},
    sync::Mutex,
    time,
};

use cafebabe::attributes::{AnnotationElement, AnnotationElementValue, AttributeData};
use cm_zipext::FileMap;
use serde::Serialize;

use crate::ext::{self, Extension};
use cm_jclass::{self, pool::PoolIter, JClassReader};

pub static PARSE_TIMES: std::sync::LazyLock<Mutex<HashMap<Box<str>, time::Duration>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

fn parse_class_safe(
    b: &[u8],
    bytecode: bool,
) -> Result<cafebabe::ClassFile<'_>, cafebabe::ParseError> {
    let now = time::Instant::now();
    match cafebabe::parse_class_with_options(
        b,
        cafebabe::ParseOptions::default().parse_bytecode(bytecode),
    ) {
        Ok(x) => {
            PARSE_TIMES
                .lock()
                .unwrap()
                .insert(x.this_class.to_string().into_boxed_str(), now.elapsed());
            Ok(x)
        }
        e => e,
    }
}

pub struct CFOwned(
    Box<cafebabe::ClassFile<'static>>,
    #[allow(unused)] Box<[u8]>,
);
impl CFOwned {
    #[inline]
    pub fn new(x: cafebabe::ClassFile<'static>, buf: Box<[u8]>) -> Self {
        Self(Box::new(x), buf)
    }

    pub fn from_vec(v: Vec<u8>, bytecode: bool) -> anyhow::Result<Self> {
        let bb = unsafe { std::mem::transmute::<&[u8], &[u8]>(v.as_slice()) };
        Ok(Self::new(
            parse_class_safe(bb, bytecode)?,
            v.into_boxed_slice(),
        ))
    }
}
impl std::ops::Deref for CFOwned {
    type Target = cafebabe::ClassFile<'static>;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn gather_inheritance_v2<RS: Read + Seek>(
    fm: &FileMap,
    rs: &mut RS,
) -> anyhow::Result<ext::Inheritance> {
    use std::str::from_utf8;
    let mut inh = ext::Inheritance::default();
    for (_, fe) in fm
        .iter()
        .filter(|(k, _)| Extension::Class.matches(k.as_ref()))
    {
        let cr = fe.reader(rs)?;
        let jcr = JClassReader::new(cr)?;
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
pub struct Complexity(pub HashMap<Box<str>, ClassCounting>);

#[derive(Debug, Clone, Serialize)]
pub struct ClassCounting {
    total: usize,
    fields: usize,
    methods: usize,
    pub code: Vec<(Box<str>, usize)>,
}
impl Complexity {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    fn fill_from(&mut self, cf: &cafebabe::ClassFile) {
        let s = &cf.this_class;
        if !self.0.contains_key(s.to_string().as_str()) {
            let mut total = cf.fields.len() + cf.methods.len();
            let mut v = vec![];
            for m in &cf.methods {
                let Some(mcode) = m.attributes.iter().find_map(|a| {
                    if let cafebabe::attributes::AttributeData::Code(code) = &a.data {
                        Some(code)
                    } else {
                        None
                    }
                }) else {
                    continue;
                };
                let Some(bc) = &mcode.bytecode else {
                    continue;
                };
                let mn = m.name.replacen("lambda$", "lambda ", 1);
                let x = &m.descriptor;
                total += bc.opcodes.len();
                v.push(((format!("{mn} {x:?}")).into_boxed_str(), bc.opcodes.len()));
            }
            if (s.to_string().ends_with("package-info") || s.to_string().ends_with("module-info"))
                && v.is_empty()
            {
                return;
            }
            self.0.insert(
                s.to_string().into_boxed_str(),
                ClassCounting {
                    total,
                    fields: cf.fields.len(),
                    methods: cf.methods.len(),
                    code: v,
                },
            );
        }
    }
    pub fn extend(&mut self, other: &Self) {
        self.0.extend(
            other
                .0
                .iter()
                .map(|(k, v)| (k.to_string().into_boxed_str(), v.clone())),
        );
    }
}
impl<R> FromIterator<R> for Complexity
where
    R: AsRef<Self>,
{
    fn from_iter<T: IntoIterator<Item = R>>(iter: T) -> Self {
        iter.into_iter().fold(Self::new(), |mut acc, x| {
            acc.extend(x.as_ref());
            acc
        })
    }
}

pub fn gather_complexity<RS: Read + Seek>(fm: &FileMap, rs: &mut RS) -> anyhow::Result<Complexity> {
    let mut cmplx = Complexity(HashMap::new());
    for (_, fe) in fm
        .iter()
        .filter(|(k, _)| Extension::Class.matches(k.as_ref()))
    {
        let cf = CFOwned::from_vec(fe.vec_from(rs)?, true)?;
        cmplx.fill_from(&cf);
    }
    Ok(cmplx)
}

fn find_annotation<'a>(
    cf: &'a CFOwned,
    name: &str,
) -> Option<&'a cafebabe::attributes::Annotation<'a>> {
    // We compare strings to avoid constructing FieldDescriptor (whcih has private fields)
    cf.attributes.iter().find_map(|a| {
        match &a.data {
            AttributeData::RuntimeVisibleAnnotations(van)
            | AttributeData::RuntimeInvisibleAnnotations(van) => van,
            _ => return None,
        }
        .iter()
        .find(|an| an.type_descriptor.to_string() == name)
    })
}

pub struct StrIndex {
    pub classes: Vec<Box<str>>,
    pub strings: HashMap<Box<str>, Vec<usize>>,
}

type Slice<T> = Box<[T]>;

#[derive(Serialize, Clone)]
pub struct StrIndexMapped {
    pub classes: Slice<Box<str>>,
    pub strings: Slice<(Box<str>, Slice<usize>)>,
}
impl From<StrIndex> for StrIndexMapped {
    fn from(x: StrIndex) -> Self {
        let classes = x.classes.into_boxed_slice();
        let mut strings = x
            .strings
            .into_iter()
            .map(|(k, v)| (k, v.into_boxed_slice()))
            .collect::<Box<_>>();
        strings.sort();
        Self { classes, strings }
    }
}

pub fn gather_str_index_v2<RS: Read + Seek>(
    fm: &FileMap,
    rs: &mut RS,
) -> anyhow::Result<StrIndexMapped> {
    let mut sidx = StrIndex {
        classes: vec![],
        strings: HashMap::new(),
    };
    for (_, fe) in fm
        .iter()
        .filter(|(k, _)| Extension::Class.matches(k.as_ref()))
    {
        let jcr = JClassReader::new(fe.reader(rs)?)?;
        let name = jcr.class_name()?.to_string().into_boxed_str();
        let sz = sidx.classes.len();
        sidx.classes.push(name);
        for x in jcr.iter_pool().by_type::<cm_jclass::idx::Utf8>() {
            sidx.strings
                .entry(x.to_string().into_boxed_str())
                .or_default()
                .push(sz);
        }
    }
    Ok(sidx.into())
}

#[derive(Serialize)]
pub struct ModEntries {
    pub classes: Box<[Box<str>]>,
}

pub fn scan_forge_mod_entries<RS: Read + Seek>(
    names: &[&str],
    fm: &FileMap,
    rs: &mut RS,
) -> anyhow::Result<Box<[Box<str>]>> {
    let mut found = vec![None; names.len()];
    for (_, fe) in fm
        .iter()
        .filter(|(k, _)| Extension::Class.matches(k.as_ref()))
    {
        let cf = CFOwned::from_vec(fe.vec_from(rs)?, false)?;
        let Some(a) = find_annotation(&cf, "Lnet/minecraftforge/fml/common/Mod;") else {
            continue;
        };
        for e in &a.elements {
            if let AnnotationElement {
                name: x,
                value: AnnotationElementValue::StringConstant(s),
            } = e
            {
                if x != "value" {
                    continue;
                }
                if let Some(i) = names.iter().position(|n| *n == *s) {
                    found[i] = Some(cf.this_class.to_string().into_boxed_str());
                    break;
                }
            }
        }
    }
    Ok(found.into_iter().flatten().collect())
}

pub fn scan_fabric_mod_entry<RS: Read + Seek>(
    classpath: &str,
    fm: &FileMap,
    rs: &mut RS,
) -> anyhow::Result<Box<str>> {
    let mut classfile = classpath.replace('.', "/");
    classfile.push_str(".class");
    let classfile = classfile.into_boxed_str();
    let fe = fm
        .get(&classfile)
        .ok_or_else(|| anyhow::anyhow!("Classfile not found: {}", classfile))?;
    let buf = fe.vec_from(rs)?;
    let jcr = JClassReader::new(buf.as_slice())?;
    jcr.class_name().map(|x| x.to_string().into_boxed_str())
}
