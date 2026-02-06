#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
    pbr_functions::apply_pbr_lighting,
    pbr_functions::main_pass_post_lighting_processing,
    forward_io::{VertexOutput, FragmentOutput},
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    // Tile the UV within the atlas tile.
    // UV_0 (in.uv) has tiling coordinates that extend beyond the tile boundary for merged quads.
    // UV_1 (in.uv_b) has the tile origin [u_min, v_min].
    var modified = in;
    let tile_size = vec2<f32>(1.0 / 16.0, 1.0 / 16.0);
    let local = in.uv - in.uv_b;
    // Wrap local coordinates within one tile using modulo
    let wrapped_local = local - floor(local / tile_size) * tile_size;
    modified.uv = in.uv_b + wrapped_local;

    // Standard PBR pipeline with tiled UVs
    var pbr_input = pbr_input_from_standard_material(modified, is_front);
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
