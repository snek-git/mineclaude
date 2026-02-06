use serde::{Deserialize, Serialize};

use crate::block::BlockType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Item {
    Block(BlockType),
    Stick,
    Coal,
    IronIngot,
    GoldIngot,
    Diamond,
    WoodenPickaxe,
    WoodenAxe,
    WoodenShovel,
    WoodenSword,
    StonePickaxe,
    StoneAxe,
    StoneShovel,
    StoneSword,
    IronPickaxe,
    IronAxe,
    IronShovel,
    IronSword,
    DiamondPickaxe,
    DiamondAxe,
    DiamondShovel,
    DiamondSword,
    Apple,
    Bread,
    CookedPorkchop,
    RawPorkchop,
    RawBeef,
    CookedBeef,
    Leather,
    RawMutton,
    CookedMutton,
    Wool,
    RottenFlesh,
    Bone,
    WoodenHoe,
    StoneHoe,
    IronHoe,
    DiamondHoe,
    Seeds,
    Wheat,
    LeatherHelmet,
    LeatherChestplate,
    LeatherLeggings,
    LeatherBoots,
    IronHelmet,
    IronChestplate,
    IronLeggings,
    IronBoots,
    DiamondHelmet,
    DiamondChestplate,
    DiamondLeggings,
    DiamondBoots,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolKind {
    Pickaxe,
    Axe,
    Shovel,
    Sword,
    Hoe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ToolTier {
    Wooden = 0,
    Stone = 1,
    Iron = 2,
    Gold = 3,
    Diamond = 4,
}

impl Item {
    pub fn max_stack(self) -> u8 {
        match self {
            Self::WoodenPickaxe | Self::WoodenAxe | Self::WoodenShovel | Self::WoodenSword
            | Self::StonePickaxe | Self::StoneAxe | Self::StoneShovel | Self::StoneSword
            | Self::IronPickaxe | Self::IronAxe | Self::IronShovel | Self::IronSword
            | Self::DiamondPickaxe | Self::DiamondAxe | Self::DiamondShovel | Self::DiamondSword
            | Self::WoodenHoe | Self::StoneHoe | Self::IronHoe | Self::DiamondHoe
            | Self::LeatherHelmet | Self::LeatherChestplate | Self::LeatherLeggings | Self::LeatherBoots
            | Self::IronHelmet | Self::IronChestplate | Self::IronLeggings | Self::IronBoots
            | Self::DiamondHelmet | Self::DiamondChestplate | Self::DiamondLeggings | Self::DiamondBoots => 1,
            _ => 64,
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Block(bt) => bt.display_name(),
            Self::Stick => "Stick",
            Self::Coal => "Coal",
            Self::IronIngot => "Iron Ingot",
            Self::GoldIngot => "Gold Ingot",
            Self::Diamond => "Diamond",
            Self::WoodenPickaxe => "Wooden Pickaxe",
            Self::WoodenAxe => "Wooden Axe",
            Self::WoodenShovel => "Wooden Shovel",
            Self::WoodenSword => "Wooden Sword",
            Self::StonePickaxe => "Stone Pickaxe",
            Self::StoneAxe => "Stone Axe",
            Self::StoneShovel => "Stone Shovel",
            Self::StoneSword => "Stone Sword",
            Self::IronPickaxe => "Iron Pickaxe",
            Self::IronAxe => "Iron Axe",
            Self::IronShovel => "Iron Shovel",
            Self::IronSword => "Iron Sword",
            Self::DiamondPickaxe => "Diamond Pickaxe",
            Self::DiamondAxe => "Diamond Axe",
            Self::DiamondShovel => "Diamond Shovel",
            Self::DiamondSword => "Diamond Sword",
            Self::Apple => "Apple",
            Self::Bread => "Bread",
            Self::CookedPorkchop => "Cooked Porkchop",
            Self::RawPorkchop => "Raw Porkchop",
            Self::RawBeef => "Raw Beef",
            Self::CookedBeef => "Cooked Beef",
            Self::Leather => "Leather",
            Self::RawMutton => "Raw Mutton",
            Self::CookedMutton => "Cooked Mutton",
            Self::Wool => "Wool",
            Self::RottenFlesh => "Rotten Flesh",
            Self::Bone => "Bone",
            Self::WoodenHoe => "Wooden Hoe",
            Self::StoneHoe => "Stone Hoe",
            Self::IronHoe => "Iron Hoe",
            Self::DiamondHoe => "Diamond Hoe",
            Self::Seeds => "Seeds",
            Self::Wheat => "Wheat",
            Self::LeatherHelmet => "Leather Helmet",
            Self::LeatherChestplate => "Leather Chestplate",
            Self::LeatherLeggings => "Leather Leggings",
            Self::LeatherBoots => "Leather Boots",
            Self::IronHelmet => "Iron Helmet",
            Self::IronChestplate => "Iron Chestplate",
            Self::IronLeggings => "Iron Leggings",
            Self::IronBoots => "Iron Boots",
            Self::DiamondHelmet => "Diamond Helmet",
            Self::DiamondChestplate => "Diamond Chestplate",
            Self::DiamondLeggings => "Diamond Leggings",
            Self::DiamondBoots => "Diamond Boots",
        }
    }

    pub fn is_block(self) -> bool {
        matches!(self, Self::Block(_))
    }

    pub fn as_block(self) -> Option<BlockType> {
        match self {
            Self::Block(bt) => Some(bt),
            _ => None,
        }
    }

    pub fn is_tool(self) -> bool {
        self.tool_kind().is_some()
    }

    pub fn tool_kind(self) -> Option<ToolKind> {
        match self {
            Self::WoodenPickaxe | Self::StonePickaxe | Self::IronPickaxe | Self::DiamondPickaxe => Some(ToolKind::Pickaxe),
            Self::WoodenAxe | Self::StoneAxe | Self::IronAxe | Self::DiamondAxe => Some(ToolKind::Axe),
            Self::WoodenShovel | Self::StoneShovel | Self::IronShovel | Self::DiamondShovel => Some(ToolKind::Shovel),
            Self::WoodenSword | Self::StoneSword | Self::IronSword | Self::DiamondSword => Some(ToolKind::Sword),
            Self::WoodenHoe | Self::StoneHoe | Self::IronHoe | Self::DiamondHoe => Some(ToolKind::Hoe),
            _ => None,
        }
    }

    pub fn tool_tier(self) -> Option<ToolTier> {
        match self {
            Self::WoodenPickaxe | Self::WoodenAxe | Self::WoodenShovel | Self::WoodenSword | Self::WoodenHoe => {
                Some(ToolTier::Wooden)
            }
            Self::StonePickaxe | Self::StoneAxe | Self::StoneShovel | Self::StoneSword | Self::StoneHoe => {
                Some(ToolTier::Stone)
            }
            Self::IronPickaxe | Self::IronAxe | Self::IronShovel | Self::IronSword | Self::IronHoe => {
                Some(ToolTier::Iron)
            }
            Self::DiamondPickaxe | Self::DiamondAxe | Self::DiamondShovel | Self::DiamondSword | Self::DiamondHoe => {
                Some(ToolTier::Diamond)
            }
            _ => None,
        }
    }

    /// Returns (food_restore, saturation_restore) if this item is food.
    pub fn food_value(self) -> Option<(f32, f32)> {
        match self {
            Self::Apple => Some((4.0, 2.4)),
            Self::Bread => Some((5.0, 6.0)),
            Self::CookedPorkchop => Some((8.0, 12.8)),
            Self::RawPorkchop => Some((3.0, 1.8)),
            Self::RawBeef => Some((3.0, 1.8)),
            Self::CookedBeef => Some((8.0, 12.8)),
            Self::RawMutton => Some((2.0, 1.2)),
            Self::CookedMutton => Some((6.0, 9.6)),
            Self::RottenFlesh => Some((4.0, 0.8)),
            _ => None,
        }
    }

    /// Returns true if this item is a food item.
    pub fn is_food(self) -> bool {
        self.food_value().is_some()
    }

    /// Maximum durability (uses) for tools and armor. Returns 0 for non-tools/non-armor.
    pub fn max_durability(self) -> u16 {
        if let Some(dur) = self.armor_durability() {
            return dur;
        }
        match self.tool_tier() {
            Some(ToolTier::Wooden) => 59,
            Some(ToolTier::Stone) => 131,
            Some(ToolTier::Iron) => 250,
            Some(ToolTier::Gold) => 32,
            Some(ToolTier::Diamond) => 1561,
            None => 0,
        }
    }

    /// Returns true if this item is an armor piece.
    pub fn is_armor(self) -> bool {
        self.armor_slot().is_some()
    }

    /// Returns the armor slot index: 0=helmet, 1=chestplate, 2=leggings, 3=boots.
    pub fn armor_slot(self) -> Option<usize> {
        match self {
            Self::LeatherHelmet | Self::IronHelmet | Self::DiamondHelmet => Some(0),
            Self::LeatherChestplate | Self::IronChestplate | Self::DiamondChestplate => Some(1),
            Self::LeatherLeggings | Self::IronLeggings | Self::DiamondLeggings => Some(2),
            Self::LeatherBoots | Self::IronBoots | Self::DiamondBoots => Some(3),
            _ => None,
        }
    }

    /// Returns the defense points for this armor piece (0 if not armor).
    pub fn armor_points(self) -> u8 {
        match self {
            Self::LeatherHelmet => 1, Self::LeatherChestplate => 3, Self::LeatherLeggings => 2, Self::LeatherBoots => 1,
            Self::IronHelmet => 2, Self::IronChestplate => 6, Self::IronLeggings => 5, Self::IronBoots => 2,
            Self::DiamondHelmet => 3, Self::DiamondChestplate => 8, Self::DiamondLeggings => 6, Self::DiamondBoots => 3,
            _ => 0,
        }
    }

    /// Returns the max durability for armor pieces.
    pub fn armor_durability(self) -> Option<u16> {
        match self {
            Self::LeatherHelmet => Some(55), Self::LeatherChestplate => Some(80),
            Self::LeatherLeggings => Some(75), Self::LeatherBoots => Some(65),
            Self::IronHelmet => Some(165), Self::IronChestplate => Some(240),
            Self::IronLeggings => Some(225), Self::IronBoots => Some(195),
            Self::DiamondHelmet => Some(363), Self::DiamondChestplate => Some(528),
            Self::DiamondLeggings => Some(495), Self::DiamondBoots => Some(429),
            _ => None,
        }
    }
}
