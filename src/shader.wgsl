struct G {
  screen: vec4f,
  low: vec4f,
  amb_out: vec4f,
  amb_in: vec4f,
  misc: vec4f,
};
struct Light { p: vec4f, c: vec4f };

@group(0) @binding(0) var<uniform> g: G;
@group(0) @binding(1) var<storage, read> lights: array<Light>;
@group(0) @binding(2) var t0: texture_2d<f32>;
@group(0) @binding(3) var t1: texture_2d<f32>;
@group(0) @binding(4) var t2: texture_2d<f32>;
@group(0) @binding(5) var t3: texture_2d<f32>;
@group(0) @binding(6) var sn: sampler;
@group(0) @binding(7) var sl: sampler;

struct VO { @builtin(position) pos: vec4f, @location(0) uv: vec2f };

@vertex fn vs(@builtin(vertex_index) i: u32) -> VO {
  let x = f32((i << 1u) & 2u);
  let y = f32(i & 2u);
  var o: VO;
  o.pos = vec4f(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
  o.uv = vec2f(x, y);
  return o;
}

fn lin(c: vec3f) -> vec3f { return pow(max(c, vec3f(0.0)), vec3f(2.2)); }

fn hash(p: vec2f) -> f32 {
  return fract(sin(dot(p, vec2f(12.9898, 78.233))) * 43758.5453);
}

// lighting: every low-res pixel gathers the point lights with ray-marched wall shadows
@fragment fn fs_light(i: VO) -> @location(0) vec4f {
  let dim = g.low.xy;
  let px = i.uv * dim;
  let ip = vec2i(px);
  let alb = textureLoad(t0, ip, 0);
  let em = textureLoad(t1, ip, 0);
  let inf = textureLoad(t2, ip, 0);
  let dy = inf.r * 128.0;
  let sp = vec2f(px.x, px.y + dy);
  var light = mix(g.amb_out.rgb, g.amb_in.rgb, inf.b);
  let n = u32(g.misc.x);
  for (var k = 0u; k < n; k++) {
    let L = lights[k];
    let d = sp - L.p.xy;
    let dist = length(d);
    if (dist > L.p.z) { continue; }
    var att = 1.0 - dist / L.p.z;
    att = att * att;
    var vis = 1.0;
    let steps = min(i32(dist / 1.5), 96);
    for (var s = 2; s < steps; s++) {
      let t = f32(s) / f32(steps);
      if (dist * (1.0 - t) < 2.5) { break; }
      let q = L.p.xy + d * t;
      if (q.x < 0.0 || q.y < 0.0 || q.x >= dim.x || q.y >= dim.y) { continue; }
      let o = textureLoad(t2, vec2i(q), 0).g;
      if (o > 0.5) { vis = vis * 0.55; if (vis < 0.1) { vis = 0.0; break; } }
    }
    light += L.c.rgb * L.p.w * att * vis;
  }
  let col = lin(alb.rgb) * light + lin(em.rgb) * 2.6;
  return vec4f(col, 1.0);
}

@fragment fn fs_bright(i: VO) -> @location(0) vec4f {
  let ts = 1.0 / vec2f(textureDimensions(t0));
  var c = textureSampleLevel(t0, sl, i.uv + ts * vec2f(-0.5, -0.5), 0.0).rgb;
  c += textureSampleLevel(t0, sl, i.uv + ts * vec2f(0.5, -0.5), 0.0).rgb;
  c += textureSampleLevel(t0, sl, i.uv + ts * vec2f(-0.5, 0.5), 0.0).rgb;
  c += textureSampleLevel(t0, sl, i.uv + ts * vec2f(0.5, 0.5), 0.0).rgb;
  c *= 0.25;
  let l = max(c.r, max(c.g, c.b));
  let k = max(l - 0.7, 0.0) / max(l, 0.0001);
  return vec4f(c * k, 1.0);
}

@fragment fn fs_down(i: VO) -> @location(0) vec4f {
  let ts = 1.0 / vec2f(textureDimensions(t0));
  var c = textureSampleLevel(t0, sl, i.uv + ts * vec2f(-1.0, -1.0), 0.0).rgb;
  c += textureSampleLevel(t0, sl, i.uv + ts * vec2f(1.0, -1.0), 0.0).rgb;
  c += textureSampleLevel(t0, sl, i.uv + ts * vec2f(-1.0, 1.0), 0.0).rgb;
  c += textureSampleLevel(t0, sl, i.uv + ts * vec2f(1.0, 1.0), 0.0).rgb;
  return vec4f(c * 0.25, 1.0);
}

fn blur(uv: vec2f, dir: vec2f) -> vec4f {
  let ts = dir / vec2f(textureDimensions(t0));
  var c = textureSampleLevel(t0, sl, uv, 0.0).rgb * 0.227027;
  c += (textureSampleLevel(t0, sl, uv + ts * 1.384615, 0.0).rgb + textureSampleLevel(t0, sl, uv - ts * 1.384615, 0.0).rgb) * 0.316216;
  c += (textureSampleLevel(t0, sl, uv + ts * 3.230769, 0.0).rgb + textureSampleLevel(t0, sl, uv - ts * 3.230769, 0.0).rgb) * 0.070270;
  return vec4f(c, 1.0);
}
@fragment fn fs_blur_h(i: VO) -> @location(0) vec4f { return blur(i.uv, vec2f(1.0, 0.0)); }
@fragment fn fs_blur_v(i: VO) -> @location(0) vec4f { return blur(i.uv, vec2f(0.0, 1.0)); }

fn aces(x: vec3f) -> vec3f {
  let a = 2.51; let b = 0.03; let c = 2.43; let d = 0.59; let e = 0.14;
  return clamp((x * (a * x + b)) / (x * (c * x + d) + e), vec3f(0.0), vec3f(1.0));
}

// final: pixel-perfect upscale, bloom, tonemap, grade, vignette, grain, UI on top
@fragment fn fs_final(i: VO) -> @location(0) vec4f {
  let sc = g.screen.z;
  let p = i.uv * g.screen.xy - g.low.zw;
  let uv = p / sc / g.low.xy;
  if (uv.x < 0.0 || uv.y < 0.0 || uv.x > 1.0 || uv.y > 1.0) { return vec4f(0.0, 0.0, 0.0, 1.0); }
  let hurt = g.amb_in.w;
  let v = uv - vec2f(0.5);
  let r2 = dot(v, v);
  let ca = (0.0015 + hurt * 0.006) * r2 * 4.0;
  var c = vec3f(
    textureSampleLevel(t0, sn, uv + v * ca, 0.0).r,
    textureSampleLevel(t0, sn, uv, 0.0).g,
    textureSampleLevel(t0, sn, uv - v * ca, 0.0).b);
  let b1 = textureSampleLevel(t1, sl, uv, 0.0).rgb;
  let b2 = textureSampleLevel(t2, sl, uv, 0.0).rgb;
  c += b1 * 0.55 + b2 * 0.9;
  c = c * (1.0 + g.amb_out.w * 3.0) + vec3f(0.75, 0.82, 1.0) * g.amb_out.w * 0.12;
  c = aces(c * g.misc.w);
  // cool shadows, warm highlights
  let lum = dot(c, vec3f(0.299, 0.587, 0.114));
  c = mix(c * vec3f(0.92, 0.97, 1.08), c * vec3f(1.05, 1.0, 0.94), smoothstep(0.15, 0.7, lum));
  c *= 1.0 - r2 * g.misc.z;
  c = mix(c, vec3f(0.45, 0.0, 0.02), clamp(hurt * smoothstep(0.12, 0.5, r2) * 0.9, 0.0, 0.6));
  c = pow(max(c, vec3f(0.0)), vec3f(1.0 / 2.2));
  c += (hash(floor(p / sc) + vec2f(g.screen.w * 13.1, g.screen.w * 7.7)) - 0.5) * 0.014;
  let u = textureSampleLevel(t3, sn, uv, 0.0);
  c = mix(c, u.rgb, u.a);
  return vec4f(c, 1.0);
}
