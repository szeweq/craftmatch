use std::{marker::PhantomData, num::NonZeroU16};

use super::{pool::JVal, JStr, PoolItem};



pub enum Utf8 {}
pub enum NameAndType {}
pub enum FieldRef {}
pub enum MethodRef {}
pub enum InterfaceMethodRef {}
pub enum AnyMethodRef {}
pub enum ClassInfo {}
pub enum ConstVal {}

pub trait UseIndex<'a> {
    type Out;
    fn at(pool_item: &'a PoolItem) -> Option<Self::Out>;
}

pub struct Index<R>(pub(in super) NonZeroU16, PhantomData<fn() -> R>);
impl<R> Index<R> {
    pub fn maybe(value: u16) -> Option<Self> {
        NonZeroU16::new(value).map(|x| Self(x, PhantomData))
    }
}
impl<R> Clone for Index<R> {
    fn clone(&self) -> Self { *self }
}
impl<R> Copy for Index<R> {}
impl<R> TryFrom<u16> for Index<R> {
    type Error = anyhow::Error;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match NonZeroU16::new(value) {
            Some(x) => Ok(Self(x, PhantomData)),
            None => anyhow::bail!("Pool index cannot be zero"),
        }
    }
}
impl<R> std::fmt::Debug for Index<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Index({})", self.0)
    }
}


impl<'a> UseIndex<'a> for Utf8 {
    type Out = &'a JStr;
    fn at(item: &'a PoolItem) -> Option<Self::Out> {
        if let PoolItem::Utf8(s) = item { Some(s) } else { None }
    }
}
impl UseIndex<'_> for NameAndType {
    type Out = (Index<Utf8>, Index<Utf8>);
    fn at(item: &PoolItem) -> Option<Self::Out> {
        if let PoolItem::NameAndType(a, b) = item { Some((*a, *b)) } else { None }
    }
}
impl UseIndex<'_> for FieldRef {
    type Out = (Index<ClassInfo>, Index<NameAndType>);
    fn at(item: &PoolItem) -> Option<Self::Out> {
        if let PoolItem::RefField(a, b) = item { Some((*a, *b)) } else { None }
    }
}
impl UseIndex<'_> for MethodRef {
    type Out = (Index<ClassInfo>, Index<NameAndType>);
    fn at(item: &PoolItem) -> Option<Self::Out> {
        if let PoolItem::RefMethod(a, b) = item { Some((*a, *b)) } else { None }
    }
}
impl UseIndex<'_> for InterfaceMethodRef {
    type Out = (Index<ClassInfo>, Index<NameAndType>);
    fn at(item: &PoolItem) -> Option<Self::Out> {
        if let PoolItem::RefInterfaceMethod(a, b) = item { Some((*a, *b)) } else { None }
    }
}
impl UseIndex<'_> for AnyMethodRef {
    type Out = (bool, Index<ClassInfo>, Index<NameAndType>);
    fn at(item: &PoolItem) -> Option<Self::Out> {
        match item {
            PoolItem::RefMethod(a, b) => Some((false, *a, *b)),
            PoolItem::RefInterfaceMethod(a, b) => Some((true, *a, *b)),
            _ => None
        }
    }
}
impl UseIndex<'_> for ClassInfo {
    type Out = Index<Utf8>;
    fn at(item: &PoolItem) -> Option<Self::Out> {
        if let PoolItem::Class(a) = item { Some(*a) } else { None }
    }
}
impl UseIndex<'_> for ConstVal {
    type Out = JVal;
    fn at(item: &PoolItem) -> Option<Self::Out> {
        Some(match item {
            PoolItem::Int(x) => JVal::Int(*x),
            PoolItem::Float(x) => JVal::Float(*x),
            PoolItem::Long(x) => JVal::Long(*x),
            PoolItem::Double(x) => JVal::Double(*x),
            PoolItem::String(x) => JVal::Str(*x),
            _ => return None,
        })
    }
}

