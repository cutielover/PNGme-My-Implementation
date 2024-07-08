use crc::{ Crc, CRC_32_ISO_HDLC };
use std::convert::TryFrom;
use std::fmt;
use std::io::{ BufReader, Read };

use crate::{ Error, Result };
use crate::chunk_type::ChunkType;

/// A validated PNG chunk. See the PNG Spec for more details
/// http://www.libpng.org/pub/png/spec/1.2/PNG-Structure.html
#[derive(Debug, Clone)]
pub struct Chunk {
    // Write me!
    len: u32,
    chunktype: ChunkType,
    chunk_data: Vec<u8>,
    chunk_crc: u32,
}

impl Chunk {
    /// New a Chunk
    pub fn new(chunktype_init: ChunkType, data_init: Vec<u8>) -> Chunk {
        let len_tmp: u32 = data_init.len() as u32;
        let bytes_type = chunktype_init.bytes();
        let mut check_crc: Vec<u8> = vec![
            bytes_type[0],
            bytes_type[1],
            bytes_type[2],
            bytes_type[3]
        ];
        for i in &data_init {
            check_crc.push(*i);
        }
        const CRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);
        let checksum = CRC32.checksum(&check_crc);

        Chunk {
            len: len_tmp,
            chunktype: chunktype_init,
            chunk_data: data_init,
            chunk_crc: checksum,
        }
    }

    /// The length of the data portion of this chunk.
    pub fn length(&self) -> u32 {
        self.len
    }

    /// The `ChunkType` of this chunk
    pub fn chunk_type(&self) -> &ChunkType {
        &self.chunktype
    }

    /// The raw data contained in this chunk in bytes
    pub fn data(&self) -> &[u8] {
        &self.chunk_data
    }

    /// The CRC of this chunk
    pub fn crc(&self) -> u32 {
        self.chunk_crc
    }

    /// Returns the data stored in this chunk as a `String`. This function will return an error
    /// if the stored data is not valid UTF-8.
    pub fn data_as_string(&self) -> Result<String> {
        let d = self.chunk_data.clone();
        let ans = String::from_utf8(d)?;
        Ok(ans)
    }

    /// Returns this chunk as a byte sequences described by the PNG spec.
    /// The following data is included in this byte sequence in order:
    /// 1. Length of the data *(4 bytes)*
    /// 2. Chunk type *(4 bytes)*
    /// 3. The data itself *(`length` bytes)*
    /// 4. The CRC of the chunk type and data *(4 bytes)*
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut ans: Vec<u8> = vec![];
        let len_tmp = u32::to_be_bytes(self.len);
        // 加入length
        for i in len_tmp {
            ans.push(i);
        }
        // 加入chunk type
        for j in self.chunktype.bytes() {
            ans.push(j);
        }
        // 加入data
        ans.append(&mut self.chunk_data.clone());
        // 加入crc
        let crc_tmp = u32::to_be_bytes(self.chunk_crc);
        for k in crc_tmp {
            ans.push(k);
        }

        ans
    }
}

impl TryFrom<&[u8]> for Chunk {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 12 {
            return Err("Chunk Try From Error: bytes not long enough".into());
        }
        let len_bytes: [u8; 4] = [bytes[0], bytes[1], bytes[2], bytes[3]];
        let len_tmp = u32::from_be_bytes(len_bytes);
        if (bytes.len() as u32) != 12 + len_tmp {
            return Err("Chunk Try From Error: data length wrong".into());
        }
        let chunk_type_bytes: [u8; 4] = [bytes[4], bytes[5], bytes[6], bytes[7]];
        let chunk_type_tmp = ChunkType::try_from(chunk_type_bytes)?;

        let mut check_crc: Vec<u8> = vec![bytes[4], bytes[5], bytes[6], bytes[7]];

        let mut data_tmp: Vec<u8> = vec![];
        let start = 8;
        let end = 8 + (len_tmp as usize);
        for i in start..end {
            data_tmp.push(bytes[i]);
            check_crc.push(bytes[i]);
        }

        let len_index = len_tmp as usize;
        let chunk_crc_bytes: [u8; 4] = [
            bytes[8 + len_index],
            bytes[9 + len_index],
            bytes[10 + len_index],
            bytes[11 + len_index],
        ];
        let chunk_crc_tmp = u32::from_be_bytes(chunk_crc_bytes);

        // check crc
        const CRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);
        let checksum = CRC32.checksum(&check_crc);

        if checksum != chunk_crc_tmp {
            return Err("Chunk Try From Error: crc not match".into());
        }

        let chunk_tmp = Chunk {
            len: len_tmp,
            chunktype: chunk_type_tmp,
            chunk_data: data_tmp,
            chunk_crc: chunk_crc_tmp,
        };
        Ok(chunk_tmp)
    }
}

impl fmt::Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Chunk {{")?;
        writeln!(f, "  Length: {}", self.length())?;
        writeln!(f, "  Type: {}", self.chunk_type())?;
        writeln!(f, "  Data: {} bytes", self.data().len())?;
        writeln!(f, "  Crc: {}", self.crc())?;
        writeln!(f, "}}")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk_type::ChunkType;
    use std::str::FromStr;

    // #[test]
    fn testing_chunk() -> Chunk {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        Chunk::try_from(chunk_data.as_ref()).unwrap()
    }

    #[test]
    fn test_new_chunk() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data = "This is where your secret message will be!".as_bytes().to_vec();
        let chunk = Chunk::new(chunk_type, data);
        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());

        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk: Chunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();

        let _chunk_string = format!("{}", chunk);
    }
}
