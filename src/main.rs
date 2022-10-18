use bevy::{prelude::*, 
	window::PresentMode,
};
// Please see https://github.com/IyesGames/iyes_loopless#states
// to learn about this. It's a much more simple way of managing states.
use iyes_loopless::prelude::*;
use std::convert::From;


// GAMEWIDE CONSTANTS
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub (crate) enum GameState{
	Start,
	Pause,
	Playing,
    Credits,
}


pub(crate) const TITLE: &str = "Waste";
// END GAMEWIDE CONSTANTS

// CUSTOM MODULE DEFINITIONS AND IMPORTS
//mod statements:
mod credits;
mod backgrounds;
mod player;
mod camera;
mod start_menu;
mod wfc;

//use statements:
use credits::*;
use backgrounds::*;
use player::*;
use camera::*;
use start_menu::*;
use wfc::*;

// END CUSTOM MODULES


fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: String::from(TITLE),
            width: WIN_W,
            height: WIN_H,
            present_mode: PresentMode::Fifo,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        // Starts game at main menu
        // Initial state should be "loopless"
		.add_loopless_state(GameState::Start)
		.add_plugin(MainMenuPlugin)
    .add_plugin(CreditsPlugin) 
    .add_enter_system_set(GameState::Playing, 
        // This system set is unconditional, as it is being added in an enter helper
        SystemSet::new()
            .with_system(init_background)
            .with_system(setup_game)
    )
    .add_system_set(ConditionSet::new()
        // These systems will only run in the condition that the game is in the state
        // Playing
        .run_in_state(GameState::Playing)
            .with_system(move_player)
            .with_system(move_camera)
            .with_system(animate_sprite))
        .add_system_set(SystemSet::on_exit(GameState::Playing)
            .with_system(despawn_game))
        .add_system(bevy::window::close_on_esc)
        // .add_system(move_player)
        // .add_system(move_camera)
        .run();
}

pub(crate) fn setup_game(mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    cameras: Query<Entity, (With<Camera2d>, Without<MainCamera>, Without<Player>, Without<Background>)>,
) {
    // Despawn other cameras
    cameras.for_each(|camera| {
        commands.entity(camera).despawn();
    });

    // done so that this camera doesn't mess with any UI cameras for start or pause menu
	let camera = Camera2dBundle::default();
    commands.spawn_bundle(camera).insert(MainCamera);

    let texture_handle = asset_server.load("characters/sprite_movement.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(64.0, 64.0), 4, 4);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    // Draw the player
    // He's so smol right now
    commands
        .spawn_bundle(SpriteSheetBundle { 
            //texture: asset_server.load(PLAYER_SPRITE),
            texture_atlas: texture_atlas_handle,
            transform: Transform::from_xyz(0., 0., 0.),
            ..default()
        })
        // Was considering giving player marker struct an xyz component
        // til I realized transform handles that for us.
        .insert(AnimationTimer(Timer::from_seconds(ANIM_TIME, true)))
        .insert(Player);

}



pub(crate) fn despawn_game(mut commands: Commands,
	camera_query: Query<Entity,  With<MainCamera>>,
    background_query: Query<Entity, With<Background>>,
    player_query: Query<Entity, With<Player>>,
) {
    // Despawn main camera
    camera_query.for_each(|camera| {
        commands.entity(camera).despawn();
    });

    // Despawn world
    background_query.for_each(|background| {
        commands.entity(background).despawn();
    });

    // Despawn player
    player_query.for_each(|player| {
        commands.entity(player).despawn();
    });

}
