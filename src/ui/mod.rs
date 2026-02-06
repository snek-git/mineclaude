pub mod chest_screen;
pub mod common;
pub mod crafting_table_screen;
pub mod death_screen;
pub mod debug_map;
pub mod furnace_screen;
pub mod hotbar;
pub mod hud;
pub mod inventory_screen;
pub mod main_menu;
pub mod pause_menu;

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;

#[derive(Resource)]
pub struct UiAtlas {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

fn setup_ui_atlas(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let image = asset_server.load("textures/atlas.png");
    let layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16), 16, 16, None, None,
    ));
    commands.insert_resource(UiAtlas { image, layout });
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .init_resource::<hotbar::HotbarState>()
            .init_resource::<inventory_screen::InventoryOpen>()
            .init_resource::<inventory_screen::CursorItem>()
            .init_resource::<pause_menu::PauseState>()
            .init_resource::<main_menu::InMainMenu>()
            .add_systems(Startup, (
                setup_ui_atlas,
                main_menu::setup_main_menu,
            ))
            .add_systems(
                Update,
                (
                    main_menu::main_menu_interaction,
                    main_menu::cleanup_main_menu,
                    main_menu::main_menu_button_hover,
                ),
            )
            .add_systems(
                Startup,
                (
                    hud::spawn_crosshair,
                    hud::spawn_debug_text,
                    hud::spawn_health_bar,
                    hud::spawn_air_bar,
                    hud::spawn_hunger_bar,
                    hud::spawn_armor_bar,
                    hotbar::spawn_hotbar.after(setup_ui_atlas),
                ),
            )
            .add_systems(
                Update,
                (
                    hud::update_debug_text,
                    hud::update_health_bar,
                    hud::update_air_bar,
                    hud::update_hunger_bar,
                    hud::update_armor_bar,
                    hotbar::hotbar_input,
                    hotbar::sync_hotbar_from_inventory,
                    hotbar::update_hotbar_visuals
                        .after(hotbar::hotbar_input)
                        .after(hotbar::sync_hotbar_from_inventory),
                    hotbar::update_item_name.after(hotbar::hotbar_input),
                    inventory_screen::toggle_inventory,
                    inventory_screen::spawn_inventory_ui
                        .after(inventory_screen::toggle_inventory),
                    inventory_screen::despawn_inventory_ui
                        .after(inventory_screen::toggle_inventory),
                    inventory_screen::inventory_slot_interaction,
                    inventory_screen::crafting_slot_interaction,
                    inventory_screen::crafting_output_interaction,
                    inventory_screen::armor_slot_interaction,
                    inventory_screen::update_inventory_ui,
                    inventory_screen::update_armor_slots_ui,
                    inventory_screen::update_cursor_item_display,
                ),
            )
            .add_systems(
                Update,
                (
                    pause_menu::toggle_pause,
                    pause_menu::spawn_pause_ui.after(pause_menu::toggle_pause),
                    pause_menu::despawn_pause_ui.after(pause_menu::toggle_pause),
                    pause_menu::pause_button_interaction,
                    pause_menu::pause_button_hover,
                ),
            )
            .add_systems(
                Update,
                (
                    furnace_screen::toggle_furnace,
                    furnace_screen::spawn_furnace_ui
                        .after(furnace_screen::toggle_furnace),
                    furnace_screen::despawn_furnace_ui
                        .after(furnace_screen::toggle_furnace),
                    furnace_screen::furnace_input_interaction,
                    furnace_screen::furnace_fuel_interaction,
                    furnace_screen::furnace_output_interaction,
                    furnace_screen::furnace_inv_slot_interaction,
                    furnace_screen::update_furnace_ui,
                ),
            )
            .add_systems(
                Update,
                (
                    crafting_table_screen::toggle_crafting_table,
                    crafting_table_screen::spawn_crafting_table_ui
                        .after(crafting_table_screen::toggle_crafting_table),
                    crafting_table_screen::despawn_crafting_table_ui
                        .after(crafting_table_screen::toggle_crafting_table),
                    crafting_table_screen::crafting_table_slot_interaction,
                    crafting_table_screen::crafting_table_output_interaction,
                    crafting_table_screen::crafting_table_inv_slot_interaction,
                    crafting_table_screen::update_crafting_table_ui,
                ),
            )
            .add_systems(
                Update,
                (
                    chest_screen::toggle_chest,
                    chest_screen::spawn_chest_ui
                        .after(chest_screen::toggle_chest),
                    chest_screen::despawn_chest_ui
                        .after(chest_screen::toggle_chest),
                    chest_screen::chest_slot_interaction,
                    chest_screen::chest_inv_slot_interaction,
                    chest_screen::update_chest_ui,
                ),
            )
            .init_resource::<death_screen::PlayerDead>()
            .add_systems(
                Update,
                (
                    death_screen::spawn_death_screen,
                    death_screen::despawn_death_screen,
                    death_screen::respawn_button_interaction,
                    death_screen::respawn_button_hover,
                ),
            )
            .init_resource::<debug_map::DebugMapState>()
            .add_systems(
                Update,
                (
                    debug_map::toggle_debug_map,
                    debug_map::debug_map_input
                        .after(debug_map::toggle_debug_map),
                    debug_map::spawn_debug_map_ui
                        .after(debug_map::toggle_debug_map),
                    debug_map::despawn_debug_map_ui
                        .after(debug_map::toggle_debug_map),
                    debug_map::regenerate_debug_map
                        .after(debug_map::debug_map_input)
                        .after(debug_map::spawn_debug_map_ui),
                    debug_map::update_debug_map_text
                        .after(debug_map::regenerate_debug_map),
                ),
            );
    }
}
