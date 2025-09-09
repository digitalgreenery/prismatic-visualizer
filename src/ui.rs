use bevy::{ecs::component::Component, prelude::{ResMut, Resource}, reflect::Reflect};
use prismatic_color::{ColorModel, ColorSpace};
use bevy_egui::{
    egui::{self,RichText},EguiContextSettings, EguiContexts, EguiPlugin, EguiPrimaryContextPass, EguiStartupSet,
};

use crate::visualization::{ColorModelCategory, Dimensionality, VertexShape, RotationDirection, SlicingMethod};

#[derive(Resource, Clone)]
pub struct VisualizationSettings{
    pub viz_scale: f32,
    pub visualization_alpha: f32,

    pub component_limit: (f32,f32,f32),
    pub per_component_gamma: bool,
    pub gamma: (f32,f32,f32),

    pub channel_settings: (ColorChannel,ColorChannel,ColorChannel),

    pub color_model_category: ColorModelCategory,
    pub color_model: ColorModel,
    pub color_space: ColorSpace,
    pub dimensionality: Dimensionality,

    pub mesh_shape: VertexShape,
    pub instance_scale: f32,
    pub line_width: f32,

    pub face_slicing: SlicingMethod,
    pub gamma_deform: bool,
    pub discrete_color: bool,
    pub color_space_model: ColorModel,

    // pub model_rotation: RotationDirection,
    pub model_mirrored: bool,

}

#[derive(Component, Debug, Clone, Reflect)]
pub struct ColorChannel {
    pub start: f32,
    pub end: f32,
    pub steps: usize,
    pub step_type: StepType,
}

impl Default for ColorChannel {
    fn default() -> Self {
        Self {
            start: 0.,
            end: 1.,
            steps: 8,
            step_type: StepType::Forward,
        }
    }
}

impl ColorChannel {
    pub fn generate(&self) -> Vec<ChannelIndex> {
        
        let mut values = Vec::new();

        let step_size = self.step_size();

        let range = 
            match self.step_type {
                StepType::Forward => 0..self.steps,
                StepType::Reverse => 1..self.steps+1,
                StepType::Inclusive => 0..self.steps,
            };

        for step in range {
            let value = self.start + step as f32 * step_size;
            
            values.push(ChannelIndex {value});
        }
        values
    }
    fn step_size(&self) -> f32 {
        if self.step_type == StepType::Inclusive {
            (self.end - self.start) / (self.steps as f32 - 1.)
        }
        else {
            (self.end - self.start) / (self.steps as f32)
        }
    }
}

#[derive(Clone, Copy)]
pub struct ChannelIndex {
    pub value: f32,
}

#[derive(Component, Debug, Clone, Reflect, PartialEq)]
pub enum StepType {
    Forward,
    Reverse,
    Inclusive,
}

impl Default for VisualizationSettings{
    fn default() -> Self {
        Self {
            viz_scale: 1.,
            visualization_alpha: 1.,

            component_limit: (1., 1., 1.), 
            per_component_gamma: false,
            gamma: (2.2, 2.2, 2.2),

            channel_settings: (
                ColorChannel::default(),
                ColorChannel::default(),
                ColorChannel::default(),
            ),

            color_model_category: ColorModelCategory::Spherical,
            color_model: ColorModel::SphericalHCLA,
            dimensionality: Dimensionality::Vertex,
            
            mesh_shape: VertexShape::Sphere,
            instance_scale: 1.0,
            line_width: 1.0,

            face_slicing: SlicingMethod::Y,
            gamma_deform: false,
            discrete_color: true,
            color_space: ColorSpace::XYZ,
            color_space_model: ColorModel::RGBA,

            // model_rotation: RotationDirection::None,
            model_mirrored: false,
        }
    }
}

pub fn ui_overlay(mut contexts: EguiContexts, mut settings: ResMut<VisualizationSettings>) {

    //Create window for variable sliders
    egui::Window::new("Spherical RGB Adjust")
        .resizable(true)
        .show(contexts.ctx_mut().unwrap(), |ui|{
        
        ui.label(RichText::new("Prismatic Visualizer").heading());
        ui.separator();

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

        // ui.label("Color Type");
        // ui.horizontal(|ui| {
        //     ui.radio_value(&mut settings.is_chroma_luma, true, "HCL");
        //     ui.radio_value(&mut settings.is_chroma_luma, false, "HWB");
        // });

        //if settings.is_chroma_luma {

        ui.horizontal(|ui| {
            ui.label("Channel Settings");
        });

        //Channel A
        ui.horizontal(|ui| {
            ui.label("A");
            ui.add(egui::DragValue::new( &mut settings.channel_settings.0.steps).range( 1..=24).prefix("Steps: "));
        });
        ui.horizontal(|ui| {
            ui.selectable_value(&mut settings.channel_settings.0.step_type, StepType::Forward, "Forward");
            ui.selectable_value(&mut settings.channel_settings.0.step_type, StepType::Reverse, "Reverse");
            if settings.channel_settings.0.steps == 1 {
                if settings.channel_settings.0.step_type == StepType::Inclusive {
                    settings.channel_settings.0.step_type = StepType::Forward;
                }
            }
            else {
                ui.selectable_value(&mut settings.channel_settings.0.step_type, StepType::Inclusive, "Inclusive");
            }
        });
        ui.horizontal(|ui|{
            ui.add(egui::Slider::new( &mut settings.channel_settings.0.start,0.0..=1.0).drag_value_speed(0.05).prefix("Start: "));
            ui.add(egui::Slider::new( &mut settings.channel_settings.0.end,0.0..=2.0).drag_value_speed(0.05).prefix("End: "));
        });

        //Channel B
        ui.horizontal(|ui| {
            ui.label("B");
            ui.add(egui::DragValue::new( &mut settings.channel_settings.1.steps).range( 1..=24).prefix("Steps: "));

        });
        ui.horizontal(|ui| {
            ui.selectable_value(&mut settings.channel_settings.1.step_type, StepType::Forward, "Forward");
            ui.selectable_value(&mut settings.channel_settings.1.step_type, StepType::Reverse, "Reverse");
            if settings.channel_settings.1.steps == 1 {
                if settings.channel_settings.1.step_type == StepType::Inclusive {
                    settings.channel_settings.1.step_type = StepType::Forward;
                }
            }
            else {
                ui.selectable_value(&mut settings.channel_settings.1.step_type, StepType::Inclusive, "Inclusive");
            }
        });
        ui.horizontal(|ui|{
            ui.add(egui::Slider::new( &mut settings.channel_settings.1.start,0.0..=1.0).drag_value_speed(0.05).prefix("Start: "));
            ui.add(egui::Slider::new( &mut settings.channel_settings.1.end,0.0..=2.0).drag_value_speed(0.05).prefix("End: "));
        });

        //Channel C
        ui.horizontal(|ui| {
            ui.label("C");
            ui.add(egui::DragValue::new( &mut settings.channel_settings.2.steps).range( 1..=24).prefix("Steps: "));
        });
        ui.horizontal(|ui| {
            ui.selectable_value(&mut settings.channel_settings.2.step_type, StepType::Forward, "Forward");
            ui.selectable_value(&mut settings.channel_settings.2.step_type, StepType::Reverse, "Reverse");
            if settings.channel_settings.2.steps == 1 {
                if settings.channel_settings.2.step_type == StepType::Inclusive {
                    settings.channel_settings.2.step_type = StepType::Forward;
                }
            }
            else {
                ui.selectable_value(&mut settings.channel_settings.2.step_type, StepType::Inclusive, "Inclusive");
            }
            // ui.selectable_label(settings.channel_settings.0, text)
        });
        ui.horizontal(|ui|{
            ui.add(egui::Slider::new( &mut settings.channel_settings.2.start,0.0..=1.0).drag_value_speed(0.05).prefix("Start: "));
            ui.add(egui::Slider::new( &mut settings.channel_settings.2.end,0.0..=2.0).drag_value_speed(0.05).prefix("End: "));
        });        

        ui.separator();

        ui.label("Color Model");
        ui.horizontal(|ui| {
            ui.selectable_value(&mut settings.color_model_category, ColorModelCategory::Spherical, "Spherical");
            ui.selectable_value(&mut settings.color_model_category, ColorModelCategory::Cubic, "Cubic");
            ui.selectable_value(&mut settings.color_model_category, ColorModelCategory::LumaChroma, "Luma-Chroma");
        });

        ui.separator();

        ui.horizontal(|ui| {
            match settings.color_model_category {
            ColorModelCategory::Spherical => {
                ui.selectable_value(&mut settings.color_model, ColorModel::SphericalHCLA, "HCL");
            },
            ColorModelCategory::Cubic => {
                ui.selectable_value(&mut settings.color_model, ColorModel::CubicHSVA, "HSV");
                ui.selectable_value(&mut settings.color_model, ColorModel::CubicHSLA, "HSL");
            },
            ColorModelCategory::LumaChroma => {
                ui.selectable_value(&mut settings.color_model, ColorModel::YUVA, "YUV");
            },
        }});
        

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
            ui.selectable_value(&mut settings.color_space, ColorSpace::XYZ, "XYZ");
            ui.selectable_value(&mut settings.color_space, ColorSpace::Cylindrical, "Cylindrical");
        });

        ui.horizontal(|ui| {

            // let mut is_cw_prev = false;
            // let mut is_ccw_prev = false;
            // let mut is_cw = false;
            // let mut is_ccw = false;
            // match settings.model_rotation {
            //     RotationDirection::None => {},
            //     RotationDirection::Clockwise => {
            //         is_cw = true;
            //         is_cw_prev = true;
            //     },
            //     RotationDirection::Counterclockwise => {
            //         is_ccw = true;
            //         is_ccw_prev = true;
            //     },
            // }
            
            // ui.checkbox(&mut is_cw, "Rotate ↻");
            // ui.checkbox(&mut is_ccw, "Rotate ↺");

            // if is_cw && is_ccw_prev {
            //     is_ccw = false;
            // }
            // if is_ccw && is_cw_prev {
            //     is_cw = false;
            // }

            // settings.model_rotation = 
            // if is_cw {
            //     RotationDirection::Clockwise
            // }
            // else if is_ccw {
            //     RotationDirection::Counterclockwise
            // } 
            // else  {
            //     RotationDirection::None
            // };

            ui.checkbox(&mut settings.model_mirrored, "Mirror");

        });

        ui.separator();

        ui.label("Shape");
        ui.horizontal(|ui| {
            ui.selectable_value(&mut settings.dimensionality, Dimensionality::Vertex, "Vertex");
            ui.selectable_value(&mut settings.dimensionality, Dimensionality::Edge, "Edge");
            ui.selectable_value(&mut settings.dimensionality, Dimensionality::Face, "Face");
            ui.selectable_value(&mut settings.dimensionality, Dimensionality::Volume, "Volume");
        });
  
        match settings.dimensionality {
            Dimensionality::Vertex => {
                // ui.label("Mesh Shape");
                ui.horizontal(|ui| {
                    // ui.selectable_value(&mut settings.mesh_shape, VertexShape::Sphere, "Spheres");
                    // ui.selectable_value(&mut settings.mesh_shape, VertexShape::Cube, "Cubes");
                    // ui.selectable_value(&mut settings.mesh_shape, VertexShape::Tetrahedron, "Tetrehedrons");
                    ui.add(egui::Slider::new( &mut settings.instance_scale ,0.0..=2.0).text("Shape Scale"));
                });
            },
            Dimensionality::Edge => {
                ui.label("Edge Direction");
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut settings.face_slicing, SlicingMethod::Y, "X|Axial");
                    ui.selectable_value(&mut settings.face_slicing, SlicingMethod::X, "Y|Radial");
                    ui.selectable_value(&mut settings.face_slicing, SlicingMethod::Z, "Z|Concentric");
                    ui.add(egui::Slider::new( &mut settings.line_width ,0.0..=10.0).text("Line Width"));
                    ui.checkbox(&mut settings.discrete_color, "Discrete Color");
                });
            },
            Dimensionality::Face => {
                ui.label("Quad Direction");
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut settings.face_slicing, SlicingMethod::Y, "X|Axial");
                    ui.selectable_value(&mut settings.face_slicing, SlicingMethod::X, "Y|Radial");
                    ui.selectable_value(&mut settings.face_slicing, SlicingMethod::Z, "Z|Concentric");
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