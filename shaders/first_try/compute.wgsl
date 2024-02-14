@group(0)
@binding(0)
var first_swap_texture: texture_storage_2d<rgba32float, read_write>; 
 
@group(0)
@binding(1)
var second_swap_texture: texture_storage_2d<rgba32float, read_write>; 

@group(0)
@binding(2)
var third_swap_texture: texture_storage_2d<rgba32float, read_write>; 
 
@group(0)
@binding(4)
var parameter_texture: texture_storage_2d<rgba32float, read_write>; 


struct ComputeShaderLightData {
    time: f32,
    time_delta: f32,
    lightspeed: f32,
    wavelength: f32,
    sizeW: f32,
    sizeH: f32,
    ping_pong: u32
}

@group(0)
@binding(5)
var<uniform> data: ComputeShaderLightData;

struct CumulativeStruct {
    c_first: vec2f,
    c_second: vec2f,
    c_third: vec2f
}

@group(0)
@binding(6)
var<storage, read_write> cumulative: CumulativeStruct;

fn f_f_fast_sum(a: f32, b: f32) -> vec2f {
    var s = a + b;
    let z = (s-a);
    return vec2f(s,b - z);
}

fn f_f_sum(a: f32, b: f32) -> vec2f {
    let s = a + b;
    let aa = s-b;
    let bb = s-aa;
    let da = a-aa;
    let db = b-bb;
    // let v: f32 = s - a;
    // let e: f32 = ( a - ( s - v ) ) + ( b - v ) ;
    // return vec2f(s,e);
    return vec2f(s,da+db);
}

    const split: f32 = 4097;
fn f_split(a: f32) -> vec2f {
    let t = split * a;
    let n = t-a;
    let ahi = t-n;
    return vec2f(ahi,a-ahi);
}

fn f_f_sub(a: f32, b: f32) -> vec2f {
    return f_f_sum(a,-b);
}

fn f_f_mul(a: f32, b: f32) -> vec2f {
    let r: f32 = a*b;
    let e = fma(a,b,-r);
    return vec2f(r,e);
}

fn f_f_div(a: f32, b: f32) -> vec2f {
    let th = a/b;
    let p = f_f_mul(th,b);
    let dh = a -p.x;
    let d = dh - p.y;
    let tl = d/b;
    return f_f_fast_sum(th,tl);
}

fn d_f_add(a: vec2f, b: f32) -> vec2f {
    let s = f_f_sum(a.x, b);
    let v = s.y + a.y;
    return f_f_fast_sum(s.x,v);
}

fn d_d_add(a: vec2f, b: vec2f) -> vec2f {
    var s = f_f_sum(a.x, b.x) ;
    let t = f_f_sum(a.y, b.y) ;
    let c = s.y + t.x;
    let v = f_f_fast_sum(s.x, c);
    let w = t.y + v.y;
    return f_f_fast_sum(v.x, w);
}

fn d_neg(a: vec2f) -> vec2f {
    return vec2f(-a.x,-a.y);
}

fn d_f_sub(a: vec2f, b: f32) -> vec2f {
    return d_f_add(a,-b);
}

fn d_d_sub(a: vec2f, b: vec2f) -> vec2f
{
    return d_d_add(a,d_neg(b));
}


fn d_f_mul(a: vec2f, b: f32) -> vec2f
{
    let c = f_f_mul(a.x,b);
    let cl3 = fma(a.y,b,c.y);
    return f_f_fast_sum(c.x, cl3);
}

fn d_d_mul(a: vec2f, b: vec2f) -> vec2f
{
    let c = f_f_mul(a.x,b.x);
    let tl0 = a.y * b.y;
    let tl1 = fma(a.x, b.y, tl0);
    let cl2 = fma(a.y,b.x,tl1);
    let cl3 = c.y + cl2;
    return f_f_fast_sum(c.x, cl3);
}


fn d_f_div(a: vec2f, b: f32) -> vec2f
{
    let th = a.x / b;
    let p = f_f_mul(th,b);
    let dh = a.x - p.x;
    let dt = dh - p.y;
    let d = dt + a.y;
    let tl = d/b;
    return f_f_fast_sum(th,tl);
}

fn f_d_div(a: f32, b: vec2f) -> vec2f
{
    let th = 1. / b.x;
    let rh = 1. - b.x * th;
    let rl = -(b.y * th);
    let e = f_f_fast_sum(rh,rl);
    let d = d_f_mul(e,th);
    let m = d_f_add(d,th);
    let c = f_f_mul(m.x,a);
    let cl3 = fma(m.y,a,c.y);
    return f_f_fast_sum(c.x,cl3);
}


fn d_d_div(a: vec2f, b: vec2f) -> vec2f
{
    let th = 1/b.x;
    let rh = 1. - b.x * th;
    let rl = -(b.y * th);
    let e = f_f_fast_sum(rh,rl);
    let d = d_f_mul(e,th);
    let m = d_f_add(d,th);
    return d_d_mul(a,m);
}


fn d_sqrt(A: vec2f) -> vec2f
{
    let y: f32 = sqrt(A.x)   ;
    let t: vec2f = f_f_mul(y,y);
    let x = (A.x - t.x - t.y + A.y) * 0.5 / y;
    return f_f_fast_sum(y,x);
}

fn d_inverse(a: vec2f) -> vec2f {
    let y = 1 / a.x;
    let t = f_f_mul(y, a.x);
    let x = (1 - t.x - t.y - y*a.y) * y;
    return f_f_fast_sum(y,x);
}

// fn d_d_add(a: vec2f, b: vec2f) -> vec2f {
//     return vec2f(a.x+b.x,0.);
// }

// fn d_neg(a: vec2f) -> vec2f {
//     return vec2f(-a.x,-a.y);
// }

// fn d_d_sub(a: vec2f, b: vec2f) -> vec2f
// {
//     return vec2f(a.x-b.x);
// }

// fn d_d_mul(a: vec2f, b: vec2f) -> vec2f
// {
//     return vec2f(a.x*b.x,0.);
// }


// fn d_d_div(a: vec2f, b: vec2f) -> vec2f
// {
//     return vec2f(a.x/b.x,0.);
// }


// fn d_sqrt(A: vec2f) -> vec2f
// {
//     return vec2f(sqrt(A.x),0.);
// }

// fn d_inverse(a: vec2f) -> vec2f {
//     return vec2f(1./a.x,0.);
// }


@compute
@workgroup_size(1)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(num_workgroups) num_workgroups: vec3<u32>
) {

    let N = 2;

    var prev_texel:   vec4f;
    var curr_texel:   vec4f;
    var left_texel:   vec4f;
    var right_texel:  vec4f;
    var bottom_texel: vec4f;
    var top_texel:    vec4f;

    var next_state_texel: vec4f;


    if(data.ping_pong % 3 == 0) {
        curr_texel   = textureLoad( third_swap_texture, global_id.xy);
        prev_texel   = textureLoad(second_swap_texture, global_id.xy);
        left_texel   = textureLoad( third_swap_texture, vec2u(global_id.x-1,global_id.y  ));
        right_texel  = textureLoad( third_swap_texture, vec2u(global_id.x+1,global_id.y  ));
        top_texel    = textureLoad( third_swap_texture, vec2u(global_id.x  ,global_id.y-1));
        bottom_texel = textureLoad( third_swap_texture, vec2u(global_id.x  ,global_id.y+1));
    } else if (data.ping_pong % 3 == 1) {
        curr_texel   = textureLoad( first_swap_texture, global_id.xy);
        prev_texel   = textureLoad( third_swap_texture, global_id.xy);
        left_texel   = textureLoad( first_swap_texture, vec2u(global_id.x-1,global_id.y  ));
        right_texel  = textureLoad( first_swap_texture, vec2u(global_id.x+1,global_id.y  ));
        top_texel    = textureLoad( first_swap_texture, vec2u(global_id.x  ,global_id.y-1));
        bottom_texel = textureLoad( first_swap_texture, vec2u(global_id.x  ,global_id.y+1));
    } else if (data.ping_pong % 3 == 2) {
        curr_texel   = textureLoad(second_swap_texture, global_id.xy);
        prev_texel   = textureLoad( first_swap_texture, global_id.xy);
        left_texel   = textureLoad(second_swap_texture, vec2u(global_id.x-1,global_id.y  ));
        right_texel  = textureLoad(second_swap_texture, vec2u(global_id.x+1,global_id.y  ));
        top_texel    = textureLoad(second_swap_texture, vec2u(global_id.x  ,global_id.y-1));
        bottom_texel = textureLoad(second_swap_texture, vec2u(global_id.x  ,global_id.y+1));
    }
    
    var parameter_texel = textureLoad(parameter_texture, global_id.xy);

    var refraction_idx = parameter_texel.r;
    var source_amp = parameter_texel.g;
    var source_time =parameter_texel.b;
    var _unassigned = parameter_texel.a;

    if(data.time == 0) {
        refraction_idx = 1.;
        source_time = 10. * data.wavelength / data.lightspeed;
    }

    if (N==1 && 
        global_id.x == 1 && data.time <= source_time
    ) {
        // source_amp = (data.time/source_time)*(data.time/source_time) * 1.;
        source_amp =  1.;
    }

    if (N==2 &&
        global_id.x == 1 && data.time <= source_time && 
        abs(f32(global_id.y)/f32(num_workgroups.y)-0.5)<0.08
    ) {
        // source_amp = (data.time/source_time)*(data.time/source_time) * 1.;
        source_amp =  1.;
    }

    let k = 1.;

    if (N==2 && 
        f32(global_id.y) < k * f32(global_id.x) + f32(num_workgroups.y/2 - num_workgroups.x/2)
    ) {
        refraction_idx = 3.402823466E+12;
    }

    parameter_texel = vec4f(refraction_idx,source_amp,source_time,_unassigned);
    textureStore(parameter_texture, global_id.xy,parameter_texel);
    
    // var refraction = parameter_texel.r;
    // var amps = parameter_texel.g;
    // var maxtime = parameter_texel.b;

    if (source_amp != 0. && data.time < source_time && data.time>=0) {
        // next_state_texel.b = source_amp * sin(data.time/(data.wavelength/data.lightspeed*refraction_idx));
        next_state_texel.r = source_amp * sin(data.time/(data.wavelength/data.lightspeed*refraction_idx));
        next_state_texel.g = 0.;
        // next_state_im_tex.b = source_amp * cos(data.time/(data.wavelength/data.lightspeed*refraction_idx));
        next_state_texel.b = - source_amp * cos(data.time/(data.wavelength/data.lightspeed*refraction_idx));
        next_state_texel.a = 0.;
    }
     else if(global_id.x == 0 || global_id.x == num_workgroups.x-1 || global_id.y == 0 || global_id.y == num_workgroups.y-1) {

    } else {
        let TWO = vec2f(2.,0.);
        var difWRe = d_d_sub( d_d_add(left_texel.xy, right_texel.xy), d_d_mul(TWO,curr_texel.xy) );
        var difWIm = d_d_sub( d_d_add(left_texel.zw, right_texel.zw), d_d_mul(TWO,curr_texel.zw) );
        var difHRe = d_d_sub( d_d_add(top_texel.xy, bottom_texel.xy), d_d_mul(TWO,curr_texel.xy) );
        var difHIm = d_d_sub( d_d_add(top_texel.zw, bottom_texel.zw), d_d_mul(TWO,curr_texel.zw) );

        let next_state_re
            = 
                d_d_sub(
                d_d_add(
                    d_d_mul(
                        d_d_div(
                            d_d_mul(
                                d_d_mul(vec2f(data.lightspeed,0.),vec2f(data.lightspeed,0.)),
                                d_d_mul(vec2f(data.time_delta,0.),vec2f(data.time_delta,0.)),
                            ),
                            d_d_mul(vec2f(refraction_idx,0.),vec2f(refraction_idx,0.)),
                        ),
                        d_d_add(
                            d_d_div(
                                difWRe,
                                d_d_mul(vec2f(data.sizeW,0.),vec2f(data.sizeW,0.)),
                            ),
                            d_d_div(
                                difHRe,
                                d_d_mul(vec2f(data.sizeH,0.),vec2f(data.sizeH,0.)),
                            ),
                        )
                    ),
                    d_d_mul(vec2f(2.,0.),curr_texel.xy)
                ),
                    prev_texel.xy
                );

        next_state_texel.x = next_state_re.x;
        next_state_texel.y = next_state_re.y;

        let next_state_im
            = 
                d_d_sub(
                d_d_add(
                    d_d_mul(
                        d_d_div(
                            d_d_mul(
                                d_d_mul(vec2f(data.lightspeed,0.),vec2f(data.lightspeed,0.)),
                                d_d_mul(vec2f(data.time_delta,0.),vec2f(data.time_delta,0.)),
                            ),
                            d_d_mul(vec2f(refraction_idx,0.),vec2f(refraction_idx,0.)),
                        ),
                        d_d_add(
                            d_d_div(
                                difWIm,
                                d_d_mul(vec2f(data.sizeW,0.),vec2f(data.sizeW,0.)),
                            ),
                            d_d_div(
                                difHIm,
                                d_d_mul(vec2f(data.sizeH,0.),vec2f(data.sizeH,0.)),
                            ),
                        )
                    ),
                    d_d_mul(vec2f(2.,0.),curr_texel.zw)
                ),
                    prev_texel.zw
                );

        next_state_texel.z = next_state_im.x;
        next_state_texel.w = next_state_im.y;
        
        // next_state.g      =data.lightspeed*data.lightspeed*data.time_delta*data.time_delta/(parameter_texel.r*parameter_texel.r)(difW.x / (data.sizeW*data.sizeW) + difH.x / (data.sizeH * data.sizeH)) + 2.*curr_state.g - curr_state.r;
        // next_state.a = data.lightspeed*data.lightspeed * data.time_delta*data.time_delta / (parameter_texel.r*parameter_texel.r) * (difW.y / (data.sizeW*data.sizeW) + difH.y / (data.sizeH * data.sizeH)) + 2.*curr_state.a - curr_state.b;
    }


    if(data.ping_pong % 3 == 0) {
        textureStore(first_swap_texture, global_id.xy, next_state_texel);
        cumulative.c_first += next_state_texel.xz;
    } else if (data.ping_pong % 3 == 1) {
        textureStore(second_swap_texture, global_id.xy, next_state_texel);
        cumulative.c_second += next_state_texel.xz;
    } else if (data.ping_pong % 3 == 2) {
        textureStore(third_swap_texture, global_id.xy, next_state_texel);
        cumulative.c_third += next_state_texel.xz;
    }
}