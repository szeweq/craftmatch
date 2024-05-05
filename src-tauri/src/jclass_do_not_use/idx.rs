use std::{marker::PhantomData, num::NonZeroU16};

use super::{pool::JVal, JStr, PoolItem};



pub(in super) enum Utf8 {}
pub(in super) enum NameAndType {}
pub(in super) enum FieldRef {}
pub(in super) enum MethodRef {}
pub(in super) enum InterfaceMethodRef {}
pub(in super) enum AnyMethodRef {}
pub(in super) enum ClassInfo {}
pub(in super) enum ConstVal {}

pub(in super) trait UseIndex {
    type Out;
    fn at(pool_item: &PoolItem) -> Option<Self::Out>;
}

pub(in super) struct Index<R>(pub(in super) NonZeroU16, PhantomData<R>);
impl<R> Index<R> {
    pub(in super) fn maybe(value: u16) -> Option<Self> {
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
impl UseIndex for Utf8 {
    type Out = JStr;
    fn at(item: &PoolItem) -> Option<Self::Out> {
        if let PoolItem::Utf8(s) = item { Some(s.clone()) } else { None }
    }
}
impl UseIndex for NameAndType {
    type Out = (Index<Utf8>, Index<Utf8>);
    fn at(item: &PoolItem) -> Option<Self::Out> {
        if let PoolItem::NameAndType(a, b) = item { Some((*a, *b)) } else { None }
    }
}
impl UseIndex for FieldRef {
    type Out = (Index<ClassInfo>, Index<NameAndType>);
    fn at(item: &PoolItem) -> Option<Self::Out> {
        if let PoolItem::RefField(a, b) = item { Some((*a, *b)) } else { None }
    }
}
impl UseIndex for MethodRef {
    type Out = (Index<ClassInfo>, Index<NameAndType>);
    fn at(item: &PoolItem) -> Option<Self::Out> {
        if let PoolItem::RefMethod(a, b) = item { Some((*a, *b)) } else { None }
    }
}
impl UseIndex for InterfaceMethodRef {
    type Out = (Index<ClassInfo>, Index<NameAndType>);
    fn at(item: &PoolItem) -> Option<Self::Out> {
        if let PoolItem::RefInterfaceMethod(a, b) = item { Some((*a, *b)) } else { None }
    }
}
impl UseIndex for AnyMethodRef {
    type Out = (bool, Index<ClassInfo>, Index<NameAndType>);
    fn at(item: &PoolItem) -> Option<Self::Out> {
        match item {
            PoolItem::RefMethod(a, b) => Some((false, *a, *b)),
            PoolItem::RefInterfaceMethod(a, b) => Some((true, *a, *b)),
            _ => None
        }
    }
}
impl UseIndex for ClassInfo {
    type Out = Index<Utf8>;
    fn at(item: &PoolItem) -> Option<Self::Out> {
        if let PoolItem::Class(a) = item { Some(*a) } else { None }
    }
}
impl UseIndex for ConstVal {
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

