use bevy::prelude::*;
use super::chunk::CHUNK_SIZE;

pub fn world_to_chunk_pos(world_pos: Vec3) -> IVec3 {
    let size = CHUNK_SIZE as i32;
    IVec3::new(
        (world_pos.x.floor() as i32).div_euclid(size),
        (world_pos.y.floor() as i32).div_euclid(size),
        (world_pos.z.floor() as i32).div_euclid(size),
    )
}

pub fn world_to_local_pos(world_pos: IVec3) -> UVec3 {
    let size = CHUNK_SIZE as i32;
    UVec3::new(
        world_pos.x.rem_euclid(size) as u32,
        world_pos.y.rem_euclid(size) as u32,
        world_pos.z.rem_euclid(size) as u32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positive_coords_chunk_pos() {
        let chunk = world_to_chunk_pos(Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(chunk, IVec3::new(0, 0, 0));

        let chunk = world_to_chunk_pos(Vec3::new(15.0, 15.0, 15.0));
        assert_eq!(chunk, IVec3::new(0, 0, 0));

        let chunk = world_to_chunk_pos(Vec3::new(16.0, 0.0, 0.0));
        assert_eq!(chunk, IVec3::new(1, 0, 0));

        let chunk = world_to_chunk_pos(Vec3::new(32.0, 48.0, 64.0));
        assert_eq!(chunk, IVec3::new(2, 3, 4));
    }

    #[test]
    fn negative_coords_chunk_pos_uses_div_euclid() {
        // -1 should be in chunk -1 (not 0)
        let chunk = world_to_chunk_pos(Vec3::new(-1.0, 0.0, 0.0));
        assert_eq!(chunk, IVec3::new(-1, 0, 0));

        let chunk = world_to_chunk_pos(Vec3::new(-16.0, 0.0, 0.0));
        assert_eq!(chunk, IVec3::new(-1, 0, 0));

        let chunk = world_to_chunk_pos(Vec3::new(-17.0, 0.0, 0.0));
        assert_eq!(chunk, IVec3::new(-2, 0, 0));
    }

    #[test]
    fn positive_coords_local_pos() {
        let local = world_to_local_pos(IVec3::new(0, 0, 0));
        assert_eq!(local, UVec3::new(0, 0, 0));

        let local = world_to_local_pos(IVec3::new(15, 15, 15));
        assert_eq!(local, UVec3::new(15, 15, 15));

        let local = world_to_local_pos(IVec3::new(16, 0, 0));
        assert_eq!(local, UVec3::new(0, 0, 0));

        let local = world_to_local_pos(IVec3::new(17, 18, 19));
        assert_eq!(local, UVec3::new(1, 2, 3));
    }

    #[test]
    fn negative_coords_local_pos_uses_rem_euclid() {
        // -1 should map to local 15
        let local = world_to_local_pos(IVec3::new(-1, -1, -1));
        assert_eq!(local, UVec3::new(15, 15, 15));

        // -16 should map to local 0
        let local = world_to_local_pos(IVec3::new(-16, 0, 0));
        assert_eq!(local, UVec3::new(0, 0, 0));

        // -17 should map to local 15
        let local = world_to_local_pos(IVec3::new(-17, 0, 0));
        assert_eq!(local, UVec3::new(15, 0, 0));
    }

    #[test]
    fn boundary_values() {
        // At exact chunk boundaries
        let chunk = world_to_chunk_pos(Vec3::new(0.0, 0.0, 0.0));
        let local = world_to_local_pos(IVec3::new(0, 0, 0));
        assert_eq!(chunk, IVec3::ZERO);
        assert_eq!(local, UVec3::ZERO);

        // Last block in chunk 0
        let local = world_to_local_pos(IVec3::new(15, 15, 15));
        assert_eq!(local, UVec3::new(15, 15, 15));

        // First block in chunk 1
        let chunk = world_to_chunk_pos(Vec3::new(16.0, 16.0, 16.0));
        let local = world_to_local_pos(IVec3::new(16, 16, 16));
        assert_eq!(chunk, IVec3::new(1, 1, 1));
        assert_eq!(local, UVec3::ZERO);
    }
}

