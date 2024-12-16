use bevy::render::render_asset::RenderAssetUsages;
use prismatic_color::{Color as P_Color, ColorModel, ColorSpace, IntoColor};

use bevy::{
    prelude::{*},
    render::render_resource::PrimitiveTopology,
    render::mesh::Indices,
};

use crate::ui::VisualizationSettings;

#[derive(Debug, Clone)]
struct ColorQuad {
    points: [P_Color;4],
}

// A marker component for our components so we can query them separately from the ground plane
#[derive(Component)]
pub struct VisualizationMeshes;



#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RotationDirection {
    None,
    Clockwise,
    Counterclockwise,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorModelCategory {
    Spherical,           // Hold hue constant, vary chroma and lightness
    Cubic,            // Hold chroma constant, vary hue and lightness
    LumaChroma, // Hold lightness constant, vary hue and chroma
}



#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SlicingMethod {
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
pub enum MeshShape {
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
pub const SCALE: f32 = 5.0;

trait BevyColorConvert {
    fn to_bevy_color(&self) -> Color;
}

impl BevyColorConvert for P_Color {
    fn to_bevy_color(&self) -> Color {
        let color = self.to_rgb().to_array();
        Color::srgba(color[0], color[1], color[2], color[3])
    }
}

pub fn spawn_spherical_visualization(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    settings: &VisualizationSettings)
{

    if settings.is_instance_visualization {
        let mesh = settings.mesh_shape.get_shape(settings.instance_scale);
        let entities: Vec<(Mesh3d, MeshMaterial3d<StandardMaterial>, Transform, VisualizationMeshes)> = generate_point_colors(&settings).iter().map(|color|{
            let (point, color) = get_point_and_color(color.clone(), settings);
            (
                Mesh3d(meshes.add(mesh.clone())),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: color.to_bevy_color(),
                    unlit: true,
                    emissive: color.to_bevy_color().to_linear(),
                    ..Default::default()
                })),
                Transform::from_translation(point.map(|axis| axis * SCALE * settings.viz_scale)),
                VisualizationMeshes,
            )
        }).collect();
        commands.spawn_batch(entities);
        
    } else {
        let quad_meshes: Vec<Mesh> = generate_quads(&settings).iter().map(|color_quad| create_quad(color_quad.clone(), settings)).collect();

        for mesh in quad_meshes {
            commands.spawn((
                Mesh3d(meshes.add(mesh.clone())),
                MeshMaterial3d(materials.add(StandardMaterial {
                    unlit: true,
                    cull_mode: None,
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
                VisualizationMeshes,
            ));
        }
        
    }

}


//Need to figure out how to merge points and quads
fn generate_point_colors(settings: &VisualizationSettings) -> Vec<P_Color> {
    let (a_steps,b_steps,c_steps) = settings.hcl_adjust;
    let color_model: ColorModel = settings.color_model;
    let yuv_offset = if settings.color_model.is_luma_chroma() {-0.5} else {0.};
    
    let (a_step,b_step,c_step) = (1. / a_steps as f32, 1. / b_steps as f32, 1. / c_steps as f32 );
    let mut points = Vec::new();
    let one: f32 = 1.;
    let hwb_offset = if settings.is_chroma_luma { 0} else { 1};

    for a in 0..a_steps {
        for b in (0 + hwb_offset)..(b_steps + hwb_offset) {
            for c in (0 + hwb_offset)..(c_steps + hwb_offset) {
                // Generate the four points of the quad
                let point: P_Color = 
                    (
                        a as f32 * a_step,
                        b as f32 * b_step + yuv_offset,
                        c as f32 * c_step + yuv_offset,
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
    let (a_steps,b_steps,c_steps) = settings.hcl_adjust;
    let method: SlicingMethod = settings.quad_shape;
    let color_model: ColorModel = settings.color_model;
    
    let (a_step,b_step,c_step) = (1. / a_steps as f32, 1. / b_steps as f32, 1. / c_steps as f32 );
    let quad_offsets = method.get_offsets(); // Get the offsets for this slicing method
    let mut quads = Vec::new();
    let one: f32 = 1.;
    let hwb_offset = if settings.is_chroma_luma {0} else {1};
    let quad_direction = if settings.is_chroma_luma {1.} else {-1.};
    let yuv_offset = if settings.color_model.is_luma_chroma() {-0.5} else {0.};

    for a in 0..a_steps {
        for b in (0 + hwb_offset)..(b_steps + hwb_offset) {
            for c in (0 + hwb_offset * 2)..(c_steps + hwb_offset) {
                // Generate the four points of the quad
                let points: [P_Color; 4] = std::array::from_fn(|n| {
                    (
                        (a as f32 + quad_offsets[n][0]) * a_step,
                        (b as f32 + quad_offsets[n][1] * quad_direction) * b_step + yuv_offset,
                        (c as f32 + quad_offsets[n][2] * quad_direction)* c_step + yuv_offset,
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

fn get_point_and_color(base_color: P_Color, settings: &VisualizationSettings) -> (Vec3, P_Color){
    let (r_gamma,g_gamma,b_gamma) = if settings.gamma_deform {(1.,1.,1.)} else {settings.gamma};
    let gamma_adjust = 2.2;
    let gamma = [
        (r_gamma/gamma_adjust) as f32,
        (g_gamma/gamma_adjust) as f32,
        (b_gamma/gamma_adjust) as f32,
    ];

    let raw_color = base_color.to_rgb();
    let chroma = base_color.to_array()[1];
    
    let mut point: Vec3 = {
        let point = base_color.to_color(settings.color_space_model).from_space_to_space(settings.color_space, ColorSpace::XYZ);
        let point = match settings.model_rotation {
            RotationDirection::None => point,
            RotationDirection::Clockwise => point.rotate_colorspace_clockwise(),
            RotationDirection::Counterclockwise => point.rotate_colorspace_counterclockwise(),
        };
        let point = if settings.model_mirrored {point.mirror_colorspace()} else {point};
        let (x,y,z, _) = point.to_tuple(); 
        Vec3 {x, y, z}
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
            let color = color.to_color(settings.color_space_model);
            let (x,y,z, _) = color.from_space_to_space(settings.color_space, ColorSpace::XYZ).to_tuple(); 
            Vec3 {x, y, z}
        };
    }

    (point, color)
} 