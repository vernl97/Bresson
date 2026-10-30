// Editor preview: density → print curves → look → display sRGB.
// Mirrors pipeline::Renderer::positive / output (without sharpening).

struct Params {
    // Part of the negative to draw: u0, v0, u1, v1.
    uv: vec4<f32>,
    // Linear MelissaRGB → linear sRGB, rows; w: sRGB luminance weights.
    out0: vec4<f32>,
    out1: vec4<f32>,
    out2: vec4<f32>,
    // Look: strength, saturation, has tables (0/1), identity (0/1).
    look: vec4<f32>,
    // Curve tables: density at entry 0, at the last entry, entries,
    // linear output (1 for sRGB render targets).
    lut: vec4<f32>,
    // Negative width, height.
    size: vec4<f32>,
    // Dodge: key density, light and dark compression (0 = off), knee².
    dodge: vec4<f32>,
    // Dodge gain per channel.
    gain: vec4<f32>,
};

@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var neg: texture_2d<f32>;
// Curve tables (rgb per entry), then the look map, then its grey curves.
@group(0) @binding(2) var<storage, read> tables: array<vec4<f32>>;

const HUES: u32 = 24u;
const SATS: u32 = 5u;
const VALUES: u32 = 8u;

struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs(@builtin(vertex_index) i: u32) -> VOut {
    // One triangle covering the viewport (the callback's rectangle).
    let x = f32((i << 1u) & 2u);
    let y = f32(i & 2u);
    var o: VOut;
    o.pos = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    o.uv = vec2<f32>(x, y);
    return o;
}

// Densities and (alpha) the low-pass density.
fn density(uv: vec2<f32>) -> vec4<f32> {
    let dims = p.size.xy;
    let xy = uv * dims - 0.5;
    let f = fract(xy);
    let i0 = vec2<i32>(floor(xy));
    let mx = vec2<i32>(dims) - vec2<i32>(1, 1);
    let a = textureLoad(neg, clamp(i0, vec2<i32>(0, 0), mx), 0);
    let b = textureLoad(neg, clamp(i0 + vec2<i32>(1, 0), vec2<i32>(0, 0), mx), 0);
    let c = textureLoad(neg, clamp(i0 + vec2<i32>(0, 1), vec2<i32>(0, 0), mx), 0);
    let d = textureLoad(neg, clamp(i0 + vec2<i32>(1, 1), vec2<i32>(0, 0), mx), 0);
    return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}

fn curve(ch: u32, d: f32) -> f32 {
    let n = p.lut.z;
    let x = clamp((d - p.lut.x) / (p.lut.y - p.lut.x) * (n - 1.0), 0.0, n - 1.0);
    let i = min(u32(x), u32(n) - 1u);
    let j = min(i + 1u, u32(n) - 1u);
    let t = x - f32(i);
    let a = tables[i][ch];
    return a + (tables[j][ch] - a) * t;
}

fn rem(x: f32, m: f32) -> f32 {
    return x - m * floor(x / m);
}

fn rgb_to_hsv(c: vec3<f32>) -> vec3<f32> {
    let mx = max(c.r, max(c.g, c.b));
    let mn = min(c.r, min(c.g, c.b));
    let d = mx - mn;
    var s = 0.0;
    if (mx > 0.0) {
        s = d / mx;
    }
    var h = 0.0;
    if (d > 1e-9) {
        if (mx == c.r) {
            h = 60.0 * rem((c.g - c.b) / d, 6.0);
        } else if (mx == c.g) {
            h = 60.0 * ((c.b - c.r) / d + 2.0);
        } else {
            h = 60.0 * ((c.r - c.g) / d + 4.0);
        }
    }
    return vec3<f32>(h, s, mx);
}

fn hsv_to_rgb(hue: f32, s: f32, v: f32) -> vec3<f32> {
    if (s <= 0.0) {
        return vec3<f32>(v, v, v);
    }
    let h = rem(hue, 360.0) / 60.0;
    let i = floor(h);
    let f = h - i;
    let pp = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));
    switch (i32(i)) {
        case 0: { return vec3<f32>(v, t, pp); }
        case 1: { return vec3<f32>(q, v, pp); }
        case 2: { return vec3<f32>(pp, v, t); }
        case 3: { return vec3<f32>(pp, q, v); }
        case 4: { return vec3<f32>(t, pp, v); }
        default: { return vec3<f32>(v, pp, q); }
    }
}

// Node index and weight of the next node, for `x` in 0..1 over `n` nodes.
fn node(x: f32, n: u32) -> vec2<f32> {
    let q = clamp(x, 0.0, 1.0) * f32(n - 1u);
    let i = min(u32(q), n - 2u);
    return vec2<f32>(f32(i), q - f32(i));
}

fn grid(h: u32, s: u32, v: u32) -> vec3<f32> {
    let base = u32(p.lut.z);
    return tables[base + (h * SATS + s) * VALUES + v].xyz;
}

fn grey(v: f32) -> vec3<f32> {
    let base = u32(p.lut.z) + HUES * SATS * VALUES;
    let n = node(v, 11u);
    let i = u32(n.x);
    return mix(tables[base + i].xyz, tables[base + i + 1u].xyz, n.y);
}

// Saturation scale eased back to 1 on skin tones (engine::look::skin_kept).
fn skin_kept(h: f32, s: f32, sat: f32) -> f32 {
    let dh = abs(rem(h - 25.0 + 180.0, 360.0) - 180.0);
    let hue = clamp(1.0 - dh / 20.0, 0.0, 1.0);
    let chroma = clamp((0.75 - s) / 0.25, 0.0, 1.0);
    let skin = hue * hue * (3.0 - 2.0 * hue) * chroma;
    return sat + (1.0 - sat) * skin;
}

fn look(c: vec3<f32>) -> vec3<f32> {
    if (p.look.w > 0.5) {
        return c;
    }
    let hsv = rgb_to_hsv(clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)));
    let h = hsv.x;
    let s = hsv.y;
    let v = hsv.z;
    let k = p.look.x;
    let sat = skin_kept(h, s, p.look.y);
    if (p.look.z < 0.5 || k <= 0.0) {
        return hsv_to_rgb(h, min(s * sat, 1.0), v);
    }
    let hp = rem(h, 360.0) / 360.0 * f32(HUES);
    let hi = u32(hp) % HUES;
    let ht = hp - floor(hp);
    let sn = node(s, SATS);
    let vn = node(v, VALUES);
    var acc = vec3<f32>(0.0);
    for (var a = 0u; a < 2u; a++) {
        let hh = (hi + a) % HUES;
        let hw = select(1.0 - ht, ht, a == 1u);
        for (var b = 0u; b < 2u; b++) {
            let sw = select(1.0 - sn.y, sn.y, b == 1u);
            for (var e = 0u; e < 2u; e++) {
                let vw = select(1.0 - vn.y, vn.y, e == 1u);
                acc += hw * sw * vw * grid(hh, u32(sn.x) + b, u32(vn.x) + e);
            }
        }
    }
    let g = grey(v);
    let gv = (g.r + g.g + g.b) / 3.0;
    let v2 = v + (gv + acc.z - v) * k;
    let s2 = clamp(s * (1.0 + (acc.y - 1.0) * k) * sat, 0.0, 1.0);
    let o = hsv_to_rgb(h + acc.x * k, s2, clamp(v2, 0.0, 1.0));
    return o + (g - vec3<f32>(gv)) * (1.0 - s) * k;
}

fn srgb_decode(v: f32) -> f32 {
    if (v <= 0.04045) {
        return v / 12.92;
    }
    return pow((v + 0.055) / 1.055, 2.4);
}

fn srgb_encode(v: f32) -> f32 {
    if (v <= 0.0031308) {
        return v * 12.92;
    }
    return 1.055 * pow(v, 1.0 / 2.4) - 0.055;
}

// Soft gamut compression towards the grey of the same luminance
// (color::compress_gamut).
const GAMUT_KNEE: f32 = 0.8;

fn compress_gamut(o: vec3<f32>) -> vec3<f32> {
    let g = clamp(dot(vec3<f32>(p.out0.w, p.out1.w, p.out2.w), o), 0.0, 1.0);
    var d = 0.0;
    for (var c = 0u; c < 3u; c++) {
        let v = o[c];
        if (v < g) {
            d = max(d, (g - v) / max(g, 1e-6));
        } else if (v > g) {
            d = max(d, (v - g) / max(1.0 - g, 1e-6));
        }
    }
    if (d <= GAMUT_KNEE) {
        return o;
    }
    let x = (d - GAMUT_KNEE) / (1.0 - GAMUT_KNEE);
    let k = (GAMUT_KNEE + (1.0 - GAMUT_KNEE) * x / (1.0 + x)) / d;
    return vec3<f32>(g) + (o - vec3<f32>(g)) * k;
}

@fragment
fn fs(in: VOut) -> @location(0) vec4<f32> {
    let uv = mix(p.uv.xy, p.uv.zw, in.uv);
    let n = density(uv);
    // Dodge (see engine::dodge): offset from the low-pass density's
    // departure from the key, nothing near it.
    let u = n.a - p.dodge.x;
    var g = p.dodge.z;
    if (u > 0.0) {
        g = p.dodge.y;
    }
    let off = -g * u * u * u / (u * u + p.dodge.w + 1e-9);
    let d = n.rgb + off * p.gain.xyz;
    let pos = look(vec3<f32>(curve(0u, d.r), curve(1u, d.g), curve(2u, d.b)));
    let c = clamp(pos, vec3<f32>(0.0), vec3<f32>(1.0));
    let lin = vec3<f32>(srgb_decode(c.r), srgb_decode(c.g), srgb_decode(c.b));
    let o = clamp(
        compress_gamut(vec3<f32>(dot(p.out0.xyz, lin), dot(p.out1.xyz, lin), dot(p.out2.xyz, lin))),
        vec3<f32>(0.0),
        vec3<f32>(1.0),
    );
    if (p.lut.w > 0.5) {
        return vec4<f32>(o, 1.0);
    }
    return vec4<f32>(srgb_encode(o.r), srgb_encode(o.g), srgb_encode(o.b), 1.0);
}
