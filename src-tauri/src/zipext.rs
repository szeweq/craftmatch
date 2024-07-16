use std::{io::{self, Read, Seek}, ops::Deref};


pub trait ZipExt {
    fn read_mem(&mut self, path: &str) -> Option<anyhow::Result<Vec<u8>>>;
    fn file_map(&mut self) -> anyhow::Result<FileMap>;
}

impl <RS: Read + Seek> ZipExt for zip::ZipArchive<RS> {
    #[inline]
    fn read_mem(&mut self, path: &str) -> Option<anyhow::Result<Vec<u8>>> {
        let mut file = match self.by_name(path) {
            Ok(file) => file,
            Err(zip::result::ZipError::FileNotFound) => return None,
            Err(e) => return Some(Err(e.into()))
        };
        let mut buf = vec![0; file.size() as usize];
        if let Err(e) = file.read_exact(&mut buf) {
            return Some(Err(e.into()));
        }
        Some(Ok(buf))
    }

    fn file_map(&mut self) -> anyhow::Result<FileMap> {
        let mut m = indexmap::IndexMap::new();
        for i in 0..self.len() {
            let file = self.by_index_raw(i)?;
            let comp = match file.compression() {
                zip::CompressionMethod::Stored => Some(false),
                zip::CompressionMethod::Deflated => Some(true),
                _ => None
            };
            m.insert(file.name().into(), FileEntry(file.size(), file.compressed_size(), file.data_start(), comp));
        }
        Ok(FileMap(m))
    }
}

pub struct FileEntry(u64, u64, u64, Option<bool>);
impl FileEntry {
    pub fn vec_from<RS: Read + Seek>(&self, rs: &mut RS) -> anyhow::Result<Vec<u8>> {
        self.reader(rs).and_then(|mut cr| {
            let mut buf = vec![0; self.0 as usize];
            cr.read_exact(&mut buf)?;
            Ok(buf)
        })
    }
    pub fn string_from<RS: Read + Seek>(&self, rs: &mut RS) -> anyhow::Result<String> {
        self.reader(rs).and_then(|mut cr| {
            let mut buf = String::with_capacity(self.0 as usize);
            cr.read_to_string(&mut buf)?;
            Ok(buf)
        })
    }
    pub fn reader<'a, RS: Read + Seek>(&self, rs: &'a mut RS) -> anyhow::Result<CompressRead<io::Take<&'a mut RS>>> {
        rs.seek(std::io::SeekFrom::Start(self.2))?;
        let rs = rs.take(self.1);
        self.3.map_or_else(|| Err(anyhow::anyhow!("Bad compression")), |x| {
            let cr = if x {
                CompressRead::Deflate(flate2::read::DeflateDecoder::new(rs))
            } else {
                CompressRead::Store(rs)
            };
            Ok(cr)
        })
    }

    pub const fn size(&self) -> u64 {
        self.0
    }
    pub const fn compressed(&self) -> u64 {
        self.1
    }
}

#[repr(transparent)]
pub struct FileMap(pub indexmap::IndexMap<Box<str>, FileEntry>);
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