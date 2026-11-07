// GPU side: the CPU paints low-res albedo/emission/info/UI buffers each frame,
// the GPU lights them (shadowed point lights), blooms, tonemaps and upscales.
use std::sync::Arc;
use winit::window::Window;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct Globals {
    pub screen: [f32; 4],
    pub low: [f32; 4],
    pub amb_out: [f32; 4],
    pub amb_in: [f32; 4],
    pub misc: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Default)]
pub struct GLight {
    pub p: [f32; 4],
    pub c: [f32; 4],
}

pub const MAX_LIGHTS: usize = 192;

pub struct Frame {
    pub w: usize,
    pub h: usize,
    pub alb: Vec<u32>,
    pub emi: Vec<u32>,
    pub inf: Vec<u32>,
    pub ui: Vec<u32>,
    pub z: Vec<f32>,
}
impl Frame {
    pub fn new(w: usize, h: usize) -> Self {
        let n = w * h;
        Frame { w, h, alb: vec![0; n], emi: vec![0; n], inf: vec![0; n], ui: vec![0; n], z: vec![0.0; n] }
    }
}

struct Tex {
    tex: wgpu::Texture,
    view: wgpu::TextureView,
}

pub struct Gfx {
    pub window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    layout: wgpu::BindGroupLayout,
    p_light: wgpu::RenderPipeline,
    p_bright: wgpu::RenderPipeline,
    p_down: wgpu::RenderPipeline,
    p_blur_h: wgpu::RenderPipeline,
    p_blur_v: wgpu::RenderPipeline,
    p_final: wgpu::RenderPipeline,
    ubuf: wgpu::Buffer,
    lbuf: wgpu::Buffer,
    sn: wgpu::Sampler,
    sl: wgpu::Sampler,
    tg: Option<Targets>,
    pub scale: u32,
    pub rw: u32,
    pub rh: u32,
}

struct Targets {
    alb: Tex,
    emi: Tex,
    inf: Tex,
    ui: Tex,
    scene: Tex,
    b1a: Tex,
    b1b: Tex,
    b2a: Tex,
    b2b: Tex,
    bg_light: wgpu::BindGroup,
    bg_bright: wgpu::BindGroup,
    bg_b1h: wgpu::BindGroup,
    bg_b1v: wgpu::BindGroup,
    bg_down: wgpu::BindGroup,
    bg_b2h: wgpu::BindGroup,
    bg_b2v: wgpu::BindGroup,
    bg_final: wgpu::BindGroup,
}

const HDR: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

impl Gfx {
    pub async fn new(window: Arc<Window>) -> Gfx {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());
        let surface = instance.create_surface(window.clone()).expect("surface");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            })
            .await
            .expect("adapter");
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .expect("device");
        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats.iter().copied().find(|f| !f.is_srgb()).unwrap_or(caps.formats[0]);
        let mut config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .expect("config");
        config.format = format;
        config.usage = wgpu::TextureUsages::RENDER_ATTACHMENT;
        config.present_mode = wgpu::PresentMode::AutoVsync;
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let tex_entry = |b: u32| wgpu::BindGroupLayoutEntry {
            binding: b,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
                tex_entry(2),
                tex_entry(3),
                tex_entry(4),
                tex_entry(5),
                wgpu::BindGroupLayoutEntry { binding: 6, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
                wgpu::BindGroupLayoutEntry { binding: 7, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
            ],
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pl"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let mk = |entry: &str, fmt: wgpu::TextureFormat| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(entry),
                layout: Some(&pl),
                vertex: wgpu::VertexState { module: &shader, entry_point: Some("vs"), compilation_options: Default::default(), buffers: &[] },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(entry),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState { format: fmt, blend: None, write_mask: wgpu::ColorWrites::ALL })],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let p_light = mk("fs_light", HDR);
        let p_bright = mk("fs_bright", HDR);
        let p_down = mk("fs_down", HDR);
        let p_blur_h = mk("fs_blur_h", HDR);
        let p_blur_v = mk("fs_blur_v", HDR);
        let p_final = mk("fs_final", format);
        let ubuf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("u"),
            size: std::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let lbuf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("l"),
            size: (std::mem::size_of::<GLight>() * MAX_LIGHTS) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let sn = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let sl = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let mut g = Gfx {
            window,
            surface,
            device,
            queue,
            config,
            layout,
            p_light,
            p_bright,
            p_down,
            p_blur_h,
            p_blur_v,
            p_final,
            ubuf,
            lbuf,
            sn,
            sl,
            tg: None,
            scale: 3,
            rw: 0,
            rh: 0,
        };
        g.resize(size.width, size.height);
        g
    }

    pub fn resize(&mut self, w: u32, h: u32) {
        if w == 0 || h == 0 {
            return;
        }
        let w = w.max(1);
        let h = h.max(1);
        self.config.width = w;
        self.config.height = h;
        self.surface.configure(&self.device, &self.config);
        let scale = ((h as f32 / 300.0).round() as u32).max(2);
        let rw = (w / scale).max(64);
        let rh = (h / scale).max(48);
        if rw != self.rw || rh != self.rh || scale != self.scale || self.tg.is_none() {
            self.scale = scale;
            self.rw = rw;
            self.rh = rh;
            self.tg = Some(self.make_targets());
        }
    }

    fn tex(&self, w: u32, h: u32, fmt: wgpu::TextureFormat, rt: bool) -> Tex {
        let mut usage = wgpu::TextureUsages::TEXTURE_BINDING;
        if rt {
            usage |= wgpu::TextureUsages::RENDER_ATTACHMENT;
        } else {
            usage |= wgpu::TextureUsages::COPY_DST;
        }
        let tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width: w.max(1), height: h.max(1), depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: fmt,
            usage,
            view_formats: &[],
        });
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        Tex { tex, view }
    }

    fn bg(&self, a: &Tex, b: &Tex, c: &Tex, d: &Tex) -> wgpu::BindGroup {
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: self.ubuf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: self.lbuf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&a.view) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&b.view) },
                wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(&c.view) },
                wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::TextureView(&d.view) },
                wgpu::BindGroupEntry { binding: 6, resource: wgpu::BindingResource::Sampler(&self.sn) },
                wgpu::BindGroupEntry { binding: 7, resource: wgpu::BindingResource::Sampler(&self.sl) },
            ],
        })
    }

    fn make_targets(&self) -> Targets {
        let (w, h) = (self.rw, self.rh);
        let u8f = wgpu::TextureFormat::Rgba8Unorm;
        let alb = self.tex(w, h, u8f, false);
        let emi = self.tex(w, h, u8f, false);
        let inf = self.tex(w, h, u8f, false);
        let ui = self.tex(w, h, u8f, false);
        let scene = self.tex(w, h, HDR, true);
        let b1a = self.tex(w / 2, h / 2, HDR, true);
        let b1b = self.tex(w / 2, h / 2, HDR, true);
        let b2a = self.tex(w / 8, h / 8, HDR, true);
        let b2b = self.tex(w / 8, h / 8, HDR, true);
        let bg_light = self.bg(&alb, &emi, &inf, &ui);
        let bg_bright = self.bg(&scene, &emi, &inf, &ui);
        let bg_b1h = self.bg(&b1a, &emi, &inf, &ui);
        let bg_b1v = self.bg(&b1b, &emi, &inf, &ui);
        let bg_down = self.bg(&b1a, &emi, &inf, &ui);
        let bg_b2h = self.bg(&b2a, &emi, &inf, &ui);
        let bg_b2v = self.bg(&b2b, &emi, &inf, &ui);
        let bg_final = self.bg(&scene, &b1a, &b2a, &ui);
        Targets { alb, emi, inf, ui, scene, b1a, b1b, b2a, b2b, bg_light, bg_bright, bg_b1h, bg_b1v, bg_down, bg_b2h, bg_b2v, bg_final }
    }

    fn upload(&self, t: &Tex, data: &[u32]) {
        let (w, h) = (self.rw, self.rh);
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo { texture: &t.tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            bytemuck::cast_slice(data),
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(w * 4), rows_per_image: Some(h) },
            wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        );
    }

    fn pass(enc: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, p: &wgpu::RenderPipeline, bg: &wgpu::BindGroup) {
        let mut rp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        rp.set_pipeline(p);
        rp.set_bind_group(0, bg, &[]);
        rp.draw(0..3, 0..1);
    }

    /// Draw a frame. If `shot` is set, the final image is also saved as a PNG.
    pub fn render(&mut self, f: &Frame, mut gl: Globals, lights: &[GLight], shot: Option<&str>) {
        if f.w as u32 != self.rw || f.h as u32 != self.rh {
            return;
        }
        let tg = self.tg.as_ref().unwrap();
        let sw = self.config.width;
        let sh = self.config.height;
        gl.screen[0] = sw as f32;
        gl.screen[1] = sh as f32;
        gl.screen[2] = self.scale as f32;
        gl.low = [self.rw as f32, self.rh as f32, (sw.saturating_sub(self.rw * self.scale) / 2) as f32, (sh.saturating_sub(self.rh * self.scale) / 2) as f32];
        let n = lights.len().min(MAX_LIGHTS);
        gl.misc[0] = n as f32;
        self.queue.write_buffer(&self.ubuf, 0, bytemuck::bytes_of(&gl));
        if n > 0 {
            self.queue.write_buffer(&self.lbuf, 0, bytemuck::cast_slice(&lights[..n]));
        }
        self.upload(&tg.alb, &f.alb);
        self.upload(&tg.emi, &f.emi);
        self.upload(&tg.inf, &f.inf);
        self.upload(&tg.ui, &f.ui);
        let st = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t) | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            _ => return,
        };
        let view = st.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut enc = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        Self::pass(&mut enc, &tg.scene.view, &self.p_light, &tg.bg_light);
        Self::pass(&mut enc, &tg.b1a.view, &self.p_bright, &tg.bg_bright);
        Self::pass(&mut enc, &tg.b1b.view, &self.p_blur_h, &tg.bg_b1h);
        Self::pass(&mut enc, &tg.b1a.view, &self.p_blur_v, &tg.bg_b1v);
        Self::pass(&mut enc, &tg.b2a.view, &self.p_down, &tg.bg_down);
        Self::pass(&mut enc, &tg.b2b.view, &self.p_blur_h, &tg.bg_b2h);
        Self::pass(&mut enc, &tg.b2a.view, &self.p_blur_v, &tg.bg_b2v);
        Self::pass(&mut enc, &view, &self.p_final, &tg.bg_final);
        let mut readback = None;
        if shot.is_some() {
            let t = self.device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d { width: sw, height: sh, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.config.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            let tv = t.create_view(&wgpu::TextureViewDescriptor::default());
            Self::pass(&mut enc, &tv, &self.p_final, &tg.bg_final);
            let bpr = (sw * 4 + 255) / 256 * 256;
            let buf = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: (bpr * sh) as u64,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
            enc.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo { texture: &t, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                wgpu::TexelCopyBufferInfo { buffer: &buf, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(bpr), rows_per_image: Some(sh) } },
                wgpu::Extent3d { width: sw, height: sh, depth_or_array_layers: 1 },
            );
            readback = Some((buf, bpr));
        }
        self.queue.submit(Some(enc.finish()));
        self.queue.present(st);
        if let (Some((buf, bpr)), Some(path)) = (readback, shot) {
            let slice = buf.slice(..);
            slice.map_async(wgpu::MapMode::Read, |_| {});
            let _ = self.device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None });
            let data = slice.get_mapped_range().expect("map");
            let bgr = matches!(self.config.format, wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb);
            let mut out = Vec::with_capacity((sw * sh * 4) as usize);
            for y in 0..sh {
                let row = &data[(y * bpr) as usize..(y * bpr + sw * 4) as usize];
                for px in row.chunks(4) {
                    if bgr {
                        out.extend_from_slice(&[px[2], px[1], px[0], 255]);
                    } else {
                        out.extend_from_slice(&[px[0], px[1], px[2], 255]);
                    }
                }
            }
            drop(data);
            let file = std::fs::File::create(path).expect("png");
            let mut enc = png::Encoder::new(std::io::BufWriter::new(file), sw, sh);
            enc.set_color(png::ColorType::Rgba);
            enc.set_depth(png::BitDepth::Eight);
            let mut wr = enc.write_header().unwrap();
            wr.write_image_data(&out).unwrap();
        }
    }
}
