//! GPU editor preview: draws the negative through the print curves and the
//! look in a wgpu fragment shader (`preview.wgsl`), so the preview stays at
//! full editor resolution while sliders move. The CPU renderer remains the
//! reference (exports, thumbnails, the histogram and readout) and the
//! fallback when wgpu is not available.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use eframe::egui;
use eframe::egui_wgpu::{self, CallbackResources, CallbackTrait, ScreenDescriptor, wgpu};

use crate::engine::curve::{LUT_DMAX, LUT_DMIN, LUT_SIZE};
use crate::engine::dodge;
use crate::engine::look::LOOK_GRID;
use crate::engine::pipeline::{Negative, Renderer};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

/// Grey-curve entries after the look map in the tables buffer.
const GREY_KNOTS: usize = 11;
const TABLE_LEN: usize = LUT_SIZE + LOOK_GRID[0] * LOOK_GRID[1] * LOOK_GRID[2] + GREY_KNOTS;

/// A negative uploaded as an RGBA32F density texture: the channels'
/// densities and, fourth, the low-pass density the dodge works from.
pub struct NegData {
    id: u64,
    width: u32,
    height: u32,
    texels: Vec<[f32; 4]>,
}

impl NegData {
    pub fn new(neg: &Negative) -> Arc<Self> {
        Arc::new(Self {
            id: next_id(),
            width: neg.width as u32,
            height: neg.height as u32,
            texels: neg
                .data
                .iter()
                .zip(dodge::low_pass(neg))
                .map(|(d, low)| [d[0], d[1], d[2], low])
                .collect(),
        })
    }
}

/// Everything a renderer contributes: curve tables, look and output matrix.
pub struct ViewData {
    id: u64,
    tables: Vec<[f32; 4]>,
    /// Look strength, saturation, has tables, identity.
    look: [f32; 4],
    /// Output matrix rows, luminance weights in w.
    out: [[f32; 4]; 3],
    /// Dodge key, light and dark compression, knee²; channel gains.
    dodge: [f32; 4],
    gain: [f32; 4],
}

impl ViewData {
    pub fn new(r: &Renderer) -> Arc<Self> {
        let (lut, look, out) = r.gpu_parts();
        let mut tables: Vec<[f32; 4]> = (0..LUT_SIZE)
            .map(|i| [lut.lut[0][i], lut.lut[1][i], lut.lut[2][i], 0.0])
            .collect();
        let (look_params, look_tables) = match look {
            Some(l) => {
                let (k, sat, identity) = l.params();
                let t = l.tables();
                let flag = |b: bool| if b { 1.0 } else { 0.0 };
                ([k, sat, flag(t.is_some()), flag(identity)], t)
            }
            None => ([0.0, 1.0, 0.0, 1.0], None),
        };
        match look_tables {
            Some(t) => {
                tables.extend(t.grid);
                tables.extend(
                    (0..GREY_KNOTS).map(|i| [t.grey[0][i], t.grey[1][i], t.grey[2][i], 0.0]),
                );
            }
            None => tables.resize(TABLE_LEN, [0.0; 4]),
        }
        debug_assert_eq!(tables.len(), TABLE_LEN);
        let (dodge, gain) = match r.dodge() {
            Some(d) => (
                [d.key, d.light, d.dark, dodge::KNEE * dodge::KNEE],
                [d.gain[0], d.gain[1], d.gain[2], 0.0],
            ),
            None => ([0.0; 4], [0.0; 4]),
        };
        Arc::new(Self {
            id: next_id(),
            tables,
            look: look_params,
            out,
            dodge,
            gain,
        })
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Params {
    uv: [f32; 4],
    out0: [f32; 4],
    out1: [f32; 4],
    out2: [f32; 4],
    look: [f32; 4],
    lut: [f32; 4],
    size: [f32; 4],
    dodge: [f32; 4],
    gain: [f32; 4],
}

/// One drawable image (main view, or the "before" side of the split).
struct Slot {
    view: u64,
    neg: u64,
    tables: wgpu::Buffer,
    params: wgpu::Buffer,
    bind_group: Option<wgpu::BindGroup>,
}

/// GPU state kept in egui's callback resources.
pub struct GpuPreview {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    linear_target: bool,
    neg: Option<(u64, wgpu::TextureView)>,
    slots: HashMap<u8, Slot>,
}

impl GpuPreview {
    /// `linear_output`: the target stores linear values (sRGB or float
    /// formats); otherwise the shader writes display-encoded sRGB.
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat, linear_output: bool) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("preview"),
            source: wgpu::ShaderSource::Wgsl(include_str!("preview.wgsl").into()),
        });
        let entry = |binding, ty| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty,
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("preview"),
            entries: &[
                entry(
                    0,
                    wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                ),
                entry(
                    1,
                    wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                ),
                entry(
                    2,
                    wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                ),
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("preview"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("preview"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        Self {
            pipeline,
            layout,
            linear_target: linear_output,
            neg: None,
            slots: HashMap::new(),
        }
    }

    /// Upload changed data and keep the slot's bind group current.
    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        slot: u8,
        neg: &NegData,
        view: &ViewData,
        uv: [f32; 4],
    ) {
        if self.neg.as_ref().map(|n| n.0) != Some(neg.id) {
            let size = wgpu::Extent3d {
                width: neg.width,
                height: neg.height,
                depth_or_array_layers: 1,
            };
            let tex = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("preview negative"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &tex,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                bytemuck::cast_slice(&neg.texels),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(neg.width * 16),
                    rows_per_image: Some(neg.height),
                },
                size,
            );
            self.neg = Some((neg.id, tex.create_view(&Default::default())));
        }
        let s = self.slots.entry(slot).or_insert_with(|| Slot {
            view: 0,
            neg: 0,
            tables: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("preview tables"),
                size: (TABLE_LEN * 16) as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            params: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("preview params"),
                size: std::mem::size_of::<Params>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            bind_group: None,
        });
        if s.view != view.id {
            queue.write_buffer(&s.tables, 0, bytemuck::cast_slice(&view.tables));
            s.view = view.id;
        }
        let params = Params {
            uv,
            out0: view.out[0],
            out1: view.out[1],
            out2: view.out[2],
            look: view.look,
            lut: [
                LUT_DMIN as f32,
                LUT_DMAX as f32,
                LUT_SIZE as f32,
                if self.linear_target { 1.0 } else { 0.0 },
            ],
            size: [neg.width as f32, neg.height as f32, 0.0, 0.0],
            dodge: view.dodge,
            gain: view.gain,
        };
        queue.write_buffer(&s.params, 0, bytemuck::bytes_of(&params));
        if s.neg != neg.id || s.bind_group.is_none() {
            let tex = &self.neg.as_ref().unwrap().1;
            s.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("preview"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: s.params.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(tex),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: s.tables.as_entire_binding(),
                    },
                ],
            }));
            s.neg = neg.id;
        }
    }

    fn draw(&self, pass: &mut wgpu::RenderPass<'_>, slot: u8) {
        let Some(bg) = self.slots.get(&slot).and_then(|s| s.bind_group.as_ref()) else {
            return;
        };
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, bg, &[]);
        pass.draw(0..3, 0..1);
    }
}

/// Register the GPU preview with eframe's wgpu renderer; `false` when the
/// app runs without wgpu.
pub fn install(cc: &eframe::CreationContext<'_>) -> bool {
    let Some(rs) = cc.wgpu_render_state.as_ref() else {
        return false;
    };
    let preview = GpuPreview::new(&rs.device, rs.target_format, rs.target_format.is_srgb());
    rs.renderer.write().callback_resources.insert(preview);
    true
}

/// Paint `part` (u0, v0, u1, v1) of the negative into `rect`.
pub fn paint(
    painter: &egui::Painter,
    rect: egui::Rect,
    slot: u8,
    neg: &Arc<NegData>,
    view: &Arc<ViewData>,
    part: [f32; 4],
) {
    painter.add(egui_wgpu::Callback::new_paint_callback(
        rect,
        PreviewCallback {
            slot,
            neg: neg.clone(),
            view: view.clone(),
            uv: part,
        },
    ));
}

struct PreviewCallback {
    slot: u8,
    neg: Arc<NegData>,
    view: Arc<ViewData>,
    uv: [f32; 4],
}

impl CallbackTrait for PreviewCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen: &ScreenDescriptor,
        _encoder: &mut wgpu::CommandEncoder,
        resources: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        if let Some(p) = resources.get_mut::<GpuPreview>() {
            p.prepare(device, queue, self.slot, &self.neg, &self.view, self.uv);
        }
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        pass: &mut wgpu::RenderPass<'static>,
        resources: &CallbackResources,
    ) {
        if let Some(p) = resources.get::<GpuPreview>() {
            p.draw(pass, self.slot);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::settings::{Lut, OutputSpace, Settings};

    /// Draw `neg` with `r` offscreen at its own size; RGBA16F target, so the
    /// shader returns linear values, encoded here like the CPU does.
    fn render_gpu(neg: &Negative, r: &Renderer) -> Option<Vec<[f32; 3]>> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
                .ok()?;
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).ok()?;
        let format = wgpu::TextureFormat::Rgba32Float;
        let mut gp = GpuPreview::new(&device, format, true);
        let (neg_d, view) = (NegData::new(neg), ViewData::new(r));
        gp.prepare(&device, &queue, 0, &neg_d, &view, [0.0, 0.0, 1.0, 1.0]);
        let (w, h) = (neg.width as u32, neg.height as u32);
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let tv = target.create_view(&Default::default());
        let row = (w * 16).div_ceil(256) * 256;
        let out = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (row * h) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut enc = device.create_command_encoder(&Default::default());
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &tv,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            gp.draw(&mut pass, 0);
        }
        enc.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &target,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &out,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(row),
                    rows_per_image: Some(h),
                },
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
        queue.submit([enc.finish()]);
        out.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: None,
            })
            .ok()?;
        let data = out.slice(..).get_mapped_range();
        let mut px = Vec::with_capacity((w * h) as usize);
        for y in 0..h as usize {
            let line: &[[f32; 4]] =
                bytemuck::cast_slice(&data[y * row as usize..][..w as usize * 16]);
            px.extend(
                line.iter()
                    .map(|p| [p[0], p[1], p[2]].map(crate::engine::color::srgb_encode)),
            );
        }
        Some(px)
    }

    /// The shader draws what the CPU renderer draws.
    #[test]
    fn gpu_preview_matches_the_cpu_renderer() {
        let chart = crate::engine::testchart::negative(0.0);
        for (lut, dodge) in [
            (Lut::None, 0.0),
            (Lut::Frontier, 0.0),
            (Lut::Pakon, 0.0),
            (Lut::Frontier, 80.0),
        ] {
            let s = Settings {
                lut,
                post_saturation: 7,
                hyper_tone_highlights: dodge,
                hyper_tone_shadows: dodge,
                ..Settings::default()
            };
            let base = crate::engine::pipeline::resolve_base(&chart.src, &s, None);
            let neg = crate::engine::pipeline::prepare_negative(&chart.src, &s, base);
            let a = crate::engine::pipeline::analyze(&neg, &s, base, None);
            for levels in [Some(&a.levels), None] {
                let r = Renderer::new(&s, levels, OutputSpace::Srgb);
                let Some(gpu) = render_gpu(&neg, &r) else {
                    eprintln!("no GPU adapter; skipping");
                    return;
                };
                let cpu = r.render(&neg, 0.0);
                let worst = cpu
                    .iter()
                    .zip(&gpu)
                    .map(|(a, b)| (0..3).map(|c| (a[c] - b[c]).abs()).fold(0.0f32, f32::max))
                    .fold(0.0f32, f32::max);
                eprintln!(
                    "{lut:?} converted={}: GPU vs CPU worst {:.2}/255",
                    levels.is_some(),
                    worst * 255.0
                );
                assert!(
                    worst < 0.5 / 255.0,
                    "{lut:?} converted={}: differs by {:.2}/255",
                    levels.is_some(),
                    worst * 255.0
                );
            }
        }
    }
}
