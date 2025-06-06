
//vertex shader
struct Uniforms {
    mMatrix : mat4x4 < f32>,
    vpMatrix : mat4x4 < f32>,
    nMatrix : mat4x4 < f32>,

};

@group(0) @binding(0) var<uniform> uniforms : Uniforms;


struct Output {
@builtin(position) position : vec4<f32>,
@location(0) v_position : vec4<f32>,
@location(1) v_normal : vec4<f32>,
//@location(2) v_uv : vec2<f32>,
//@location(3) v_color: vec4<f32>
};

@vertex
fn vs_main( @location(0) pos: vec4<f32>, @location(1) norm: vec4<f32>) -> Output {
    var output : Output;
    let m_position:vec4<f32> = uniforms.mMatrix * pos;
    output.v_position = m_position;
    output.v_normal = uniforms.nMatrix * norm;
    output.position = uniforms.vpMatrix * m_position;
    //output.v_color = in.color;
    return output;
}



// frag shader stuff

struct fragUniforms {
    light_position: vec4<f32>,
    eye_position: vec4<f32>,
};

@binding(1) @group(0) var<uniform> frag_uniforms: fragUniforms;

struct lightUniforms {
    color: vec4<f32>,
    specular_color : vec4<f32>,
    ambient_intensity: f32,
    diffuse_intensity :f32,
    specular_intensity: f32,
    specular_shininess: f32,
    is_two_side: i32,
}

@binding(2) @group(0) var<uniform> light_uniforms : lightUniforms;
@binding(0) @group(1) var texture_data : texture_2d<f32>;
@binding(1) @group(1) var texture_sampler : sampler;

@fragment
fn fs_main(@location(0) v_position: vec4<f32>, @location(1) v_normal: vec4<f32>) -> @location(0) vec4<f32> {
    //let texture_color:vec4<f32> = textureSample(texture_data, texture_sampler, in.v_uv);
    let N:vec3<f32> = normalize(v_normal.xyz);
    let L:vec3<f32> = normalize(frag_uniforms.light_position.xyz - v_position.xyz);
    let V:vec3<f32> = normalize(frag_uniforms.eye_position.xyz - v_position.xyz);
    let H:vec3<f32> = normalize(L + V);
    // front side
    var diffuse:f32 = light_uniforms.diffuse_intensity * max(dot(N, L), 0.0);
    var specular: f32 = light_uniforms.specular_intensity *
    pow(max(dot(N, H),0.0), light_uniforms.specular_shininess);

    let ambient: f32 = light_uniforms.ambient_intensity;
    let res = light_uniforms.color.xyz*(ambient + diffuse) + light_uniforms.specular_color.xyz * specular;
    return vec4<f32>(res, 1.0);
    }
