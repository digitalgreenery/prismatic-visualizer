//Digital Greenery
//Spherical RGB Visualizer


use spherical_rgb::{Color as SColor, DefinedColor, Gamma, spherical_hcl, tuple_lerp};
use std::{f32::consts::PI};
mod ui;


use bevy::{
    prelude::{*},
    render::render_resource::{Extent3d, TextureDimension, TextureFormat, PrimitiveTopology},
    render::mesh::Indices,
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(EguiPlugin)
        .add_systems(Startup, (setup))
        .add_systems(Update, (rotate,ui_overlay,update_visualization))
        .run();
}

/// A marker component for our components so we can query them separately from the ground plane
#[derive(Component)]
struct Shape;

#[derive(Component)]
struct SphericalVisualizationMeshes;

#[derive(Resource, Clone)]
struct DefinedColorResource{
    component_limit: SColor,
    gamma: Gamma,
    hue_adjust: Gamma,
    visualization_needs_updated: bool,
}

const X_EXTENT: f32 = 14.5;
const SCALE: f32 = 5.0;

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
        mesh: meshes.add(shape::Plane::from_size(100.0).into()),
        material: materials.add(Color::SILVER.into()),
        transform: Transform::from_xyz(0.,0.,-0.1).with_rotation( Quat::from_rotation_x(PI/2.)),
        ..default()
    });
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(SCALE*2., SCALE*2., SCALE*2.).looking_at(Vec3::new(0., 0., 0.), Vec3::Z),
        ..default()
    });

    let defined_color = DefinedColorResource{ component_limit: SColor::new(1., 1., 1.), gamma: Gamma::new(1., 1., 1.), hue_adjust: Gamma::new(1., 1., 1.), visualization_needs_updated: false };
    let defined_color_copy = DefinedColorResource{ component_limit: SColor::new(1., 1., 1.), gamma: Gamma::new(1., 1., 1.), hue_adjust: Gamma::new(1., 1., 1.), visualization_needs_updated: false };

    commands.insert_resource(defined_color);

    spawn_spherical_visualization(commands, meshes, materials, defined_color_copy);
    
    
    

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

fn ui_overlay(mut contexts: EguiContexts, mut defined_color: ResMut<DefinedColorResource>){
    //Move current resource values to mutable variables for sliders
    let (mut red, mut green, mut blue) = defined_color.component_limit.to_tuple();
    let (mut red_gamma, mut green_gamma, mut blue_gamma) = defined_color.gamma.to_tuple();
    let (mut yellow_adjust, mut cyan_adjust, mut magenta_adjust) = defined_color.hue_adjust.to_tuple();

    //Create window for variable sliders
    egui::Window::new("Spherical RGB Adjust").show(contexts.ctx_mut(), |ui|{
        ui.label("Component Bounds");
        ui.add(egui::Slider::new( &mut red ,0.0..=1.0).text("Red"));
        ui.add(egui::Slider::new( &mut green ,0.0..=1.0).text("Green"));
        ui.add(egui::Slider::new( &mut blue ,0.0..=1.0).text("Blue"));
        ui.label("Gamma");
        ui.add(egui::Slider::new( &mut red_gamma ,0.1..=3.0).text("Red"));
        ui.add(egui::Slider::new( &mut green_gamma ,0.1..=3.0).text("Green"));
        ui.add(egui::Slider::new( &mut blue_gamma ,0.1..=3.0).text("Blue"));
        ui.label("Hue Adjust");
        ui.add(egui::Slider::new( &mut yellow_adjust ,0.1..=3.0).text("Yellow"));
        ui.add(egui::Slider::new( &mut cyan_adjust ,0.1..=3.0).text("Cyan"));
        ui.add(egui::Slider::new( &mut magenta_adjust ,0.1..=3.0).text("Magenta"));

    });

    //Check if slider has been changed
    let slider_changed = 
        (red,green,blue) != defined_color.component_limit.to_tuple() ||
        (red_gamma,green_gamma,blue_gamma) != defined_color.gamma.to_tuple() ||
        (yellow_adjust,cyan_adjust,magenta_adjust) != defined_color.hue_adjust.to_tuple();

    if slider_changed{
        //Update values to Resource
        defined_color.component_limit = SColor::from_tuple((red,green,blue));
        defined_color.gamma = Gamma::new(red_gamma,green_gamma,blue_gamma);
        defined_color.hue_adjust = Gamma::new(yellow_adjust,cyan_adjust,magenta_adjust);

        //Update mesh
        //todo
        println!("Reached");
        defined_color.visualization_needs_updated = true;

    }
}

fn update_visualization(
    mut commands: Commands,
    mut defined_color: ResMut<DefinedColorResource>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    entities: Query<Entity, With<SphericalVisualizationMeshes>>)
{
    if defined_color.visualization_needs_updated {
        defined_color.visualization_needs_updated = false;

        //Delete previous visualization 
        for mesh in entities.iter(){
            commands.entity(mesh).despawn();
        }
    
    let defined_color_copy = defined_color.clone();

        spawn_spherical_visualization(commands, meshes, materials, defined_color_copy);
    }
}

fn spawn_spherical_visualization(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    defined_color: DefinedColorResource)
{

    let spherical_rgb_meshes = draw_spherical_colorspace(defined_color);
    for(_index, mesh) in spherical_rgb_meshes.iter().enumerate() {
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(mesh.clone()),
                material: materials.add(StandardMaterial {
                    base_color: Color::WHITE,
                    unlit: true,
                    cull_mode: None,
                    emissive: Color::rgb(1.0, 1.0, 1.0),// Set emissive color
                    ..Default::default()
                }),
                transform: Transform::from_scale(Vec3 { x: SCALE, y: SCALE, z: SCALE }),
                ..Default::default()
            },
            SphericalVisualizationMeshes,
        ));

    }

}


fn create_quad(v0: Vec3, v1: Vec3, v2: Vec3, v3: Vec3, defined_color: DefinedColorResource) -> Mesh {
    
    let gamma = defined_color.gamma.clone();
    let (c0,c1,c2,c3) = (
        DefinedColor::new(SColor::new(v0.x,v0.y,v0.z),gamma).to_color(),
        DefinedColor::new(SColor::new(v0.x,v0.y,v0.z),gamma).to_color(),
        DefinedColor::new(SColor::new(v0.x,v0.y,v0.z),gamma).to_color(),
        DefinedColor::new(SColor::new(v0.x,v0.y,v0.z),gamma).to_color(),
    );

    // Create a new mesh using a triangle list topology, where each set of 3 vertices composes a triangle.
    Mesh::new(PrimitiveTopology::TriangleList)
        // Add 4 vertices, each with its own position attribute (coordinate in
        // 3D space), for each of the corners of the parallelogram.
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![v0, v1, v2, v3]
        )
        // Assign color to each vertex based on its xyz values.
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_COLOR,
            vec![
                [c0.r, c0.g, c0.b, 1.0],
                [c0.r, c0.g, c0.b, 1.0],
                [c0.r, c0.g, c0.b, 1.0],
                [c0.r, c0.g, c0.b, 1.0],
            ]
        )
        // Assign normals (everything points outwards)
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            vec![[1.0, 1.0, 1.0],
                        [1.0, 1.0, 1.0],
                        [1.0, 1.0, 1.0],
                        [1.0, 1.0, 1.0]]
        )
        // After defining all the vertices and their attributes, build each triangle using the
        // indices of the vertices that make it up in a counter-clockwise order.
        .with_indices(Some(Indices::U32(vec![
            // First triangle
            0, 1, 2,
            // Second triangle
            1, 3, 2
        ])))
}

fn draw_spherical_colorspace(defined_color: DefinedColorResource) -> Vec<Mesh>{
    let mut colorspace_meshes  = Vec::<Mesh>::new();

    let (h_step,s_step,v_step) = (24, 8, 8);

    for v in 0..v_step {
        let s_step_adjusted = s_step/(v_step-v);
        for s in 0..s_step_adjusted {
            for h in  0..h_step{   

                colorspace_meshes.push(create_quad(
                    Vec3::from(spherical_hcl(h as f32 / h_step as f32,1.-(s as f32 /(s_step/(v_step-v)) as f32),v as f32 / v_step as f32).to_tuple()),
                    Vec3::from(spherical_hcl(((h+1) % h_step) as f32 / h_step as f32, 1.-(s as f32 /(s_step/(v_step-v)) as f32), v as f32 /v_step as f32).to_tuple()),
                    Vec3::from(spherical_hcl(h as f32 / (h_step) as f32, 1.-((s+1) as f32 /(s_step/(v_step-v)) as f32), v as f32 /v_step as f32).to_tuple()),
                    Vec3::from(spherical_hcl(((h+1) % h_step) as f32 / (h_step) as f32, 1.-((s+1) as f32 /(s_step/(v_step-v)) as f32), v as f32 /v_step as f32).to_tuple()),
                    defined_color.clone(),
                ));

            }
        }
    }

    return colorspace_meshes;
    
}

/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 12;

    let mut palette = [0; TEXTURE_SIZE*4];

    for n in 0..TEXTURE_SIZE{
        let float_color = DefinedColor::new(spherical_hcl((n)as f32/(TEXTURE_SIZE)as f32, 1.0, 1.0),Gamma::new(2.2,2.2,2.2,)).to_color().to_tuple();
        palette[n*4]    =(float_color.0*255.0)as u8;
        palette[n*4+1]  =(float_color.1*255.0)as u8;
        palette[n*4+2]  =(float_color.2*255.0)as u8;
        palette[n*4+3]  =255;
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
        TextureFormat::Rgba8UnormSrgb,
    )
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