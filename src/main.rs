//! This example demonstrates the built-in 3d shapes in Bevy.
//! The scene includes a patterned texture and a rotation for visualizing the normals and UVs.

extern crate lazy_static;

use std::{f32::consts::PI, sync::Mutex};
use std::collections::HashMap;

use bevy::{
    prelude::{*},
    render::render_resource::{Extent3d, TextureDimension, TextureFormat}, gizmos,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, (setup))
        .add_systems(Update, (rotate, draw_spherical_colorspace))
        .run();
}

/// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component)]
struct Shape;

const X_EXTENT: f32 = 14.5;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

    let shapes = [
        meshes.add(shape::Cube::default().into()),
        meshes.add(shape::Icosphere::default().try_into().unwrap()),
        meshes.add(shape::UVSphere::default().into()),
    ];

    let num_shapes = shapes.len();

    for (i, shape) in shapes.into_iter().enumerate() {
        commands.spawn((
            PbrBundle {
                mesh: shape,
                material: debug_material.clone(),
                transform: Transform::from_xyz(
                    -X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * X_EXTENT,
                    7.0,
                    1.0,
                )
                .with_rotation(Quat::from_rotation_x(-PI / 4.)),
                ..default()
            },
            Shape,
        ));
    }

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 9000.0,
            range: 100.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(8.0, 16.0, 16.0),
        ..default()
    });

    // ground plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(50.0).into()),
        material: materials.add(Color::SILVER.into()),
        transform: Transform::from_xyz(0.,0.,-0.1).with_rotation( Quat::from_rotation_x(PI/2.)),
        ..default()
    });
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0., 6., 6.).looking_at(Vec3::new(0., 0., 6.), Vec3::Z),
        ..default()
    });

    // commands.insert_resource(ToggleCameraRotation(false));
 
}

fn rotate(mut query: Query<&mut Transform, With<Shape>>, 
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Shape>)>, 
    // camera_rotation: Res<ToggleCameraRotation>,
    keyboard: Res<Input<KeyCode>>, 
    time: Res<Time>) {
    
    for mut transform in &mut query {
        transform.rotate_y(time.delta_seconds() / 2.);
    }

    for mut camera_transform in &mut camera_query {

        let boost = if keyboard.pressed(KeyCode::ShiftLeft) {2.} else {0.};
        let speed = 2. + boost;

        // Define the camera's rotation speed in radians per second
        let camera_rotation_speed_horizontal = 
            if keyboard.pressed(KeyCode::Q)||keyboard.pressed(KeyCode::Left){
                speed
            }
            else if keyboard.pressed(KeyCode::E)||keyboard.pressed(KeyCode::Right) {
                -speed
            }
            else {
                0.0
        };

        let camera_rotation_speed_vertical = 
            if keyboard.pressed(KeyCode::R)||keyboard.pressed(KeyCode::Up){
                speed
            }
            else if keyboard.pressed(KeyCode::F)||keyboard.pressed(KeyCode::Down) {
                -speed
            }
            else {
                0.0
        };

        let camera_speed_horizontal = 
            if keyboard.pressed(KeyCode::D){
                speed
            }
            else if keyboard.pressed(KeyCode::A) {
                -speed
            }
            else {
                0.0
        };

        let camera_speed_forward = 
            if keyboard.pressed(KeyCode::W){
                speed
            }
            else if keyboard.pressed(KeyCode::S) {
                -speed
            }
            else {
                0.0
        };

        let camera_speed_vertical =
        if keyboard.pressed(KeyCode::Space){
            speed
        }
        else if keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::C) {
            -speed
        }
        else {
            0.0
        };

        let time_delta = time.delta_seconds();

        // Calculate the camera's rotation angle based on time and speed
        let camera_rotation_angle_horizontal = time_delta * camera_rotation_speed_horizontal;
        let camera_rotation_angle_vertical = time_delta * camera_rotation_speed_vertical;
        let camera_vertical = time_delta * camera_speed_vertical;
        let camera_horizontal = time_delta * camera_speed_horizontal;
        let camera_forward = time_delta * camera_speed_forward;

        let side_movement = camera_transform.local_x();
        let forward_movement = camera_transform.local_z();

        camera_transform.rotate(Quat::from_rotation_z(camera_rotation_angle_horizontal) * Quat::from_axis_angle(side_movement, camera_rotation_angle_vertical));
        camera_transform.translation.z += camera_vertical;
        camera_transform.translation +=  (Vec3::new(forward_movement.x,forward_movement.y,0.) * -camera_forward) + (side_movement * camera_horizontal);
    }
    
}

fn draw_spherical_colorspace(mut gizmos: Gizmos){
    //let color_map: HashMap<(f32,f32,f32),(f32,f32,f32)> = HashMap::new();

    let scale = 5.;

    let h_step = 24;
    let s_step = 240;
    let v_step = 24;

    let xyz_offset = (1.,0.,0.);
    let mut prev_color = hsv_spherical_rgb(0.0, 0.0, 0.0);

    for v in 0..v_step {
        let s_step_adjusted = s_step/(v_step-v);
        for s in 0..s_step_adjusted {
            for h in  0..h_step{
                let new_color= hsv_spherical_rgb(h as f32 / (h_step) as f32, 1.-(s as f32 /(s_step/(v_step-v)) as f32), v as f32 /v_step as f32);
                if !(s==0&&h==0) {
                    gizmos.line(
                  Vec3::new(prev_color.0*scale+xyz_offset.0,
                                  prev_color.1*scale+xyz_offset.1,
                                  prev_color.2*scale+xyz_offset.2
                                ),
                    Vec3::new(new_color.0*scale+xyz_offset.0,
                                  new_color.1*scale+xyz_offset.1,
                                  new_color.2*scale+xyz_offset.2
                                ), 
                        Color::RgbaLinear { red: (prev_color.0), green: (prev_color.1), blue: (prev_color.2), alpha: (1.) });
                }
                
                prev_color = new_color;
            }
        }
    }
    
}

/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 12;
    //assert!(hsv_spherical_rgb(0.0, 1.0, 1.0)==(1.0,0.0,0.0));

    let mut palette = [0; TEXTURE_SIZE*4];

    for n in 0..TEXTURE_SIZE{
        let float_color = hsv_spherical_rgb((n)as f32/(TEXTURE_SIZE)as f32, 1.0, 1.0);
        palette[n*4]    =(float_color.0*255.0)as u8;
        palette[n*4+1]  =(float_color.1*255.0)as u8;
        palette[n*4+2]  =(float_color.2*255.0)as u8;
        palette[n*4+3]  =170;
        println!("Color {n} is: {float_color:?}");
    }


    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8Unorm,
    )
}

// fn hsv_spherical_rgb(h: f32, s: f32, v: f32) -> (f32,f32,f32){

//     let hue_arc_length: f32 = 1.0/3.0;
//     let hue_part: f32 = (PI/2.0)*((3.0*h) % 1.0)*s+(PI/4.0)*(1.0-s);
//     let phi: f32 = 1.95968918625 - 1.1 * (1.15074-0.7893882996 * s).sin();
//     let a: f32 = v*hue_part.cos()*phi.sin();
//     let b: f32 = v*hue_part.sin()*phi.sin();
//     let c: f32 = v*phi.cos();

//     //println!("Hue_part:{hue_part}, :{h}");

//     if h < hue_arc_length {//yellow
//         (a,b,c)
//     }
//     else if h < 2.0 * hue_arc_length {//cyan
//         (c,a,b)
//     }
//     else{//magenta
//         (b,c,a)
//     }    
// }

fn hsv_spherical_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    const PI: f32 = std::f32::consts::PI;

    let key_hsv = (
        (h * 100000000.0) as u32,
        (s * 100000000.0) as u32,
        (v * 100000000.0) as u32
    );

    // Define a static hash map for caching results
    lazy_static::lazy_static! {
        static ref CACHE: Mutex<HashMap<(u32, u32, u32), (f32, f32, f32)>> = Mutex::new(HashMap::new());
    }

    // Check if the result is already cached
    if let Some(result) = CACHE.lock().unwrap().get(&key_hsv) {
        return *result;
    }

    let hue_arc_length: f32 = 1.0 / 3.0;
    let hue_part: f32 = (PI / 2.0) * ((3.0 * h) % 1.0) * s + (PI / 4.0) * (1.0 - s);
    let phi: f32 = 1.95968918625 - 1.1 * (1.15074 - 0.7893882996 * s).sin();
    let a: f32 = v * hue_part.cos() * phi.sin();
    let b: f32 = v * hue_part.sin() * phi.sin();
    let c: f32 = v * phi.cos();

    let result;

    if h < hue_arc_length {
        result = (a, b, c);
    } else if h < 2.0 * hue_arc_length {
        result = (c, a, b);
    } else {
        result = (b, c, a);
    }

    // Cache the result
    CACHE.lock().unwrap().insert(key_hsv, result);

    result
}


// struct ToggleCameraRotation(bool);
// impl bevy::prelude::Resource for ToggleCameraRotation {}

// fn toggle_camera_rotation(
//     mut toggle_camera_rotation: ResMut<ToggleCameraRotation>,
//     keyboard_input: Res<Input<KeyCode>>,
// ) {
//     // Toggle time-based rotation on/off when Space is pressed
//     if keyboard_input.just_pressed(KeyCode::Space) {
//         toggle_camera_rotation.0 = !toggle_camera_rotation.0;
//     }
// }