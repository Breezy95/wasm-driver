use std::f32::consts::PI;

use cgmath::{Angle, Deg};

use wasm_driver::{lighting::create_light_struct, run, vertex::{convert_vector_to_vertices, Vertex}};

#[path = "shared_funcs/shared_funcs.rs"]
pub mod shared_funcs;
#[path ="shared_funcs/texture.rs"]
pub mod texture;

pub mod vertex;
// ssphere example
pub fn sphere_position(r: f32, theta: Deg<f32>, phi: Deg<f32>) -> [f32; 3] {
    let snt = theta.sin();
    let cnt = theta.cos();
    let snp = phi.sin();
    let cnp = phi.cos();
    [r * snt * cnp, r * cnt, -r * snt * snp]
}



fn create_sphere_vertices(r: f32, u: usize, v: usize) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>) {
    let mut pts: Vec<[f32; 3]> = Vec::with_capacity((4 * (u - 1) * (v - 1)) as usize);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity((4 * (u - 1) * (v - 1)) as usize);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity((4 * (u - 1) * (v - 1)) as usize);

    for i in 0..u - 1 {
        for j in 0..v - 1 {
            let theta = i as f32 * 180.0 / (u as f32 - 1.0);
            let phi = j as f32 * 360.0 / (v as f32 - 1.0);
            let theta1 = (i as f32 + 1.0) * 180.0 / (u as f32 - 1.0);
            let phi1 = (j as f32 + 1.0) * 360.0 / (v as f32 - 1.0);
            let p0 = sphere_position(r, Deg(theta), Deg(phi));
            let p1 = sphere_position(r, Deg(theta1), Deg(phi));
            let p2 = sphere_position(r, Deg(theta1), Deg(phi1));
            let p3 = sphere_position(r, Deg(theta), Deg(phi1));

            // positions
            pts.push(p0);
            pts.push(p1);
            pts.push(p3);
            pts.push(p1);
            pts.push(p2);
            pts.push(p3);

            // normals
            normals.push([p0[0]/r, p0[1]/r, p0[2]/r]);
            normals.push([p1[0]/r, p1[1]/r, p1[2]/r]);
            normals.push([p3[0]/r, p3[1]/r, p3[2]/r]);
            normals.push([p1[0]/r, p1[1]/r, p1[2]/r]);
            normals.push([p2[0]/r, p2[1]/r, p2[2]/r]);
            normals.push([p3[0]/r, p3[1]/r, p3[2]/r]);

            // uvs
            let u0 = 0.5+((p0[0]/r).atan2(p0[2]/r))/PI/2.0;
            let u1 = 0.5+((p1[0]/r).atan2(p1[2]/r))/PI/2.0;
            let u2 = 0.5+((p2[0]/r).atan2(p2[2]/r))/PI/2.0;
            let u3 = 0.5+((p3[0]/r).atan2(p3[2]/r))/PI/2.0;
            let v0 = 0.5-(p0[1]/r).asin()/PI;
            let v1 = 0.5-(p1[1]/r).asin()/PI;
            let v2 = 0.5-(p2[1]/r).asin()/PI;
            let v3 = 0.5-(p3[1]/r).asin()/PI;

            uvs.push([u0, v0]);
            uvs.push([u1, v1]);
            uvs.push([u3, v3]);
            uvs.push([u1, v1]);
            uvs.push([u2, v2]);
            uvs.push([u3, v3]);
        }
    }
    (pts, normals, uvs)
}



pub fn main() {
    {
    unsafe { #[cfg(not(target_arch="wasm32"))]
    std::env::set_var("WAYLAND_DISPLAY", "") };
    }
    // pos, normal, uv
    let mesh  = create_sphere_vertices(1.5, 15, 20);
    let fname = "src/assets/bball.jpg";
    let light_data = create_light_struct([1.0, 0.0, 0.0], [1.0, 1.0, 0.0], 0.1, 0.6, 0.2, 30.0);
    let mesh_data = convert_vector_to_vertices(mesh.0, mesh.1, mesh.2);
    let u_mode = wgpu::AddressMode::ClampToEdge;
    let v_mode = wgpu::AddressMode::ClampToEdge;
    run(&mesh_data, light_data, fname, u_mode, v_mode);
}
