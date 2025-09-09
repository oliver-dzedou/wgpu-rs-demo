// The structure of our vertex input
// Must match the struct defined in lib.rs
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>
}

// The structure of the output of our vertex shader
// Used as input for the fragment shader
struct VertexOutput { 
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

// The vertex shader
// Determines the position
@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}


// The fragment shader
// Determines the color
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
