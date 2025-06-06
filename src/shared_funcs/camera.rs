use cgmath::*;
use std::{f32::consts::PI, time::Duration};
use winit::{dpi::PhysicalPosition, event::{ElementState, MouseScrollDelta}, keyboard::KeyCode};

use super::matrix_helpers::SAFE_FRAC_PI_2;
mod projection;

#[derive(Debug)]
pub struct Camera {
    pub pos: Point3<f32>,
   pitch: Rad<f32>,
    yaw: Rad<f32>,
}

impl Camera {
    // should just be another view projection matrix
    pub fn new<Point: Into<Point3<f32>>, Pitch: Into<Rad<f32>>, Yaw: Into<Rad<f32>>>(
        pos: Point,
        pitch: Pitch,
        yaw: Yaw,
    ) -> Self {
        Self {
            pos: pos.into(),
            pitch: pitch.into(),
            yaw: yaw.into(),
        }
    }

    pub fn get_eye_position(&self) -> [f32; 3] {
            [self.pos.x, self.pos.y, self.pos.z]
        }
        
    pub fn calc_view_mat(&self) -> Matrix4<f32> {
        let front = Vector3 {
            x: self.pitch.cos() * self.yaw.cos() ,
            y: self.pitch.sin(),
            z: self.pitch.cos() * self.yaw.sin(),
        };

        let up = Vector3::unit_y();
        Matrix4::look_to_rh(self.pos, front.normalize(), up)
    }
}

#[derive(Debug)]
pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_hori: f32,
    rotate_vert: f32,
    scroll: f32,
    speed: f32,
    sens: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_hori: 0.0,
            rotate_vert: 0.0,
            scroll: 0.0,
            speed,
            sens: sensitivity,
        }
    }

    

    pub fn key_handler(&mut self, key: KeyCode, state: ElementState) -> bool{
        let amount = if state == ElementState::Pressed { 1.0 } else { 0.0 };
        match key {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.amount_forward = amount;
                true
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.amount_backward = amount;
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.amount_left = amount;
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.amount_right = amount;
                true
            }
            KeyCode::Space => {
                self.amount_up = amount;
                true
            }
            KeyCode::ShiftLeft => {
                self.amount_down = amount;
                true
            }
            _ => false,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_hori = mouse_dx as f32;
        self.rotate_vert = mouse_dy as f32;
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = -match delta {
            // I'm assuming a line is about 100 pixels
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.0,
            MouseScrollDelta::PixelDelta(PhysicalPosition {
                y: scroll,
                ..
            }) => *scroll as f32,
        };
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();

        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        camera.pos += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        camera.pos += right * (self.amount_right - self.amount_left) * self.speed * dt;

        // Move in/out (aka. "zoom")
        // Note: this isn't an actual zoom. The camera's position
        // changes when zooming. I've added this to make it easier
        // to get closer to an object you want to focus on.
        let (pitch_sin, pitch_cos) = camera.pitch.0.sin_cos();
        let scrollward = Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
        camera.pos += scrollward * self.scroll * self.speed * self.sens * dt;
        self.scroll = 0.0;

        // Move up/down. Since we don't use roll, we can just
        // modify the y coordinate directly.
        camera.pos.y += (self.amount_up - self.amount_down) * self.speed * dt;

        // Rotate
        camera.yaw += Rad(self.rotate_hori) * self.sens * dt;
        camera.pitch += Rad(-self.rotate_vert) * self.sens * dt;

        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non-cardinal direction.
        self.rotate_hori = 0.0;
        self.rotate_vert = 0.0;

        // Keep the camera's angle from going too high/low.
        if camera.pitch < (-Rad(SAFE_FRAC_PI_2)).into() {
            camera.pitch = -Rad(SAFE_FRAC_PI_2);
        } else if camera.pitch > Rad(SAFE_FRAC_PI_2) {
            camera.pitch = Rad(SAFE_FRAC_PI_2);
        }
    }
}


#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}
impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_project_matrix(&mut self, camera: &Camera, project_mat: Matrix4<f32>) {
        self.view_proj = (project_mat * camera.calc_view_mat()).into()
    }
}
