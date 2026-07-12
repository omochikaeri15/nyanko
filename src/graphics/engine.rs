pub mod construct;
pub mod timeline;
pub mod transform;

use super::rig::{Animation, Unit};

#[derive(Clone, Debug, Default)]
pub struct FrameData {
    pub sprite_index: usize,
    pub final_matrix: [f32; 9],
    pub vertices: [f32; 12],
    pub uvs: [f32; 12],
    pub opacity: f32,
    pub glow: u8,
}

pub fn resolve_frame(
    unit: &Unit,
    anim: Option<&Animation>,
    frame: f32,
) -> Vec<FrameData> {
    let parts = if let Some(animation) = anim {
        let mut state_buffer = unit.model.parts.clone();
        let _ = timeline::animate(&unit.model, animation, frame, &mut state_buffer);
        state_buffer
    } else {
        unit.model.parts.clone()
    };

    let world_parts = transform::solve_hierarchy(&parts, &unit.model);

    construct::build_geometry(&world_parts, &unit.sheet)
}