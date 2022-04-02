
const MIN_LIGHT_COUNT: u64 = 16;

struct LightRenderer {
    //lighting_bind_group_layout: BindGroupLayout,
    //lighting_pipeline: RenderPipeline,
    //lighting_locals: Buffer,
    //unlit_view: TextureView,
    //lighting_bind_group: BindGroup,
    //light_count: u32,
}

impl LightRenderer {
    pub fn new(rc: &RenderingContext, globals: &Buffer, resolution: Resolution) -> Self {
        let shader = rc.device.create_shader_module(&include_wgsl!("shaders/light.wgsl"));

        //let lighting_bind_group_layout = rc.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        //    label: Some("lighting_bind_group_layout"),
        //    entries: &[
        //        globals_bind_group_layout_entry,
        //        BindGroupLayoutEntry {
        //            binding: 1,
        //            visibility: ShaderStages::VERTEX_FRAGMENT,
        //            ty: BindingType::Buffer {
        //                ty: BufferBindingType::Uniform,
        //                has_dynamic_offset: true,
        //                min_binding_size: BufferSize::new(0),
        //            },
        //            count: None,
        //        },
        //        BindGroupLayoutEntry {
        //            binding: 2,
        //            visibility: ShaderStages::FRAGMENT,
        //            ty: BindingType::Texture {
        //                multisampled: false,
        //                view_dimension: TextureViewDimension::D2,
        //                sample_type: TextureSampleType::Float { filterable: false },
        //            },
        //            count: None,
        //        },
        //    ],
        //});

        //let lighting_pipeline_layout = rc.device.create_pipeline_layout(&PipelineLayoutDescriptor {
        //    label: Some("lighting_pipeline_layout"),
        //    bind_group_layouts: &[&lighting_bind_group_layout],
        //    push_constant_ranges: &[],
        //});

        //let lighting_pipeline = rc.device.create_render_pipeline(&RenderPipelineDescriptor {
        //    label: Some("lighting_pipeline"),
        //    layout: Some(&lighting_pipeline_layout),
        //    vertex: VertexState {
        //        module: &light,
        //        entry_point: "vs_main",
        //        buffers: &[],
        //    },
        //    primitive: PrimitiveState {
        //        topology: PrimitiveTopology::TriangleStrip,
        //        strip_index_format: None,
        //        front_face: FrontFace::Ccw,
        //        cull_mode: None,
        //        unclipped_depth: false,
        //        polygon_mode: PolygonMode::Fill,
        //        conservative: false,
        //    },
        //    depth_stencil: None,
        //    multisample: MultisampleState {
        //        count: 1,
        //        mask: !0,
        //        alpha_to_coverage_enabled: false,
        //    },
        //    fragment: Some(FragmentState {
        //        module: &light,
        //        entry_point: "fs_main",
        //        targets: &[ColorTargetState {
        //            format: TEXTURE_FORMAT,
        //            blend: Some(BlendState {
        //                color: BlendComponent {
        //                    src_factor: BlendFactor::One,
        //                    dst_factor: BlendFactor::One,
        //                    operation: BlendOperation::Add,
        //                },
        //                alpha: BlendComponent::REPLACE,
        //            }),
        //            write_mask: ColorWrites::ALL,
        //        }],
        //    }),
        //    multiview: None,
        //});

        //let lighting_locals = rc.device.create_buffer(&BufferDescriptor {
        //    label: Some("lighting_locals"),
        //    size: std::mem::size_of::<Light>() as u64 * MIN_LIGHT_COUNT,
        //    usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        //    mapped_at_creation: false,
        //});
    //
        //let unlit_view = unlit_view(rc, resolution);
    //
        //let lighting_bind_group = lighting_bind_group(
        //    &rc,
        //    &lighting_bind_group_layout,
        //    &lighting_locals,
        //    &unlit_view,
        //);

        Self {
            lighting_bind_group_layout,
            lighting_pipeline,
            lighting_locals,
            unlit_view,
            lighting_bind_group,
            light_count: 0,
        }
    }

    pub fn write_lights(&mut self, rc: &RenderingContext, lights: &[Light]) {
        //self.light_count = lights.len() as _;
//
        //rc.queue.write_buffer(
        //    &self.lighting_locals,
        //    0,
        //    unsafe {
        //        std::slice::from_raw_parts(
        //            lights.as_ptr() as *const u8,
        //            lights.len() * std::mem::size_of::<Light>(),
        //        )
        //    },
        //)
    }

    pub fn resize(&mut self, rc: &RenderingContext, resolution: [u32; 2]) {
        //self.unlit_view = unlit_view(rc, resolution);
        //self.lighting_bind_group = lighting_bind_group(
        //    &rc,
        //    &self.lighting_bind_group_layout,
        //    &self.lighting_locals,
        //    &self.unlit_view,
        //);
    }

    pub fn render(
        &self,
        rc: &RenderingContext,
        surface_view: &TextureView,
    ) {
        // Build our command encoder
        let mut command_encoder = rc.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("light_renderer::command_encoder"),
        });

        // Render lights!
        {
            let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("light_renderer::render_pass"),
                color_attachments: &[RenderPassColorAttachment {
                    view: surface_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.lighting_pipeline);
            for i in 0..self.light_count {
                render_pass.set_bind_group(0, &self.lighting_bind_group, &[i * std::mem::size_of::<Light>() as u32]);
                render_pass.draw(0..4, 0..1);
            }
        }

        // Submit our work
        rc.queue.submit([command_encoder.finish()]);
    }
}

fn bind_group(
    rc: &RenderingContext,
    layout: &BindGroupLayout,
    globals: &Buffer,
    locals: &Buffer,
    unlit_view: &TextureView,
) -> BindGroup {
    rc.device.create_bind_group(&BindGroupDescriptor {
        label: Some("light_renderer::bind_group"),
        layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: globals.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: locals,
                    offset: 0,
                    size: BufferSize::new(std::mem::size_of::<Light>() as _),
                }),
            },
            BindGroupEntry {
                binding: 2,
                resource: BindingResource::TextureView(unlit_view),
            },
        ],
    })
}
