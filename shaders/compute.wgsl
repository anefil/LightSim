@group(0)
@binding(0)
var first_swap_texture_re: texture_storage_2d<rgba32float, read_write>; 
 
@group(0)
@binding(1)
var second_swap_texture_re: texture_storage_2d<rgba32float, read_write>; 

@group(0)
@binding(2)
var first_swap_texture_im: texture_storage_2d<rgba32float, read_write>; 
 
@group(0)
@binding(3)
var second_swap_texture_im: texture_storage_2d<rgba32float, read_write>; 

@group(0)
@binding(4)
var parameters_texture: texture_storage_2d<rgba32float, read_write>; 


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

@group(0)
@binding(6)
var<storage, read_write> cumulative: vec4f;

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
    
    var curr_state_re_tex = textureLoad( first_swap_texture_re, global_id.xy);
    // var curr_state_re_tex = textureLoad( first_swap_texture_im, global_id.xy);
    var next_state_re_tex = textureLoad(second_swap_texture_re, global_id.xy);
    // var next_state_re_tex = textureLoad(second_swap_texture_im, global_id.xy);
    var curr_state_im_tex = textureLoad( first_swap_texture_im, global_id.xy);
    // var curr_state_im_tex = textureLoad( first_swap_texture_re, global_id.xy);
    var next_state_im_tex = textureLoad(second_swap_texture_im, global_id.xy);
    // var next_state_im_tex = textureLoad(second_swap_texture_re, global_id.xy);

    if(data.ping_pong == 1) {
        var t = curr_state_re_tex;
        curr_state_re_tex = next_state_re_tex;
        next_state_re_tex = t;

        var l = curr_state_im_tex;
        curr_state_im_tex = next_state_im_tex;
        next_state_im_tex = l;
    }

    let prev_state_re = curr_state_re_tex.xy;
    let curr_state_re = curr_state_re_tex.zw;
    let prev_state_im = curr_state_im_tex.xy;
    let curr_state_im = curr_state_im_tex.zw;
    
    var parameters = textureLoad(parameters_texture, global_id.xy);

    var refraction_idx = parameters.r;
    var source_amp = parameters.g;
    var source_time =parameters.b;
    var _unassigned = parameters.a;
    

    if(data.time == 0) {
        refraction_idx = 1.;
        source_time = 10. * data.wavelength / data.lightspeed;
    }

    if (
        // abs(f32(global_id.y)/f32(num_workgroups.y)-0.5)<0.03 
        // && 
        N==1 && global_id.x == 1 && data.time <= source_time 
    ) {
        // source_amp = (data.time/source_time)*(data.time/source_time) * 1.;
        source_amp =  1.;
    }

    if (
        abs(f32(global_id.y)/f32(num_workgroups.y)-0.5)<0.08
        && 
        N==2 && global_id.x == 1 && data.time <= source_time 
    ) {
        // source_amp = (data.time/source_time)*(data.time/source_time) * 1.;
        source_amp =  1.;
    }

    let k = 1.;

    if(
        N==2 && f32(global_id.y) < k * f32(global_id.x) + f32(num_workgroups.y/2 - num_workgroups.x/2)
    ) {
        refraction_idx = 3.402823466E+12;
    }

    parameters = vec4f(refraction_idx,source_amp,source_time,_unassigned);
    textureStore(parameters_texture, global_id.xy,parameters);
    
    // var refraction = parameters.r;
    // var amps = parameters.g;
    // var maxtime = parameters.b;

    next_state_re_tex.r = curr_state_re.r;
    next_state_re_tex.g = curr_state_re.g;
    next_state_im_tex.r = curr_state_im.r;
    next_state_im_tex.g = curr_state_im.g;

    if (source_amp != 0. && data.time < source_time && data.time>=0) {
        // next_state_re_tex.b = source_amp * sin(data.time/(data.wavelength/data.lightspeed*refraction_idx));
        next_state_re_tex.b = source_amp * sin(data.time/(data.wavelength/data.lightspeed*refraction_idx));
        next_state_re_tex.a = 0.;
        // next_state_im_tex.b = source_amp * cos(data.time/(data.wavelength/data.lightspeed*refraction_idx));
        next_state_im_tex.b = - source_amp * cos(data.time/(data.wavelength/data.lightspeed*refraction_idx));
        next_state_im_tex.a = 0.;
    }
     else if(global_id.x == 0 || global_id.x == num_workgroups.x-1 || global_id.y == 0 || global_id.y == num_workgroups.y-1) {

    } else {
        var left_tex_re  : vec2f;
        var left_tex_im  : vec2f;
        var right_tex_re : vec2f;
        var right_tex_im : vec2f;
        var top_tex_re   : vec2f;
        var top_tex_im   : vec2f;
        var bottom_tex_re: vec2f;
        var bottom_tex_im: vec2f;
        if(data.ping_pong == 1) {
            left_tex_re   = textureLoad(second_swap_texture_re, vec2<u32>(global_id.x-1,global_id.y)).zw;
            left_tex_im   = textureLoad(second_swap_texture_im, vec2<u32>(global_id.x-1,global_id.y)).zw;
            right_tex_re  = textureLoad(second_swap_texture_re, vec2<u32>(global_id.x+1,global_id.y)).zw;
            right_tex_im  = textureLoad(second_swap_texture_im, vec2<u32>(global_id.x+1,global_id.y)).zw;
            top_tex_re    = textureLoad(second_swap_texture_re, vec2<u32>(global_id.x,global_id.y-1)).zw;
            top_tex_im    = textureLoad(second_swap_texture_im, vec2<u32>(global_id.x,global_id.y-1)).zw;
            bottom_tex_re = textureLoad(second_swap_texture_re, vec2<u32>(global_id.x,global_id.y+1)).zw;
            bottom_tex_im = textureLoad(second_swap_texture_im, vec2<u32>(global_id.x,global_id.y+1)).zw;
        } else {
            left_tex_re   = textureLoad(first_swap_texture_re, vec2<u32>(global_id.x-1,global_id.y)).zw;
            left_tex_im   = textureLoad(first_swap_texture_im, vec2<u32>(global_id.x-1,global_id.y)).zw;
            right_tex_re  = textureLoad(first_swap_texture_re, vec2<u32>(global_id.x+1,global_id.y)).zw;
            right_tex_im  = textureLoad(first_swap_texture_im, vec2<u32>(global_id.x+1,global_id.y)).zw;
            top_tex_re    = textureLoad(first_swap_texture_re, vec2<u32>(global_id.x,global_id.y-1)).zw;
            top_tex_im    = textureLoad(first_swap_texture_im, vec2<u32>(global_id.x,global_id.y-1)).zw;
            bottom_tex_re = textureLoad(first_swap_texture_re, vec2<u32>(global_id.x,global_id.y+1)).zw;
            bottom_tex_im = textureLoad(first_swap_texture_im, vec2<u32>(global_id.x,global_id.y+1)).zw;
        }

        let TWO = vec2f(2.,0.);
        var difWRe = d_d_sub( d_d_add(left_tex_re, right_tex_re), d_d_mul(TWO,curr_state_re) );
        var difWIm = d_d_sub( d_d_add(left_tex_im, right_tex_im), d_d_mul(TWO,curr_state_im) );
        var difHRe = d_d_sub( d_d_add(top_tex_re, bottom_tex_re), d_d_mul(TWO,curr_state_re) );
        var difHIm = d_d_sub( d_d_add(top_tex_im, bottom_tex_im), d_d_mul(TWO,curr_state_im) );

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
                    d_d_mul(vec2f(2.,0.),curr_state_re)
                ),
                    prev_state_re
                );

        next_state_re_tex.b = next_state_re.x;
        next_state_re_tex.a = next_state_re.y;

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
                    d_d_mul(vec2f(2.,0.),curr_state_im)
                ),
                    prev_state_im
                );

        next_state_im_tex.b = next_state_im.x;
        next_state_im_tex.a = next_state_im.y;
        
        // next_state.g      =data.lightspeed*data.lightspeed*data.time_delta*data.time_delta/(parameters.r*parameters.r)(difW.x / (data.sizeW*data.sizeW) + difH.x / (data.sizeH * data.sizeH)) + 2.*curr_state.g - curr_state.r;
        // next_state.a = data.lightspeed*data.lightspeed * data.time_delta*data.time_delta / (parameters.r*parameters.r) * (difW.y / (data.sizeW*data.sizeW) + difH.y / (data.sizeH * data.sizeH)) + 2.*curr_state.a - curr_state.b;
    }


    if(data.ping_pong == 0) {
        textureStore(second_swap_texture_re, global_id.xy, next_state_re_tex);
        textureStore(second_swap_texture_im, global_id.xy, next_state_im_tex);
        cumulative.r += next_state_re_tex.b;
        cumulative.g += next_state_im_tex.b;
    } else {
        textureStore(first_swap_texture_re, global_id.xy, next_state_re_tex);
        textureStore(first_swap_texture_im, global_id.xy, next_state_im_tex);
        cumulative.b += next_state_re_tex.b;
        cumulative.a += next_state_im_tex.b;
    }
}