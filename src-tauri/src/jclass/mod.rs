// PURELY EXPERIMENTAL! DO NOT USE IN PRODUCTION!
#![allow(dead_code)]

use std::{io::Read, marker::PhantomData, ops::Deref, sync::Arc};
use byteorder::{ReadBytesExt, BE};
use bytes::{Buf, BufMut, Bytes};
use self::{idx::{ClassInfo, Index, Utf8}, jtype::MemberType, pool::{ClassPool, PoolItem}};

pub mod attr;
pub mod idx;
pub mod jtype;
pub mod pool;
pub mod readseek;

#[derive(Clone)]
#[repr(transparent)]
pub struct JStr(Arc<[u8]>);
impl From<Box<[u8]>> for JStr {
    fn from(value: Box<[u8]>) -> Self {
        Self(Arc::from(value))
    }
}
impl Deref for JStr {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::fmt::Debug for JStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", String::from_utf8_lossy(&self.0))
    }
}
impl std::fmt::Display for JStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.0))
    }
}

fn skip_member_info<R: Read>(r: &mut R) -> anyhow::Result<()> {
    let count = r.read_u16::<BE>()?;
    for _ in 0..count {
        r.read_exact(&mut [0; 6])?;
        skip_attr_info(r)?;
    }
    Ok(())
}
fn skip_attr_info<R: Read>(r: &mut R) -> anyhow::Result<()> {
    let count = r.read_u16::<BE>()?;
    for _ in 0..count {
        r.read_exact(&mut [0; 2])?;
        let len = r.read_u32::<BE>()?;
        skip_exact(r, len as usize)?;
    }
    Ok(())
}

fn skip_exact<R: Read>(r: &mut R, n: usize) -> anyhow::Result<()> {
    let x = r.bytes().take(n).count();
    if x != n {
        anyhow::bail!("Invalid length");
    }
    Ok(())
}

fn read_member_info<R: Read>(r: &mut R, bmut: &mut bytes::BytesMut) -> anyhow::Result<()> {
    let count = r.read_u16::<BE>()?;
    bmut.put_u16(count);
    let mut n = [0; 6];
    for _ in 0..count {
        r.read_exact(&mut n)?;
        bmut.put_slice(&n);
        read_attr_info(r, bmut)?;
    }
    Ok(())
}
fn read_attr_info<R: Read>(r: &mut R, bmut: &mut bytes::BytesMut) -> anyhow::Result<()> {
    let count = r.read_u16::<BE>()?;
    bmut.put_u16(count);
    let mut n = [0; 2];
    for _ in 0..count {
        r.read_exact(&mut n)?;
        bmut.put_slice(&n);
        let len = r.read_u32::<BE>()?;
        bmut.put_u32(len);
        std::io::copy(&mut r.take(len as u64), &mut bmut.writer())?;
    }
    Ok(())
}

fn len_member_info(b: &mut Bytes, len: &mut usize) {
    let count = b.get_u16();
    *len += count as usize * 6;
    for _ in 0..count {
        b.advance(6);
        len_attr_info(b, len);
    }
}
fn len_attr_info(b: &mut Bytes, len: &mut usize) {
    let count = b.get_u16();
    *len += count as usize * 2;
    for _ in 0..count {
        b.advance(2);
        let l = b.get_u32() as usize;
        *len += l;
        b.advance(l);
    }
}

pub struct MemberIter<T: MemberType> {
    b: Bytes,
    pool: ClassPool,
    cur: u16,
    len: u16,
    _t: PhantomData<T>
}
impl<T: MemberType> Iterator for MemberIter<T> {
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

pub struct MemberInfo<T: MemberType> {
    b: Bytes,
    pool: ClassPool,
    flags: u16,
    name_idx: Index<Utf8>,
    descriptor_idx: Index<Utf8>,
    attr_count: u16,
    _t: PhantomData<T>
}
impl<T: MemberType> MemberInfo<T> {
    pub fn name(&self) -> anyhow::Result<&JStr> {
        self.pool.get(self.name_idx)
    }
    pub fn descriptor(&self) -> anyhow::Result<&JStr> {
        self.pool.get(self.descriptor_idx)
    }
    pub fn attrs(&mut self) -> Attrs<T> {
        Attrs { b: self.b.clone(), pool: self.pool.clone(), cur: 0, len: self.attr_count, _t: PhantomData }
    }
}

pub struct Attrs<T> {
    b: Bytes,
    pool: ClassPool,
    cur: u16,
    len: u16,
    _t: PhantomData<T>
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

pub struct AttrInfo<T> {
    b: Bytes,
    pool: ClassPool,
    name_idx: Index<Utf8>,
    _t: PhantomData<T>
}
impl<T> AttrInfo<T> {
    pub fn name(&self) -> anyhow::Result<&JStr> {
        self.pool.get(self.name_idx)
    }
}

pub struct ClassData {
    access_flags: u16,
    class_ref: Index<ClassInfo>,
    super_ref: Option<Index<ClassInfo>>,
}

pub trait Step {
    type Next: Step;
}

pub enum AtInterfaces {}
impl Step for AtInterfaces {
    type Next = AtFields;
}
pub enum AtFields {}
impl Step for AtFields {
    type Next = AtMethods;
}
pub enum AtMethods {}
impl Step for AtMethods {
    type Next = AtAttributes;
}
pub enum AtAttributes {}
impl Step for AtAttributes {
    type Next = ();
}
impl Step for () {
    type Next = ();
}

pub struct JClassReader<R: Read, At: Step> {
    r: R,
    pool: ClassPool,
    minor: u16,
    major: u16,
    data: ClassData,
    _t: PhantomData<At>
}

impl <R: Read, At: Step> JClassReader<R, At> {
    fn step(self) -> anyhow::Result<JClassReader<R, At::Next>> {
        Ok(JClassReader {
            r: self.r,
            pool: self.pool,
            minor: self.minor,
            major: self.major,
            data: self.data,
            _t: PhantomData
        })
    }
}

impl <R: Read, At: Step> JClassReader<R, At> {
    pub fn class_name(&self) -> anyhow::Result<&JStr> {
        self.pool.get(self.pool.get(self.data.class_ref)?)
    }
    pub fn super_class(&self) -> anyhow::Result<Option<&JStr>> {
        match self.data.super_ref {
            None => Ok(None),
            Some(super_ref) => Ok(Some(self.pool.get(self.pool.get(super_ref)?)?)),
        }
    }
    #[inline]
    pub fn iter_pool(&self) -> std::slice::Iter<PoolItem> {
        self.pool.iter()
    }
}

impl <R: Read> JClassReader<R, AtInterfaces> {
    pub fn new(mut r: R) -> anyhow::Result<Self> {
        if !matches!(r.read_u32::<BE>(), Ok(0xCAFEBABE)) {
            anyhow::bail!("Invalid magic");
        }
        let mut cv = [0u16; 3];
        let [minor, major, pool_count] = match r.read_u16_into::<BE>(&mut cv) {
            Ok(()) => cv,
            Err(e) => return Err(e.into())
        };
        let mut pool = Vec::with_capacity(pool_count as usize);
        pool.push(PoolItem::None);
        while pool.len() < pool.capacity() {
            let mut tag = 0u8;
            r.read_exact(std::slice::from_mut(&mut tag))?;
            pool.push(PoolItem::read_from(tag, major, &mut r)?);
            if tag == 5 || tag == 6 {
                pool.push(PoolItem::Reserved)
            }
        }
        let pool = ClassPool::from(pool.into_boxed_slice());
        let mut cv = [0u16; 3];
        let [access_flags, class_ref, super_ref] = match r.read_u16_into::<BE>(&mut cv) {
            Ok(()) => cv,
            Err(e) => return Err(e.into())
        };
        let class_ref = class_ref.try_into()?;
        let super_ref = Index::maybe(super_ref);
        Ok(Self { r, pool, minor, major, data: ClassData { access_flags, class_ref, super_ref }, _t: PhantomData })
    }

    #[inline]
    pub fn interfaces(mut self, f: impl FnOnce(Vec<anyhow::Result<JStr>>) -> anyhow::Result<()>) -> anyhow::Result<JClassReader<R, AtFields>> {
        let mut v = vec![0; self.r.read_u16::<BE>()? as usize];
        self.r.read_u16_into::<BE>(&mut v)?;
        let v = v.into_iter().map(|x| self.pool.get(self.pool.get_::<ClassInfo>(x)?).cloned()).collect();
        f(v)?;
        self.step()
    }
    pub fn skip_interfaces(mut self) -> anyhow::Result<JClassReader<R, AtFields>> {
        let l = self.r.read_u16::<BE>()?;
        for _ in 0..l {
            self.r.read_exact(&mut [0u8; 2])?;
        }
        self.step()
    }
}
impl <R: Read> JClassReader<R, AtFields> {
    #[inline]
    pub fn fields(mut self, f: impl FnOnce(MemberIter<jtype::OfField>) -> anyhow::Result<()>) -> anyhow::Result<JClassReader<R, AtMethods>> {
        let mut bmut = bytes::BytesMut::new();
        read_member_info(&mut self.r, &mut bmut)?;
        let b = bmut.freeze();
        let len = b.clone().get_u16();
        f(MemberIter { b, pool: self.pool.clone(), cur: 0, len, _t: PhantomData })?;
        self.step()
    }
    pub fn skip_fields(mut self) -> anyhow::Result<JClassReader<R, AtMethods>> {
        skip_member_info(&mut self.r)?;
        self.step()
    }
}
impl <R: Read> JClassReader<R, AtMethods> {
    #[inline]
    pub fn methods(mut self, f: impl FnOnce(MemberIter<jtype::OfMethod>) -> anyhow::Result<()>) -> anyhow::Result<JClassReader<R, AtAttributes>> {
        let mut bmut = bytes::BytesMut::new();
        read_member_info(&mut self.r, &mut bmut)?;
        let b = bmut.freeze();
        let len = b.clone().get_u16();
        f(MemberIter { b, pool: self.pool.clone(), cur: 0, len, _t: PhantomData })?;
        self.step()
    }
    pub fn skip_methods(mut self) -> anyhow::Result<JClassReader<R, AtAttributes>> {
        skip_member_info(&mut self.r)?;
        self.step()
    }
}
impl <R: Read> JClassReader<R, AtAttributes> {
    #[inline]
    pub fn attributes(mut self, f: impl FnOnce(Attrs<jtype::OfClass>) -> anyhow::Result<()>) -> anyhow::Result<JClassReader<R, ()>> {
        let mut bmut = bytes::BytesMut::new();
        read_attr_info(&mut self.r, &mut bmut)?;
        let b = bmut.freeze();
        let len = b.clone().get_u16();
        f(Attrs { b, pool: self.pool.clone(), cur: 0, len, _t: PhantomData })?;
        self.step()
    }
    pub fn skip_attributes(mut self) -> anyhow::Result<JClassReader<R, ()>> {
        skip_attr_info(&mut self.r)?;
        self.step()
    }
}