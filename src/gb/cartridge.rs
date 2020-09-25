use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::{fs, io};

const TITLE_BEGIN: u16 = 0x0134;
const TITLE_END: u16 = 0x0143;

/// Contains parsed metadata of Cartridge
pub struct Metadata {
    pub title: String,
}

impl Metadata {
    pub fn from_buf(buf: &[u8]) -> Self {
        Self {
            title: Metadata::parse_title(buf),
        }
    }

    /// Returns title from metadata
    /// TODO: can it contain utf8 data?
    fn parse_title(buf: &[u8]) -> String {
        buf[TITLE_BEGIN as usize..TITLE_END as usize]
            .iter()
            .filter(|b| b.is_ascii_alphanumeric())
            .map(|b| char::from(*b))
            .collect()
    }
}

/// Contains all data for a cartridge
pub struct Cartridge {
    pub meta: Metadata,
    rom: Vec<u8>,
}

impl Cartridge {
    /// Creates a new Cartridge from the given Path
    pub fn from_path(path: &Path) -> io::Result<Self> {
        let mut file = File::open(&path)?;
        let metadata = fs::metadata(&path)?;
        let mut buffer = vec![0; metadata.len() as usize];
        file.read_exact(&mut buffer)?;

        let len = buffer.len();
        let meta = Metadata::from_buf(&buffer);
        let cartridge = Self { meta, rom: buffer };
        println!(
            "Loaded '{}' from {} with {} bytes",
            cartridge.meta.title,
            path.display(),
            len
        );
        Ok(cartridge)
    }

    pub fn read(&self, address: u16) -> u8 {
        // TODO: take care of memory banking
        self.rom[address as usize]
    }
}
