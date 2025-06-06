use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct FragUniforms {
    pub light_position: [f32; 4],
    pub eye_position: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Light {
    color: [f32; 4],
    specular_color: [f32; 4],
    ambient_intensity: f32,
    diffuse_intensity: f32,
    specular_intensity: f32,
    specular_shininess: f32,
    
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct LightUniforms {
    pub specular_color: [f32; 4],
    pub ambient_intensity: f32,
    pub diffuse_intensity: f32,
    pub specular_intensity: f32,
    pub specular_shininess: f32,
    pub is_two_side: i32,
}

pub fn create_light_struct(c: [f32; 3], sc: [f32; 3], ai: f32, di: f32, si: f32, ss: f32) -> Light {
    Light {
        color: [c[0], c[1], c[2], 1.0],
        specular_color: [sc[0], sc[1], sc[2], 1.0],
        ambient_intensity: ai,
        diffuse_intensity: di,
        specular_intensity: si,
        specular_shininess: ss,
    }
}

impl Light {
    pub fn to_frag_uniforms(&self, light_pos: [f32; 3], eye_pos: [f32; 3]) -> FragUniforms {
        FragUniforms {
            light_position: [light_pos[0], light_pos[1], light_pos[2], 1.0],
            eye_position: [eye_pos[0], eye_pos[1], eye_pos[2], 1.0],
        }
    }

    pub fn to_light_uniforms(&self, is_two_side: bool) -> LightUniforms {
        LightUniforms {
            specular_color: self.specular_color,
            ambient_intensity: self.ambient_intensity,
            diffuse_intensity: self.diffuse_intensity,
            specular_intensity: self.specular_intensity,
            specular_shininess: self.specular_shininess,
            is_two_side: if is_two_side { 1 } else { 0 },
        }
    }
}
