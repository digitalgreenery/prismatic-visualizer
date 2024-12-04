//Digital Greenery
//Prismatic Color Visualizer

use bevy::render::render_asset::RenderAssetUsages;
use prismatic_color::{constants as Colors, Color as P_Color, ColorType, IntoColor};
use std::f32::consts::PI;
use bevy::input::mouse::{MouseMotion, MouseButtonInput};

use bevy::{
    prelude::{*},
    render::render_resource::{/*Extent3d, TextureDimension, TextureFormat,*/ PrimitiveTopology},
    render::mesh::Indices,
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};

trait BevyColorConvert {
    fn to_bevy_color(&self) -> Color;
}

impl BevyColorConvert for P_Color {
    fn to_bevy_color(&self) -> Color {
        let color = self.to_rgb().to_array();
        Color::srgba(color[0], color[1], color[2], color[3])
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(EguiPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (rotate,camera_controls,ui_overlay,update_visualization))
        .run();
}

// A marker component for our components so we can query them separately from the ground plane
#[derive(Component)]
struct Shape;

#[derive(Component)]
struct SphericalVisualizationMeshes;

#[derive(Resource, Clone)]
struct VisualizationSettings{
    viz_scale: f32,
    instance_scale: f32,
    component_limit: (f32,f32,f32),
    gamma: (f32,f32,f32),
    hcl_adjust: (u8,u8,u8),
    is_chroma_luma: bool,
    color_model: ColorType,
    is_instance_visualization: bool,
    mesh_shape: MeshShape,
    quad_shape: SlicingMethod,
    gamma_deform: bool,
    discrete_color: bool,
}

impl Default for VisualizationSettings{
    fn default() -> Self {
        Self {
            viz_scale: 1.,
            instance_scale: 0.1,
            component_limit: (1., 1., 1.), 
            gamma: (2.2, 2.2, 2.2), 
            hcl_adjust: (12,8,8),
            is_chroma_luma: false,
            color_model: ColorType::SphericalHCLA,
            is_instance_visualization: true,
            mesh_shape: MeshShape::Sphere,
            quad_shape: SlicingMethod::Axial,
            gamma_deform: false,
            discrete_color: true,
        }
    }
}

#[derive(Debug, Clone)]
struct ColorQuad {
    points: [P_Color;4],
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SlicingMethod {
    Radial,           // Hold hue constant, vary chroma and lightness
    Axial,            // Hold chroma constant, vary hue and lightness
    Concentric, // Hold lightness constant, vary hue and chroma
}

impl SlicingMethod {
    fn get_offsets(&self) -> [[f32; 3]; 4] {
        match self {
            SlicingMethod::Radial => [
                [0.0, 0.0, 0.0], // Point 1: Base point
                [0.0, 1.0, 0.0], // Point 2: Vary chroma
                [0.0, 1.0, 1.0], // Point 3: Vary chroma and lightness
                [0.0, 0.0, 1.0], // Point 4: Vary lightness
            ],
            SlicingMethod::Concentric => [
                [0.0, 0.0, 0.0], // Point 1: Base point
                [1.0, 0.0, 0.0], // Point 2: Vary hue
                [1.0, 0.0, 1.0], // Point 3: Vary hue and lightness
                [0.0, 0.0, 1.0], // Point 4: Vary lightness
            ],
            SlicingMethod::Axial => [
                [0.0, 0.0, 0.0], // Point 1: Base point
                [1.0, 0.0, 0.0], // Point 2: Vary hue
                [1.0, 1.0, 0.0], // Point 3: Vary hue and chroma
                [0.0, 1.0, 0.0], // Point 4: Vary chroma
            ],
        }
    }
}

#[derive(Clone,PartialEq)]
enum MeshShape {
    Sphere,
    Cube,
}

impl MeshShape {
    fn get_shape(&self, scale: f32) -> Mesh {
        match self {
            MeshShape::Sphere => Sphere::new(scale).into(),
            MeshShape::Cube => Cuboid::new(scale,scale,scale).into(),
        }
    }
}

// const X_EXTENT: f32 = 14.5;
const SCALE: f32 = 5.0;

fn setup(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    // mut images: ResMut<Assets<Image>>,
    materials: ResMut<Assets<StandardMaterial>>,
) {

    commands.spawn((
        Camera3d {..Default::default()},
        Transform::from_xyz(SCALE*2., SCALE*2., SCALE*2.).looking_at(Vec3::new(0., 0., 0.), Vec3::Z),
    ));

    let settings = VisualizationSettings::default();

    let settings_copy = settings.clone();

    commands.insert_resource(settings);

    spawn_spherical_visualization(commands, meshes, materials, &settings_copy);

 
}

fn rotate(mut query: Query<&mut Transform, With<Shape>>,
    time: Res<Time>) {
    
    for mut transform in &mut query {
        transform.rotate_y(time.delta_secs() / 2.);
    }
    
}

fn camera_controls(
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Shape>)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    // mouse_button: Res<MouseButton>,
    // mouse_motion: Res<MouseMotion>,
    time: Res<Time>,
    mut contexts: bevy_egui::EguiContexts,
){
    if !contexts.ctx_mut().is_pointer_over_area() && !contexts.ctx_mut().wants_keyboard_input(){
        for mut camera_transform in &mut camera_query {

            let boost = if keyboard.pressed(KeyCode::ShiftLeft) {2.} else {0.};
            let speed = 2. + boost;
    
            // Define the camera's rotation speed in radians per second
            let camera_rotation_speed_horizontal = 
                if keyboard.pressed(KeyCode::KeyQ)||keyboard.pressed(KeyCode::ArrowLeft){
                    speed
                }
                else if keyboard.pressed(KeyCode::KeyE)||keyboard.pressed(KeyCode::ArrowRight) {
                    -speed
                }
                else {
                    0.0
            };
    
            let camera_rotation_speed_vertical = 
                if keyboard.pressed(KeyCode::KeyR)||keyboard.pressed(KeyCode::ArrowUp){
                    speed
                }
                else if keyboard.pressed(KeyCode::KeyF)||keyboard.pressed(KeyCode::ArrowDown) {
                    -speed
                }
                else {
                    0.0
            };
    
            let camera_speed_horizontal = 
                if keyboard.pressed(KeyCode::KeyD){
                    speed
                }
                else if keyboard.pressed(KeyCode::KeyA) {
                    -speed
                }
                else {
                    0.0
            };
    
            let camera_speed_forward = 
                if keyboard.pressed(KeyCode::KeyW){
                    speed
                }
                else if keyboard.pressed(KeyCode::KeyS) {
                    -speed
                }
                else {
                    0.0
            };
    
            let camera_speed_vertical =
            if keyboard.pressed(KeyCode::Space){
                speed
            }
            else if keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::KeyC) {
                -speed
            }
            else {
                0.0
            };
    
            let time_delta = time.delta_secs();
    
            // Calculate the camera's rotation angle based on time and speed
            let camera_rotation_angle_horizontal = time_delta * camera_rotation_speed_horizontal;
            let camera_rotation_angle_vertical = time_delta * camera_rotation_speed_vertical;
            let camera_vertical = time_delta * camera_speed_vertical;
            let camera_horizontal = time_delta * camera_speed_horizontal;
            let camera_forward = time_delta * camera_speed_forward;
    
            let side_movement = camera_transform.local_x().as_vec3();
            let forward_movement = camera_transform.local_z().as_vec3();
    
            camera_transform.rotate(Quat::from_rotation_z(camera_rotation_angle_horizontal) * Quat::from_axis_angle(side_movement, camera_rotation_angle_vertical));
            camera_transform.translation.z += camera_vertical;
            camera_transform.translation +=  (Vec3::new(forward_movement.x,forward_movement.y,0.) * -camera_forward) + (side_movement * camera_horizontal);
        }
    }

}

fn ui_overlay(mut contexts: EguiContexts, mut settings: ResMut<VisualizationSettings>){

    //Create window for variable sliders
    egui::Window::new("Spherical RGB Adjust").resizable(false).anchor(egui::Align2::LEFT_TOP, [5.,5.]).show(contexts.ctx_mut(), |ui|{
        
        ui.set_max_width(ui.available_width()/2.);

        ui.label("Scale");
        ui.add(egui::Slider::new( &mut settings.viz_scale ,0.0..=2.0).text("Visualization Scale"));

        ui.label("Perceptual Offset");
        ui.add(egui::Slider::new( &mut settings.component_limit.0 ,0.0..=1.0).text("Red"));
        ui.add(egui::Slider::new( &mut settings.component_limit.1 ,0.0..=1.0).text("Green"));
        ui.add(egui::Slider::new( &mut settings.component_limit.2 ,0.0..=1.0).text("Blue"));
        ui.label("Gamma");
        ui.add(egui::Slider::new( &mut settings.gamma.0 ,0.1..=3.0).text("Red"));
        ui.add(egui::Slider::new( &mut settings.gamma.1 ,0.1..=3.0).text("Green"));
        ui.add(egui::Slider::new( &mut settings.gamma.2 ,0.1..=3.0).text("Blue"));

        ui.separator();

        ui.label("Color Type");
        ui.horizontal(|ui| {
            ui.radio_value(&mut settings.is_chroma_luma, true, "HCL");
            ui.radio_value(&mut settings.is_chroma_luma, false, "HWB");
        });

        if settings.is_chroma_luma {
            ui.label("HCL");
            ui.add(egui::Slider::new( &mut settings.hcl_adjust.0 ,1..=48).text("Hue"));
            ui.add(egui::Slider::new( &mut settings.hcl_adjust.1 ,1..=24).text("Chroma"));
            ui.add(egui::Slider::new( &mut settings.hcl_adjust.2 ,1..=24).text("Luma"));
        }
        else {
            ui.label("HWB");
            ui.add(egui::Slider::new( &mut settings.hcl_adjust.0 ,1..=48).text("Hue"));
            ui.add(egui::Slider::new( &mut settings.hcl_adjust.1 ,1..=24).text("White"));
            ui.add(egui::Slider::new( &mut settings.hcl_adjust.2 ,1..=24).text("Black"));
        }

        ui.separator();

        ui.label("Color Model");
        ui.horizontal(|ui| {
            ui.radio_value(&mut settings.color_model, ColorType::SphericalHCLA, "Spherical");
            ui.radio_value(&mut settings.color_model, ColorType::CubicHSVA, "Cubic");
        });

        ui.separator();

        ui.label("Shape");
        ui.horizontal(|ui| {
            ui.radio_value(&mut settings.is_instance_visualization, true, "Shapes");
            ui.radio_value(&mut settings.is_instance_visualization, false, "Slices");
        });
  
        if settings.is_instance_visualization {
            ui.label("Mesh Shape");
            ui.horizontal(|ui| {
                ui.radio_value(&mut settings.mesh_shape, MeshShape::Sphere, "Spheres");
                ui.radio_value(&mut settings.mesh_shape, MeshShape::Cube, "Cubes");
                ui.add(egui::Slider::new( &mut settings.instance_scale ,0.0..=2.0).text("Shape Scale"));
            });
        }
        else {
            ui.label("Quad Direction");
            ui.horizontal(|ui| {
                ui.radio_value(&mut settings.quad_shape, SlicingMethod::Axial, "Axial");
                ui.radio_value(&mut settings.quad_shape, SlicingMethod::Radial, "Radial");
                ui.radio_value(&mut settings.quad_shape, SlicingMethod::Concentric, "Concentric");
                ui.checkbox(&mut settings.discrete_color, "Discrete Color");
            });
        }

        ui.separator();

        ui.label("Additional Settings");

        ui.checkbox(&mut settings.gamma_deform, "Gamma Deform");

        ui.separator();

        ui.label("WASD - Horizontal Movement");
        ui.label("Ctrl & Space - Vertical Movement");
        ui.label("Arrow Keys - Camera");

    });

    // egui::Window::new("Colorspace Info").resizable(false).anchor(egui::Align2::RIGHT_TOP, [5.,-5.]).show(contexts.ctx_mut(), |ui|{
       
    //     ui.collapsing("Quaternary Peaks", |ui| {
    //     //    ui.painter().rect_filled(egui::Rect::from_two_pos(egui::Pos2::ZERO, egui::Pos2::new(200., 200.)), egui::Rounding::ZERO, egui::Color32::from_rgb(255,0,0));
    //         ui.painter().circle(
    //             egui::Pos2{x:250.0,y:250.0},
    //             50.0, 
    //             egui::Color32::from_rgb(255,0,0), 
    //             egui::Stroke{width: 5.0, color: egui::Color32::from_rgb(255,0,0)}
    //         );
        
    //     });

    // });
   
}

fn update_visualization(
    mut commands: Commands,
    visualization_settings: ResMut<VisualizationSettings>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    entities: Query<Entity, With<SphericalVisualizationMeshes>>)
{


    if visualization_settings.is_changed() {

        //Delete previous visualization 
        for mesh in entities.iter(){
            commands.entity(mesh).despawn();
        }
 
    spawn_spherical_visualization(commands, meshes, materials, & *visualization_settings);
 
    }
}

fn spawn_spherical_visualization(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    settings: &VisualizationSettings)
{

    if settings.is_instance_visualization {
        let mesh = settings.mesh_shape.get_shape(settings.instance_scale);
        for color in generate_point_colors(&settings) {
            let (point, color) = get_point_and_color(color, settings);
            commands.spawn((
                Mesh3d(meshes.add(mesh.clone())),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: color.to_bevy_color(),
                    unlit: true,
                    cull_mode: None,
                    emissive: color.to_bevy_color().to_linear(),
                    ..Default::default()
                })),
                Transform::from_translation(point.map(|axis| axis * SCALE * settings.viz_scale)),
                GlobalTransform::default(),
                Visibility::default(),      // To control rendering visibility
                InheritedVisibility::default(), // For frustum culling
                SphericalVisualizationMeshes,
            ));
        };
        
    } else {
        let quad_meshes: Vec<Mesh> = generate_quads(&settings).iter().map(|color_quad| create_quad(color_quad.clone(), settings)).collect();

        for mesh in quad_meshes {
            commands.spawn((
                Mesh3d(meshes.add(mesh.clone())),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::WHITE,
                    unlit: true,
                    cull_mode: None,
                    emissive: Color::WHITE.to_linear(),
                    ..Default::default()
                })),
                Transform::from_scale(Vec3 {
                    x: SCALE * settings.viz_scale,
                    y: SCALE * settings.viz_scale,
                    z: SCALE * settings.viz_scale,
                }),
                GlobalTransform::default(), // This is required for the Transform system
                Visibility::default(),      // To control rendering visibility
                InheritedVisibility::default(), // For frustum culling
                SphericalVisualizationMeshes,
            ));
        }
        
    }

}

fn generate_point_colors(settings: &VisualizationSettings) -> Vec<P_Color> {
    let (h_steps,c_steps,l_steps) = settings.hcl_adjust;
    let color_model: ColorType = settings.color_model;
    
    let (h_step,c_step,l_step) = (1. / h_steps as f32, 1. / c_steps as f32, 1. / l_steps as f32 );
    let mut points = Vec::new();
    let one: f32 = 1.;
    let hwb_offset = if settings.is_chroma_luma { 0} else { 1};

    for h in 0..h_steps {
        for c in (0 + hwb_offset)..(c_steps + hwb_offset) {
            for l in (0 + hwb_offset)..(l_steps + hwb_offset) {
                // Generate the four points of the quad
                let point: P_Color = 
                    (
                        h as f32 * h_step,
                        c as f32 * c_step,
                        l as f32 * l_step,
                        one,
                    )
                    .into_color(color_model);

                // Add the quad to the list
                points.push(point);
            }
        }
    }

    points
}

fn generate_quads(settings: &VisualizationSettings) -> Vec<ColorQuad> {
    let (h_steps,c_steps,l_steps) = settings.hcl_adjust;
    let method: SlicingMethod = settings.quad_shape;
    let color_model: ColorType = settings.color_model;
    
    let (h_step,c_step,l_step) = (1. / h_steps as f32, 1. / c_steps as f32, 1. / l_steps as f32 );
    let quad_offsets = method.get_offsets(); // Get the offsets for this slicing method
    let mut quads = Vec::new();
    let one: f32 = 1.;
    let hwb_offset = if settings.is_chroma_luma {0} else {1};
    let quad_direction = if settings.is_chroma_luma {1.} else {-1.};

    for h in (0 + hwb_offset)..(h_steps+hwb_offset) {
        for c in (0 + hwb_offset)..(c_steps + hwb_offset) {
            for l in (0 + hwb_offset * 2)..(l_steps + hwb_offset) {
                // Generate the four points of the quad
                let points: [P_Color; 4] = std::array::from_fn(|n| {
                    (
                        (h as f32 + quad_offsets[n][0] * quad_direction) * h_step,
                        (c as f32 + quad_offsets[n][1] * quad_direction) * c_step,
                        (l as f32 + quad_offsets[n][2] * quad_direction)* l_step,
                        one,
                    )
                    .into_color(color_model)
                });

                // Add the quad to the list
                quads.push(ColorQuad {points: points});
            }
        }
    }

    quads
}

fn create_quad(color_quad: ColorQuad, settings: &VisualizationSettings) -> Mesh {

   let points_and_colors: [(Vec3, P_Color); 4] = color_quad.points.map(|color| get_point_and_color(color, settings));
   let (quad,colors) = (points_and_colors.map(|(a,_)|a).to_vec(), points_and_colors.map(|(_,b)|b));
    // Create a new mesh using a triangle list topology, where each set of 3 vertices composes a triangle.
    Mesh::new(
        PrimitiveTopology::TriangleList, 
        RenderAssetUsages::RENDER_WORLD,
    )
        // Add 4 vertices, each with its own position attribute (coordinate in
        // 3D space), for each of the corners of the parallelogram.
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            quad,
        )
        // Assign color to each vertex based on its xyz values.
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_COLOR,
            
            if settings.discrete_color {
                vec! [
                    colors[0].to_array(),
                    colors[0].to_array(),
                    colors[0].to_array(),
                    colors[0].to_array(),
                ]
            }
            else {
                vec![
                    colors[0].to_array(),
                    colors[1].to_array(),
                    colors[2].to_array(),
                    colors[3].to_array(),
                ]
            }
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
        .with_inserted_indices(Indices::U32(vec![
            // First triangle
            0, 1, 2,
            // Second triangle
            2, 3, 0,
        ]))
}

fn get_point_and_color(color: P_Color, settings: &VisualizationSettings) -> (Vec3, P_Color){
    let (r_gamma,g_gamma,b_gamma) = if settings.gamma_deform {(1.,1.,1.)} else {settings.gamma};
    let gamma_adjust = 2.2;
    let gamma = [
        (r_gamma/gamma_adjust) as f32,
        (g_gamma/gamma_adjust) as f32,
        (b_gamma/gamma_adjust) as f32,
    ];

    let raw_color = color.to_rgb();
    let chroma = color.to_array()[1];
    
    let mut point: Vec3 = {
        let color = raw_color.to_array();
        Vec3 {x: color[0], y: color[1], z:color[2],}
    };

    let color: P_Color = 
        raw_color.
            remap_rgb_components(
            chroma, 
            settings.component_limit.0, 
            settings.component_limit.1, 
            settings.component_limit.2
            ).
            component_gamma_transform(
                gamma[0],
                gamma[1], 
                gamma[2],
            );

    if settings.gamma_deform {
        point = {
            let color = color.to_array();
            Vec3 {x: color[0], y: color[1], z:color[2],}
        };
    }

    (point, color)
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