@group(0)
@binding(0)
var texView: texture_2d<f32>; 

@group(0)
@binding(1)
var texSampler: sampler; 


struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texcoord: vec2<f32>,
}


@fragment
fn main(vertex_output: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texView,texSampler,vertex_output.texcoord);
}