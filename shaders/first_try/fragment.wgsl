@group(0)
@binding(0)
var tex1View: texture_2d<f32>; 

@group(0)
@binding(1)
var tex1Sampler: sampler; 


@group(0)
@binding(2)
var tex2View: texture_2d<f32>; 

@group(0)
@binding(3)
var tex2Sampler: sampler; 

@group(0)
@binding(4)
var tex3View: texture_2d<f32>; 

@group(0)
@binding(5)
var tex3Sampler: sampler; 

@group(0)
@binding(8)
var tex5View: texture_2d<f32>; 

@group(0)
@binding(9)
var tex5Sampler: sampler; 

struct ComputeShaderLightData {
    time: f32,
    time_delta: f32,
    lightspeed: f32,
    wavelength: f32,
    sizeW: f32,
    sizeH: f32,
    ping_pong: u32,
}

@group(0)
@binding(10)
var<uniform> data: ComputeShaderLightData;


struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texcoord: vec2<f32>,
}


@fragment
fn main(vertex_output: VertexOutput) -> @location(0) vec4<f32> {

    var tex1: vec4f;

    let tex5 = textureSample(tex5View,tex5Sampler,vertex_output.texcoord);

    var value: f32;

    if (data.ping_pong % 3 == 0) {
        tex1 = textureSample(tex1View,tex1Sampler,vertex_output.texcoord);
    } else if (data.ping_pong % 3 == 1) {
        tex1 = textureSample(tex2View,tex2Sampler,vertex_output.texcoord);
    } else if (data.ping_pong % 3 == 2) {
        tex1 = textureSample(tex3View,tex3Sampler,vertex_output.texcoord);
    }

    value = clamp(sqrt(tex1.x*tex1.x + tex1.z*tex1.z) * 0.2,0.,1.);
        
    if (tex5.r > 100.) {
        return vec4<f32>(1.,1.,1.,1.);
    } else {
        return vec4<f32>(value,value,value,1.);
    }
}