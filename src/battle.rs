#[allow(unused_imports)]
use bevy::{prelude::*, ui::*};
use iyes_loopless::prelude::*;
use crate::{GameState};
//use std::net::{UdpSocket, SocketAddr, Ipv4Addr, IpAddr};
use crate::backgrounds::Tile;
use crate::camera::{MainCamera, MenuCamera, SlidesCamera};
use crate::player::Player;

const BATTLE_BACKGROUND: &str = "backgrounds/battlescreen_desert_1.png";
const ENEMY_MONSTER: &str = "monsters/clean_monster.png";
const MONSTER: &str = "monsters/stickdude.png";

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.75, 0.35, 0.35);


#[derive(Component)]
pub(crate) struct BattleBackground;

#[derive(Component)]
pub(crate) struct Monster;

#[derive(Component)]
pub(crate) struct PlayerMonster;

#[derive(Component)]
pub (crate) struct EnemyMonster;

// Unit structs to help identify the specific UI components for player's or enemy's monster health/level
// since there may be many Text components
#[derive(Component)]
struct PlayerHealth;

#[derive(Component)]
struct EnemyHealth;

#[derive(Component)]
struct PlayerLevel;

#[derive(Component)]
struct EnemyLevel;

#[derive(Component)]
pub(crate) struct AbortButton;

#[derive(Component)]
pub(crate) struct AttackButton;

#[derive(Component)]
pub(crate) struct DefendButton;

#[derive(Component)]
pub(crate) struct BattleUIElement;

struct UiAssets{
	font: Handle<Font>,
	button: Handle<Image>,
	button_pressed: Handle<Image>,
}

pub(crate) struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_enter_system_set(GameState::Battle, 
                SystemSet::new()
                    .with_system(setup_battle)
                    .with_system(battle_stats)
                    .with_system(abort_button)
                    .with_system(attack_button)
                    .with_system(defend_button)
                    .with_system(spawn_player_monster)
                    .with_system(spawn_enemy_monster)
                )
            .add_system_set(ConditionSet::new()
                // Run these systems only when in Battle state
                .run_in_state(GameState::Battle)
                    // addl systems go here
                    .with_system(abort_button_handler)
                    .with_system(attack_button_handler)
                    .with_system(defend_button_handler)
                    
                .into())
            .add_exit_system(GameState::Battle, despawn_battle)
            .add_enter_system_set(GameState::HostBattle, 
                SystemSet::new()
                    .with_system(setup_battle)
                    .with_system(battle_stats)
                    .with_system(abort_button)
                    .with_system(attack_button)
                    .with_system(defend_button)
                    .with_system(spawn_player_monster)
                    .with_system(spawn_enemy_monster)
                )
            .add_system_set(ConditionSet::new()
                // Run these systems only when in Battle state
                .run_in_state(GameState::HostBattle)
                    // addl systems go here
                    .with_system(mult_abort_handler)
                    .with_system(attack_button_handler)
                    .with_system(defend_button_handler)
                .into())
            .add_enter_system(GameState::PreHost, pre_host)
            .add_enter_system(GameState::PrePeer, pre_peer)
            .add_exit_system(GameState::HostBattle, despawn_battle)
            .add_enter_system_set(GameState::PeerBattle, 
                SystemSet::new()
                    .with_system(setup_battle)
                    .with_system(battle_stats)
                    .with_system(abort_button)
                    .with_system(attack_button)
                    .with_system(defend_button)
                    .with_system(spawn_player_monster)
                    .with_system(spawn_enemy_monster)
                )
            .add_system_set(ConditionSet::new()
                // Run these systems only when in Battle state
                .run_in_state(GameState::PeerBattle)
                    // addl systems go here
                    .with_system(mult_abort_handler)
                    .with_system(attack_button_handler)
                    .with_system(defend_button_handler)
                .into())
            .add_exit_system(GameState::PeerBattle, despawn_battle);
    }
}

pub(crate) fn pre_host(mut commands: Commands){
    let camera = Camera2dBundle::default();
    commands.spawn_bundle(camera).insert(MainCamera);
    commands.insert_resource(NextState(GameState::HostBattle));
}

pub(crate) fn pre_peer(mut commands: Commands){
    let camera = Camera2dBundle::default();
    commands.spawn_bundle(camera).insert(MainCamera);
    commands.insert_resource(NextState(GameState::PeerBattle));
}

pub(crate) fn setup_battle(mut commands: Commands,
                           asset_server: Res<AssetServer>,
                           cameras: Query<(&Transform, Entity), (With<Camera2d>, Without<MenuCamera>, Without<SlidesCamera>,
                            Without<Player>, Without<Tile>)>
) { 
    //let temp 
    if cameras.is_empty() {
        // error!("No spawned camera...?");
    } else{

    }
    let (ct, _) = cameras.single();

    // Backgrounds overlayed on top of the game world (to prevent the background
    // from being despawned and needing regenerated by WFC).
    // Main background is on -1, so layer this at 0.
    // Monsters can be layered at 1. and buttons/other UI can be 2.
    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load(BATTLE_BACKGROUND),
        transform: Transform::from_xyz(ct.translation.x, ct.translation.y, 0.),
        ..default()
    })
        .insert(BattleBackground);
}

pub(crate) fn battle_stats(mut commands: Commands, asset_server: Res<AssetServer>) 
{
    commands.spawn_bundle(
            // Create a TextBundle that has a Text with a list of sections.
            TextBundle::from_sections([
                // health header for player's monster
                TextSection::new(
                    "Health:",
                    TextStyle {
                        font: asset_server.load("buttons/joystix monospace.ttf"),
                        font_size: 40.0,
                        color: Color::BLACK,
                    },
                ),
                // health of player's monster
                TextSection::new(
                    "10",
                    TextStyle {
                        font: asset_server.load("buttons/joystix monospace.ttf"),
                        font_size: 40.0,
                        color: Color::BLACK,
                    },
                )
            ])
            .with_style(Style {
                    align_self: AlignSelf::FlexEnd,
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        top: Val::Px(5.0),
                        left: Val::Px(15.0),
                        ..default()
                    },
                    ..default()
                },
            ),
        )
        .insert(PlayerHealth)
        .insert(BattleUIElement);

        commands.spawn_bundle(
            // Create a TextBundle that has a Text with a list of sections.
            TextBundle::from_sections([
                // level header for player's monster
                TextSection::new(
                    "Level:",
                    TextStyle {
                        font: asset_server.load("buttons/joystix monospace.ttf"),
                        font_size: 40.0,
                        color: Color::BLACK,
                    },
                ),
                // level of player's monster
                TextSection::new(
                    "1",
                    TextStyle {
                        font: asset_server.load("buttons/joystix monospace.ttf"),
                        font_size: 40.0,
                        color: Color::BLACK,
                    },
                )
            ])
            .with_style(Style {
                    align_self: AlignSelf::FlexEnd,
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        top: Val::Px(40.0),
                        left: Val::Px(15.0),
                        ..default()
                    },
                    ..default()
                },
            ),
        )
        .insert(PlayerLevel)
        .insert(BattleUIElement);

        commands.spawn_bundle(
            // Create a TextBundle that has a Text with a list of sections.
            TextBundle::from_sections([
                // health header for enemy's monster
                TextSection::new(
                    "Health:",
                    TextStyle {
                        font: asset_server.load("buttons/joystix monospace.ttf"),
                        font_size: 40.0,
                        color: Color::BLACK,
                    },
                ),
                // health of enemy's monster
                TextSection::new(
                    "20",
                    TextStyle {
                        font: asset_server.load("buttons/joystix monospace.ttf"),
                        font_size: 40.0,
                        color: Color::BLACK,
                    },
                )
            ])
            .with_style(Style {
                    align_self: AlignSelf::FlexEnd,
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        top: Val::Px(5.0),
                        right: Val::Px(15.0),
                        ..default()
                    },
                    ..default()
                },
            ),
        )
        .insert(EnemyHealth)
        .insert(BattleUIElement);

        commands.spawn_bundle(
            // Create a TextBundle that has a Text with a list of sections.
            TextBundle::from_sections([
                // level header for player's monster
                TextSection::new(
                    "Level:",
                    TextStyle {
                        font: asset_server.load("buttons/joystix monospace.ttf"),
                        font_size: 40.0,
                        color: Color::BLACK,
                    },
                ),
                // level of player's monster
                TextSection::new(
                    "1",
                    TextStyle {
                        font: asset_server.load("buttons/joystix monospace.ttf"),
                        font_size: 40.0,
                        color: Color::BLACK,
                    },
                )
            ])
            .with_style(Style {
                    align_self: AlignSelf::FlexEnd,
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        top: Val::Px(40.0),
                        right: Val::Px(15.0),
                        ..default()
                    },
                    ..default()
                },
            ),
        )
        .insert(EnemyLevel)
        .insert(BattleUIElement);

}

pub(crate) fn spawn_player_monster(mut commands: Commands, 
    asset_server: Res<AssetServer>,
    cameras: Query<(&Transform, Entity), (With<Camera2d>, Without<MenuCamera>, Without<SlidesCamera>)>,
) {
    if cameras.is_empty() 
    {
        error!("No spawned camera...?");
    }

      let (ct, _) = cameras.single();

      commands.spawn_bundle(
        SpriteBundle {
        texture: asset_server.load(MONSTER),
        transform: Transform::from_xyz(ct.translation.x - 400., ct.translation.y - 100., 1.), 
        ..default()
    })
        .insert(PlayerMonster)
        .insert(Monster);

}


pub(crate) fn spawn_enemy_monster(mut commands: Commands,
    asset_server: Res<AssetServer>,
    cameras: Query<(&Transform, Entity), (With<Camera2d>, Without<MenuCamera>, Without<SlidesCamera>)>
) {

    if cameras.is_empty() 
    {
        error!("No spawned camera...?");
    }

    let (ct, _) = cameras.single();

    commands.spawn_bundle(
        SpriteBundle {
        texture: asset_server.load(ENEMY_MONSTER),
        transform: Transform::from_xyz(ct.translation.x + 400., ct.translation.y - 100., 1.), 
        ..default()
    })
        .insert(EnemyMonster)
        .insert(Monster);
}

// handles abort button for multplayer battles 
pub (crate) fn mult_abort_handler (
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &Children),
        (Changed<Interaction>, With<AbortButton>),
    >,
    mut text_query: Query<&mut Text>,
    mut commands: Commands
) {

    for (interaction, mut color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(*children.iter().next().unwrap()).unwrap();
        match *interaction {
            Interaction::Clicked => {
                text.sections[0].value = "Abort".to_string();
                *color = PRESSED_BUTTON.into();
                commands.insert_resource(NextState(GameState::Start));
            }
            Interaction::Hovered => {
                text.sections[0].value = "Abort".to_string();
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                text.sections[0].value = "Abort".to_string();
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

pub (crate) fn abort_button_handler(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &Children),
        (Changed<Interaction>, With<AbortButton>),
    >,
    mut text_query: Query<&mut Text>,
    mut commands: Commands
) {

    for (interaction, mut color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(*children.iter().next().unwrap()).unwrap();
        match *interaction {
            Interaction::Clicked => {
                text.sections[0].value = "Abort".to_string();
                *color = PRESSED_BUTTON.into();
                // This is gonna cause us problems as is, until we modify
                // states so that the initial transition from Start -> StartPlaying (a new state)
                // is the only one that spawns the world. In this paradigm,
                // it will regenerate the whole world as if it just started.
                commands.insert_resource(NextState(GameState::Playing));
            }
            Interaction::Hovered => {
                text.sections[0].value = "Abort".to_string();
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                text.sections[0].value = "Abort".to_string();
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

pub (crate) fn attack_button_handler (
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &Children),
        (Changed<Interaction>, With<AttackButton>),
    >,
    mut text_query: Query<&mut Text>,
    mut commands: Commands
) {

    for (interaction, mut color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(*children.iter().next().unwrap()).unwrap();
        match *interaction {
            Interaction::Clicked => {
                text.sections[0].value = "Attack".to_string();
                *color = PRESSED_BUTTON.into();

                // Attack button handler -> send msg to other player when pressed
                
                
            }
            Interaction::Hovered => {
                text.sections[0].value = "Attack".to_string();
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                text.sections[0].value = "Attack".to_string();
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

pub (crate) fn defend_button_handler (
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &Children),
        (Changed<Interaction>, With<DefendButton>),
    >,
    mut text_query: Query<&mut Text>,
    mut commands: Commands
) {

    for (interaction, mut color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(*children.iter().next().unwrap()).unwrap();
        match *interaction {
            Interaction::Clicked => {
                text.sections[0].value = "Defend".to_string();
                *color = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => {
                text.sections[0].value = "Defend".to_string();
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                text.sections[0].value = "Defend".to_string();
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

pub(crate) fn abort_button(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(175.0), Val::Px(65.0)),
                // center button
                margin: UiRect::all(Val::Auto),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                position: UiRect {
                    bottom: Val::Px(100.0),
                    left: Val::Px(100.0),
                    ..default()
                },
                ..default()
            },
            color: NORMAL_BUTTON.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                "Abort",
                TextStyle {
                    font: asset_server.load("buttons/joystix monospace.ttf"),
                    font_size: 40.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            ));
        })
        .insert(AbortButton)
        .insert(BattleUIElement);
}

pub(crate) fn attack_button(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(175.0), Val::Px(65.0)),
                // center button
                margin: UiRect::all(Val::Auto),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                position: UiRect {
                    bottom: Val::Px(100.0),
                    left: Val::Px(325.0),
                    ..default()
                },
                ..default()
            },
            color: NORMAL_BUTTON.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                "Attack",
                TextStyle {
                    font: asset_server.load("buttons/joystix monospace.ttf"),
                    font_size: 40.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            ));
        })
        .insert(AttackButton)
        .insert(BattleUIElement);
}

pub(crate) fn defend_button(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(175.0), Val::Px(65.0)),
                // center button
                margin: UiRect::all(Val::Auto),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                position: UiRect {
                    bottom: Val::Px(100.0),
                    left: Val::Px(550.0),
                    ..default()
                },
                ..default()
            },
            color: NORMAL_BUTTON.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                "Defend",
                TextStyle {
                    font: asset_server.load("buttons/joystix monospace.ttf"),
                    font_size: 40.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            ));
        })
        .insert(DefendButton)
        .insert(BattleUIElement);
}

pub(crate) fn despawn_battle(mut commands: Commands,
    background_query: Query<Entity, With<BattleBackground>>,
    monster_query: Query<Entity, With<Monster>>,
    battle_ui_element_query: Query<Entity, With<BattleUIElement>>
) {
    if background_query.is_empty() 
    {
        error!("background is here!");
    }

   background_query.for_each(|background| {
        commands.entity(background).despawn();
   });

   if monster_query.is_empty() 
   {
        error!("monsters are here!");
   }

   monster_query.for_each(|monster| {
        commands.entity(monster).despawn();
   });


   if battle_ui_element_query.is_empty() 
    {
    error!("ui elements are here!");
    }

   battle_ui_element_query.for_each(|battle_ui_element| {
        commands.entity(battle_ui_element).despawn_recursive();
   });

}