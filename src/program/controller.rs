use nalgebra::Vector3;
use winit::event::{KeyboardInput, VirtualKeyCode, ElementState};

#[derive(Default)]
pub struct CameraController {
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,
}

impl CameraController {
    pub fn cur_dir(&self) -> Vector3<f64> {
        let mut dir = Vector3::zeros();
        if self.is_left_pressed {
            dir -= Vector3::x();
        }
        if self.is_right_pressed {
            dir += Vector3::x();
        }
        if self.is_forward_pressed {
            dir += Vector3::z();
        }
        if self.is_backward_pressed {
            dir -= Vector3::z();
        }
        if self.is_up_pressed {
            dir += Vector3::y();
        }
        if self.is_down_pressed {
            dir -= Vector3::y();
        }

        if dir.norm() < 0.1 { dir } else { dir.normalize() }
    }

    pub fn handle_key_event(&mut self, event: KeyboardInput) {
        match event.virtual_keycode {
            Some(VirtualKeyCode::A) => self.is_left_pressed = event.state == ElementState::Pressed,
            Some(VirtualKeyCode::D) => self.is_right_pressed = event.state == ElementState::Pressed,
            Some(VirtualKeyCode::Space) => self.is_up_pressed = event.state == ElementState::Pressed,
            Some(VirtualKeyCode::LShift) => self.is_down_pressed = event.state == ElementState::Pressed,
            Some(VirtualKeyCode::W) => self.is_forward_pressed = event.state == ElementState::Pressed,
            Some(VirtualKeyCode::S) => self.is_backward_pressed = event.state == ElementState::Pressed,
            _ => (),
        }
    }
}

