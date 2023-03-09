use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_editor_pls::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::prelude::*;

const PLAYER_SIZE_HALF: f32 = PLAYER_SIZE / 2.0;
const PLAYER_SIZE: f32 = 100.0;
const PLAYER_STARTING_POSITION: Vec3 = Vec3::new(-500.0, -100.0, 0.0);
const PLAYER_VELOCITY: f32 = 100.0;
const GROUND_SIZE_HALF_X: f32 = 1000.0;
const GROUND_SIZE_HALF_Y: f32 = 25.0;
const GROUND_SIZE: Vec2 = Vec2 { x: 2000.0, y: 50.0 };
const GROUND_STARTING_POSITION: Vec3 = Vec3::new(0.0, -300.0, 0.0);

const ENEMY_SIZE: f32 = 25.0;
const ENEMY_SIZE_HALF: f32 = ENEMY_SIZE / 2.0;
const ENEMY_STARTING_POSITION: Vec3 = Vec3::new(500.0, -300.0, 0.0);
const ENEMY_VELOCITY: f32 = 100.0;

#[derive(Component)]
struct Player;
#[derive(Component)]
struct Ground;
#[derive(Component)]
struct Enemy;

pub fn exec() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_startup_system(startup)
        .add_system(player_move)
        .add_system(player_jump)
        // .add_system(enemy_move)
        .add_system(enemy_collision)
        .add_plugin(WorldInspectorPlugin::default())
        .add_plugin(EditorPlugin)
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
        RigidBody::Dynamic,
        Collider::cuboid(PLAYER_SIZE_HALF, PLAYER_SIZE_HALF),
        Velocity::zero(),
        LockedAxes::ROTATION_LOCKED, // bad ;(
        Player,
        Dominance::group(10),
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
        Collider::cuboid(ENEMY_SIZE_HALF, ENEMY_SIZE_HALF),
        Velocity::zero(),
        LockedAxes::ROTATION_LOCKED,
        Enemy,
    ));
}

fn player_move(input: Res<Input<KeyCode>>, mut query: Query<&mut Velocity, With<Player>>) {
    let mut velocity = query.single_mut();

    if input.any_pressed([KeyCode::Right, KeyCode::D]) {
        velocity.linvel.x = PLAYER_VELOCITY;
    }
    if input.any_pressed([KeyCode::Left, KeyCode::A]) {
        velocity.linvel.x = -PLAYER_VELOCITY;
    }
}

fn player_jump(
    rapier_context: Res<RapierContext>,
    mut player_query: Query<(Entity, &mut Velocity), With<Player>>,
    ground_query: Query<Entity, With<Ground>>,
    input: Res<Input<KeyCode>>,
) {
    if input.pressed(KeyCode::Space) {
        let (player, mut velocity) = player_query.single_mut();
        let ground = ground_query.single();

        if rapier_context.contact_pair(player, ground).is_some() {
            velocity.linvel.y = PLAYER_VELOCITY;
        }
    }
}

fn enemy_move(mut enemy_query: Query<(&mut Velocity), With<Enemy>>) {
    let mut enemy_vel = enemy_query.single_mut();
    enemy_vel.linvel.x = -100.0;
}

fn enemy_collision(
    rapier_context: Res<RapierContext>,
    player_query: Query<Entity, With<Player>>,
    enemy_query: Query<Entity, With<Enemy>>,
) {
    if let Some(collision) =
        rapier_context.contact_pair(player_query.single(), enemy_query.single())
    {
        for manifold in collision.manifolds() {
            // println!("{:?}", manifold.local1);
            // println!("{:?}", manifold.local2);
            for contact_point in manifold.points() {
                println!("{:?}", contact_point.dist());
            }
        }
    }
}
