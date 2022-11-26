use crate::backgrounds::Tile;
use crate::camera::{MenuCamera, SlidesCamera};
use crate::monster::{
    get_monster_sprite_for_type, Boss, Defense, Element, Enemy, Health, Level, MonsterStats,
    PartyMonster, SelectedMonster, Strength,
};
use crate::player::Player;
use crate::quests::*;
use crate::world::{GameProgress, TypeSystem};
use crate::GameState;
use bevy::prelude::*;
use iyes_loopless::prelude::*;
use rand::*;

const BATTLE_BACKGROUND: &str = "backgrounds/battlescreen_desert_1.png";

#[derive(Component)]
pub(crate) struct BattleBackground;

#[derive(Component)]
pub(crate) struct Monster;

#[derive(Component)]
pub(crate) struct PlayerMonster;

#[derive(Component)]
pub(crate) struct EnemyMonster;

// Unit structs to help identify the specific UI components for player's or enemy's monster health/level
// since there may be many Text components
#[derive(Component)]
pub(crate) struct PlayerHealth;

#[derive(Component)]
pub(crate) struct EnemyHealth;

#[derive(Component)]
pub(crate) struct PlayerLevel;

#[derive(Component)]
pub(crate) struct EnemyLevel;

#[derive(Component)]
pub(crate) struct BattleUIElement;

pub(crate) struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system_set(
            GameState::Battle,
            SystemSet::new()
                .with_system(setup_battle)
                .with_system(setup_battle_stats),
        )
        .add_system_set(
            ConditionSet::new()
                // Run these systems only when in Battle state
                .run_in_state(GameState::Battle)
                // addl systems go here
                .with_system(spawn_player_monster)
                .with_system(spawn_enemy_monster)
                .with_system(update_battle_stats)
                .with_system(key_press_handler)
                .into(),
        )
        .add_exit_system(GameState::Battle, despawn_battle);
    }
}

macro_rules! end_battle {
    ($commands:expr, $game_progress:expr, $my_monster:expr, $enemy_monster:expr) => {
        // remove the monster from the enemy stats
        $game_progress.enemy_stats.remove(&$enemy_monster);
        // reset selected monster back to the first one in our bag
        let first_monster = $game_progress.monster_id_entity.get(&0).unwrap().clone();
        $commands.entity($my_monster).remove::<SelectedMonster>();
        $commands.entity(first_monster).insert(SelectedMonster);
        // the battle is over, remove enemy from monster anyways
        $commands.entity($enemy_monster).remove::<Enemy>();
        $commands.insert_resource(NextState(GameState::Playing));
    };
}

macro_rules! monster_level_up {
    ($commands:expr, $game_progress:expr, $my_monster:expr, $up_by:expr) => {
        let mut stats = $game_progress
            .monster_entity_to_stats
            .get_mut(&$my_monster)
            .unwrap();
        stats.lvl.level += 1 * $up_by;
        stats.hp.max_health += 10 * $up_by;
        stats.hp.health = stats.hp.max_health as isize;
        stats.stg.atk += 2 * $up_by;
        stats.stg.crt += 5 * $up_by;
        stats.def.def += 1 * $up_by;
        // we have to remove the old stats and add the new one
        // because we cannot change the stats in place
        $commands.entity($my_monster).remove::<MonsterStats>();
        $commands.entity($my_monster).insert_bundle(stats.clone());
    };
}

pub(crate) fn setup_battle(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    cameras: Query<
        (&Transform, Entity),
        (
            With<Camera2d>,
            Without<MenuCamera>,
            Without<SlidesCamera>,
            Without<Player>,
            Without<Tile>,
        ),
    >,
) {
    // what is this??
    if cameras.is_empty() {
        error!("No spawned camera...?");
        return;
    }
    let (ct, _) = cameras.single();

    // Backgrounds overlayed on top of the game world (to prevent the background
    // from being despawned and needing regenerated by WFC).
    // Main background is on -1, so layer this at 0.
    // Monsters can be layered at 1. and buttons/other UI can be 2.
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load(BATTLE_BACKGROUND),
            transform: Transform::from_xyz(ct.translation.x, ct.translation.y, 0.),
            ..default()
        })
        .insert(BattleBackground);
}

// -----------------------------------------------------------------------------------------------------------

pub(crate) fn setup_battle_stats(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut set: ParamSet<(
        Query<&mut Level, With<SelectedMonster>>,
        Query<&mut Level, With<Enemy>>,
    )>,
) {
    let mut my_lvl = 0;
    let mut enemy_lvl = 0;
    for my_monster in set.p0().iter_mut() {
        my_lvl = my_monster.level;
    }

    for enemy_monster in set.p1().iter_mut() {
        enemy_lvl = enemy_monster.level;
    }

    commands
        .spawn_bundle(
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
                TextSection::from_style(TextStyle {
                    font: asset_server.load("buttons/joystix monospace.ttf"),
                    font_size: 40.0,
                    color: Color::BLACK,
                }),
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
            }),
        )
        .insert(PlayerHealth)
        .insert(BattleUIElement);

    commands
        .spawn_bundle(
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
                    my_lvl.to_string(),
                    TextStyle {
                        font: asset_server.load("buttons/joystix monospace.ttf"),
                        font_size: 40.0,
                        color: Color::BLACK,
                    },
                ),
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
            }),
        )
        .insert(PlayerLevel)
        .insert(BattleUIElement);

    commands
        .spawn_bundle(
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
                TextSection::from_style(TextStyle {
                    font: asset_server.load("buttons/joystix monospace.ttf"),
                    font_size: 40.0,
                    color: Color::BLACK,
                }),
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
            }),
        )
        //.insert(MonsterBundle::default())
        .insert(EnemyHealth)
        .insert(BattleUIElement);

    commands
        .spawn_bundle(
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
                    enemy_lvl.to_string(),
                    TextStyle {
                        font: asset_server.load("buttons/joystix monospace.ttf"),
                        font_size: 40.0,
                        color: Color::BLACK,
                    },
                ),
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
            }),
        )
        .insert(EnemyLevel)
        .insert(BattleUIElement);
}

pub(crate) fn update_battle_stats(
    _commands: Commands,
    _asset_server: Res<AssetServer>,
    mut set: ParamSet<(
        Query<&mut Health, With<SelectedMonster>>,
        Query<&mut Health, With<Enemy>>,
    )>,
    mut enemy_health_text_query: Query<&mut Text, (With<EnemyHealth>, Without<PlayerHealth>)>,
    mut player_health_text_query: Query<&mut Text, (With<PlayerHealth>, Without<EnemyHealth>)>,
) {
    let mut my_health = 0;
    let mut enemy_health = 0;
    for my_monster in set.p0().iter_mut() {
        my_health = my_monster.health;
    }

    for enemy_monster in set.p1().iter_mut() {
        enemy_health = enemy_monster.health;
    }

    for mut text in &mut enemy_health_text_query {
        text.sections[1].value = format!("{}", enemy_health);
    }

    for mut text in &mut player_health_text_query {
        text.sections[1].value = format!("{}", my_health);
    }
}

pub(crate) fn spawn_player_monster(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    cameras: Query<
        (&Transform, Entity),
        (With<Camera2d>, Without<MenuCamera>, Without<SlidesCamera>),
    >,
    selected_monster_query: Query<(&Element, Entity), (With<SelectedMonster>, Without<Enemy>)>,
) {
    if cameras.is_empty() {
        error!("No spawned camera...?");
        return;
    }

    if selected_monster_query.is_empty() {
        error!("No selected monster...?");
        return;
    }

    let (ct, _) = cameras.single();

    // why doesn't this update
    let (selected_type, selected_monster) = selected_monster_query.single();

    commands
        .entity(selected_monster)
        .remove_bundle::<SpriteBundle>()
        .insert_bundle(SpriteBundle {
            sprite: Sprite {
                flip_y: false, // flips our little buddy, you guessed it, in the y direction
                flip_x: true,  // guess what this does
                ..default()
            },
            texture: asset_server.load(&get_monster_sprite_for_type(*selected_type)),
            transform: Transform::from_xyz(ct.translation.x - 400., ct.translation.y - 100., 1.),
            ..default()
        })
        .insert(PlayerMonster)
        .insert(Monster);
}

pub(crate) fn spawn_enemy_monster(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    cameras: Query<
        (&Transform, Entity),
        (With<Camera2d>, Without<MenuCamera>, Without<SlidesCamera>),
    >,
    selected_type_query: Query<&Element, (Without<SelectedMonster>, With<Enemy>)>,
) {
    if cameras.is_empty() {
        error!("No spawned camera...?");
        return;
    }

    if selected_type_query.is_empty() {
        error!("No selected monster...?");
        return;
    }

    let selected_type = selected_type_query.single();

    let (ct, _) = cameras.single();

    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load(&get_monster_sprite_for_type(*selected_type)),
            transform: Transform::from_xyz(ct.translation.x + 400., ct.translation.y - 100., 1.),
            ..default()
        })
        .insert(EnemyMonster)
        .insert(Monster);
    // .insert(monster_info.clone());
}

pub(crate) fn despawn_battle(
    mut commands: Commands,
    background_query: Query<Entity, With<BattleBackground>>,
    monster_query: Query<Entity, With<Monster>>,
    battle_ui_element_query: Query<Entity, With<BattleUIElement>>,
) {
    if background_query.is_empty() {
        error!("background is here!");
    }

    background_query.for_each(|background| {
        commands.entity(background).despawn();
    });

    if monster_query.is_empty() {
        error!("monsters are here!");
    }

    monster_query.for_each(|monster| {
        commands
            .entity(monster)
            .remove_bundle::<SpriteBundle>()
            .remove::<PlayerMonster>()
            .remove::<EnemyMonster>()
            .remove::<Monster>();
    });

    if battle_ui_element_query.is_empty() {
        error!("ui elements are here!");
    }

    battle_ui_element_query.for_each(|battle_ui_element| {
        commands.entity(battle_ui_element).despawn_recursive();
    });
}

/// Handler system to enact battle actions based on key presses
pub(crate) fn key_press_handler(
    input: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut game_progress: ResMut<GameProgress>,
    // placeholder for another resource dedicated to battle
    mut my_monster: Query<
        (&mut Health, &mut Strength, &mut Defense, Entity, &Element),
        (With<SelectedMonster>, Without<Enemy>),
    >,
    mut enemy_monster: Query<
        (
            &mut Health,
            &mut Strength,
            &mut Defense,
            Entity,
            Option<&Boss>,
            &Element,
        ),
        (Without<SelectedMonster>, With<Enemy>),
    >,
    mut party_monsters: Query<
        (&mut Health, &mut Strength, &mut Defense, Entity, &Element),
        (With<PartyMonster>, Without<SelectedMonster>, Without<Enemy>),
    >,
    type_system: Res<TypeSystem>,
    camera: Query<
        (&Transform, Entity),
        (With<Camera2d>, Without<MenuCamera>, Without<SlidesCamera>),
    >,
    asset_server: Res<AssetServer>,
) {
    if my_monster.is_empty() || enemy_monster.is_empty() {
        info!("Monsters are missing!");
        commands.insert_resource(NextState(GameState::Playing));
        return;
    }

    if camera.is_empty() {
        error!("No spawned camera?");
        commands.insert_resource(NextState(GameState::Playing));
        return;
    }

    let (transform, _) = camera.single();

    // Get player and enemy monster data out of the query
    let (mut player_health, mut player_stg, player_def, player_entity, player_type) =
        my_monster.single_mut();

    let (mut enemy_health, enemy_stg, enemy_def, enemy_entity, enemy_boss, enemy_type) =
        enemy_monster.single_mut();

    if player_health.health <= 0 {
        let next_monster = game_progress.next_monster_cyclic(player_entity);
        if next_monster.is_none() {
            info!("Your monster was defeated.");
            end_battle!(commands, game_progress, player_entity, enemy_entity);
        } else {
            info!("Your monster was defeated. Switching to next monster.");
            commands.entity(player_entity).remove::<SelectedMonster>();
            commands
                .entity(player_entity)
                .remove_bundle::<SpriteBundle>();
            commands.entity(player_entity).remove::<PlayerMonster>();
            commands.entity(player_entity).remove::<Monster>();
            commands
                .entity(*next_monster.unwrap())
                .insert(SelectedMonster);
        }
    }

    if input.just_pressed(KeyCode::A) {
        // ATTACK HANDLER
        // Actions:
        // 0: attack 1: defend: 2: elemental: 3: special
        let enemy_action = rand::thread_rng().gen_range(0..=3);
        info!("You attack!");

        if enemy_action == 0 {
            info!("Enemy attacks!")
        } else if enemy_action == 1 {
            info!("Enemy defends!")
        } else if enemy_action == 2 {
            info!("Enemy uses an elemental attack!")
        } else {
            info!("Enemy uses its special ability!")
        }

        let str_buff_damage = if game_progress.turns_left_of_buff[0] > 0 {
            info!("You will deal extra damage this turn.");
            game_progress.turns_left_of_buff[0] -= 1;
            game_progress.current_level
        } else {
            0
        };

        // Temporarily increase strength for the turn calculation
        player_stg.atk += str_buff_damage;
        let turn_result = calculate_turn(
            &player_stg,
            &player_def,
            player_type,
            0,
            &enemy_stg,
            &enemy_def,
            enemy_type,
            enemy_action,
            *type_system,
        );
        // Reset strength for next turn
        player_stg.atk -= str_buff_damage;

        player_health.health -= turn_result.1;
        enemy_health.health -= turn_result.0;

        if enemy_health.health <= 0 {
            info!("Enemy monster defeated. Your monsters will level up!");
            // at this point this monster is already "ours", we just need to register is with the resource
            // get the stats from the monster
            let mut new_monster_stats = *game_progress.enemy_stats.get(&enemy_entity).unwrap();
            // Clamp health down so we don't keep boss health
            new_monster_stats.hp.health = game_progress.current_level as isize * 10;
            new_monster_stats.hp.max_health = game_progress.current_level * 10;
            // remove the monster from the enemy stats
            game_progress.enemy_stats.remove(&enemy_entity);
            // add the monster to the monster bag
            commands.entity(enemy_entity).insert(PartyMonster);
            game_progress.new_monster(enemy_entity, new_monster_stats);
            // TODO: see the discrepancy between the type we see and the type we get
            info!(
                "new member type: {:?}",
                game_progress
                    .monster_entity_to_stats
                    .get(&enemy_entity)
                    .unwrap()
                    .typing
            );
            // update game progress
            // check for boss
            if enemy_boss.is_some() {
                info!("Boss defeated!");
                game_progress.get_quest_rewards(*enemy_type);
                game_progress.win_boss();
                // if boss level up twice
                for pm in party_monsters.iter_mut() {
                    monster_level_up!(commands, game_progress, pm.3, 1);
                }
                monster_level_up!(commands, game_progress, player_entity, 1);
                monster_level_up!(commands, game_progress, enemy_entity, 1);
                commands.entity(enemy_entity).remove::<Boss>();

                // Spawn an NPC if enemy_boss is some and we won
                let new_quest = Quest::random();
                info!("Someone appears in the dust!");
                commands
                    .spawn_bundle(SpriteBundle {
                        texture: asset_server.load(NPC_PATH),
                        transform: Transform::from_xyz(
                            transform.translation.x,
                            transform.translation.y,
                            0.,
                        ),
                        ..default()
                    })
                    .insert(NPC { quest: new_quest });
            } else {
                game_progress.win_battle();
                game_progress.get_quest_rewards(*enemy_type);
                // if not boss level up once
                for pm in party_monsters.iter_mut() {
                    monster_level_up!(commands, game_progress, pm.3, 1);
                }
                monster_level_up!(commands, game_progress, player_entity, 1);
                monster_level_up!(commands, game_progress, enemy_entity, 1);
            }
            end_battle!(commands, game_progress, player_entity, enemy_entity);
        } else if player_health.health <= 0 {
            game_progress.num_living_monsters -= 1;
            let next_monster = game_progress.next_monster_cyclic(player_entity);
            if next_monster.is_none() {
                info!("Your monster was defeated.");
                end_battle!(commands, game_progress, player_entity, enemy_entity);
            } else {
                info!("Your monster was defeated. Switching to next monster.");
                commands.entity(player_entity).remove::<SelectedMonster>();
                commands
                    .entity(player_entity)
                    .remove_bundle::<SpriteBundle>();
                commands.entity(player_entity).remove::<PlayerMonster>();
                commands.entity(player_entity).remove::<Monster>();
                commands
                    .entity(*next_monster.unwrap())
                    .insert(SelectedMonster);
            }
        }
    } else if input.just_pressed(KeyCode::E) {
        // ELEMENTAL ATTACK HANDLER
        // Actions:
        // 0: attack 1: defend: 2: elemental: 3: special
        let enemy_action = rand::thread_rng().gen_range(0..=3);
        info!("You use your type {:?} elemental attack!", player_type);

        if enemy_action == 0 {
            info!("Enemy attacks!")
        } else if enemy_action == 1 {
            info!("Enemy defends!")
        } else if enemy_action == 2 {
            info!("Enemy uses an elemental attack!")
        } else {
            info!("Enemy uses its special ability!")
        }

        let str_buff_damage = if game_progress.turns_left_of_buff[0] > 0 {
            info!("You will deal extra damage this turn.");
            game_progress.turns_left_of_buff[0] -= 1;
            game_progress.current_level
        } else {
            0
        };

        // Temporarily increase strength for the turn calculation
        player_stg.atk += str_buff_damage;
        let turn_result = calculate_turn(
            &player_stg,
            &player_def,
            player_type,
            2,
            &enemy_stg,
            &enemy_def,
            enemy_type,
            enemy_action,
            *type_system,
        );
        // Reset strength for next turn
        player_stg.atk -= str_buff_damage;

        player_health.health -= turn_result.1;
        enemy_health.health -= turn_result.0;

        if enemy_health.health <= 0 {
            info!("Enemy monster defeated. Your monsters will level up!");
            // at this point this monster is already "ours", we just need to register is with the resource
            // get the stats from the monster
            let mut new_monster_stats = *game_progress.enemy_stats.get(&enemy_entity).unwrap();
            // Clamp health down so we don't keep boss health
            new_monster_stats.hp.health = game_progress.current_level as isize * 10;
            new_monster_stats.hp.max_health = game_progress.current_level * 10;
            // remove the monster from the enemy stats
            game_progress.enemy_stats.remove(&enemy_entity);
            // add the monster to the monster bag
            commands.entity(enemy_entity).insert(PartyMonster);
            game_progress.new_monster(enemy_entity, new_monster_stats);
            // TODO: see the discrepancy between the type we see and the type we get
            info!(
                "new member type: {:?}",
                game_progress
                    .monster_entity_to_stats
                    .get(&enemy_entity)
                    .unwrap()
                    .typing
            );
            // update game progress
            // check for boss
            if enemy_boss.is_some() {
                info!("Boss defeated!");
                game_progress.win_boss();
                game_progress.get_quest_rewards(*enemy_type);
                // if boss level up twice
                for pm in party_monsters.iter_mut() {
                    monster_level_up!(commands, game_progress, pm.3, 1);
                }
                monster_level_up!(commands, game_progress, player_entity, 1);
                monster_level_up!(commands, game_progress, enemy_entity, 1);
                commands.entity(enemy_entity).remove::<Boss>();
                // Spawn an NPC if enemy_boss is some and we won
                let new_quest = Quest::random();
                info!("Someone appears in the dust!");
                commands
                    .spawn_bundle(SpriteBundle {
                        texture: asset_server.load(NPC_PATH),
                        transform: Transform::from_xyz(
                            transform.translation.x,
                            transform.translation.y,
                            0.,
                        ),
                        ..default()
                    })
                    .insert(NPC { quest: new_quest });
            } else {
                game_progress.win_battle();
                game_progress.get_quest_rewards(*enemy_type);
                // if not boss level up once
                for pm in party_monsters.iter_mut() {
                    monster_level_up!(commands, game_progress, pm.3, 1);
                }
                monster_level_up!(commands, game_progress, player_entity, 1);
                monster_level_up!(commands, game_progress, enemy_entity, 1);
            }
            end_battle!(commands, game_progress, player_entity, enemy_entity);
        } else if player_health.health <= 0 {
            game_progress.num_living_monsters -= 1;
            let next_monster = game_progress.next_monster_cyclic(player_entity);
            if next_monster.is_none() {
                info!("Your monster was defeated.");
                end_battle!(commands, game_progress, player_entity, enemy_entity);
            } else {
                info!("Your monster was defeated. Switching to next monster.");
                commands.entity(player_entity).remove::<SelectedMonster>();
                commands
                    .entity(player_entity)
                    .remove_bundle::<SpriteBundle>();
                commands.entity(player_entity).remove::<PlayerMonster>();
                commands.entity(player_entity).remove::<Monster>();
                commands
                    .entity(*next_monster.unwrap())
                    .insert(SelectedMonster);
            }
        }
    } else if input.just_pressed(KeyCode::Q) {
        // ABORT HANDLER
        commands.entity(enemy_entity).remove::<Enemy>();
        commands.insert_resource(NextState(GameState::Playing));
    } else if input.just_pressed(KeyCode::D) {
        // DEFEND HANDLER
    } else if input.just_pressed(KeyCode::C) {
        // CYCLE HANDLER
        if my_monster.is_empty() {
            error!("No monster spawned, cannot switch!");
            return;
        }
        // They want to cycle their monster
        let next_monster = game_progress.next_monster_cyclic(player_entity);
        if next_monster.is_none() {
            info!("No monster to cycle to.");
        } else {
            info!("Cycling to next monster in party.");
            commands.entity(player_entity).remove::<SelectedMonster>();
            commands
                .entity(player_entity)
                .remove_bundle::<SpriteBundle>();
            commands.entity(player_entity).remove::<PlayerMonster>();
            commands.entity(player_entity).remove::<Monster>();
            commands
                .entity(*next_monster.unwrap())
                .insert(SelectedMonster);
        }
    } else if input.just_pressed(KeyCode::Key1) {
        // USE HEAL ITEM HANDLER
        // Must first check that they have enough healing items
        if game_progress.player_inventory[0] > 0 {
            // Remove the item, it is used now
            game_progress.player_inventory[0] -= 1;

            // Calculate heal amount
            let heal_amount = (game_progress.current_level * 3) as isize;

            // Heal whole party
            for mut pm in party_monsters.iter_mut() {
                // Check if this is a resurrection
                if pm.0.health <= 0 {
                    game_progress.num_living_monsters += 1;
                }

                // Clamped heal
                if pm.0.health + heal_amount > pm.0.max_health as isize {
                    pm.0.health = pm.0.max_health as isize;
                } else {
                    pm.0.health += heal_amount;
                }
            }

            // Now heal selected monster
            if player_health.health + heal_amount > player_health.max_health as isize {
                player_health.health = player_health.max_health as isize;
            } else {
                player_health.health += heal_amount;
            }

            info!("{} health restored.", heal_amount);
        }
    } else if input.just_pressed(KeyCode::Key2) {
        // USE STRENGTH BUFF HANDLER
        // Check that we have a buff item
        if game_progress.player_inventory[1] > 0 {
            info!("You used a strength buff. The next five turns you will deal extra damage.");
            // Decrement
            game_progress.player_inventory[1] -= 1;
            // Make it so we have turns left of this buff
            game_progress.turns_left_of_buff[0] = 5;
        }
    }
}

fn calculate_turn(
    player_stg: &Strength,
    player_def: &Defense,
    player_type: &Element,
    player_action: usize,
    enemy_stg: &Strength,
    enemy_def: &Defense,
    enemy_type: &Element,
    enemy_action: usize,
    type_system: TypeSystem,
) -> (isize, isize) {
    if player_action == 1 || enemy_action == 1 {
        // if either side defends this turn will not have any damage on either side
        return (0, 0);
    }
    // More actions can be added later, we can also consider decoupling the actions from the damage
    let mut result = (
        0, // Your damage to enemy
        0, // Enemy's damage to you
    );
    // player attacks
    // If our attack is less than the enemy's defense, we do 0 damage
    if player_stg.atk <= enemy_def.def {
        result.0 = 0;
    } else {
        // if we have damage, we do that much damage
        // I've only implemented crits for now, dodge and element can follow
        result.0 = player_stg.atk - enemy_def.def;
        if player_stg.crt > enemy_def.crt_res {
            // calculate crit chance and apply crit damage
            let crit_chance = player_stg.crt - enemy_def.crt_res;
            let crit = rand::thread_rng().gen_range(0..=100);
            if crit <= crit_chance {
                info!("You had a critical strike!");
                result.0 *= player_stg.crt_dmg;
            }
        }
    }
    // same for enemy
    if enemy_stg.atk <= player_def.def {
        result.1 = 0;
    } else {
        result.1 = enemy_stg.atk - player_def.def;
        if enemy_stg.crt > player_def.crt_res {
            let crit_chance = enemy_stg.crt - player_def.crt_res;
            let crit = rand::thread_rng().gen_range(0..=100);
            if crit <= crit_chance {
                info!("Enemy had a critical strike!");
                result.1 *= enemy_stg.crt_dmg;
            }
        }
    }

    if player_action == 2 {
        result.0 = (type_system.type_modifier[*player_type as usize][*enemy_type as usize]
            * result.0 as f32)
            .trunc() as usize;
    }

    if enemy_action == 2 {
        result.1 = (type_system.type_modifier[*enemy_type as usize][*player_type as usize]
            * result.1 as f32)
            .trunc() as usize;
    }

    info!(
        "Player deals {} damage, enemy deals {} damage...",
        result.0, result.1
    );

    (result.0 as isize, result.1 as isize)
}
