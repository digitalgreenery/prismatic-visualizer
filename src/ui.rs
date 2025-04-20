use bevy::prelude::{ResMut, Resource};
use prismatic_color::{ColorModel, ColorSpace};
use bevy_egui::{egui::{self, epaint::Vertex}, EguiContexts};

use crate::visualization::{ColorModelCategory, Dimensionality, MeshShape, RotationDirection, SlicingMethod};

#[derive(Resource, Clone)]
pub struct VisualizationSettings{
    pub viz_scale: f32,
    pub instance_scale: f32,
    pub component_limit: (f32,f32,f32),
    pub per_component_gamma: bool,
    pub gamma: (f32,f32,f32),
    pub hcl_adjust: (u8,u8,u8),
    // pub offset: ((u8,u8),(u8,u8),(u8,u8)),
    pub is_chroma_luma: bool,
    pub color_model_category: ColorModelCategory,
    pub color_model: ColorModel,
    pub color_space: ColorSpace,
    pub dimensionality: Dimensionality,
    pub mesh_shape: MeshShape,
    pub quad_shape: SlicingMethod,
    pub gamma_deform: bool,
    pub discrete_color: bool,
    pub color_space_model: ColorModel,
    pub model_rotation: RotationDirection,
    pub model_mirrored: bool,
}

impl Default for VisualizationSettings{
    fn default() -> Self {
        Self {
            viz_scale: 1.,
            instance_scale: 0.1,
            component_limit: (1., 1., 1.), 
            per_component_gamma: false,
            gamma: (2.2, 2.2, 2.2), 
            hcl_adjust: (12,8,8),
            is_chroma_luma: false,
            color_model_category: ColorModelCategory::Spherical,
            color_model: ColorModel::SphericalHCLA,
            dimensionality: Dimensionality::Vertex,
            mesh_shape: MeshShape::Sphere,
            quad_shape: SlicingMethod::Axial,
            gamma_deform: false,
            discrete_color: true,
            color_space: ColorSpace::XYZ,
            color_space_model: ColorModel::RGBA,
            model_rotation: RotationDirection::None,
            model_mirrored: false,
        }
    }
}

pub fn ui_overlay(mut contexts: EguiContexts, mut settings: ResMut<VisualizationSettings>){

    //Create window for variable sliders
    egui::SidePanel::left("Spherical RGB Adjust").resizable(false).show(contexts.ctx_mut(), |ui|{
        
        // ui.set_max_width(ui.available_width()/2.);

        ui.label("Scale");
        ui.add(egui::Slider::new( &mut settings.viz_scale ,0.0..=2.0).text("Visualization Scale"));

        ui.label("Perceptual Offset");
        ui.add(egui::Slider::new( &mut settings.component_limit.0 ,0.0..=1.0).text("Red"));
        ui.add(egui::Slider::new( &mut settings.component_limit.1 ,0.0..=1.0).text("Green"));
        ui.add(egui::Slider::new( &mut settings.component_limit.2 ,0.0..=1.0).text("Blue"));

        ui.horizontal(|ui| {
            ui.label("Gamma");
            ui.checkbox(&mut settings.per_component_gamma, "per component");
        });
        if settings.per_component_gamma {
            ui.add(egui::Slider::new( &mut settings.gamma.0 ,0.1..=3.0).text("Red"));
            ui.add(egui::Slider::new( &mut settings.gamma.1 ,0.1..=3.0).text("Green"));
            ui.add(egui::Slider::new( &mut settings.gamma.2 ,0.1..=3.0).text("Blue"));
        }
        else {
            ui.add(egui::Slider::new( &mut settings.gamma.0 ,0.1..=3.0));
            settings.gamma.1 = settings.gamma.0;
            settings.gamma.2 = settings.gamma.0;
        }

        ui.separator();

        ui.label("Color Type");
        ui.horizontal(|ui| {
            ui.radio_value(&mut settings.is_chroma_luma, true, "HCL");
            ui.radio_value(&mut settings.is_chroma_luma, false, "HWB");
        });

        if settings.is_chroma_luma {
            ui.add(egui::Slider::new( &mut settings.hcl_adjust.0 ,1..=48).text("Hue"));
            ui.add(egui::Slider::new( &mut settings.hcl_adjust.1 ,1..=24).text("Chroma"));
            ui.add(egui::Slider::new( &mut settings.hcl_adjust.2 ,1..=24).text("Luma"));
        }
        else {
            ui.add(egui::Slider::new( &mut settings.hcl_adjust.0 ,1..=48).text("Hue"));
            ui.add(egui::Slider::new( &mut settings.hcl_adjust.1 ,1..=24).text("White"));
            ui.add(egui::Slider::new( &mut settings.hcl_adjust.2 ,1..=24).text("Black"));
        }

        ui.separator();

        ui.label("Color Model");
        ui.horizontal(|ui| {
            ui.radio_value(&mut settings.color_model_category, ColorModelCategory::Spherical, "Spherical");
            ui.radio_value(&mut settings.color_model_category, ColorModelCategory::Cubic, "Cubic");
            ui.radio_value(&mut settings.color_model_category, ColorModelCategory::LumaChroma, "Luma-Chroma");
        });

        ui.separator();

        match settings.color_model_category {
            ColorModelCategory::Spherical => {
                ui.radio_value(&mut settings.color_model, ColorModel::SphericalHCLA, "HCL");
            },
            ColorModelCategory::Cubic => {
                ui.radio_value(&mut settings.color_model, ColorModel::CubicHSVA, "HSV");
                ui.radio_value(&mut settings.color_model, ColorModel::CubicHSLA, "HSL");
            },
            ColorModelCategory::LumaChroma => {
                ui.radio_value(&mut settings.color_model, ColorModel::YUVA, "YUV");
            },
        }

        ui.separator();

        let current_color_model = settings.color_model;

        ui.label("Color Space");
        egui::ComboBox::from_label("Axis")
        .selected_text(format!("{:?}", settings.color_space_model))
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut settings.color_space_model, current_color_model, "Current Color Model");
            ui.selectable_value(&mut settings.color_space_model, ColorModel::RGBA, "RGB");
            ui.selectable_value(&mut settings.color_space_model, ColorModel::CMYA, "CMY");
            ui.selectable_value(&mut settings.color_space_model, ColorModel::SphericalHCLA, "Spherical HCL");
            ui.selectable_value(&mut settings.color_space_model, ColorModel::CubicHSVA, "Cubic HSV");
            ui.selectable_value(&mut settings.color_space_model, ColorModel::YUVA, "YUV");
        });

        ui.horizontal(|ui| {
            ui.radio_value(&mut settings.color_space, ColorSpace::XYZ, "XYZ");
            ui.radio_value(&mut settings.color_space, ColorSpace::Cylindrical, "Cylindrical");
            ui.radio_value(&mut settings.color_space, ColorSpace::Symmetric, "Symmetric");
        });

        ui.horizontal(|ui| {

            let mut is_cw_prev = false;
            let mut is_ccw_prev = false;
            let mut is_cw = false;
            let mut is_ccw = false;
            match settings.model_rotation {
                RotationDirection::None => {},
                RotationDirection::Clockwise => {
                    is_cw = true;
                    is_cw_prev = true;
                },
                RotationDirection::Counterclockwise => {
                    is_ccw = true;
                    is_ccw_prev = true;
                },
            }
            
            ui.checkbox(&mut is_cw, "Rotate ↻");
            ui.checkbox(&mut is_ccw, "Rotate ↺");

            if is_cw && is_ccw_prev {
                is_ccw = false;
            }
            if is_ccw && is_cw_prev {
                is_cw = false;
            }

            settings.model_rotation = 
            if is_cw {
                RotationDirection::Clockwise
            }
            else if is_ccw {
                RotationDirection::Counterclockwise
            } 
            else  {
                RotationDirection::None
            };

            ui.checkbox(&mut settings.model_mirrored, "Mirror");

        });

        ui.separator();

        ui.label("Shape");
        ui.horizontal(|ui| {
            ui.radio_value(&mut settings.dimensionality, Dimensionality::Vertex, "Vertex");
            ui.radio_value(&mut settings.dimensionality, Dimensionality::Face, "Faces");
        });
  
        match settings.dimensionality {
            Dimensionality::Vertex => {
                ui.label("Mesh Shape");
                ui.horizontal(|ui| {
                    ui.radio_value(&mut settings.mesh_shape, MeshShape::Sphere, "Spheres");
                    ui.radio_value(&mut settings.mesh_shape, MeshShape::Cube, "Cubes");
                    ui.add(egui::Slider::new( &mut settings.instance_scale ,0.0..=2.0).text("Shape Scale"));
                });
            },
            Dimensionality::Edge => {},
            Dimensionality::Face => {
                ui.label("Quad Direction");
                ui.horizontal(|ui| {
                    ui.radio_value(&mut settings.quad_shape, SlicingMethod::Axial, "Axial");
                    ui.radio_value(&mut settings.quad_shape, SlicingMethod::Radial, "Radial");
                    ui.radio_value(&mut settings.quad_shape, SlicingMethod::Concentric, "Concentric");
                    ui.checkbox(&mut settings.discrete_color, "Discrete Color");
                });

            },
            Dimensionality::Volume => {},
        }

        ui.separator();

        // ui.label("Additional Settings");

        // ui.checkbox(&mut settings.gamma_deform, "Gamma Deform");

        // ui.separator();

        ui.label("WASD - Horizontal Movement");
        ui.label("Ctrl & Space - Vertical Movement");
        ui.label("Arrow Keys - Camera Rotation");

    });
   
}