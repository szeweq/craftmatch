use std::io::{Read, Seek, SeekFrom};

use byteorder::{ReadBytesExt, BE};
use bytes::Bytes;

use super::{
    idx::{ClassInfo, Index},
    iter::MemberIter,
    jtype,
    pool::{ClassPool, PoolItem},
    read_magic, skip_attr_info, skip_member_info, Attrs, JStr,
};

pub struct JClassSeekReader<R: Read + Seek> {
    r: R,
    pool: ClassPool,
    minor: u16,
    major: u16,
    access_flags: u16,
    class_ref: Index<ClassInfo>,
    super_ref: Option<Index<ClassInfo>>,
    pos_interfaces: u64,
    pos_fields: u64,
    pos_methods: u64,
    pos_attributes: u64,
    end: u64,
}

impl<R: Read + Seek> JClassSeekReader<R> {
    pub fn new(mut r: R) -> anyhow::Result<Self> {
        read_magic(&mut r)?;
        let mut cv = [0u16; 3];
        let [minor, major, pool_count] = match r.read_u16_into::<BE>(&mut cv) {
            Ok(()) => cv,
            Err(e) => return Err(e.into()),
        };
        let mut pool = vec![PoolItem::None];
        while pool.len() < pool_count as usize {
            let tag = r.read_u8()?;
            pool.push(PoolItem::read_from(tag, major, &mut r)?);
            if tag == 5 || tag == 6 {
                pool.push(PoolItem::Reserved);
            }
        }
        let pool = ClassPool::from(pool.into_boxed_slice());
        let mut cv = [0u16; 3];
        let [access_flags, class_ref, super_ref] = match r.read_u16_into::<BE>(&mut cv) {
            Ok(()) => cv,
            Err(e) => return Err(e.into()),
        };
        let class_ref = class_ref.try_into()?;
        let super_ref = Index::maybe(super_ref);
        let pos_interfaces = r.stream_position()?;
        let iccount = r.read_u16::<BE>()? as usize;
        r.seek(SeekFrom::Current(iccount as i64 * 2))?;
        let pos_fields = r.stream_position()?;
        skip_member_info(&mut r)?;
        let pos_methods = r.stream_position()?;
        skip_member_info(&mut r)?;
        let pos_attributes = r.stream_position()?;
        skip_attr_info(&mut r)?;
        let end = r.stream_position()?;
        if r.read(&mut [0; 4])? > 0 {
            anyhow::bail!("Invalid end");
        }
        Ok(Self {
            r,
            pool,
            minor,
            major,
            access_flags,
            class_ref,
            super_ref,
            pos_interfaces,
            pos_fields,
            pos_methods,
            pos_attributes,
            end,
        })
    }

    pub fn class_name(&self) -> anyhow::Result<&JStr> {
        self.pool.get(self.pool.get(self.class_ref)?)
    }

    pub fn super_class(&self) -> anyhow::Result<Option<&JStr>> {
        match self.super_ref {
            None => Ok(None),
            Some(super_ref) => Ok(Some(self.pool.get(self.pool.get(super_ref)?)?)),
        }
    }

    pub fn interfaces(
        &mut self,
    ) -> anyhow::Result<impl Iterator<Item = anyhow::Result<&JStr>> + '_> {
        self.r.seek(SeekFrom::Start(self.pos_interfaces))?;
        let mut v = vec![0; self.r.read_u16::<BE>()? as usize];
        self.r.read_u16_into::<BE>(&mut v)?;
        Ok(v.into_iter()
            .map(|x| self.pool.get(self.pool.get_::<ClassInfo>(x)?)))
    }
    fn fill_data(&mut self, from: u64, to: u64) -> anyhow::Result<(u16, Bytes)> {
        self.r.seek(SeekFrom::Start(from))?;
        let len = self.r.read_u16::<BE>()?;
        let mut buf = vec![0; to.abs_diff(from) as usize];
        self.r.read_exact(&mut buf)?;
        let cur = Bytes::from(buf.into_boxed_slice());
        Ok((len, cur))
    }
    pub fn fields(&mut self) -> anyhow::Result<MemberIter<jtype::OfField>> {
        let (len, b) = self.fill_data(self.pos_fields, self.pos_methods)?;
        Ok(MemberIter::new(b, self.pool.clone(), len))
    }
    pub fn methods(&mut self) -> anyhow::Result<MemberIter<jtype::OfMethod>> {
        let (len, b) = self.fill_data(self.pos_methods, self.pos_attributes)?;
        Ok(MemberIter::new(b, self.pool.clone(), len))
    }
    pub fn class_attrs(&mut self) -> anyhow::Result<Attrs<jtype::OfClass>> {
        let (len, b) = self.fill_data(self.pos_attributes, self.end)?;
        Ok(Attrs::new(b, self.pool.clone(), len))
    }
}
