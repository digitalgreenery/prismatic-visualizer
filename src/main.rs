//Digital Greenery
//Prismatic Color Visualizer

use bevy::prelude::{*};
use bevy_egui::EguiPlugin;

mod camera;
use camera::camera_controls;

mod ui;
use ui::{ui_overlay, VisualizationSettings};

mod visualization;
use visualization::{spawn_3d_visualization, VisualizationMeshes, SCALE};


fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(EguiPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (ui_overlay,update_visualization))
        .add_systems(FixedUpdate, camera_controls)
        .run();
}

fn setup(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    // mut images: ResMut<Assets<Image>>,
    materials: ResMut<Assets<StandardMaterial>>,
) {

    //Needs moved into camera.rs
    commands.spawn((
        Camera3d {..Default::default()},
        Transform::from_xyz(SCALE*2., SCALE*2., SCALE*2.).looking_at(Vec3::new(0., 0., 0.), Vec3::Z),
    ));

    let settings = VisualizationSettings::default();

    let settings_copy = settings.clone();

    commands.insert_resource(settings);

    spawn_3d_visualization(commands, meshes, materials, &settings_copy);

 
}

fn update_visualization(
    mut commands: Commands,
    visualization_settings: ResMut<VisualizationSettings>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    entities: Query<Entity, With<VisualizationMeshes>>)
{


    if visualization_settings.is_changed() {

        //Delete previous visualization 
        for mesh in entities.iter(){
            commands.entity(mesh).despawn();
        }
 
    spawn_3d_visualization(commands, meshes, materials, & *visualization_settings);
 
    }
}