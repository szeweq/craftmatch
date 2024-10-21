use std::{io::{self, Read, Seek}, ops::Deref};

pub struct FileEntry {
    len: u64,
    comp_len: u64,
    start: u64,
    comp: Option<bool>
}
impl FileEntry {
    pub fn vec_from<RS: Read + Seek>(&self, rs: &mut RS) -> anyhow::Result<Vec<u8>> {
        self.reader(rs).and_then(|mut cr| {
            let mut buf = vec![0; self.len as usize];
            cr.read_exact(&mut buf)?;
            Ok(buf)
        })
    }
    pub fn string_from<RS: Read + Seek>(&self, rs: &mut RS) -> anyhow::Result<String> {
        self.reader(rs).and_then(|mut cr| {
            let mut buf = String::with_capacity(self.len as usize);
            cr.read_to_string(&mut buf)?;
            Ok(buf)
        })
    }
    pub fn reader<'a, RS: Read + Seek>(&self, rs: &'a mut RS) -> anyhow::Result<CompressRead<io::Take<&'a mut RS>>> {
        rs.seek(std::io::SeekFrom::Start(self.start))?;
        let rs = rs.take(self.comp_len);
        self.comp.map_or_else(|| Err(anyhow::anyhow!("Bad compression")), |x| {
            let cr = if x {
                CompressRead::Deflate(flate2::read::DeflateDecoder::new(rs))
            } else {
                CompressRead::Store(rs)
            };
            Ok(cr)
        })
    }

    pub const fn size(&self) -> u64 {
        self.len
    }
    pub const fn compressed(&self) -> u64 {
        self.comp_len
    }
}

#[repr(transparent)]
pub struct FileMap(pub indexmap::IndexMap<Box<str>, FileEntry>);
impl FileMap {
    pub fn from_zip_read_seek(mut rs: impl Read + Seek) -> anyhow::Result<Self> {
        let mut z = zip::ZipArchive::new(&mut rs)?;
        let mut m = indexmap::IndexMap::new();
        for i in 0..z.len() {
            if z.name_for_index(i).is_some_and(|n| n.ends_with(['/', '\\'])) { continue; }
            let file = z.by_index_raw(i)?;
            let comp = match file.compression() {
                zip::CompressionMethod::Stored => Some(false),
                zip::CompressionMethod::Deflated => Some(true),
                _ => None
            };
            m.insert(file.name().into(), FileEntry {
                len: file.size(),
                comp_len: file.compressed_size(),
                start: file.data_start(), comp
            });
        }
        Ok(Self(m))
    }
}
impl Deref for FileMap {
    type Target = indexmap::IndexMap<Box<str>, FileEntry>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
pub enum CompressRead<R: Read> {
    Deflate(flate2::read::DeflateDecoder<R>),
    Store(R)
}
impl <R: Read> Read for CompressRead<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Deflate(r) => r.read(buf),
            Self::Store(r) => r.read(buf)
        }
    }
}