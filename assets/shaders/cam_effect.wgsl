#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct PostProcessSettings {
    intensity: f32, // 0.0 = normal, 1.0 = full dream effect
#ifdef SIXTEEN_BYTE_ALIGNMENT
    _webgl2_padding: vec3<f32>
#endif
}
@group(0) @binding(2) var<uniform> settings: PostProcessSettings;

fn hash(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3(p.xyx) * 0.13);
    p3 += dot(p3, p3.yzx + 3.333);
    return fract((p3.x + p3.y) * p3.z);
}

fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    
    let a = hash(i);
    let b = hash(i + vec2(1.0, 0.0));
    let c = hash(i + vec2(0.0, 1.0));
    let d = hash(i + vec2(1.0, 1.0));
    
    let u = f * f * (3.0 - 2.0 * f);
    
    return mix(a, b, u.x) + (c - a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let intensity = settings.intensity;
    
    let blur_amount = intensity * 0.003;
    let samples = 8;
    var color_sum = vec3(0.0);
    for (var i = 0; i < samples; i++) {
        let angle = f32(i) * 0.785398; // PI/4
        let offset = vec2(cos(angle), sin(angle)) * blur_amount;
        
        // Chromatic aberration on blur samples
        let r = textureSample(screen_texture, texture_sampler, in.uv + offset * 1.2).r;
        let g = textureSample(screen_texture, texture_sampler, in.uv + offset).g;
        let b = textureSample(screen_texture, texture_sampler, in.uv + offset * 0.8).b;
        
        color_sum += vec3(r, g, b);
    }
    
    let blurred = color_sum / f32(samples);
    let original = textureSample(screen_texture, texture_sampler, in.uv).rgb;
    let vignette_strength = intensity * 0.5;
    let vignette_center = in.uv - 0.5;
    let vignette = 1.0 - dot(vignette_center, vignette_center) * vignette_strength;
    
    let hue_shift = intensity * 0.1;
    let shifted = vec3(
        blurred.r + hue_shift,
        blurred.g,
        blurred.b - hue_shift * 0.5
    );
    var final_color = mix(original, shifted, intensity);
    final_color *= vignette;
    let luminance = dot(final_color, vec3(0.299, 0.587, 0.114));
    final_color = mix(final_color, vec3(luminance), intensity * 0.3);
    final_color *= 1.0 + intensity * 0.2;
    
    return vec4(final_color, 1.0);
}
