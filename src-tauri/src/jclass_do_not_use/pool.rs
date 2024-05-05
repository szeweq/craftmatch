use std::sync::Arc;

use super::{idx::{AnyMethodRef, ClassInfo, FieldRef, Index, InterfaceMethodRef, MethodRef, NameAndType, UseIndex, Utf8}, JStr};

pub(in super) enum PoolItem {
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

#[allow(dead_code)]
pub(in super) enum RefKind {
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
pub(in super) struct ClassPool(Arc<[PoolItem]>);
impl ClassPool {
    pub(in super) fn get<R: UseIndex>(&self, idx: Index<R>) -> anyhow::Result<R::Out> {
        match self.0.get(idx.0.get() as usize) {
            Some(pi) => R::at(pi).ok_or_else(|| anyhow::anyhow!("Invalid pool item")),
            None => anyhow::bail!("Invalid pool index"),
        }
    }
    pub(in super) fn get_<R: UseIndex>(&self, idx: u16) -> anyhow::Result<R::Out> {
        self.get::<R>(idx.try_into()?)
    }
    pub(in super) fn str_to_index(&self, s: &str) -> Option<Index<Utf8>> {
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

pub enum JVal {
    Int(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    Str(Index<Utf8>),
}