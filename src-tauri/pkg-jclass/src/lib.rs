// PURELY EXPERIMENTAL! DO NOT USE IN PRODUCTION!
#![allow(dead_code)]

use std::{io::{self, Read}, marker::PhantomData, ops::Deref, sync::Arc};
use byteorder::{ReadBytesExt, BE};
use bytes::{Buf, BufMut, Bytes};
use iter::Attrs;
use self::{idx::{ClassInfo, Index, Utf8}, jtype::MemberType, pool::{ClassPool, PoolItem}};

pub mod attr;
pub mod idx;
pub mod jtype;
pub mod pool;
pub mod iter;
pub mod read;
pub mod readseek;

pub use read::JClassReader;

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

fn read_magic<R: Read>(r: &mut R) -> anyhow::Result<()> {
    let magic = r.read_u32::<BE>()?;
    if magic != 0xCAFEBABE {
        anyhow::bail!("Invalid magic");
    }
    Ok(())
}

fn skip_member_info<R: Read>(r: &mut R) -> io::Result<()> {
    let count = r.read_u16::<BE>()?;
    for _ in 0..count {
        r.read_exact(&mut [0; 6])?;
        skip_attr_info(r)?;
    }
    Ok(())
}
fn skip_attr_info<R: Read>(r: &mut R) -> io::Result<()> {
    let count = r.read_u16::<BE>()?;
    for _ in 0..count {
        r.read_exact(&mut [0; 2])?;
        let len = r.read_u32::<BE>()?;
        skip_exact(r, len as usize)?;
    }
    Ok(())
}

fn skip_exact<R: Read>(r: &mut R, n: usize) -> io::Result<()> {
    let x = r.bytes().take(n).count();
    if x != n {
        return Err(io::Error::other("Invalid length"));
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

#[derive(Clone)]
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
        Attrs::new(self.b.clone(), self.pool.clone(), self.attr_count)
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
