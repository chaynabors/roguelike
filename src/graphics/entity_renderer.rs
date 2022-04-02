
const MIN_ENTITY_COUNT: u64 = 16;

struct EntityRenderer {
    pipeline: RenderPipeline,
    locals: Buffer,
    bind_group: BindGroup,
    count: u32,
}

impl EntityRenderer {
    pub fn new(rc: &RenderingContext, resolution: Resolution) -> Self {
        let shader = rc.device.create_shader_module(&include_wgsl!("shaders/fs_entity.wgsl"));

        let bind_group_layout = rc.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("entity_bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: BufferSize::new(0),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: false },
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = rc.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("entity_pipeline_layout"),
            bind_group_layouts: &[&entity_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = rc.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("entity_pipeline"),
            layout: Some(&entity_pipeline_layout),
            vertex: VertexState {
                module: &vs_quad,
                entry_point: "vs_main",
                buffers: &[],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                module: &fs_entity,
                entry_point: "fs_main",
                targets: &[ColorTargetState {
                    format: TEXTURE_FORMAT,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                }],
            }),
            multiview: None,
        });
    
        let entity_locals = rc.device.create_buffer(&BufferDescriptor {
            label: Some("entity_locals"),
            size: std::mem::size_of::<Entity>() as u64 * MIN_ENTITY_COUNT,
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let (entity_atlas_data, entity_atlas_size) = entities();
        let entity_atlas = rc.device.create_texture(&TextureDescriptor {
            label: Some("entity_atlas"),
            size: entity_atlas_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TEXTURE_FORMAT,
            usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
        });

        rc.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &entity_atlas,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &entity_atlas_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(entity_atlas_size.width * TEXTURE_FORMAT.describe().block_size as u32),
                rows_per_image: None,
            },
            entity_atlas_size,
        );

        let entity_atlas_view = entity_atlas.create_view(&TextureViewDescriptor {
            label: Some("entity_atlas_view"),
            format: Some(TEXTURE_FORMAT),
            dimension: Some(TextureViewDimension::D2),
            aspect: TextureAspect::All,
            ..Default::default()
        });

        let entity_bind_group = rc.device.create_bind_group(&BindGroupDescriptor {
            label: Some("entity_bind_group"),
            layout: &entity_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &entity_locals,
                        offset: 0,
                        size: BufferSize::new(std::mem::size_of::<Entity>() as _),
                    }),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&entity_atlas_view),
                },
            ],
        });
    }

    pub fn write_entities(&mut self, rc: &RenderingContext, entities: &[Entity]) {
        self.entity_count = entities.len() as _;

        rc.queue.write_buffer(
            &self.entity_locals,
            0,
            unsafe {
                std::slice::from_raw_parts(
                    entities.as_ptr() as *const u8,
                    entities.len() * std::mem::size_of::<Entity>(),
                )
            },
        )
    }

    pub fn render(
        &self,
        rc: &RenderingContext,
        surface_view: &TextureView,
    ) {
        // Build our command encoder
        let mut command_encoder = rc.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("command_encoder"),
        });

        // Render chunks!
        {
            let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("entity_renderer::render_pass"),
                color_attachments: &[RenderPassColorAttachment {
                    view: surface_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(CHUNK_CLEAR_COLOR),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.entity_pipeline);
            for i in 0..self.entity_count {
                render_pass.set_scissor_rect(0, 0, TILE_SIZE, TILE_SIZE);
                render_pass.set_bind_group(0, &self.entity_bind_group, &[i * std::mem::size_of::<Entity>() as u32]);
                render_pass.draw(0..4, 0..1);
            }
        }

        // Submit our work
        rc.queue.submit([command_encoder.finish()]);
    }
}

fn entities() -> (Vec<u8>, Extent3d) {
    let bytes = include_bytes!("textures/entities.gif");
    let decoder = GifDecoder::new(&bytes[..]).unwrap();
    let dimensions = decoder.dimensions();

    let mut buf = vec![0; decoder.total_bytes() as usize];
    decoder.read_image(&mut buf).unwrap();

    (buf, Extent3d { width: dimensions.0, height: dimensions.1, depth_or_array_layers: 1 })
}
