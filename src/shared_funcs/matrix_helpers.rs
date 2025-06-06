use std::f32::consts::PI;
use winit::window::Window;
use cgmath::*;
// cgmaths coordinate system is built for opengl's coord system
pub const SAFE_FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2 - 0.0001;

pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

// creates a matrix that
pub fn create_view_matrix(camera_position: Point3<f32>, look_direction: Point3<f32>,
up_direction: Vector3<f32>) -> Matrix4<f32> {
Matrix4::look_at_rh(camera_position, look_direction, up_direction)
}

pub fn create_projection_matrix(aspect:f32, is_perspective:bool) -> Matrix4<f32> {
let project_mat:Matrix4<f32>;
if is_perspective {
project_mat = OPENGL_TO_WGPU_MATRIX * perspective(Rad(2.0*PI/5.0), aspect, 0.1, 100.0);
} else {
project_mat = OPENGL_TO_WGPU_MATRIX * ortho(-4.0, 4.0, -3.0, 3.0, -1.0, 6.0);
}
project_mat
}

pub fn create_view_projection_matrix(
    position: Point3<f32>,
    look_direction: Point3<f32>,
    up_direction: Vector3<f32>,
    aspect: f32,
    is_perspective: bool,
) -> (Matrix4<f32>, Matrix4<f32>, Matrix4<f32>) {
    let view_mat = Matrix4::look_at_rh(position, look_direction, up_direction);

    // construct projection matrix
    let project_mat: Matrix4<f32> = create_projection_matrix(aspect, is_perspective);

    // construct view-projection matrix
    let view_project_mat = project_mat * view_mat;

    // return various matrices
    (view_mat, project_mat, view_project_mat)
}

// we need 
pub fn create_transforms_matrix(translation:[f32; 3], rotation:[f32; 3], scaling:[f32; 3]) -> Matrix4<f32> {

let trans_mat = Matrix4::from_translation(Vector3::new(translation[0],
translation[1], translation[2]));
let rotate_mat_x = Matrix4::from_angle_x(Rad(rotation[0]));
let rotate_mat_y = Matrix4::from_angle_y(Rad(rotation[1]));
let rotate_mat_z = Matrix4::from_angle_z(Rad(rotation[2]));
let scale_mat = Matrix4::from_nonuniform_scale(scaling[0], scaling[1], scaling[2]);

let model_mat = trans_mat * rotate_mat_z * rotate_mat_y * rotate_mat_x * scale_mat;

model_mat
}