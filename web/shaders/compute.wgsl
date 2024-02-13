@group(0)
@binding(0)
var tex: texture_storage_2d_array<rgba8unorm, read_write>; // this is used as both input and output for convenience

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var v = textureLoad(tex, global_id.xy, 0);
    v += vec4<f32>(0.02,0.02,0.02,0.02);
    v.w = 1.;
    textureStore(tex, global_id.xy,v);
}