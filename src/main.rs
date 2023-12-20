//Digital Greenery
//Spherical RGB Visualizer


use spherical_rgb::{Color as SColor, DefinedColor, Gamma, spherical_hcl, cubic_hsv, tuple_lerp};
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
        .add_systems(Startup, setup)
        .add_systems(Update, (rotate,camera_controls,ui_overlay,update_visualization))
        .run();
}

/// A marker component for our components so we can query them separately from the ground plane
#[derive(Component)]
struct Shape;

#[derive(Component)]
struct SphericalVisualizationMeshes;

#[derive(Resource, Clone)]
struct VisualizationSettings{
    component_limit: SColor,
    gamma: Gamma,
    hcl_adjust: (u8,u8,u8),
    visualization_needs_updated: bool,
    visualization_shape: VisualiztionShape,
}

#[derive(Clone, PartialEq, Eq)]
enum VisualiztionShape{
    Spherical,
    Cubic,
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

    // // ground plane
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(shape::Plane::from_size(100.0).into()),
    //     material: materials.add(Color::SILVER.into()),
    //     transform: Transform::from_xyz(0.,0.,-0.1).with_rotation( Quat::from_rotation_x(PI/2.)),
    //     ..default()
    // });
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(SCALE*2., SCALE*2., SCALE*2.).looking_at(Vec3::new(0., 0., 0.), Vec3::Z),
        ..default()
    });

    let settings = VisualizationSettings{ 
        component_limit: SColor::new(1., 1., 1.), 
        gamma: Gamma::new(2.2, 1.2, 1.75), 
        hcl_adjust: (24,8,8), 
        visualization_needs_updated: false, 
        visualization_shape: VisualiztionShape::Spherical
    };

    let settings_copy = settings.clone();

    commands.insert_resource(settings);

    spawn_spherical_visualization(commands, meshes, materials, settings_copy);
    
    
    

    // commands.insert_resource(ToggleCameraRotation(false));
 
}

fn rotate(mut query: Query<&mut Transform, With<Shape>>,
    time: Res<Time>) {
    
    for mut transform in &mut query {
        transform.rotate_y(time.delta_seconds() / 2.);
    }
    
}

fn camera_controls(
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Shape>)>,
    keyboard: Res<Input<KeyCode>>, 
    time: Res<Time>,
    mut contexts: bevy_egui::EguiContexts,
){

    if !contexts.ctx_mut().is_pointer_over_area() && !contexts.ctx_mut().wants_keyboard_input(){
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

}

fn ui_overlay(mut contexts: EguiContexts, mut settings: ResMut<VisualizationSettings>){
    //Move current resource values to mutable variables for sliders
    let (mut red, mut green, mut blue) = settings.component_limit.to_tuple();
    let (mut red_gamma, mut green_gamma, mut blue_gamma) = settings.gamma.to_tuple();
    let (mut hue_adjust, mut chroma_adjust, mut luminance_adjust) = settings.hcl_adjust;
    let mut v_shape = settings.visualization_shape.clone();

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
        ui.label("HCL Adjust");
        ui.add(egui::Slider::new( &mut hue_adjust ,3..=48).text("Hue"));
        ui.add(egui::Slider::new( &mut chroma_adjust ,1..=24).text("Chroma"));
        ui.add(egui::Slider::new( &mut luminance_adjust ,1..=16).text("Luminance"));
        ui.separator();
        ui.label("Shape");
        ui.horizontal(|ui| {
            ui.radio_value(&mut v_shape, VisualiztionShape::Spherical, "Sphere");
            ui.radio_value(&mut v_shape, VisualiztionShape::Cubic, "Cube");
        })

    });

    //Check if slider has been changed
    let slider_changed = 
        (red,green,blue) != settings.component_limit.to_tuple() ||
        (red_gamma,green_gamma,blue_gamma) != settings.gamma.to_tuple() ||
        (hue_adjust,chroma_adjust,luminance_adjust) != settings.hcl_adjust ||
        (v_shape != settings.visualization_shape);

    if slider_changed{
        //Update values to Resource
        settings.component_limit = SColor::from_tuple((red,green,blue));
        settings.gamma = Gamma::new(red_gamma,green_gamma,blue_gamma);
        settings.hcl_adjust = (hue_adjust,chroma_adjust,luminance_adjust);
        settings.visualization_shape = v_shape;

        //Update mesh
        //todo
        println!("Reached");
        settings.visualization_needs_updated = true;

    }
}

fn update_visualization(
    mut commands: Commands,
    mut defined_color: ResMut<VisualizationSettings>,
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
    defined_color: VisualizationSettings)
{

    let (x_scale, y_scale, z_scale) = defined_color.component_limit.to_tuple();

    let spherical_rgb_meshes = draw_spherical_colorspace(defined_color);
    for(_index, mesh) in spherical_rgb_meshes.iter().enumerate() {
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(mesh.clone()),
                material: materials.add(StandardMaterial {
                    base_color: Color::rgb(1.,1.,1.),
                    unlit: true,
                    cull_mode: None,
                    emissive: Color::rgb(1.0, 1.0, 1.0),// Set emissive color
                    ..Default::default()
                }),
                transform: Transform::from_scale(Vec3 { x: SCALE*x_scale, y: SCALE*y_scale, z: SCALE*z_scale }),
                ..Default::default()
            },
            SphericalVisualizationMeshes,
        ));

    }

}


fn create_quad(v0: Vec3, v1: Vec3, v2: Vec3, v3: Vec3, settings: VisualizationSettings) -> Mesh {
    
    let (r_gamma,g_gamma,b_gamma) = settings.gamma.to_tuple();
    let gamma = Gamma::new(r_gamma/2.2,g_gamma/2.2,b_gamma/2.2);
    let (c0,c1,c2,c3) = (
        DefinedColor::new(SColor::new(v0.x,v0.y,v0.z),gamma).to_color(),
        DefinedColor::new(SColor::new(v1.x,v1.y,v1.z),gamma).to_color(),
        DefinedColor::new(SColor::new(v2.x,v2.y,v2.z),gamma).to_color(),
        DefinedColor::new(SColor::new(v3.x,v3.y,v3.z),gamma).to_color(),
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

fn draw_spherical_colorspace(settings: VisualizationSettings) -> Vec<Mesh>{
    let mut colorspace_meshes  = Vec::<Mesh>::new();

    let (h_step,s_step,v_step) = settings.hcl_adjust;

    for v in 0..v_step {
        let s_step_adjusted = s_step/(v_step-v);
        for s in 0..s_step_adjusted {
            for h in  0..h_step{   

                // let (red, green, blue) = spherical_hcl(h as f32 / h_step as f32,1.-(s as f32 /(s_step/(v_step-v)) as f32),v as f32 / v_step as f32).to_tuple();

                // if  red > defined_color.component_limit.r ||
                //     green > defined_color.component_limit.g ||
                //     blue > defined_color.component_limit.b  {
                //     break;
                // }

                let (hue,hue_next) = (h as f32 / h_step as f32, ((h+1) % h_step) as f32 / h_step as f32);
                let (chroma, chroma_next) = (1.-(s as f32 /(s_step/(v_step-v)) as f32), 1.-((s+1) as f32 /(s_step/(v_step-v)) as f32));
                let luminance = v as f32 / v_step as f32;
                
                let (v0, v1, v2, v3);

                match settings.visualization_shape {
                    VisualiztionShape::Spherical =>
                        (v0,v1,v2,v3)= (
                            Vec3::from(spherical_hcl(hue,chroma, luminance).to_tuple()),
                            Vec3::from(spherical_hcl(hue_next, chroma, luminance).to_tuple()),
                            Vec3::from(spherical_hcl(hue, chroma_next, luminance).to_tuple()),
                            Vec3::from(spherical_hcl(hue_next, chroma_next, luminance).to_tuple()),
                        ),
                    VisualiztionShape::Cubic =>
                        (v0,v1,v2,v3)= (
                            Vec3::from(cubic_hsv(hue,chroma, luminance).to_tuple()),
                            Vec3::from(cubic_hsv(hue_next, chroma, luminance).to_tuple()),
                            Vec3::from(cubic_hsv(hue, chroma_next, luminance).to_tuple()),
                            Vec3::from(cubic_hsv(hue_next, chroma_next, luminance).to_tuple()),
                        ),
                }

                colorspace_meshes.push(create_quad(
                    v0,
                    v1,
                    v2,
                    v3,
                    settings.clone(),
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