use bevy::{diagnostic::{FrameTimeDiagnosticsPlugin, PrintDiagnosticsPlugin}, prelude::*, render::camera::Camera};
use core::f32;

/*
 * TODO WT:
 * Planets random colours.
 * Planets have gravity.
 */

fn main() {
    App::build()
        .add_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        // .add_plugin(FrameTimeDiagnosticsPlugin)
        // .add_plugin(PrintDiagnosticsPlugin::default())
        .add_startup_system(startup_system.system())
        // .add_system(update_camera.system())
        .add_system_to_stage(stage::UPDATE, gravity.system())
        .add_system_to_stage(stage::UPDATE, apply_velocity.system())
        .run();
}

fn startup_system(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let m = 1000.0;
    let sun_rad = calc_rad(m, 4.0);
    let target_ball = spawn_ball(
        commands,
        &mut meshes,
        &mut materials,
        Color::AZURE,
        sun_rad,
        m,
        Vec3::zero(),
    );

    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 100.0, 0.0))
                .looking_at(-Vec3::unit_y(), Vec3::unit_z()),
            ..Default::default()
        })
        // .insert_resource(CameraFollow(target_ball.unwrap(), 100.0))
        .spawn(LightBundle {
            light: Light {
                color: Color::WHITE,
                ..Default::default()
            },
            ..Default::default()
        });

    let rad = 200.0;
    let num_objects = 200;

    use rand::prelude::*;

    let mut rng = rand::thread_rng();

    for _ in 0..num_objects {
        let x = rng.gen::<f32>() * rad - rad / 2.0;
        let y = rng.gen::<f32>() * rad - rad / 2.0;
        // let y = 0.0;
        let z = rng.gen::<f32>() * rad - rad / 2.0;

        let vec = Vec3::new(x, y, z);
        if vec.length() < sun_rad {
            continue;
        }

        let m = rng.gen::<f32>() * 10.0 + 1.0;

        spawn_ball(
            commands,
            &mut meshes,
            &mut materials,
            Color::WHITE,
            calc_rad(m, 1.0),
            m,
            vec,
        );
    }
}

struct Planet {
    radius: f32,
    mass: f32,
    velocity: Vec3,
}

struct CameraFollow(Entity, f32);

fn update_camera(
    cam_follow: Res<CameraFollow>,
    mut queries: QuerySet<(
        Query<&Transform>,
        Query<&mut Transform, With<Camera>>
    )>,
) {
    let trans = if let Ok(transform) = queries.q0().get(cam_follow.0) {
        transform.translation
    } else {
        return;
    };

    for mut cam_trans in queries.q1_mut().iter_mut() {
        cam_trans.translation.x = trans.x;
        cam_trans.translation.y = trans.y + cam_follow.1;
        cam_trans.translation.z = trans.z;
    }
}

fn spawn_ball(
    commands: &mut Commands,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut materials: &mut ResMut<Assets<StandardMaterial>>,
    color: Color,
    radius: f32,
    mass: f32,
    translation: Vec3,
) -> Option<Entity> {
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius,
                subdivisions: 5,
            })),
            material: materials.add(StandardMaterial {
                albedo: color,
                albedo_texture: None,
                shaded: true,
            }),
            transform: Transform::from_translation(translation),
            ..Default::default()
        })
        .with(Planet {
            radius,
            mass,
            velocity: Vec3::zero(),
        })
        .current_entity()
}

// f = m1 a
// f = G(m1 m2) / r^2
// m1 a = G(m1 m2) / r^2
// r^2 m1 a = G m1 m2
// r^2 a = G m2
// a = G m2 / r^2

fn gravity(mut query: Query<(Entity, &mut Planet, &mut Transform)>, time: Res<Time>) {
    let mass_positions: Vec<_> = query
        .iter_mut()
        .map(|(entity, planet, transform)| (entity, planet.mass, transform.translation))
        .collect();

    let gravity_constant = 1.0;

    let delta_seconds = time.delta_seconds();

    for (entity, mut planet, mut transform) in query.iter_mut() {
        let mut accel = Vec3::zero();

        for (e_other, mass, translation) in mass_positions.iter() {
            if entity == *e_other {
                continue;
            }

            let vec_to: Vec3 = *translation - transform.translation;

            let g = (gravity_constant * mass) / vec_to.length_squared();
            accel += g * vec_to.normalize();
        }

        planet.velocity += accel * delta_seconds;
    }
}

fn apply_velocity(mut query: Query<(Entity, &mut Planet, &mut Transform)>, time: Res<Time>) {
    let pos_rads: Vec<_> = query
        .iter_mut()
        .map(|(entity, planet, trans)| (entity, trans.translation, planet.radius))
        .collect();

    for (entity, mut planet, mut transform) in query.iter_mut() {
        pos_rads.iter().for_each(|(e, pos, rad)| {
            if entity == *e {
                return;
            }

            let vec_to = *pos - transform.translation;
            if vec_to.length() - (planet.radius + rad) < 0.0 {
                planet.velocity = reflect_vec(planet.velocity, vec_to.normalize()) * 0.995;
            }
        });

        // TODO WT: CCD
        transform.translation += planet.velocity * time.delta_seconds();
    }
}

fn reflect_vec(vector: Vec3, normal: Vec3) -> Vec3 {
    -2.0 * (vector.dot(normal)) * normal + vector
}

fn calc_rad(mass: f32, density: f32) -> f32 {
    let v = mass / density;
    ((3.0 * v) / (4.0 * f32::consts::PI)).powf(1.0 / 3.0)
}
// fn collision(
//     mut query: Query<(entity, )
// )

// v = (4 / 3) pi r^3
// v / r^3 = 4pi/3
// 3v = 4pir^3
// root3(3v/4pi) = r
