use std::{io::Read, marker::PhantomData};
use bytes::{Buf, Bytes};
use byteorder::{ReadBytesExt, BE};
use super::{idx::{ClassInfo, Index}, jtype::MemberType, len_member_info, pool::ClassPool, read_member_info, AttrInfo, JStr, MemberInfo};

pub struct Interfaces {
    inner: std::vec::IntoIter<u16>,
    pool: ClassPool
}
impl Interfaces {
    pub fn from_read(r: &mut impl Read, pool: &ClassPool) -> anyhow::Result<Self> {
        let mut v = vec![0; r.read_u16::<BE>()? as usize];
        r.read_u16_into::<BE>(&mut v)?;
        Ok(Self { inner: v.into_iter(), pool: pool.clone()})
    }
}
impl Iterator for Interfaces {
    type Item = anyhow::Result<JStr>;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|x| self.pool.get(self.pool.get_::<ClassInfo>(x)?).cloned())
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.inner.count()
    }
}

#[derive(Clone)]
pub struct MemberIter<T: MemberType> {
    b: Bytes,
    pool: ClassPool,
    cur: u16,
    len: u16,
    _t: PhantomData<fn() -> T>
}
impl <T: MemberType> MemberIter<T> {
    pub fn from_read(r: &mut impl Read, pool: &ClassPool) -> anyhow::Result<Self> {
        let mut bmut = bytes::BytesMut::new();
        read_member_info(r, &mut bmut)?;
        let b = bmut.freeze();
        let len = b.clone().get_u16();
        Ok(Self { b, pool: pool.clone(), cur: 0, len, _t: PhantomData })
    }
    pub const fn new(b: Bytes, pool: ClassPool, len: u16) -> Self {
        Self { b, pool, cur: 0, len, _t: PhantomData }
    }
}
impl <T: MemberType> Iterator for MemberIter<T> {
    type Item = anyhow::Result<MemberInfo<T>>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.cur == self.len {
            return None
        }
        let mut b = self.b.clone();
        let mut l = 0;
        len_member_info(&mut b, &mut l);
        let mut b = self.b.split_to(l);
        let flags = b.get_u16();
        let iname = b.get_u16();
        let idesc = b.get_u16();
        let attr_count = b.get_u16();
        let name_idx = match Index::try_from(iname) {
            Ok(idx) => idx,
            Err(err) => return Some(Err(err))
        };
        let descriptor_idx = match Index::try_from(idesc) {
            Ok(idx) => idx,
            Err(err) => return Some(Err(err))
        };
        self.cur += 1;
        Some(Ok(MemberInfo {
            b,
            pool: self.pool.clone(),
            flags,
            name_idx,
            descriptor_idx,
            attr_count,
            _t: PhantomData
        }))
    }
}

#[derive(Clone)]
pub struct Attrs<T> {
    b: Bytes,
    pool: ClassPool,
    cur: u16,
    len: u16,
    _t: PhantomData<fn() -> T>
}
impl <T> Attrs<T> {
    pub const fn new(b: Bytes, pool: ClassPool, len: u16) -> Self {
        Self { b, pool, cur: 0, len, _t: PhantomData }
    }
}
impl<T> Iterator for Attrs<T> {
    type Item = anyhow::Result<AttrInfo<T>>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.cur == self.len {
            return None
        }
        let name_idx = match Index::try_from(self.b.get_u16()) {
            Ok(idx) => idx,
            Err(err) => return Some(Err(err))
        };
        let len = self.b.get_u32() as usize;
        let b = self.b.split_to(len);
        self.cur += 1;
        Some(Ok(AttrInfo { b, pool: self.pool.clone(), name_idx, _t: PhantomData }))
    }
}