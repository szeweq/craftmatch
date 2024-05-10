use std::{marker::PhantomData, ops::Deref};

use bytes::{Buf, Bytes};

use super::{idx::{ClassInfo, ConstVal, Index, NameAndType, Utf8}, jtype::{OfClass, OfField, OfMethod}, pool::{ClassPool, JVal, PoolItem}, JStr};

pub enum JAttr<T> {
    AnnotationDefault,
    BootstrapMethods,
    ConstantValue,
    Code(AttrKey<KeyCode>),
    Deprecated,
    EnclosingMethod(AttrKey<KeyEnclosingMethod>),
    Exceptions,
    InnerClasses(AttrKey<KeyInnerClasses>),
    LineNumberTable,
    LocalVariableTable,
    LocalVariableTypeTable,
    MethodParameters,
    Module,
    ModuleMainClass,
    ModulePackages,
    NestHost,
    NestMembers,
    PermittedSubclasses,
    Record,
    RuntimeInvisibleAnnotations(AttrKey<KeyAnnotations>),
    RuntimeInvisibleParameterAnnotations(AttrKey<KeyParamAnnotations>),
    RuntimeInvisibleTypeAnnotations(AttrKey<KeyTypeAnnotations>),
    RuntimeVisibleAnnotations(AttrKey<KeyAnnotations>),
    RuntimeVisibleParameterAnnotations(AttrKey<KeyParamAnnotations>),
    RuntimeVisibleTypeAnnotations(AttrKey<KeyTypeAnnotations>),
    Signature(AttrKey<KeySignature>),
    SourceDebugExtension(AttrKey<KeySourceDebugExtension>),
    SourceFile(AttrKey<KeySourceFile>),
    StackMapTable,
    Synthetic,
    Unsupported(PhantomData<T>)
}
impl<T> JAttr<T> {
    const fn from_name(value: &[u8]) -> Self {
        match value {
            b"AnnotationDefault" => Self::AnnotationDefault,
            b"BootstrapMethods" => Self::BootstrapMethods,
            b"ConstantValue" => Self::ConstantValue,
            b"Code" => Self::Code(ak()),
            b"Deprecated" => Self::Deprecated,
            b"EnclosingMethod" => Self::EnclosingMethod(ak()),
            b"Exceptions" => Self::Exceptions,
            b"InnerClasses" => Self::InnerClasses(ak()),
            b"LineNumberTable" => Self::LineNumberTable,
            b"LocalVariableTable" => Self::LocalVariableTable,
            b"LocalVariableTypeTable" => Self::LocalVariableTypeTable,
            b"MethodParameters" => Self::MethodParameters,
            b"Module" => Self::Module,
            b"ModuleMainClass" => Self::ModuleMainClass,
            b"ModulePackages" => Self::ModulePackages,
            b"NestHost" => Self::NestHost,
            b"NestMembers" => Self::NestMembers,
            b"PermittedSubclasses" => Self::PermittedSubclasses,
            b"Record" => Self::Record,
            b"RuntimeInvisibleAnnotations" => Self::RuntimeInvisibleAnnotations(ak()),
            b"RuntimeInvisibleParameterAnnotations" => Self::RuntimeInvisibleParameterAnnotations(ak()),
            b"RuntimeInvisibleTypeAnnotations" => Self::RuntimeInvisibleTypeAnnotations(ak()),
            b"RuntimeVisibleAnnotations" => Self::RuntimeVisibleAnnotations(ak()),
            b"RuntimeVisibleParameterAnnotations" => Self::RuntimeVisibleParameterAnnotations(ak()),
            b"RuntimeVisibleTypeAnnotations" => Self::RuntimeVisibleTypeAnnotations(ak()),
            b"Signature" => Self::Signature(ak()),
            b"SourceDebugExtension" => Self::SourceDebugExtension(ak()),
            b"SourceFile" => Self::SourceFile(ak()),
            b"StackMapTable" => Self::StackMapTable,
            b"Synthetic" => Self::Synthetic,
            _ => Self::Unsupported(PhantomData),
        }
    }
}

#[inline]
const fn ak<T: UseAttr>() -> AttrKey<T> {
    AttrKey(PhantomData)
}

trait AttrMatch {
    fn matches(a: &JAttr<Self>) -> bool where Self: std::marker::Sized;
}
impl AttrMatch for OfField {
    fn matches(a: &JAttr<Self>) -> bool {
        matches!(a,
            JAttr::ConstantValue |
            JAttr::Deprecated |
            JAttr::RuntimeInvisibleAnnotations(_) |
            JAttr::RuntimeInvisibleTypeAnnotations(_) |
            JAttr::RuntimeVisibleAnnotations(_) |
            JAttr::RuntimeVisibleTypeAnnotations(_) |
            JAttr::Signature(_) |
            JAttr::Synthetic
        )
    }
}
impl AttrMatch for OfMethod {
    fn matches(a: &JAttr<Self>) -> bool {
        matches!(a,
            JAttr::AnnotationDefault |
            JAttr::Code(_) |
            JAttr::Deprecated |
            JAttr::Exceptions |
            JAttr::MethodParameters |
            JAttr::RuntimeInvisibleAnnotations(_) |
            JAttr::RuntimeInvisibleParameterAnnotations(_) |
            JAttr::RuntimeInvisibleTypeAnnotations(_) |
            JAttr::RuntimeVisibleAnnotations(_) |
            JAttr::RuntimeVisibleParameterAnnotations(_) |
            JAttr::RuntimeVisibleTypeAnnotations(_) |
            JAttr::Signature(_) |
            JAttr::Synthetic
        )
    }
}
impl AttrMatch for OfClass {
    fn matches(a: &JAttr<Self>) -> bool {
        matches!(a,
            JAttr::BootstrapMethods |
            JAttr::Deprecated |
            JAttr::EnclosingMethod(_) |
            JAttr::InnerClasses(_) |
            JAttr::Module |
            JAttr::ModuleMainClass |
            JAttr::ModulePackages |
            JAttr::PermittedSubclasses |
            JAttr::Record |
            JAttr::RuntimeInvisibleAnnotations(_) |
            JAttr::RuntimeInvisibleTypeAnnotations(_) |
            JAttr::RuntimeVisibleAnnotations(_) |
            JAttr::RuntimeVisibleTypeAnnotations(_) |
            JAttr::Signature(_) |
            JAttr::SourceDebugExtension(_) |
            JAttr::SourceFile(_) |
            JAttr::Synthetic
        )
    }
}

impl<T: AttrMatch> TryFrom<&[u8]> for JAttr<T> {
    type Error = anyhow::Error;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let a = Self::from_name(value);
        if T::matches(&a) {
            Ok(a)
        } else {
            anyhow::bail!("Invalid attribute name")
        }
    }
}

pub trait UseAttr {
    type Out;
    fn parse(b: Bytes, pool: &ClassPool) -> anyhow::Result<Self::Out>;
}
pub struct AttrKey<T: UseAttr>(PhantomData<T>);

macro_rules! key_enums {
    ($($name:ident),*) => {
        $(pub enum $name {})*
    };
}

key_enums! {
    KeyAnnotationDefault,
    KeyBootstrapMethods,
    KeyConstantValue,
    KeyCode,
    KeyEnclosingMethod,
    KeyExceptions,
    KeyInnerClasses,
    KeyLineNumberTable,
    KeyLocalVariableTable,
    KeyLocalVariableTypeTable,
    KeyMethodParameters,
    KeyModule,
    KeyModuleMainClass,
    KeyModulePackages,
    KeyNestHost,
    KeyNestMembers,
    KeyPermittedSubclasses,
    KeyRecord,
    KeyAnnotations,
    KeyParamAnnotations,
    KeyTypeAnnotations,
    KeySignature,
    KeySourceDebugExtension,
    KeySourceFile,
    KeyStackMapTable
}

// impl UseAttr for KeyAnnotationDefault {}
// impl UseAttr for KeyBootstrapMethods {}
impl UseAttr for KeyConstantValue {
    type Out = JVal;
    fn parse(mut b: Bytes, pool: &ClassPool) -> anyhow::Result<Self::Out> {
        pool.get_::<ConstVal>(b.get_u16())
    }
}
impl UseAttr for KeyCode {
    type Out = ();
    fn parse(_b: Bytes, _pool: &ClassPool) -> anyhow::Result<Self::Out> {
        Ok(())
    }
}
impl UseAttr for KeyEnclosingMethod {
    type Out = (JStr, Option<(JStr, JStr)>);
    fn parse(mut b: Bytes, pool: &ClassPool) -> anyhow::Result<Self::Out> {
        let class_name = pool.get(pool.get_::<ClassInfo>(b.get_u16())?)?;
        let method = if let Some(idx) = Index::<NameAndType>::maybe(b.get_u16()) {
            let (mn, mt) = pool.get(idx)?;
            Some((pool.get(mn)?.clone(), pool.get(mt)?.clone()))
        } else {
            None
        };
        Ok((class_name.clone(), method))
    }
}
// impl UseAttr for KeyExceptions {}
impl UseAttr for KeyInnerClasses {
    type Out = Iter<InnerClass>;
    fn parse(mut b: Bytes, pool: &ClassPool) -> anyhow::Result<Self::Out> {
        let len = b.get_u16();
        Ok(Iter { b, pool: pool.clone(), cur: 0, len, _t: PhantomData })
    }
}
// impl UseAttr for KeyLineNumberTable {}
// impl UseAttr for KeyLocalVariableTable {}
// impl UseAttr for KeyLocalVariableTypeTable {}
// impl UseAttr for KeyMethodParameters {}
// impl UseAttr for KeyModule {}
// impl UseAttr for KeyModuleMainClass {}
// impl UseAttr for KeyModulePackages {}
// impl UseAttr for KeyNestHost {}
// impl UseAttr for KeyNestMembers {}
// impl UseAttr for KeyPermittedSubclasses {}
// impl UseAttr for KeyRecord {}
impl UseAttr for KeyAnnotations {
    type Out = ();
    fn parse(_b: Bytes, _pool: &ClassPool) -> anyhow::Result<Self::Out> {
        Ok(())
    }
}
impl UseAttr for KeyParamAnnotations {
    type Out = ();
    fn parse(_b: Bytes, _pool: &ClassPool) -> anyhow::Result<Self::Out> {
        Ok(())
    }
}
impl UseAttr for KeyTypeAnnotations {
    type Out = ();
    fn parse(_b: Bytes, _pool: &ClassPool) -> anyhow::Result<Self::Out> {
        Ok(())
    }
}
impl UseAttr for KeySignature {
    type Out = JStr;
    fn parse(mut b: Bytes, pool: &ClassPool) -> anyhow::Result<Self::Out> {
        pool.get_::<Utf8>(b.get_u16()).cloned()
    }
}
impl UseAttr for KeySourceDebugExtension {
    type Out = JStr;
    fn parse(b: Bytes, _pool: &ClassPool) -> anyhow::Result<Self::Out> {
        Ok(JStr::from(b.to_vec().into_boxed_slice()))
    }
}
impl UseAttr for KeySourceFile {
    type Out = JStr;
    fn parse(mut b: Bytes, pool: &ClassPool) -> anyhow::Result<Self::Out> {
        pool.get_::<Utf8>(b.get_u16()).cloned()
    }
}
// impl UseAttr for KeyStackMapTable {}

pub trait Parsing {
    fn parse(b: &mut Bytes, pool: &ClassPool) -> anyhow::Result<Self> where Self: Sized;
}

pub struct Data<T> {
    pool: ClassPool,
    data: T
}
impl<T> Deref for Data<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl<T: Parsing> Parsing for Data<T> {
    fn parse(b: &mut Bytes, pool: &ClassPool) -> anyhow::Result<Self> {
        let d = T::parse(b, pool)?;
        Ok(Self { pool: pool.clone(), data: d })
    }
}

pub struct Iter<T: Parsing> {
    b: Bytes,
    pool: ClassPool,
    cur: u16,
    len: u16,
    _t: PhantomData<T>
}
impl<T: Parsing> Iter<T> {
    pub fn new(mut b: Bytes, pool: ClassPool) -> Self {
        let len = b.get_u16();
        Self { b, pool, cur: 0, len, _t: PhantomData }
    }
}
impl<T: Parsing> Iterator for Iter<T> {
    type Item = anyhow::Result<T>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.cur == self.len {
            return None
        }
        let d = T::parse(&mut self.b, &self.pool);
        self.cur += 1;
        Some(d)
    }
}

pub struct InnerClass {
    inner_class: Index<ClassInfo>,
    outer_class: Option<Index<ClassInfo>>,
    name: Option<Index<Utf8>>,
    access_flags: u16
}
impl Parsing for InnerClass {
    fn parse(b: &mut Bytes, _pool: &ClassPool) -> anyhow::Result<Self> {
        let inner_class = Index::try_from(b.get_u16())?;
        let outer_class = Index::maybe(b.get_u16());
        let name = Index::maybe(b.get_u16());
        let access_flags = b.get_u16();
        Ok(Self { inner_class, outer_class, name, access_flags })
    }
}

pub struct Annotation {
    type_idx: Index<Utf8>,
    elems_len: u16,
    b: Bytes
}
impl Data<Annotation> {
    pub fn type_name(&self) -> anyhow::Result<JStr> {
        self.pool.get(self.type_idx).cloned()
    }
    pub fn elems(&self) -> Iter<Data<AnnElemPair>> {
        Iter::new(self.b.clone(), self.pool.clone())
    }
}

pub struct AnnElemPair {
    name_idx: Index<Utf8>,
    val: ElemVal
}
impl Parsing for AnnElemPair {
    fn parse(b: &mut Bytes, pool: &ClassPool) -> anyhow::Result<Self> {
        let name_idx = Index::try_from(b.get_u16())?;
        let val = ElemVal{};
        Ok(Self { name_idx, val })
    }
}

pub struct ElemVal {}