use crate::block::BlockType;

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

#[derive(Clone)]
pub struct Chunk {
    blocks: [u8; CHUNK_VOLUME],
}

impl serde::Serialize for Chunk {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(&self.blocks)
    }
}

impl<'de> serde::Deserialize<'de> for Chunk {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ChunkVisitor;

        impl<'de> serde::de::Visitor<'de> for ChunkVisitor {
            type Value = Chunk;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "a byte array of length {}", CHUNK_VOLUME)
            }

            fn visit_bytes<E: serde::de::Error>(self, v: &[u8]) -> Result<Chunk, E> {
                if v.len() != CHUNK_VOLUME {
                    return Err(E::invalid_length(v.len(), &self));
                }
                let mut blocks = [0u8; CHUNK_VOLUME];
                blocks.copy_from_slice(v);
                Ok(Chunk { blocks })
            }

            fn visit_byte_buf<E: serde::de::Error>(self, v: Vec<u8>) -> Result<Chunk, E> {
                self.visit_bytes(&v)
            }
        }

        deserializer.deserialize_bytes(ChunkVisitor)
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            blocks: [0; CHUNK_VOLUME],
        }
    }
}

impl Chunk {
    #[inline]
    pub fn index(x: usize, y: usize, z: usize) -> usize {
        y * CHUNK_SIZE * CHUNK_SIZE + z * CHUNK_SIZE + x
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> BlockType {
        BlockType::from_id(self.blocks[Self::index(x, y, z)])
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, z: usize, block: BlockType) {
        self.blocks[Self::index(x, y, z)] = block as u8;
    }

    pub fn is_empty(&self) -> bool {
        self.blocks.iter().all(|&b| b == 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_chunk_is_empty() {
        let chunk = Chunk::default();
        assert!(chunk.is_empty());
    }

    #[test]
    fn new_chunk_is_all_air() {
        let chunk = Chunk::default();
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    assert_eq!(chunk.get(x, y, z), BlockType::Air);
                }
            }
        }
    }

    #[test]
    fn set_get_roundtrip() {
        let mut chunk = Chunk::default();
        chunk.set(5, 10, 3, BlockType::Stone);
        assert_eq!(chunk.get(5, 10, 3), BlockType::Stone);

        chunk.set(5, 10, 3, BlockType::Dirt);
        assert_eq!(chunk.get(5, 10, 3), BlockType::Dirt);
    }

    #[test]
    fn set_get_boundary_positions() {
        let mut chunk = Chunk::default();

        // Origin corner
        chunk.set(0, 0, 0, BlockType::Cobblestone);
        assert_eq!(chunk.get(0, 0, 0), BlockType::Cobblestone);

        // Max corner
        chunk.set(15, 15, 15, BlockType::DiamondOre);
        assert_eq!(chunk.get(15, 15, 15), BlockType::DiamondOre);
    }

    #[test]
    fn chunk_not_empty_after_set() {
        let mut chunk = Chunk::default();
        chunk.set(0, 0, 0, BlockType::Stone);
        assert!(!chunk.is_empty());
    }

    #[test]
    fn chunk_empty_after_clearing() {
        let mut chunk = Chunk::default();
        chunk.set(8, 8, 8, BlockType::Stone);
        assert!(!chunk.is_empty());
        chunk.set(8, 8, 8, BlockType::Air);
        assert!(chunk.is_empty());
    }

    #[test]
    fn index_yzx_ordering() {
        // YZX means y * 256 + z * 16 + x
        assert_eq!(Chunk::index(0, 0, 0), 0);
        assert_eq!(Chunk::index(1, 0, 0), 1);
        assert_eq!(Chunk::index(0, 0, 1), CHUNK_SIZE);
        assert_eq!(Chunk::index(0, 1, 0), CHUNK_SIZE * CHUNK_SIZE);
    }

    #[test]
    fn different_positions_independent() {
        let mut chunk = Chunk::default();
        chunk.set(0, 0, 0, BlockType::Stone);
        chunk.set(1, 0, 0, BlockType::Dirt);
        chunk.set(0, 1, 0, BlockType::Sand);
        assert_eq!(chunk.get(0, 0, 0), BlockType::Stone);
        assert_eq!(chunk.get(1, 0, 0), BlockType::Dirt);
        assert_eq!(chunk.get(0, 1, 0), BlockType::Sand);
        assert_eq!(chunk.get(0, 0, 1), BlockType::Air);
    }
}
