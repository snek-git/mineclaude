pub mod atlas;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum BlockType {
    #[default]
    Air = 0,
    Stone = 1,
    Dirt = 2,
    Grass = 3,
    Cobblestone = 4,
    Planks = 5,
    Sand = 6,
    Gravel = 7,
    OakLog = 8,
    OakLeaves = 9,
    Glass = 10,
    CoalOre = 11,
    IronOre = 12,
    GoldOre = 13,
    DiamondOre = 14,
    Bedrock = 15,
    Water = 16,
    CraftingTable = 17,
    Furnace = 18,
    Torch = 19,
    Snow = 20,
    Clay = 21,
    Sandstone = 22,
    BirchLog = 23,
    BirchLeaves = 24,
    TallGrass = 25,
    Chest = 26,
    Bed = 27,
    DoorBottom = 28,
    DoorTop = 29,
    DoorBottomOpen = 30,
    DoorTopOpen = 31,
    OakSapling = 32,
    BirchSapling = 33,
    Farmland = 34,
    WheatStage0 = 35,
    WheatStage1 = 36,
    WheatStage2 = 37,
    WheatStage3 = 38,
}

impl BlockType {
    pub fn from_id(id: u8) -> Self {
        match id {
            0 => Self::Air,
            1 => Self::Stone,
            2 => Self::Dirt,
            3 => Self::Grass,
            4 => Self::Cobblestone,
            5 => Self::Planks,
            6 => Self::Sand,
            7 => Self::Gravel,
            8 => Self::OakLog,
            9 => Self::OakLeaves,
            10 => Self::Glass,
            11 => Self::CoalOre,
            12 => Self::IronOre,
            13 => Self::GoldOre,
            14 => Self::DiamondOre,
            15 => Self::Bedrock,
            16 => Self::Water,
            17 => Self::CraftingTable,
            18 => Self::Furnace,
            19 => Self::Torch,
            20 => Self::Snow,
            21 => Self::Clay,
            22 => Self::Sandstone,
            23 => Self::BirchLog,
            24 => Self::BirchLeaves,
            25 => Self::TallGrass,
            26 => Self::Chest,
            27 => Self::Bed,
            28 => Self::DoorBottom,
            29 => Self::DoorTop,
            30 => Self::DoorBottomOpen,
            31 => Self::DoorTopOpen,
            32 => Self::OakSapling,
            33 => Self::BirchSapling,
            34 => Self::Farmland,
            35 => Self::WheatStage0,
            36 => Self::WheatStage1,
            37 => Self::WheatStage2,
            38 => Self::WheatStage3,
            _ => Self::Air,
        }
    }

    pub fn is_solid(self) -> bool {
        matches!(
            self,
            Self::Stone
                | Self::Dirt
                | Self::Grass
                | Self::Cobblestone
                | Self::Planks
                | Self::Sand
                | Self::Gravel
                | Self::OakLog
                | Self::OakLeaves
                | Self::Glass
                | Self::CoalOre
                | Self::IronOre
                | Self::GoldOre
                | Self::DiamondOre
                | Self::Bedrock
                | Self::CraftingTable
                | Self::Furnace
                | Self::Snow
                | Self::Clay
                | Self::Sandstone
                | Self::BirchLog
                | Self::BirchLeaves
                | Self::Chest
                | Self::Bed
                | Self::DoorBottom
                | Self::DoorTop
                | Self::Farmland
        )
    }

    pub fn is_transparent(self) -> bool {
        matches!(
            self,
            Self::Air
                | Self::Water
                | Self::Glass
                | Self::OakLeaves
                | Self::BirchLeaves
                | Self::Torch
                | Self::TallGrass
                | Self::DoorBottom
                | Self::DoorTop
                | Self::DoorBottomOpen
                | Self::DoorTopOpen
                | Self::OakSapling
                | Self::BirchSapling
                | Self::WheatStage0
                | Self::WheatStage1
                | Self::WheatStage2
                | Self::WheatStage3
        )
    }

    /// Returns true for blocks that should not be rendered as cube geometry.
    /// These blocks are non-solid decorations (torches, saplings, tall grass).
    pub fn is_non_cube(self) -> bool {
        matches!(
            self,
            Self::Torch | Self::TallGrass | Self::OakSapling | Self::BirchSapling
            | Self::WheatStage0 | Self::WheatStage1 | Self::WheatStage2 | Self::WheatStage3
        )
    }

    pub fn is_air(self) -> bool {
        self == Self::Air
    }

    pub fn is_liquid(self) -> bool {
        self == Self::Water
    }

    /// Returns true if the player's raycast should be able to target this block.
    /// Everything except Air and Water is targetable.
    pub fn is_targetable(self) -> bool {
        !matches!(self, Self::Air | Self::Water)
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Air => "",
            Self::Stone => "Stone",
            Self::Dirt => "Dirt",
            Self::Grass => "Grass Block",
            Self::Cobblestone => "Cobblestone",
            Self::Planks => "Oak Planks",
            Self::Sand => "Sand",
            Self::Gravel => "Gravel",
            Self::OakLog => "Oak Log",
            Self::OakLeaves => "Oak Leaves",
            Self::Glass => "Glass",
            Self::CoalOre => "Coal Ore",
            Self::IronOre => "Iron Ore",
            Self::GoldOre => "Gold Ore",
            Self::DiamondOre => "Diamond Ore",
            Self::Bedrock => "Bedrock",
            Self::Water => "Water",
            Self::CraftingTable => "Crafting Table",
            Self::Furnace => "Furnace",
            Self::Torch => "Torch",
            Self::Snow => "Snow",
            Self::Clay => "Clay",
            Self::Sandstone => "Sandstone",
            Self::BirchLog => "Birch Log",
            Self::BirchLeaves => "Birch Leaves",
            Self::TallGrass => "Tall Grass",
            Self::Chest => "Chest",
            Self::Bed => "Bed",
            Self::DoorBottom | Self::DoorBottomOpen => "Oak Door",
            Self::DoorTop | Self::DoorTopOpen => "Oak Door",
            Self::OakSapling => "Oak Sapling",
            Self::BirchSapling => "Birch Sapling",
            Self::Farmland => "Farmland",
            Self::WheatStage0 | Self::WheatStage1 | Self::WheatStage2 | Self::WheatStage3 => "Wheat",
        }
    }

    /// Returns the item(s) dropped when this block is broken.
    /// Returns None for blocks that drop nothing (glass, leaves, etc.).
    pub fn drop_item(self) -> Option<crate::inventory::item::Item> {
        use crate::inventory::item::Item;
        match self {
            Self::Air | Self::Water | Self::Bedrock => None,
            Self::Glass => None,
            Self::OakLeaves => {
                // Independent rolls: 5% apple, 5% sapling (apple takes priority)
                let apple_roll = rand::random::<f32>();
                let sapling_roll = rand::random::<f32>();
                if apple_roll < 0.05 {
                    Some(crate::inventory::item::Item::Apple)
                } else if sapling_roll < 0.05 {
                    Some(Item::Block(Self::OakSapling))
                } else {
                    None
                }
            }
            Self::BirchLeaves => {
                if rand::random::<f32>() < 0.05 {
                    Some(Item::Block(Self::BirchSapling))
                } else {
                    None
                }
            }
            Self::TallGrass => {
                if rand::random::<f32>() < 0.3 {
                    Some(Item::Seeds)
                } else {
                    None
                }
            }
            Self::DoorTop | Self::DoorTopOpen => Some(Item::Block(Self::DoorBottom)),
            Self::DoorBottomOpen => Some(Item::Block(Self::DoorBottom)),
            Self::Grass => Some(Item::Block(Self::Dirt)),
            Self::Stone => Some(Item::Block(Self::Cobblestone)),
            Self::CoalOre => Some(Item::Coal),
            Self::DiamondOre => Some(Item::Diamond),
            Self::Farmland => Some(Item::Block(Self::Dirt)),
            Self::WheatStage0 | Self::WheatStage1 | Self::WheatStage2 => Some(Item::Seeds),
            Self::WheatStage3 => Some(Item::Wheat),
            _ => Some(Item::Block(self)),
        }
    }

    /// Returns additional random drops when this block is broken (beyond drop_item).
    /// Returns a list of (item, count) pairs.
    pub fn bonus_drops(self) -> Vec<(crate::inventory::item::Item, u8)> {
        use crate::inventory::item::Item;
        match self {
            Self::WheatStage3 => {
                // Mature wheat drops 0-3 seeds in addition to wheat
                let seed_count = (rand::random::<f32>() * 4.0).floor() as u8; // 0,1,2,3
                if seed_count > 0 {
                    vec![(Item::Seeds, seed_count)]
                } else {
                    vec![]
                }
            }
            Self::Grass => {
                // 10% chance to drop seeds when digging grass blocks
                if rand::random::<f32>() < 0.1 {
                    vec![(Item::Seeds, 1)]
                } else {
                    vec![]
                }
            }
            _ => vec![],
        }
    }

    /// Returns the base time in seconds to break this block by hand (without tools).
    pub fn break_time(self) -> f32 {
        match self {
            Self::Air | Self::Water => 0.0,
            Self::TallGrass | Self::Torch | Self::OakSapling | Self::BirchSapling
            | Self::WheatStage0 | Self::WheatStage1 | Self::WheatStage2 | Self::WheatStage3 => 0.0, // instant break
            Self::OakLeaves | Self::BirchLeaves => 0.3,
            Self::Glass => 0.45,
            Self::Dirt | Self::Sand | Self::Farmland => 0.75,
            Self::Gravel | Self::Clay => 0.9,
            Self::Snow => 0.3,
            Self::Grass => 0.9,
            Self::OakLog | Self::BirchLog | Self::Planks
            | Self::DoorBottom | Self::DoorTop | Self::DoorBottomOpen | Self::DoorTopOpen => 3.0,
            Self::CraftingTable => 3.75,
            Self::Sandstone => 4.0,
            Self::Stone => 7.5,
            Self::Cobblestone => 10.0,
            Self::CoalOre | Self::IronOre | Self::GoldOre | Self::DiamondOre => 15.0,
            Self::Furnace => 17.5,
            Self::Chest => 3.75,
            Self::Bed => 0.3,
            Self::Bedrock => f32::MAX, // unbreakable
        }
    }

    /// Returns the minimum tool tier required to get drops from this block.
    /// None means hand/any tool works. Some(tier) means at least that pickaxe tier.
    pub fn required_pickaxe_tier(self) -> Option<crate::inventory::item::ToolTier> {
        use crate::inventory::item::ToolTier;
        match self {
            Self::Stone | Self::Cobblestone | Self::Sandstone | Self::Furnace | Self::CoalOre => {
                Some(ToolTier::Wooden) // any pickaxe
            }
            Self::IronOre => Some(ToolTier::Stone),
            Self::GoldOre | Self::DiamondOre => Some(ToolTier::Iron),
            _ => None,
        }
    }
}

/// Face direction for block face lookups
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Face {
    Top,
    Bottom,
    North,
    South,
    East,
    West,
}

impl Face {
    pub fn normal(self) -> [f32; 3] {
        match self {
            Self::Top => [0.0, 1.0, 0.0],
            Self::Bottom => [0.0, -1.0, 0.0],
            Self::North => [0.0, 0.0, -1.0],
            Self::South => [0.0, 0.0, 1.0],
            Self::East => [1.0, 0.0, 0.0],
            Self::West => [-1.0, 0.0, 0.0],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solid_blocks_are_solid() {
        assert!(BlockType::Stone.is_solid());
        assert!(BlockType::Dirt.is_solid());
        assert!(BlockType::Cobblestone.is_solid());
        assert!(BlockType::Planks.is_solid());
        assert!(BlockType::Bedrock.is_solid());
        assert!(BlockType::OakLog.is_solid());
    }

    #[test]
    fn non_solid_blocks() {
        assert!(!BlockType::Air.is_solid());
        assert!(!BlockType::Water.is_solid());
        assert!(!BlockType::Torch.is_solid());
        assert!(!BlockType::TallGrass.is_solid());
    }

    #[test]
    fn transparent_blocks() {
        assert!(BlockType::Air.is_transparent());
        assert!(BlockType::Glass.is_transparent());
        assert!(BlockType::Water.is_transparent());
        assert!(BlockType::OakLeaves.is_transparent());
        assert!(BlockType::BirchLeaves.is_transparent());
        assert!(BlockType::Torch.is_transparent());
        assert!(BlockType::TallGrass.is_transparent());
    }

    #[test]
    fn opaque_blocks_not_transparent() {
        assert!(!BlockType::Stone.is_transparent());
        assert!(!BlockType::Dirt.is_transparent());
        assert!(!BlockType::Bedrock.is_transparent());
    }

    #[test]
    fn from_id_roundtrip() {
        for id in 0..=38u8 {
            let bt = BlockType::from_id(id);
            assert_eq!(bt as u8, id);
        }
    }

    #[test]
    fn from_id_unknown_returns_air() {
        assert_eq!(BlockType::from_id(255), BlockType::Air);
        assert_eq!(BlockType::from_id(100), BlockType::Air);
        assert_eq!(BlockType::from_id(39), BlockType::Air);
    }

    #[test]
    fn break_time_instant_for_air_and_water() {
        assert_eq!(BlockType::Air.break_time(), 0.0);
        assert_eq!(BlockType::Water.break_time(), 0.0);
        assert_eq!(BlockType::TallGrass.break_time(), 0.0);
        assert_eq!(BlockType::Torch.break_time(), 0.0);
    }

    #[test]
    fn break_time_positive_for_solid_blocks() {
        assert!(BlockType::Stone.break_time() > 0.0);
        assert!(BlockType::Dirt.break_time() > 0.0);
        assert!(BlockType::OakLog.break_time() > 0.0);
    }

    #[test]
    fn bedrock_unbreakable() {
        assert_eq!(BlockType::Bedrock.break_time(), f32::MAX);
    }

    #[test]
    fn dirt_breaks_faster_than_stone() {
        assert!(BlockType::Dirt.break_time() < BlockType::Stone.break_time());
    }

    #[test]
    fn display_name_empty_for_air() {
        assert!(BlockType::Air.display_name().is_empty());
    }

    #[test]
    fn display_name_not_empty_for_blocks() {
        assert!(!BlockType::Stone.display_name().is_empty());
        assert!(!BlockType::Dirt.display_name().is_empty());
        assert!(!BlockType::CraftingTable.display_name().is_empty());
    }

    #[test]
    fn is_air_only_for_air() {
        assert!(BlockType::Air.is_air());
        assert!(!BlockType::Stone.is_air());
        assert!(!BlockType::Water.is_air());
    }

    #[test]
    fn is_liquid_only_for_water() {
        assert!(BlockType::Water.is_liquid());
        assert!(!BlockType::Air.is_liquid());
        assert!(!BlockType::Stone.is_liquid());
    }

    #[test]
    fn face_normals_unit_length() {
        for face in [Face::Top, Face::Bottom, Face::North, Face::South, Face::East, Face::West] {
            let n = face.normal();
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            assert!((len - 1.0).abs() < 0.001);
        }
    }
}
