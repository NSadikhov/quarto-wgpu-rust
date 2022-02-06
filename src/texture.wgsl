// Vertex shader

struct TextureVertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] tex_coords: vec2<f32>;
    [[location(2)]] normal: vec3<f32>;
    // [[location(3)]] tangent: vec3<f32>;
    // [[location(4)]] bitangent: vec3<f32>;
};

struct TextureVertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] tex_coords: vec2<f32>;
    [[location(1)]] world_normal: vec3<f32>;
    [[location(2)]] world_position: vec3<f32>;
    // [[location(1)]] tangent_position: vec3<f32>;
    // [[location(2)]] tangent_light_position: vec3<f32>;
    // [[location(3)]] tangent_view_position: vec3<f32>;
};

[[block]]
struct CameraUniform {
    view_position: vec4<f32>;
    view_proj: mat4x4<f32>;
};

[[group(1), binding(0)]]
var<uniform> camera: CameraUniform;

[[block]]
struct LightUniform {
    position: vec3<f32>;
    color: vec3<f32>;
};

[[group(2), binding(0)]]
var<uniform> light: LightUniform;

struct InstanceInput {
    // *Model
    [[location(5)]] model_matrix_0: vec4<f32>;
    [[location(6)]] model_matrix_1: vec4<f32>;
    [[location(7)]] model_matrix_2: vec4<f32>;
    [[location(8)]] model_matrix_3: vec4<f32>;
    // *Normal
    [[location(9)]] normal_matrix_0: vec3<f32>;
    [[location(10)]] normal_matrix_1: vec3<f32>;
    [[location(11)]] normal_matrix_2: vec3<f32>;
};


[[stage(vertex)]]
fn vs_main(
    model: TextureVertexInput, // set_vertex_buffer -> slot(0)
    instance: InstanceInput, // set_vertex_buffer -> slot(1)
) -> TextureVertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    let normal_matrix = mat3x3<f32>(
        instance.normal_matrix_0,
        instance.normal_matrix_1,
        instance.normal_matrix_2,
    );

    // Construct the tangent matrix
    // let world_normal = normalize(normal_matrix * model.normal);
    // let world_tangent = normalize(normal_matrix * model.tangent);
    // let world_bitangent = normalize(normal_matrix * model.bitangent);
    // let tangent_matrix = transpose(mat3x3<f32>(
    //     world_tangent,
    //     world_bitangent,
    //     world_normal,
    // ));
    // let world_position = model_matrix * vec4<f32>(model.position, 1.0);
    

    var out: TextureVertexOutput;
    out.tex_coords = model.tex_coords;
    // out.color = model.color;

    out.world_normal = normal_matrix * model.normal;
    var world_position: vec4<f32> = model_matrix * vec4<f32>(model.position, 1.0); 
    out.world_position = world_position.xyz;

    out.clip_position = camera.view_proj * world_position;
    
    // out.tangent_position = tangent_matrix * world_position.xyz;
    // out.tangent_view_position = tangent_matrix * camera.view_position.xyz;
    // out.tangent_light_position = tangent_matrix * light.position;
    return out;
}

// Fragment shader

// *Diffuse map
[[group(0), binding(0)]]
var t_diffuse: texture_2d<f32>;
[[group(0), binding(1)]]
var s_diffuse: sampler;

// *Normal map
// [[group(0), binding(2)]]
// var t_normal: texture_2d<f32>;
// [[group(0), binding(3)]]
// var s_normal: sampler;


// let coordinate_system = mat3x3<f32>(
//     vec3(1, 0, 0), // x axis (right)
//     vec3(0, 1, 0), // y axis (up)
//     vec3(0, 0, 1)  // z axis (forward)
// );

[[stage(fragment)]]
fn fs_arrow_main(in: TextureVertexOutput) -> [[location(0)]] vec4<f32> {
    var object_color : vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    return object_color;
}

[[stage(fragment)]]
fn fs_main(in: TextureVertexOutput) -> [[location(0)]] vec4<f32> {
    // return textureSample(t_diffuse, s_diffuse, in.tex_coords);

      var object_color : vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);


    // let object_normal: vec4<f32> = textureSample(t_normal, s_normal, in.tex_coords);

    // Create the lighting vectors
    // let tangent_normal = object_normal.xyz * 2.0 - 1.0;
    // let light_direction = normalize(in.tangent_light_position - in.tangent_position);
    // let view_direction = normalize(in.tangent_view_position - in.tangent_position);

    // *Ambient
    // We don't need (or want) much ambient light, so 0.1 is fine
    let ambient_strength = 0.1;
    let ambient_color = light.color * ambient_strength;

    // *Diffuse
    let light_direction = normalize(light.position - in.world_position);
    let diffuse_strength = max(dot(in.world_normal, light_direction), 0.0);
    // let diffuse_strength = max(dot(tangent_normal, light_direction), 0.0);

    let diffuse_color = light.color * diffuse_strength;

    // *Specular
    let view_direction = normalize(camera.view_position.xyz - in.world_position);
    // let reflect_direction = reflect(-light_direction, in.world_normal); // used in Phong model
    let half_dir = normalize(view_direction + light_direction); 

    // let specular_strength = pow(max(dot(view_direction, reflect_direction), 0.0), 32.0);
    let specular_strength = pow(max(dot(in.world_normal, half_dir), 0.0), 32.0);
    // let specular_strength = pow(max(dot(tangent_normal, half_dir), 0.0), 32.0);
    let specular_color = light.color * specular_strength;

    let result = (ambient_color + diffuse_color + specular_color) * object_color.xyz;

    return vec4<f32>(result, object_color.a);
}



// TODO: Check out if condition statement in wgsl
// to have 1 vertex main and 1 fragment main
// render_pipeline can have many buffers in vertex field
