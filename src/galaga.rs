use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
    sprite::MaterialMesh2dBundle,
};

const INITIAL_PROJECTILE_DIRECTION: Vec2 = Vec2::new(0.5, -0.5);
const PLAYER_COLOR: Color = Color::rgb(0.3, 0.3, 0.7);
const PLAYER_SIZE: Vec3 = Vec3::new(120.0, 20.0, 0.0);
const PLAYER_SPEED: f32 = 500.0;
const PLAYER_STARTING_POSITION: Vec3 = Vec3::new(0.0, 0.0, 0.0);
const PROJECTILE_COLOR: Color = Color::rgb(0.3, 0.6, 0.3);
const PROJECTILE_SIZE: Vec3 = Vec3::new(30.0, 30.0, 0.0);
const PROJECTILE_SPEED: f32 = 400.0;
const PROJECTILE_STARTING_POSITION: Vec3 = Vec3::new(0.0, 300.0, 1.0);
const PROJECTILE_TIME_LIMIT: f32 = 0.3;
const TIME_STEP: f32 = 1.0 / 60.0;

#[derive(Component)]
struct Player;
#[derive(Component)]
struct Collider;
#[derive(Component)]
struct Projectile;
#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);
#[derive(Resource)]
struct ProjectileTimer(Timer);
// The Enemy object
#[derive(Component)]
struct Enemy;

pub fn exec() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup_game)
        .insert_resource(ProjectileTimer(Timer::from_seconds(
            PROJECTILE_TIME_LIMIT,
            TimerMode::Once,
        )))
        .add_system(check_for_collisions)
        .add_systems(
            (
                move_player.before(check_for_collisions),
                shoot_projectile,
                move_projectiles,
                destroy_projectiles,
            )
                .in_schedule(CoreSchedule::FixedUpdate),
        )
        .insert_resource(FixedTime::new_from_secs(TIME_STEP))
        .add_system(bevy::window::close_on_esc)
        .run();
}

fn setup_game(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    // Player
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: PLAYER_STARTING_POSITION,
                scale: PLAYER_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: PLAYER_COLOR,
                ..default()
            },
            ..default()
        },
        Player,
        Collider,
    ));

    // Enemy
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::default().into()).into(),
            material: materials.add(ColorMaterial::from(PROJECTILE_COLOR)),
            transform: Transform::from_translation(PROJECTILE_STARTING_POSITION)
                .with_scale(PROJECTILE_SIZE * Vec3::new(2.0, 2.0, 2.0)),
            ..default()
        },
        Enemy,
        Collider,
    ));
}

fn move_player(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let mut paddle_transform = query.single_mut();
    let mut direction = 0.0;

    if keyboard_input.pressed(KeyCode::A) {
        direction -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::D) {
        direction += 1.0;
    }

    // Calculate the new horizontal paddle position based on player input
    let new_paddle_position = paddle_transform.translation.x + direction * PLAYER_SPEED * TIME_STEP;

    paddle_transform.translation.x = new_paddle_position;
}

fn shoot_projectile(
    time: Res<Time>,
    mut projectile_timer: ResMut<ProjectileTimer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&Transform, With<Player>>,
) {
    let player_transform = query.single_mut();

    if keyboard_input.pressed(KeyCode::Space) {
        if projectile_timer.0.tick(time.delta()).finished() {
            // Reset the timer
            projectile_timer.0.reset();
            // run logic

            // Spawn projectile
            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: meshes.add(shape::Circle::default().into()).into(),
                    material: materials.add(ColorMaterial::from(PROJECTILE_COLOR)),
                    transform: Transform::from_translation(player_transform.translation)
                        .with_scale(PROJECTILE_SIZE),
                    ..default()
                },
                Projectile,
                Velocity(INITIAL_PROJECTILE_DIRECTION.normalize() * PROJECTILE_SPEED),
            ));
        }
    }
}

fn move_projectiles(mut query: Query<&mut Transform, With<Projectile>>) {
    for mut collider_transform in &mut query {
        // Calculate the new horizontal player position based on player input
        let new_projectile_position = collider_transform.translation.y + 250.0 * TIME_STEP;
        // TODO: make sure player doesn't exceed bounds of game area

        collider_transform.translation.y = new_projectile_position;
    }
}

fn destroy_projectiles(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<Projectile>>,
) {
    for (collider_entity, collider_transform) in &query {
        if collider_transform.translation.y > 350.0 {
            commands.entity(collider_entity).despawn();
        }
    }
}

fn check_for_collisions(
    mut commands: Commands,
    projectiles_query: Query<(Entity, &Transform), With<Projectile>>,
    collider_query: Query<(Entity, &Transform, Option<&Enemy>), With<Collider>>,
) {
    // Loop through all the projectiles on screen
    for (projectile_entity, projectile_transform) in &projectiles_query {
        // Loop through all collidable elements on the screen
        // TODO: Figure out how to flatten this - 2 for loops no bueno
        for (collider_entity, collider_transform, enemy_check) in &collider_query {
            let collision = collide(
                projectile_transform.translation,
                projectile_transform.scale.truncate(),
                collider_transform.translation,
                collider_transform.scale.truncate(),
            );

            if let Some(collision) = collision {
                // If it's an enemy, destroy!
                if enemy_check.is_some() {
                    println!("Collided!");

                    // Enemy is destroyed
                    commands.entity(collider_entity).despawn();

                    // Projectile disappears too? Prevents "cutting through" a line of enemies all at once
                    commands.entity(projectile_entity).despawn();
                }
            }
        }
    }
}
