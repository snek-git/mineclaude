pub mod dropped_item;
pub mod mob;

use bevy::prelude::*;

pub struct EntityPlugin;

impl Plugin for EntityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<mob::SunburnTimer>()
            .add_systems(Startup, (mob::setup_mob_materials, dropped_item::setup_dropped_item_assets))
            .add_systems(
                Update,
                (
                    mob::spawn_mobs,
                    mob::update_mob_ai,
                    mob::move_mobs.after(mob::update_mob_ai),
                    mob::hostile_attack_player.after(mob::move_mobs),
                    mob::hostile_sunburn,
                    mob::despawn_dead_mobs,
                    mob::despawn_distant_mobs,
                    mob::despawn_void_mobs,
                    dropped_item::dropped_item_physics,
                    dropped_item::dropped_item_bob,
                    dropped_item::pickup_dropped_items,
                    dropped_item::dropped_item_despawn,
                    dropped_item::dropped_item_despawn_void,
                ),
            );
    }
}
