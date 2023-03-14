use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_editor_pls::prelude::*;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_rapier2d::prelude::*;

const WINDOW_WIDTH: f32 = 1080.0;
const WINDOW_HEIGHT: f32 = 720.0;

const PLAYER_SIZE_HALF: f32 = PLAYER_SIZE / 2.0;
const PLAYER_SIZE: f32 = 50.0;
const PLAYER_STARTING_POSITION: Vec3 =
    Vec3::new(((WINDOW_WIDTH / 2.0) * -1.0) + 100.0, -100.0, 0.0);

const GROUND_SIZE_HALF_X: f32 = WINDOW_WIDTH / 2.0;
const GROUND_SIZE_HALF_Y: f32 = 25.0;
const GROUND_SIZE: Vec2 = Vec2 {
    x: WINDOW_WIDTH,
    y: 50.0,
};

const ENEMY_SIZE: f32 = 25.0;
const ENEMY_SIZE_HALF: f32 = ENEMY_SIZE / 2.0;
const ENEMY_STARTING_POSITION: Vec3 = Vec3::new(200.0, -300.0, 0.0);
const JUMP_SIZE: f32 = 300.0;
#[derive(Component)]
struct Ground;
#[derive(Component)]
struct Enemy;
#[derive(Component, Reflect, Resource, Default, InspectorOptions, Debug)]
#[reflect(Resource, InspectorOptions)]
struct Configuration {
    jump_speed: f32,
    speed: f32,
    clamp: f32,
    friction: f32,
    restart: bool,
    enemy: bool,
}
#[derive(Component)]
struct Player;
#[derive(Component, Deref, DerefMut, Debug, Default)]
struct Velocity(Vec2);

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "MARIO".into(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(startup)
        .add_system(enemy_move)
        .add_system(enemy_collision)
        .add_system(camerman)
        .add_system(block_left_move)
        .add_system(player_kinematics)
        .add_plugin(EditorPlugin)
        .init_resource::<Configuration>()
        .insert_resource(Configuration {
            jump_speed: 130.00,
            speed: 30.0,
            friction: 0.9,
            clamp: 500.0,
            restart: false,
            enemy: true,
        })
        .insert_resource(RapierConfiguration {
            gravity: Vect::Y * -9.81 * 20.0,
            ..Default::default()
        })
        .add_system(config)
        .add_plugin(ResourceInspectorPlugin::<Configuration>::default())
        .run();
}

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((Name::new("Camera"), Camera2dBundle::default()));

    for n in 0..10 {
        // Ground
        commands.spawn((
            Name::new("Ground"),
            MaterialMesh2dBundle {
                mesh: meshes
                    .add(
                        shape::Quad {
                            size: GROUND_SIZE,
                            flip: false,
                        }
                        .into(),
                    )
                    .into(),
                material: materials.add(ColorMaterial::from(Color::BLUE)),
                transform: Transform::from_translation(Vec3::new(
                    (WINDOW_WIDTH + JUMP_SIZE) * n as f32,
                    (WINDOW_HEIGHT / 2.0 - 25.0) * -1.0,
                    5.0,
                )),
                ..default()
            },
            RigidBody::Fixed,
            Collider::cuboid(GROUND_SIZE_HALF_X, GROUND_SIZE_HALF_Y),
            Dominance::group(50),
            Ground,
        ));
    }

    // Enemy
    commands.spawn((
        Name::new("Enemy"),
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Cube { size: ENEMY_SIZE }.into()).into(),
            material: materials.add(ColorMaterial::from(Color::BLACK)),
            transform: Transform::from_translation(ENEMY_STARTING_POSITION),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::cuboid(ENEMY_SIZE_HALF - 0.1, ENEMY_SIZE_HALF),
        bevy_rapier2d::prelude::Velocity::default(),
        LockedAxes::ROTATION_LOCKED,
        Enemy,
    ));

    // Player
    commands.spawn((
        Name::new("Player"),
        Player,
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Cube { size: PLAYER_SIZE }.into()).into(),
            material: materials.add(ColorMaterial::from(Color::RED)),
            transform: Transform::from_translation(PLAYER_STARTING_POSITION),
            ..default()
        },
        RigidBody::KinematicVelocityBased,
        KinematicCharacterController {
            snap_to_ground: Some(CharacterLength::Absolute(0.1)),
            autostep: None,
            apply_impulse_to_dynamic_bodies: false,
            up: Vec2::Y,
            slide: true,
            ..default()
        },
        Collider::cuboid(PLAYER_SIZE_HALF, PLAYER_SIZE_HALF),
        LockedAxes::ROTATION_LOCKED,
        Velocity::default(),
    ));
}

fn config(
    config: ResMut<Configuration>,
    mut commands: Commands,
    enemy_query: Query<Entity, With<Enemy>>,
) {
    for enemy in &enemy_query {
        if config.enemy {
            // TODO
            // commands.entity(enemy).spawn();
        } else {
            commands.entity(enemy).despawn();
        }
    }
}

fn player_kinematics(
    time: Res<Time>,
    rapier_config: Res<RapierConfiguration>,
    mut controller_query: Query<(
        &mut Velocity,
        &mut KinematicCharacterController,
        Option<&KinematicCharacterControllerOutput>,
    )>,
    input: Res<Input<KeyCode>>,
    config: Res<Configuration>,
) {
    if controller_query.is_empty() {
        return;
    }
    let (mut velocity, mut controller, controller_output) =
        controller_query.single_mut();

    let grounded = match controller_output {
        Some(output) => output.grounded,
        None => false,
    };

    let dt = time.delta_seconds();
    let mut instant_acceleration = Vec2::ZERO;
    let mut instant_velocity = velocity.0;

    // physics simulation
    // friction

    if grounded {
        instant_velocity.x *= config.friction;
    } else {
        // gravity
        instant_acceleration += Vec2::Y * rapier_config.gravity;
        // to make the air velocity slitly faster
        instant_velocity.x *= config.friction + 0.01;
    }

    if input.any_pressed([KeyCode::Right, KeyCode::D]) {
        instant_velocity.x += config.speed;
    }
    if input.any_pressed([KeyCode::Left, KeyCode::A]) {
        instant_velocity.x -= config.speed;
    }

    if input.pressed(KeyCode::Space) && grounded {
        instant_velocity.y = config.jump_speed;
    }

    instant_velocity = instant_velocity
        .clamp(Vec2::splat(-config.clamp), Vec2::splat(config.clamp));

    velocity.0 = (instant_acceleration * dt) + instant_velocity;
    controller.translation = Some(velocity.0 * dt);
}

fn enemy_move(
    mut enemy_query: Query<&mut bevy_rapier2d::prelude::Velocity, With<Enemy>>,
) {
    for mut velocity in enemy_query.iter_mut() {
        velocity.linvel.x = -100.0;
    }
}

fn enemy_collision(
    mut commands: Commands,
    player_query: Query<
        Option<&KinematicCharacterControllerOutput>,
        With<Player>,
    >,
    enemy_query: Query<Entity, With<Enemy>>,
) {
    let handle_enemy_query = |c: &CharacterCollision| {
        let entity = c.entity.clone();
        enemy_query.iter().filter(move |enemy| *enemy == entity)
    };

    let handle_despawn = |entity: Entity| {
        commands.entity(entity).despawn();
    };

    let handle_collisions = |player: &KinematicCharacterControllerOutput| {
        player
            .collisions
            .iter()
            .filter(|c| c.toi.normal1.y == 1.0)
            .flat_map(handle_enemy_query)
            .for_each(handle_despawn);
    };

    player_query.single().map(handle_collisions);
}

fn camerman(
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    player_query: Query<(&Transform, &Velocity), With<Player>>,
) {
    let (transform, velocity) = player_query.single();

    if velocity.x > 0.0 {
        for mut camera in camera_query.iter_mut() {
            // TODO: move to an external function.
            // Calculates the "normalized" player position. The player position will start at 0.0
            let norm_pos = (transform.translation.x + (WINDOW_WIDTH / 2.0))
                - camera.translation.x;
            if norm_pos > WINDOW_WIDTH / 2.0 {
                camera.translation.x += velocity.x / 60.0;
            }
        }
    }
}

fn block_left_move(
    camera_query: Query<&Transform, With<Camera2d>>,
    mut player_query: Query<
        (&mut Transform, &mut Velocity),
        (With<Player>, Without<Camera2d>),
    >,
) {
    let (mut player_pos, mut velocity) = player_query.single_mut();
    for camera in &camera_query {
        // TODO: move to an external function.
        let norm_pos = (player_pos.translation.x + (WINDOW_WIDTH / 2.0))
            - camera.translation.x
            - PLAYER_SIZE_HALF;

        if norm_pos < 0.0 {
            player_pos.translation.x += norm_pos * -1.0;
            velocity.x = 0.0;
        }
    }
}
