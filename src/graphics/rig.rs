mod imgcut;
mod maanim;
mod mamodel;

use super::engine::{timeline, transform};
use super::tools::{boundary, periodicity};

pub use imgcut::{ImgRect, ImgVec2, SpriteCut, SpriteSheet};
pub use maanim::{Animation, AnimModification};
pub use mamodel::{Model, ModelPart};

pub struct Unit {
    pub model: Model,
    pub sheet: SpriteSheet,
}

impl Unit {
    #[inline(always)]
    pub fn parse(
        png: impl AsRef<[u8]>,
        imgcut: impl AsRef<[u8]>,
        mamodel: impl AsRef<[u8]>,
    ) -> Option<Self> {
        Self::parse_inner(png.as_ref(), imgcut.as_ref(), mamodel.as_ref())
    }

    fn parse_inner(png: &[u8], imgcut: &[u8], mamodel: &[u8]) -> Option<Self> {
        let sheet = SpriteSheet::parse(png, imgcut)?;
        let model = Model::parse(mamodel)?;

        Some(Self { model, sheet })
    }

    pub fn calculate_bounds(
        &self,
        animations: &[&Animation],
        tolerance: f32
    ) -> Option<(f32, f32, f32, f32)> {
        let tolerance = boundary::Tolerance::new(tolerance);
        let bounds = boundary::calculate_animation_bounds(&self.model, &self.sheet, animations, tolerance)?;

        Some((bounds.min_x, bounds.min_y, bounds.width(), bounds.height()))
    }

    pub fn calculate_cycle(
        &self,
        animation: &Animation,
        tolerance: f32,
        minimum_frame: Option<i32>,
        maximum_frame: Option<i32>,
        mut progress_callback: impl FnMut(usize) -> bool,
    ) -> Option<(i32, i32)> {
        let mut frame_states: Vec<Vec<([f32; 9], f32)>> = Vec::new();
        let mut state_buffer = self.model.parts.clone();
        let mut current_frame = 0;

        let minimum_loop_length = minimum_frame.unwrap_or(1);

        loop {
            if !progress_callback(current_frame) {
                return None;
            }

            if let Some(maximum) = maximum_frame {
                if current_frame > maximum as usize {
                    return None;
                }
            }

            let frame_float = current_frame as f32;
            let _ = timeline::animate(&self.model, animation, frame_float, &mut state_buffer);
            let world_parts = transform::solve_hierarchy(&state_buffer, &self.model);

            let mut current_state = Vec::with_capacity(world_parts.len());
            for part in &world_parts {
                current_state.push((part.matrix, part.opacity));
            }

            for (past_frame_index, past_state) in frame_states.iter().enumerate() {
                let loop_length = current_frame as i32 - past_frame_index as i32;

                if loop_length < minimum_loop_length {
                    continue;
                }

                let difference = periodicity::calculate_difference(&current_state, past_state);

                if difference <= tolerance {
                    return Some((past_frame_index as i32, current_frame as i32));
                }
            }

            frame_states.push(current_state);
            current_frame += 1;
        }
    }
}