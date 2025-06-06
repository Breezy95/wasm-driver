use bytemuck::{Pod, Zeroable};
use core::slice;
use std::mem;
use wgpu::util::DeviceExt;

use cgmath::*;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 4],
    pub normal: [f32; 4],
    pub uv: [f32; 2]
}

pub fn slices_to_vertex(p: [f32; 3], n: [f32; 3], t: [f32; 2]) -> Vertex {
    Vertex {
        position: [p[0], p[1], p[2], 1.0],
        normal: [n[0], n[1], n[2], 1.0],
        uv: [t[0],t[1]],
    }
}

pub fn convert_vector_to_vertices(
    p: Vec<[f32; 3]>,
    n: Vec<[f32; 3]>,
    uv: Vec<[f32; 2]>,
) -> Vec<Vertex> {
    let mut data: Vec<Vertex> = Vec::with_capacity(p.len());
    for i in 0..p.len() {
        data.push(slices_to_vertex(p[i], n[i], uv[i]));
    }
    data.to_vec()
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x4, 1 =>Float32x4, 2 => Float32x2];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}
