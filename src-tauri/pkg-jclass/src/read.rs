use std::{io::Read, marker::PhantomData};
use anyhow::Result;
use byteorder::{ReadBytesExt, BE};
use bytes::Buf;
use super::{idx::Index, iter::{Interfaces, MemberIter}, jtype, pool::{ClassPool, PoolItem}, read_attr_info, read_magic, skip_attr_info, skip_member_info, Attrs, ClassData, JStr};

pub trait Step {
    type Next: Step;
}
macro_rules! step {
    ($at:ident, $next:ty) => {
        pub enum $at {}
        impl Step for $at { type Next = $next; }
    };
}

step!(AtInterfaces, AtFields);
step!(AtFields, AtMethods);
step!(AtMethods, AtAttributes);
step!(AtAttributes, ());
impl Step for () { type Next = (); }

pub struct JClassReader<R: Read, At: Step> {
    r: R,
    pool: ClassPool,
    minor: u16,
    major: u16,
    data: ClassData,
    _t: PhantomData<fn() -> At>
}

impl <R: Read, At: Step> JClassReader<R, At> {
    #[inline]
    #[allow(clippy::missing_const_for_fn)]
    fn step(self) -> JClassReader<R, At::Next> {
        unsafe {
            let p = &self as *const Self as *const JClassReader<R, At::Next>;
            std::mem::forget(self);
            std::ptr::read(p)
        }
    }
}

impl <R: Read, At: Step> JClassReader<R, At> {
    pub fn class_name(&self) -> Result<&JStr> {
        self.pool.get(self.pool.get(self.data.class_ref)?)
    }
    pub fn super_class(&self) -> Result<Option<&JStr>> {
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
    pub fn new(mut r: R) -> Result<Self> {
        read_magic(&mut r)?;
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
    pub fn interfaces(mut self) -> Result<(JClassReader<R, AtFields>, Interfaces)> {
        let ii = Interfaces::from_read(&mut self.r, &self.pool)?;
        Ok((self.step(), ii))
    }
    pub fn skip_interfaces(mut self) -> Result<JClassReader<R, AtFields>> {
        let l = self.r.read_u16::<BE>()?;
        for _ in 0..l {
            self.r.read_exact(&mut [0u8; 2])?;
        }
        Ok(self.step())
    }
}
impl <R: Read> JClassReader<R, AtFields> {
    #[inline]
    pub fn fields(mut self) -> Result<(JClassReader<R, AtMethods>, MemberIter<jtype::OfField>)> {
        let mi = MemberIter::from_read(&mut self.r, &self.pool)?;
        Ok((self.step(), mi))
    }
    pub fn skip_fields(mut self) -> Result<JClassReader<R, AtMethods>> {
        skip_member_info(&mut self.r)?;
        Ok(self.step())
    }
}
impl <R: Read> JClassReader<R, AtMethods> {
    #[inline]
    pub fn methods(mut self) -> Result<(JClassReader<R, AtAttributes>, MemberIter<jtype::OfMethod>)> {
        let mi = MemberIter::from_read(&mut self.r, &self.pool)?;
        Ok((self.step(), mi))
    }
    pub fn skip_methods(mut self) -> Result<JClassReader<R, AtAttributes>> {
        skip_member_info(&mut self.r)?;
        Ok(self.step())
    }
}
impl <R: Read> JClassReader<R, AtAttributes> {
    #[inline]
    pub fn attributes(mut self) -> Result<(JClassReader<R, ()>, Attrs<jtype::OfClass>)> {
        let mut bmut = bytes::BytesMut::new();
        read_attr_info(&mut self.r, &mut bmut)?;
        let b = bmut.freeze();
        let len = b.clone().get_u16();
        let pool = self.pool.clone();
        Ok((self.step(), Attrs::new(b, pool, len)))
    }
    pub fn skip_attributes(mut self) -> Result<JClassReader<R, ()>> {
        skip_attr_info(&mut self.r)?;
        Ok(self.step())
    }
}