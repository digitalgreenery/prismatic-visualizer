use bevy::{color, gizmos::gizmos, render::render_asset::RenderAssetUsages};
use bevy_egui::egui::emath::OrderedFloat;
use indexmap::IndexMap;
use prismatic_color::{Color as P_Color, ColorModel, ColorSpace, IntoColor, };

use bevy_pointcloud::{point_cloud, PointCloudPlugin};
use bevy_pointcloud::loader::las::LasLoaderPlugin;
use bevy_pointcloud::point_cloud::{PointCloud, PointCloud3d, PointCloudData};
use bevy_pointcloud::point_cloud_material::{PointCloudMaterial, PointCloudMaterial3d};
use bevy_pointcloud::render::PointCloudRenderMode;

use bevy::{
    prelude::{*},
    render::render_resource::PrimitiveTopology,
    render::mesh::Indices,
};

use crate::ui::VisualizationSettings;

// A marker component for our components so we can query them separately from the ground plane
#[derive(Component)]
pub struct VisualizationMesh;



#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RotationDirection {
    None,
    Clockwise,
    Counterclockwise,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorModelCategory {
    Spherical,
    Cubic,
    LumaChroma,
}



#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SlicingMethod {
    X,
    Y,
    Z,
}

impl SlicingMethod {
    fn get_face_offsets(&self) -> [[usize; 3]; 4] {
        match self {
            SlicingMethod::X => [
                [0, 0, 0],
                [0, 1, 0],
                [0, 1, 1],
                [0, 0, 1],
            ],
            SlicingMethod::Y => [                
                [0, 0, 0], 
                [1, 0, 0], 
                [1, 0, 1], 
                [0, 0, 1], 
            ],
            SlicingMethod::Z => [
                [0, 0, 0],
                [1, 0, 0],
                [1, 1, 0],
                [0, 1, 0], 
            ],

        }
    }

    fn get_edge_offsets(&self) -> [[usize; 3]; 2] {
        match self {
            SlicingMethod::X => [
                [0, 0, 0],
                [1, 0, 0],
            ],
            SlicingMethod::Y => [
                [0, 0, 0],
                [0, 1, 0], 
            ],
            SlicingMethod::Z => [
                [0, 0, 0], 
                [0, 0, 1], 
            ],

        }
    }

    fn get_vertex_offset(&self) -> [[usize; 3]; 1] {
        [[0,0,0]]
    }

}

#[derive(Clone,PartialEq)]
pub enum VertexShape {
    Sphere,
    Cube,
    Tetrahedron,
}

impl VertexShape {
    fn get_shape(&self, scale: f32) -> Mesh {
        match self {
            VertexShape::Sphere => Sphere::new(scale).into(),
            VertexShape::Cube => Cuboid::new(scale,scale,scale).into(),
            VertexShape::Tetrahedron => Tetrahedron::new(
                Vec3::new(0.5 * scale, 0.5 * scale, 0.5 * scale),
                Vec3::new(-0.5 * scale, 0.5 * scale, -0.5 * scale),
                Vec3::new(-0.5 * scale, -0.5 * scale, 0.5 * scale),
                Vec3::new(0.5 * scale, -0.5 * scale, -0.5 * scale),
            ).into()
        }
    }
}



#[derive(Clone,PartialEq)]
pub enum Dimensionality {
    Vertex,
    Edge,
    Face,
    Volume,
}

pub enum DimensionList {
    Vertex(VertexList),
    Edge(EdgeList),
    Face(FaceList),
    Volume(FaceList),
}

impl DimensionList {
    pub fn render(
        &self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        point_clouds: &mut Assets<PointCloud>,
        point_cloud_material: &mut Assets<PointCloudMaterial>,
        gizmos: &mut Gizmos,
        settings: &VisualizationSettings,
    ) {
        match self {
            DimensionList::Vertex(vertex_list) => {
                // Render vertices as a point cloud
                let points: Vec<PointCloudData> = vertex_list
                    .iter()
                    .map(|(vertex_object, _)| {
                        let position: Vec3 =
                            vertex_object.point.clone().into_vec3() * SCALE * settings.viz_scale;
                        let color = P_Color::from_array(
                            vertex_object.color.map(|x| x.into_inner()),
                            settings.color_model,
                        )
                        .to_bevy_color()
                        .to_srgba()
                        .to_f32_array();
                        PointCloudData {
                            position,
                            point_size: settings.instance_scale * SCALE,
                            color,
                        }
                    })
                    .collect::<Vec<_>>();

                    let my_material = point_cloud_material.add(PointCloudMaterial {
                        point_size: 50.0 * settings.instance_scale * SCALE,
                        ..default()
                    });

                    let point_cloud: Handle<PointCloud> = point_clouds.add(PointCloud{
                        points
                    });
                    commands.spawn((
                        PointCloud3d(point_cloud),
                        PointCloudMaterial3d(my_material.clone()),
                        VisualizationMesh,
                    )); 
             },
             DimensionList::Edge(edge_list) => {
                // Render edges with gizmo lines
                for edge in &edge_list.edges {
                    let vertex_1 = edge_list.vertex_registry.get_index(edge.0).unwrap().0;
                    let vertex_2 = edge_list.vertex_registry.get_index(edge.1).unwrap().0;
                    
                    if settings.discrete_color {
                        gizmos.line(vertex_1.point.map(|axis| axis.into_inner() * SCALE * settings.viz_scale).into(), vertex_2.point.map(|axis| axis.into_inner() * SCALE * settings.viz_scale).into(), vertex_1.color.into_color(settings));
                    } 
                    else {
                        gizmos.line_gradient(vertex_1.point.map(|axis| axis.into_inner() * SCALE * settings.viz_scale).into(), vertex_2.point.map(|axis| axis.into_inner() * SCALE * settings.viz_scale).into(), vertex_1.color.into_color(settings), vertex_2.color.into_color(settings));
                    }
                }
            },       
            DimensionList::Face(face_list) => {
                //Render faces with a triangle based mesh
                // Collect positions, normals, and colors
                let mut positions: Vec<[f32; 3]> = Vec::new();
                let mut normals: Vec<[f32; 3]> = Vec::new();
                let mut colors: Vec<[f32; 4]> = Vec::new();
                let mut indices: Vec<u32> = Vec::new();

                for (i1, i2, i3, i4) in &face_list.faces {
                    // Lookup vertices from registry
                    let v1 = face_list.vertex_registry.get_index(*i1).unwrap().0;
                    let v2 = face_list.vertex_registry.get_index(*i2).unwrap().0;
                    let v3 = face_list.vertex_registry.get_index(*i3).unwrap().0;
                    let v4 = face_list.vertex_registry.get_index(*i4).unwrap().0;

                    let verts = [v1, v2, v3, v4];

                    // Push positions/colors

                    
                    let base = positions.len() as u32;
                    for v in &verts {
                        positions.push(v.point.map(|p| p.into_inner() * SCALE * settings.viz_scale));
                        let color = 
                            if settings.discrete_color {
                                P_Color::from_array(v1.color.map(|x| x.into_inner()), settings.color_model)
                                .to_bevy_color()
                                .to_linear()
                                .to_f32_array()
                            }
                            else {
                                P_Color::from_array(v.color.map(|x| x.into_inner()), settings.color_model)
                                .to_bevy_color()
                                .to_linear()
                                .to_f32_array()
                        };
                        colors.push(color);
                    }

                    // Compute a simple normal (cross product of two edges)
                    let p1 = Vec3::from(positions[base as usize]);
                    let p2 = Vec3::from(positions[base as usize + 1]);
                    let p3 = Vec3::from(positions[base as usize + 2]);
                    let normal = (p2 - p1).cross(p3 - p1).normalize_or_zero();
                    for _ in 0..4 {
                        normals.push(normal.into());
                    }

                    // Add indices for two triangles: (0,1,2) and (0,2,3)
                    indices.extend_from_slice(&[
                        base, base + 1, base + 2,
                        base, base + 2, base + 3,
                    ]);
                }

                // Build mesh
                let mut mesh = Mesh::new(
                    bevy::render::mesh::PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(), // or RenderAssetUsages::RENDER_WORLD if you only need rendering
                );
                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
                mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
                mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
                mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));

                // Unlit so vertex colors are shown directly
                let material = materials.add(StandardMaterial {
                    base_color: Color::WHITE,
                    unlit: true,
                    ..default()
                });

                commands.spawn((
                     
                    Mesh3d( meshes.add(mesh)),
                    MeshMaterial3d(material),
                    VisualizationMesh,
                ));
            },
            DimensionList::Volume(face_list) => {
                //Render faces with a triangle based mesh
                // Collect positions, normals, and colors
                let mut positions: Vec<[f32; 3]> = Vec::new();
                let mut normals: Vec<[f32; 3]> = Vec::new();
                let mut colors: Vec<[f32; 4]> = Vec::new();
                let mut indices: Vec<u32> = Vec::new();

                for (i1, i2, i3, i4) in &face_list.faces {
                    // Lookup vertices from registry
                    let v1 = face_list.vertex_registry.get_index(*i1).unwrap().0;
                    let v2 = face_list.vertex_registry.get_index(*i2).unwrap().0;
                    let v3 = face_list.vertex_registry.get_index(*i3).unwrap().0;
                    let v4 = face_list.vertex_registry.get_index(*i4).unwrap().0;

                    let verts = [v1, v2, v3, v4];

                    // Push positions/colors

                    
                    let base = positions.len() as u32;
                    for v in &verts {
                        positions.push(v.point.map(|p| p.into_inner() * SCALE * settings.viz_scale));
                        let color = 
                            if settings.discrete_color {
                                P_Color::from_array(v1.color.map(|x| x.into_inner()), settings.color_model)
                                .to_bevy_color()
                                .to_linear()
                                .to_f32_array()
                            }
                            else {
                                P_Color::from_array(v.color.map(|x| x.into_inner()), settings.color_model)
                                .to_bevy_color()
                                .to_linear()
                                .to_f32_array()
                        };
                        colors.push(color);
                    }

                    // Compute a simple normal (cross product of two edges)
                    let p1 = Vec3::from(positions[base as usize]);
                    let p2 = Vec3::from(positions[base as usize + 1]);
                    let p3 = Vec3::from(positions[base as usize + 2]);
                    let normal = (p2 - p1).cross(p3 - p1).normalize_or_zero();
                    for _ in 0..4 {
                        normals.push(normal.into());
                    }

                    // Add indices for two triangles: (0,1,2) and (0,2,3)
                    indices.extend_from_slice(&[
                        base, base + 1, base + 2,
                        base, base + 2, base + 3,
                    ]);
                }

                // Build mesh
                let mut mesh = Mesh::new(
                    bevy::render::mesh::PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(), // or RenderAssetUsages::RENDER_WORLD if you only need rendering
                );
                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
                mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
                mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
                mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));

                // Unlit so vertex colors are shown directly
                let material = materials.add(StandardMaterial {
                    base_color: Color::WHITE,
                    unlit: true,
                    ..default()
                });

                commands.spawn((
                     
                    Mesh3d( meshes.add(mesh)),
                    MeshMaterial3d(material),
                    VisualizationMesh,
                ));
            },
        }
    }
}

trait IntoVec3 {
    fn into_vec3(self) -> Vec3;
}

impl IntoVec3 for [OrderedFloat<f32>; 3] {
    fn into_vec3(self) -> Vec3 {
        let [x, y, z] = self;
        Vec3::new(x.into_inner(), y.into_inner(), z.into_inner())
    }
}

trait OrderedArrayIntoColor {
    fn into_color(self, settings: &VisualizationSettings) -> Color;
}

impl OrderedArrayIntoColor for [OrderedFloat<f32>; 4] {
    fn into_color(self, settings: &VisualizationSettings) -> Color {
        P_Color::from_array(self.map(|x| x.into_inner()), settings.color_model).to_bevy_color()
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct VertexObject {
    pub point: [OrderedFloat<f32>;3],
    pub color: [OrderedFloat<f32>;4],
}

impl VertexObject {
    fn new(point: [f32;3], color: P_Color) -> VertexObject {
        VertexObject { point: point.map(OrderedFloat::from), color: color.to_array().map(OrderedFloat::from) }
    }
    fn from_tuple(point_color: ([f32;3], P_Color)) -> VertexObject {
        VertexObject::new(point_color.0,point_color.1)
    }
}

pub struct VertexList {
    registry: IndexMap<VertexObject, usize>,
}

impl VertexList {
    pub fn iter(&self) -> impl Iterator<Item = (&VertexObject, &usize)> {
        self.registry.iter()
    }
}

pub struct EdgeList {
    vertex_registry: IndexMap<VertexObject, usize>,
    edges: Vec<(usize, usize)>,
}

pub struct FaceList {
    vertex_registry: IndexMap<VertexObject, usize>,
    faces: Vec<(usize, usize, usize, usize)>,
}

// Common trait for vertex management
pub trait VertexCollection {
    fn add_vertex(&mut self, v1: &VertexObject) -> usize{
        self.get_or_insert_index(v1)
    }
    
    fn get_or_insert_index(&mut self, vertex: &VertexObject) -> usize {
        if let Some(&index) = self.vertex_registry().get(vertex) {
            index
        } else {
            let index = self.vertex_registry().len();
            self.vertex_registry_mut().insert(vertex.clone(), index);
            index
        }
    }
    
    fn vertex_registry(&self) -> &IndexMap<VertexObject, usize>;
    fn vertex_registry_mut(&mut self) -> &mut IndexMap<VertexObject, usize>;
}

impl VertexCollection for VertexList {
    fn vertex_registry(&self) -> &IndexMap<VertexObject, usize> {
        &self.registry
    }

    fn vertex_registry_mut(&mut self) -> &mut IndexMap<VertexObject, usize> {
        &mut self.registry
    }
}

impl VertexList {
    pub fn new() -> Self {
        Self {
            registry: IndexMap::new(),
        }
    }
}

impl EdgeList {
    pub fn new() -> Self {
        Self {
            vertex_registry: IndexMap::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_edge(&mut self, v1: VertexObject, v2: VertexObject) -> usize {
        let i1 = self.get_or_insert_index(&v1);
        let i2 = self.get_or_insert_index(&v2);
        self.edges.push((i1, i2));
        self.edges.len() - 1
    }
}

impl VertexCollection for EdgeList {

    fn vertex_registry(&self) -> &IndexMap<VertexObject, usize> {
        &self.vertex_registry
    }

    fn vertex_registry_mut(&mut self) -> &mut IndexMap<VertexObject, usize> {
        &mut self.vertex_registry
    }
}

impl FaceList {
    pub fn new() -> Self {
        Self {
            vertex_registry: IndexMap::new(),
            faces: Vec::new(),
        }
    }

    pub fn add_quad(
        &mut self,
        v1: VertexObject,
        v2: VertexObject,
        v3: VertexObject,
        v4: VertexObject,
    ) -> usize {
        let i1 = self.get_or_insert_index(&v1);
        let i2 = self.get_or_insert_index(&v2);
        let i3 = self.get_or_insert_index(&v3);
        let i4 = self.get_or_insert_index(&v4);
        self.faces.push((i1, i2, i3, i4));
        self.faces.len() - 1
    }
}

impl VertexCollection for FaceList {

    fn vertex_registry(&self) -> &IndexMap<VertexObject, usize> {
        &self.vertex_registry
    }

    fn vertex_registry_mut(&mut self) -> &mut IndexMap<VertexObject, usize> {
        &mut self.vertex_registry
    }
}


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

pub fn spawn_3d_visualization(
    mut gizmos: Gizmos,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut point_clouds: ResMut<Assets<PointCloud>>,
    mut point_cloud_materials: ResMut<Assets<PointCloudMaterial>>,
    settings: &VisualizationSettings)
{
    generate_dimension_lists(settings)
        .render(&mut commands, &mut meshes, &mut materials, &mut point_clouds ,&mut point_cloud_materials, &mut gizmos, settings);
}

fn wrap_index(index: usize, offset: usize, length: usize) -> usize {
    let raw = index + offset;
    return raw % length;
}


fn generate_dimension_lists(settings: &VisualizationSettings) ->  DimensionList{

    let yuv_offset = if settings.color_model.is_luma_chroma() {-0.5} else {0.};
    
    let mut dim_list: DimensionList = 
    match settings.dimensionality {
        Dimensionality::Vertex => DimensionList::Vertex(VertexList::new()),
        Dimensionality::Edge => DimensionList::Edge(EdgeList::new()),
        Dimensionality::Face => DimensionList::Face(FaceList::new()),
        Dimensionality::Volume => DimensionList::Volume(FaceList::new()),
    };


    let channel_a_list = settings.channel_settings.0.generate();
    let channel_b_list = settings.channel_settings.1.generate();
    let channel_c_list = settings.channel_settings.2.generate();

    for (index_of_a, a) in channel_a_list.iter().enumerate() {
        if settings.dimensionality != Dimensionality::Vertex && channel_a_list.len() < 1 {continue};
        for (index_of_b, b ) in channel_b_list.iter().enumerate() {
            if settings.dimensionality != Dimensionality::Vertex && channel_b_list.len() < 1 {continue};
            for (index_of_c, c ) in channel_c_list.iter().enumerate() {
                if settings.dimensionality != Dimensionality::Vertex && channel_c_list.len() < 1 {continue};
                    // Generate points of the dimension
                    let color_point_1 = (
                        a.value,
                        b.value + yuv_offset,
                        c.value + yuv_offset,
                    );
                    let point_1 = VertexObject::from_tuple(get_point_and_color(color_point_1, settings));
                    match &mut dim_list {
                        DimensionList::Vertex(vertex_list) => {
                            vertex_list.add_vertex(&point_1);
                        },
                        DimensionList::Edge(edge_list) => {
                            let slice_offset = settings.face_slicing.get_edge_offsets();
                            // let (a_end, b_end,c_end) = (index_of_a == channel_a_list.len() - 1, index_of_b == channel_b_list.len() - 1,index_of_c == channel_c_list.len() - 1);
                            let (a2,b2,c2) = (
                                channel_a_list.get(wrap_index(index_of_a, slice_offset[1][0], channel_a_list.len())).unwrap().value,
                                channel_b_list.get(wrap_index(index_of_b, slice_offset[1][1], channel_b_list.len())).unwrap().value + yuv_offset,
                                channel_c_list.get(wrap_index(index_of_c, slice_offset[1][2], channel_c_list.len())).unwrap().value + yuv_offset,
                            );
                            let color_point_2: (f32,f32,f32) = (
                                a2,
                                b2,
                                c2,
                            );
                            let point_2 = VertexObject::from_tuple(get_point_and_color(color_point_2, settings));
                            edge_list.add_edge(point_1, point_2);
                        },
                        DimensionList::Face(face_list) => {
                            let slice_offset = settings.face_slicing.get_face_offsets();
                            let (a2,b2,c2) = (
                                channel_a_list.get(wrap_index(index_of_a, slice_offset[1][0], channel_a_list.len())).unwrap().value,
                                channel_b_list.get(wrap_index(index_of_b, slice_offset[1][1], channel_b_list.len())).unwrap().value + yuv_offset,
                                channel_c_list.get(wrap_index(index_of_c, slice_offset[1][2], channel_c_list.len())).unwrap().value + yuv_offset,
                            );
                            let color_point_2: (f32,f32,f32) = (
                                a2,
                                b2,
                                c2,
                            );
                            let point_2 = VertexObject::from_tuple(get_point_and_color(color_point_2, settings));

                            let (a3,b3,c3) = (
                                channel_a_list.get(wrap_index(index_of_a, slice_offset[2][0], channel_a_list.len())).unwrap().value,
                                channel_b_list.get(wrap_index(index_of_b, slice_offset[2][1], channel_b_list.len())).unwrap().value + yuv_offset,
                                channel_c_list.get(wrap_index(index_of_c, slice_offset[2][2], channel_c_list.len())).unwrap().value + yuv_offset,
                            );
                            let color_point_3: (f32,f32,f32) = (
                                a3,
                                b3,
                                c3,
                            );
                            let point_3 = VertexObject::from_tuple(get_point_and_color(color_point_3, settings));

                            let (a4,b4,c4) = (
                                channel_a_list.get(wrap_index(index_of_a, slice_offset[3][0], channel_a_list.len())).unwrap().value,
                                channel_b_list.get(wrap_index(index_of_b, slice_offset[3][1], channel_b_list.len())).unwrap().value + yuv_offset,
                                channel_c_list.get(wrap_index(index_of_c, slice_offset[3][2], channel_c_list.len())).unwrap().value + yuv_offset,
                            );
                            let color_point_4: (f32,f32,f32) = (
                                a4,
                                b4,
                                c4,
                            );
                            let point_4 = VertexObject::from_tuple(get_point_and_color(color_point_4, settings));
                            
                            face_list.add_quad(point_1, point_2, point_3, point_4);
                        },
                        DimensionList::Volume(face_list) => {
                            // Check each axis separately
                            // X-min and X-max
                            if index_of_a == 0 || index_of_a == channel_a_list.len() - 1 {
                                let slice = SlicingMethod::X;
                                let offsets = slice.get_face_offsets();
                                let mut verts = Vec::new();
                                for o in &offsets {
                                    let ca = channel_a_list.get(wrap_index(index_of_a, o[0], channel_a_list.len())).unwrap().value;
                                    let cb = channel_b_list.get(wrap_index(index_of_b, o[1], channel_b_list.len())).unwrap().value + yuv_offset;
                                    let cc = channel_c_list.get(wrap_index(index_of_c, o[2], channel_c_list.len())).unwrap().value + yuv_offset;
                                    verts.push(VertexObject::from_tuple(get_point_and_color((ca,cb,cc), settings)));
                                }
                                face_list.add_quad(verts[0].clone(), verts[1].clone(), verts[2].clone(), verts[3].clone());
                            }

                            // Y-min and Y-max
                            if index_of_b == 0 || index_of_b == channel_b_list.len() - 1 {
                                let slice = SlicingMethod::Y;
                                let offsets = slice.get_face_offsets();
                                let mut verts = Vec::new();
                                for o in &offsets {
                                    let ca = channel_a_list.get(wrap_index(index_of_a, o[0], channel_a_list.len())).unwrap().value;
                                    let cb = channel_b_list.get(wrap_index(index_of_b, o[1], channel_b_list.len())).unwrap().value + yuv_offset;
                                    let cc = channel_c_list.get(wrap_index(index_of_c, o[2], channel_c_list.len())).unwrap().value + yuv_offset;
                                    verts.push(VertexObject::from_tuple(get_point_and_color((ca,cb,cc), settings)));
                                }
                                face_list.add_quad(verts[0].clone(), verts[1].clone(), verts[2].clone(), verts[3].clone());
                            }

                            // Z-min and Z-max
                            if index_of_c == 0 || index_of_c == channel_c_list.len() - 1 {
                                let slice = SlicingMethod::Z;
                                let offsets = slice.get_face_offsets();
                                let mut verts = Vec::new();
                                for o in &offsets {
                                    let ca = channel_a_list.get(wrap_index(index_of_a, o[0], channel_a_list.len())).unwrap().value;
                                    let cb = channel_b_list.get(wrap_index(index_of_b, o[1], channel_b_list.len())).unwrap().value + yuv_offset;
                                    let cc = channel_c_list.get(wrap_index(index_of_c, o[2], channel_c_list.len())).unwrap().value + yuv_offset;
                                    verts.push(VertexObject::from_tuple(get_point_and_color((ca,cb,cc), settings)));
                                }
                                face_list.add_quad(verts[0].clone(), verts[1].clone(), verts[2].clone(), verts[3].clone());
                            }
                        },
                    }
                
            }
        }
    }

    dim_list
}

fn get_point_and_color(base_color: (f32,f32,f32), settings: &VisualizationSettings) -> ([f32;3], P_Color){
    let (r_gamma,g_gamma,b_gamma) = if settings.gamma_deform {(1.,1.,1.)} else {settings.gamma};
    let gamma_adjust = 2.2;
    let gamma = [
        (r_gamma/gamma_adjust) as f32,
        (g_gamma/gamma_adjust) as f32,
        (b_gamma/gamma_adjust) as f32,
    ];
    
    let base_color = (base_color.0,base_color.1,base_color.2,settings.visualization_alpha);
    let raw_color = P_Color::from_tuple(base_color, settings.color_model);
    let chroma = base_color.1;

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

    let base_color = if settings.gamma_deform {color} else {raw_color};
    
    let point: Vec3 = {
        let point = base_color.convert_color(settings.color_space_model).from_space_to_space(settings.color_space, ColorSpace::XYZ);
        let point = if settings.model_mirrored {point.mirror_colorspace()} else {point};
        let (x,y,z, _) = point.to_tuple(); 
        Vec3 {x, y, z}
    };

    (point.into(), color)
}
