use std::io::{Read, Seek};


pub trait ZipExt {
    fn read_mem(&mut self, path: &str) -> Option<anyhow::Result<Vec<u8>>>;
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
}