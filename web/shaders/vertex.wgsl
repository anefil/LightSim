
// @group(0) @binding(0)
// var<uniform> position: vec3<f32>;

struct VertexInput {
    @location(0)
    position: vec2<f32>
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) texcoord: vec2<f32>,
}


@vertex
fn main(model: VertexInput) -> VertexOutput {
    var vsOutput: VertexOutput;
    vsOutput.position = vec4<f32>(model.position,0.,1.);
    vsOutput.texcoord = model.position;
    return vsOutput;
}