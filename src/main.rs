use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

mod camera;
mod controller;
use crate::camera::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(
            0xF9 as f32 / 255.0,
            0xF9 as f32 / 255.0,
            0xFF as f32 / 255.0,
        )))
        .insert_resource(Msaa::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(FlyCameraPlugin)
        .add_startup_system(setup_graphics)
        .add_startup_system(setup_physics)
        .add_system(cast_shape)
        .add_system(collision_events)
        .insert_resource(RapierConfiguration {
            gravity: Vec3::Y * -98.1,
            ..Default::default()
        })
        .run();
}

fn setup_graphics(mut commands: Commands) {
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(-30.0, 30.0, 100.0)
                .looking_at(Vec3::new(0.0, 10.0, 0.0), Vec3::Y),
            ..Default::default()
        })
        .insert(FlyCamera::default());
}

pub fn setup_physics(mut commands: Commands) {
    /*
     * Ground
     */
    let ground_size = 200.1;
    let ground_height = 0.1;

    commands
        .spawn(TransformBundle::from(Transform::from_xyz(
            0.0,
            -ground_height,
            0.0,
        )))
        .insert(Collider::cuboid(ground_size, ground_height, ground_size));

    /*
     * Create the cubes
     */
    let num = 8;
    let rad = 1.0;

    let shift = rad * 2.0 + rad;
    let centerx = shift * (num / 2) as f32;
    let centery = shift / 2.0;
    let centerz = shift * (num / 2) as f32;

    let mut offset = -(num as f32) * (rad * 2.0 + rad) * 0.5;

    for j in 0usize..20 {
        for i in 0..num {
            for k in 0usize..num {
                let x = i as f32 * shift - centerx + offset;
                let y = j as f32 * shift + centery + 3.0;
                let z = k as f32 * shift - centerz + offset;

                // Build the rigid body.
                commands
                    .spawn(TransformBundle::from(Transform::from_xyz(x, y, z)))
                    .insert(RigidBody::Dynamic)
                    .insert(Collider::cuboid(rad, rad, rad));
            }
        }

        offset -= 0.05 * rad * (num as f32 - 1.0);
    }

    // Insert player
    commands
        .spawn(TransformBundle::from(Transform::from_xyz(
            -30.0, 30.0, 50.0,
        )))
        .insert(Collider::capsule(Vec3::Y * 0.5, Vec3::Y * 1.5, 0.5))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(Velocity::zero())
        .insert(RigidBody::Dynamic)
        .insert(Sleeping::disabled())
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(AdditionalMassProperties::Mass(1.0))
        .insert(GravityScale(1.0))
        .insert(Ccd { enabled: true });
}

/* Cast a shape inside of a system. */
fn cast_shape(
    mut commands: Commands,
    windows: Res<Windows>,
    mouse_button_input: Res<Input<MouseButton>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        let shape = Collider::cuboid(1.0, 1.0, 1.0);

        for (camera, camera_transform) in cameras.iter() {
            // First, compute a ray from the mouse position.
            let (ray_pos, ray_dir) =
                ray_from_mouse_position(windows.get_primary().unwrap(), camera, camera_transform);

            commands
                .spawn(TransformBundle::from(Transform::from_xyz(
                    ray_pos.x, ray_pos.y, ray_pos.z,
                )))
                .insert(RigidBody::Dynamic)
                .insert(Velocity {
                    linvel: ray_dir,
                    angvel: Vec3::new(1.0, 1.0, 1.0),
                })
                .insert(ExternalImpulse {
                    impulse: camera_transform.forward() * 5000.0,
                    torque_impulse: Vec3::new(1.0, 1.0, 1.0),
                })
                .insert(Sleeping::disabled())
                .insert(Ccd::enabled())
                .insert(shape.clone())
                .insert(ActiveEvents::COLLISION_EVENTS);
        }
    }
}

/* A system that compute collision events. */
fn collision_events(mut commands: Commands, mut collision_events: EventReader<CollisionEvent>) {
    for collision_event in collision_events.iter() {
        println!("Received collision event: {:?}", collision_event);

        if let CollisionEvent::Started(entity1, entity2, _) = collision_event {
            let color = Color::YELLOW;
            commands
                .entity(entity1.clone())
                .insert(ColliderDebugColor(color));
            commands
                .entity(entity2.clone())
                .insert(ColliderDebugColor(color));
        }

        if let CollisionEvent::Stopped(entity1, entity2, _) = collision_event {
            let color = Color::BLUE;
            commands
                .entity(entity1.clone())
                .insert(ColliderDebugColor(color));
            commands
                .entity(entity2.clone())
                .insert(ColliderDebugColor(color));
        }
    }
}

// Credit to @doomy on discord.
fn ray_from_mouse_position(
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> (Vec3, Vec3) {
    let mouse_position = window.cursor_position().unwrap_or(Vec2::new(0.0, 0.0));

    let x = 2.0 * (mouse_position.x / window.width() as f32) - 1.0;
    let y = 2.0 * (mouse_position.y / window.height() as f32) - 1.0;

    let camera_inverse_matrix =
        camera_transform.compute_matrix() * camera.projection_matrix().inverse();
    let near = camera_inverse_matrix * Vec3::new(x, y, -1.0).extend(1.0);
    let far = camera_inverse_matrix * Vec3::new(x, y, 1.0).extend(1.0);

    let near = near.truncate() / near.w;
    let far = far.truncate() / far.w;
    let dir: Vec3 = far - near;
    (near, dir)
}
