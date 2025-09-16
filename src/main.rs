use bevy::prelude::*;
use avian2d::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugins::default())
        .add_systems(Startup, setup)
        .add_systems(Update, player_move_system)
        .add_event::<AttacEvent>()
        .add_systems(Update, 
            (enemy_attack_system, handle_attack_events, toward_player_system))
        .add_systems(Update, lifebar_system)
        .add_plugins(EguiPlugin::default())
        .add_systems(EguiPrimaryContextPass, ui_system)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    
    //プレイヤー
    commands.spawn((
        Sprite{
            color: Color::srgb(0.25, 0.75, 0.25),
            custom_size: Some(Vec2::new(30.0, 30.0)),
            ..Default::default()
        },
        Player,
        RigidBody::Kinematic,
        Collider::rectangle(30.0, 30.0),
        CollisionEventsEnabled,
        Health { current: 100, max: 100 },
        children![
            (LifebarBase,
                Sprite {
                    color: Srgba::new(0.0, 0.0, 0.0, 0.8).into(),
                    custom_size: Some(Vec2::new(30.0, 5.0)),
                    ..Default::default()
                },
                Transform::from_translation(Vec3::new(0.0, 20.0, 0.0)),),
            (LifebarFill,
                Sprite {
                    color: Srgba::new(0.0, 1.0, 0.0, 0.8).into(),
                    custom_size: Some(Vec2::new(30.0, 5.0)),
                    ..Default::default()
                },
                Transform::from_translation(Vec3::new(0.0, 20.0, 0.1)),),
        ],
    ));
    //敵
    commands.spawn((
        Sprite{
            color: Color::srgb(0.75, 0.25, 0.25),
            custom_size: Some(Vec2::new(30.0, 30.0)),
            ..Default::default()
        },
        Transform::from_translation(Vec3::new(100.0, 0.0, 0.0)),
        Enemy,
        TowardPlayer { speed: 50.0 },
        RigidBody::Kinematic,
        Collider::rectangle(30.0, 30.0),
        Health { current: 50, max: 50 },
        Attack { damage: 10 },
    )); 
}

#[derive(Component)]
struct Player;

fn player_move_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    let mut direction = Vec3::ZERO;
    if keyboard_input.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    const MOVE_SPEED:f32 = 100.0;
    if direction != Vec3::ZERO {
        direction = direction.normalize() * MOVE_SPEED * time.delta_secs(); // Move speed
        for mut transform in player_query.iter_mut() {
            transform.translation += direction;
        }
    }
}

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct Health {
    current: i32,
    max: i32,
}

#[derive(Component)]
struct LifebarBase;

#[derive(Component)]
struct LifebarFill;

#[derive(Component)]
struct Attack {
    damage: i32,
}

#[derive(Event)]
struct AttacEvent{
    attacker: Entity,
    target: Entity,
}

// write attack event when enemy collides with player
fn enemy_attack_system(
    mut collisions: EventReader<CollisionStarted>,
    mut attack_events: EventWriter<AttacEvent>,
    player_query: Query<Entity, With<Player>>,
    enemy_query: Query<Entity, With<Enemy>>,
) {
    for CollisionStarted(entity1, entity2) in collisions.read(){
        let entity1_is_player = player_query.get(*entity1).is_ok();
        let entity2_is_player = player_query.get(*entity2).is_ok();
        let entity1_is_enemy = enemy_query.get(*entity1).is_ok();
        let entity2_is_enemy = enemy_query.get(*entity2).is_ok();

        if (entity1_is_player && entity2_is_enemy) || (entity1_is_enemy && entity2_is_player) {
            let (attacker, target) = if entity1_is_enemy {
                (*entity1, *entity2)
            } else {
                (*entity2, *entity1)
            };
            attack_events.write(AttacEvent { attacker, target });
        }
    }
}

#[derive(Component)]
struct TowardPlayer{
    speed: f32,
}

fn toward_player_system(
    mut enemy_query: Query<(&mut Transform, &TowardPlayer), Without<Player>>,
    player_query: Query<&Transform, (With<Player>, Without<TowardPlayer>)>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_query.single() {
        for (mut enemy_transform, toward_player) in enemy_query.iter_mut() {
            let direction = (player_transform.translation - enemy_transform.translation).normalize();
            enemy_transform.translation += direction * toward_player.speed * time.delta_secs();
        }
    }
}

fn handle_attack_events(
    mut attack_events: EventReader<AttacEvent>,
    mut health_query: Query<&mut Health>,
    attack_query: Query<&Attack>,
) {
    for event in attack_events.read() {

        if let Ok(mut health) = health_query.get_mut(event.target) {
            if let Ok(attack) = attack_query.get(event.attacker) {
                health.current -= attack.damage;
                println!("Entity {:?} attacked Entity {:?} for {} damage. Remaining health: {}", event.attacker, event.target, attack.damage, health.current);
                if health.current <= 0 {
                    println!("Entity {:?} has been defeated!", event.target);
                    // Here you might want to despawn the entity or trigger some death logic
                }
            }
        }
    }
}

fn lifebar_system(
    mut health_query: Query<(Entity, &Health), Changed<Health>>,
    mut lifebar_fill_query: Query<(&mut Sprite, &mut Transform ,&ChildOf), With<LifebarFill>>,
    //mut lifebar_base_query: Query<&Sprite, With<LifebarBase>>,
) {
    for (mut life_fill, mut transform, ChildOf(parent)) in lifebar_fill_query.iter_mut() {
        for (entity, health) in health_query.iter_mut() {
            if *parent != entity {
                continue;
            }
            let health_ratio = health.current as f32 / health.max as f32;
            life_fill.custom_size = Some(Vec2::new(30.0 * health_ratio, 5.0));
            transform.translation.x = -15.0 * (1.0 - health_ratio);
        }
    }
}


fn ui_system(
    mut contexts: EguiContexts,
    player_query: Query<&Health, With<Player>>,
) -> Result {
    egui::Window::new("Vampire Surviver").show(contexts.ctx_mut()?, |ui| {
        ui.label("Use WASD to move the player (green square).");
        if let Ok(health) = player_query.single() {
            ui.label(format!("Health: {}/{}", health.current, health.max));
        } else {
            ui.label("Player not found");
        }
    });
    Ok(())
}