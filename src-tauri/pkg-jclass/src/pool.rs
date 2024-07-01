use std::{ops::Deref, sync::Arc};

use byteorder::{BE, ReadBytesExt};

use super::{idx::{AnyMethodRef, ClassInfo, FieldRef, Index, InterfaceMethodRef, MethodRef, NameAndType, UseIndex, Utf8}, JStr};

#[derive(Debug)]
pub enum PoolItem {
    None,
    Utf8(JStr),
    Int(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    Class(Index<Utf8>),
    String(Index<Utf8>),
    RefField(Index<ClassInfo>, Index<NameAndType>),
    RefMethod(Index<ClassInfo>, Index<NameAndType>),
    RefInterfaceMethod(Index<ClassInfo>, Index<NameAndType>),
    NameAndType(Index<Utf8>, Index<Utf8>),
    MethodHandle(RefKind),
    MethodType(Index<Utf8>),
    Dynamic(u16, Index<NameAndType>),
    InvokeDynamic(u16, Index<NameAndType>),
    Module(Index<Utf8>),
    Package(Index<Utf8>),
    Reserved
}
impl PoolItem {
    pub fn read_from(tag: u8, major: u16, r: &mut impl std::io::Read) -> anyhow::Result<Self> {
        Ok(match tag {
            1 => {
                let len = r.read_u16::<BE>()? as usize;
                let mut b = vec![0; len];
                r.read_exact(&mut b)?;
                Self::Utf8(JStr::from(b.into_boxed_slice()))
            }
            3 => Self::Int(r.read_i32::<BE>()?),
            4 => Self::Float(r.read_f32::<BE>()?),
            5 => Self::Long(r.read_i64::<BE>()?),
            6 => Self::Double(r.read_f64::<BE>()?),
            7 => Self::Class(r.read_u16::<BE>()?.try_into()?),
            8 => Self::String(r.read_u16::<BE>()?.try_into()?),
            9 => {
                let r1 = r.read_u16::<BE>()?.try_into()?;
                let r2 = r.read_u16::<BE>()?.try_into()?;
                Self::RefField(r1, r2)
            }
            10 => {
                let r1 = r.read_u16::<BE>()?.try_into()?;
                let r2 = r.read_u16::<BE>()?.try_into()?;
                Self::RefMethod(r1, r2)
            }
            11 => {
                let r1 = r.read_u16::<BE>()?.try_into()?;
                let r2 = r.read_u16::<BE>()?.try_into()?;
                Self::RefInterfaceMethod(r1, r2)
            }
            12 => {
                let r1 = r.read_u16::<BE>()?.try_into()?;
                let r2 = r.read_u16::<BE>()?.try_into()?;
                Self::NameAndType(r1, r2)
            }
            15 if major >= 51 => {
                let kind = r.read_u8()?;
                let r = r.read_u16::<BE>()?;
                Self::MethodHandle((kind, r).try_into()?)
            }
            16 if major >= 51 => Self::MethodType(r.read_u16::<BE>()?.try_into()?),
            17 if major >= 55 => {
                let r1 = r.read_u16::<BE>()?;
                let r2 = r.read_u16::<BE>()?.try_into()?;
                Self::Dynamic(r1, r2)
            }
            18 if major >= 51 => {
                let r1 = r.read_u16::<BE>()?;
                let r2 = r.read_u16::<BE>()?.try_into()?;
                Self::InvokeDynamic(r1, r2)
            }
            19 if major >= 53 => Self::Module(r.read_u16::<BE>()?.try_into()?),
            20 if major >= 53 => Self::Package(r.read_u16::<BE>()?.try_into()?),
            n => anyhow::bail!("Invalid tag: {}", n),
        })
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum RefKind {
    GetField(Index<FieldRef>),
    GetStatic(Index<FieldRef>),
    PutField(Index<FieldRef>),
    PutStatic(Index<FieldRef>),
    InvokeVirtual(Index<MethodRef>),
    InvokeStatic(Index<AnyMethodRef>),
    InvokeSpecial(Index<AnyMethodRef>),
    NewInvokeSpecial(Index<MethodRef>),
    InvokeInterface(Index<InterfaceMethodRef>),
}
impl TryFrom<(u8, u16)> for RefKind {
    type Error = anyhow::Error;
    fn try_from((kind, index): (u8, u16)) -> Result<Self, Self::Error> {
        Ok(match kind {
            1 => Self::GetField(index.try_into()?),
            2 => Self::GetStatic(index.try_into()?),
            3 => Self::PutField(index.try_into()?),
            4 => Self::PutStatic(index.try_into()?),
            5 => Self::InvokeVirtual(index.try_into()?),
            6 => Self::InvokeStatic(index.try_into()?),
            7 => Self::InvokeSpecial(index.try_into()?),
            8 => Self::NewInvokeSpecial(index.try_into()?),
            9 => Self::InvokeInterface(index.try_into()?),
            _ => anyhow::bail!("Invalid kind"),
        })
    }
}

#[derive(Clone)]
pub struct ClassPool(Arc<[PoolItem]>);
impl ClassPool {
    pub fn get<R: for <'a> UseIndex<'a>>(&self, idx: Index<R>) -> anyhow::Result<<R as UseIndex<'_>>::Out> {
        match self.0.get(idx.0.get() as usize) {
            Some(pi) => R::at(pi).ok_or_else(|| anyhow::anyhow!("Invalid pool item")),
            None => anyhow::bail!("Invalid pool index"),
        }
    }
    pub fn get_<R: for <'a> UseIndex<'a>>(&self, idx: u16) -> anyhow::Result<<R as UseIndex<'_>>::Out> {
        self.get::<R>(idx.try_into()?)
    }
    pub fn str_to_index(&self, s: &str) -> Option<Index<Utf8>> {
        let b = s.as_bytes();
        self.0.iter().position(|i| match i {
            PoolItem::Utf8(a) => &**a == b,
            _ => false
        }).and_then(|i| Index::maybe(i as u16))
    }
}
impl<T> From<T> for ClassPool where T: Into<Arc<[PoolItem]>> {
    fn from(t: T) -> Self {
        Self(t.into())
    }
}
impl Deref for ClassPool {
    type Target = [PoolItem];
    fn deref(&self) -> &Self::Target { &self.0 }
}

#[derive(Clone)]
pub enum JVal {
    Int(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    Str(Index<Utf8>),
}

pub trait PoolIter<'a> {
    fn by_type<U: UseIndex<'a>>(self) -> impl Iterator<Item = <U as UseIndex<'a>>::Out>;
}
impl<'a, I: Iterator<Item = &'a PoolItem>> PoolIter<'a> for I {
    #[inline]
    fn by_type<U: UseIndex<'a>>(self) -> impl Iterator<Item = <U as UseIndex<'a>>::Out> {
        self.filter_map(U::at)
    }
}