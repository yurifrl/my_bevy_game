use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_editor_pls::prelude::*;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_rapier2d::prelude::*;

const WINDOW_WIDTH: f32 = 1080.0;
const WINDOW_HEIGHT: f32 = 720.0;
const PLAYER_SIZE_HALF: f32 = PLAYER_SIZE / 2.0;
const PLAYER_SIZE: f32 = 100.0;
const PLAYER_STARTING_POSITION: Vec3 =
    Vec3::new(((WINDOW_WIDTH / 2.0) * -1.0) + 100.0, -100.0, 0.0);
const PLAYER_VELOCITY: f32 = 100.0;
const GROUND_SIZE_HALF_X: f32 = WINDOW_WIDTH / 2.0;
const GROUND_SIZE_HALF_Y: f32 = 25.0;
const GROUND_SIZE: Vec2 = Vec2 {
    x: WINDOW_WIDTH,
    y: 50.0,
};
const GROUND_STARTING_POSITION: Vec3 = Vec3::new(0.0, (WINDOW_HEIGHT / 2.0 - 25.0) * -1.0, 5.0);
const ENEMY_SIZE: f32 = 25.0;
const ENEMY_SIZE_HALF: f32 = ENEMY_SIZE / 2.0;
const ENEMY_STARTING_POSITION: Vec3 = Vec3::new(200.0, -300.0, 0.0);
const ENEMY_VELOCITY: f32 = 100.0;

#[derive(Component)]
struct Ground;
#[derive(Component)]
struct Enemy;
#[derive(Component, Reflect, Resource, Default, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
struct Configuration {
    jump_speed: f32,
    speed: f32,
    clamp: f32,
    friction: f32,
}
#[derive(Component, Default, Copy, Clone, PartialEq, Debug)]
struct Player {
    velocity: Vec2,
}

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
        // .add_system(player_move)
        // .add_system(player_jump)
        .add_system(enemy_move)
        .add_system(enemy_collision)
        .add_system(camerman)
        .add_system(block_left_move)
        .add_system(player_kinematics)
        .add_plugin(EditorPlugin)
        .init_resource::<Configuration>()
        .insert_resource(Configuration {
            jump_speed: 100.00,
            speed: 40.0,
            friction: 0.9,
            clamp: 400.0,
        })
        .insert_resource(RapierConfiguration {
            gravity: Vect::Y * -9.81 * 20.0,
            ..Default::default()
        })
        .add_plugin(ResourceInspectorPlugin::<Configuration>::default())
        .run();
}

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((Name::new("Camera"), Camera2dBundle::default()));

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
            transform: Transform::from_translation(GROUND_STARTING_POSITION),
            ..default()
        },
        RigidBody::Fixed,
        Collider::cuboid(GROUND_SIZE_HALF_X, GROUND_SIZE_HALF_Y),
        Dominance::group(50),
        Ground,
    ));

    // Ground 2
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
            material: materials.add(ColorMaterial::from(Color::DARK_GREEN)),
            transform: Transform::from_translation(Vec3::new(
                WINDOW_WIDTH,
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

    // Player
    commands.spawn((
        Name::new("Player"),
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Cube { size: PLAYER_SIZE }.into()).into(),
            material: materials.add(ColorMaterial::from(Color::RED)),
            transform: Transform::from_translation(PLAYER_STARTING_POSITION),
            ..default()
        },
        RigidBody::KinematicPositionBased,
        KinematicCharacterController {
            snap_to_ground: Some(CharacterLength::Absolute(0.1)),
            autostep: None,
            apply_impulse_to_dynamic_bodies: false,
            up: Vec2::Y,
            // offset: CharacterLength::Absolute(0.01),
            slide: true,
            ..default()
        },
        Collider::cuboid(PLAYER_SIZE_HALF, PLAYER_SIZE_HALF),
        LockedAxes::ROTATION_LOCKED, // bad ;(
        Player::default(),
    ));

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
        Velocity::zero(),
        LockedAxes::ROTATION_LOCKED,
        Enemy,
    ));
}

fn enemy_move(mut enemy_query: Query<&mut Velocity, With<Enemy>>) {
    for mut enemy_vel in enemy_query.iter_mut() {
        enemy_vel.linvel.x = -100.0;
    }
}

fn enemy_collision(
    mut commands: Commands,
    rapier_context: Res<RapierContext>,
    player_query: Query<Entity, With<Player>>,
    enemy_query: Query<Entity, With<Enemy>>,
) {
    for enemy in &enemy_query {
        if let Some(collision) = rapier_context.contact_pair(player_query.single(), enemy) {
            for manifold in collision.manifolds() {
                if manifold.normal().y == -1.0 {
                    commands.entity(enemy).despawn();
                    return;
                }

                // println!("MORREU");
            }
        }
    }
}

fn camerman(
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    player_query: Query<(&Transform, &Player)>,
) {
    let (transform, player) = player_query.single();

    if player.velocity.x > 0.0 {
        for mut camera in camera_query.iter_mut() {
            // TODO: move to an external function.
            // Calculates the "normalized" player position. The player position will start at 0.0
            let norm_pos = (transform.translation.x + (WINDOW_WIDTH / 2.0)) - camera.translation.x;
            if norm_pos > WINDOW_WIDTH / 2.0 {
                camera.translation.x += player.velocity.x / 100.0;
            }
        }
    }
}

fn block_left_move(
    camera_query: Query<&Transform, (With<Camera2d>, Without<Player>)>,
    mut player_query: Query<(&mut Transform, &mut Player), Without<Camera2d>>,
) {
    let (mut player_pos, mut player) = player_query.single_mut();
    for camera in &camera_query {
        // TODO: move to an external function.
        let norm_pos = (player_pos.translation.x + (WINDOW_WIDTH / 2.0))
            - camera.translation.x
            - PLAYER_SIZE_HALF;

        if norm_pos < 0.0 {
            player_pos.translation.x += norm_pos * -1.0;
            player.velocity.x = 0.0;
        }
    }
}

fn player_kinematics(
    time: Res<Time>,
    rapier_config: Res<RapierConfiguration>,
    mut controller_query: Query<(
        &mut Player,
        &mut KinematicCharacterController,
        Option<&KinematicCharacterControllerOutput>,
    )>,
    input: Res<Input<KeyCode>>,
    config: Res<Configuration>,
) {
    if controller_query.is_empty() {
        return;
    }
    let (mut player, mut controller, controller_output) = controller_query.single_mut();
    let grounded = match controller_output {
        Some(output) => output.grounded,
        None => false,
    };

    let dt = time.delta_seconds();
    let mut instant_acceleration = Vec2::ZERO;
    let mut instant_velocity = player.velocity;

    // physics simulation
    if grounded {
        // friction
        instant_velocity.x *= config.friction;
    } else {
        // gravity
        instant_acceleration += Vec2::Y * rapier_config.gravity;
    }

    if input.any_pressed([KeyCode::Right, KeyCode::D]) {
        instant_velocity.x += config.speed;
    }
    if input.any_pressed([KeyCode::Left, KeyCode::A]) {
        instant_velocity.x -= config.speed;
    }

    if input.pressed(KeyCode::Space) {
        if grounded {
            instant_velocity.y = config.jump_speed;
        }
    }

    instant_velocity =
        instant_velocity.clamp(Vec2::splat(-config.clamp), Vec2::splat(config.clamp));

    player.velocity = (instant_acceleration * dt) + instant_velocity;
    controller.translation = Some(player.velocity * dt);
}
