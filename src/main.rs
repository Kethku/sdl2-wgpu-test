use sdl2::{
    self,
    event::{
        Event,
        WindowEvent
    }
};

use wgpu::*;

use wgpu_glyph::{Section, GlyphBrushBuilder, Scale};

fn create_swap_chain(width: u32, height: u32, surface: & Surface, device: &Device) -> SwapChain {
    let sc_desc = SwapChainDescriptor {
        usage: TextureUsage::OUTPUT_ATTACHMENT,
        format: TextureFormat::Bgra8UnormSrgb,
        width: width as u32,
        height: height as u32,
        present_mode: PresentMode::Vsync,
    };

    device.create_swap_chain(&surface, &sc_desc)
}

fn main() {
    let context = sdl2::init().unwrap();
    let video_subsystem = context.video().unwrap();

    let window = video_subsystem.window("SDL2 WebGL Test", 640, 480)
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let surface = Surface::create(&window);

    let adapter = Adapter::request(
        &RequestAdapterOptions {
            power_preference: PowerPreference::Default,
            backends: BackendBit::PRIMARY
        }).unwrap();

    let (device, mut queue) = adapter.request_device(&DeviceDescriptor {
        extensions: Extensions {
            anisotropic_filtering: false
        },
        limits: Limits::default()
    });

    let vs = include_bytes!("shader.vert.spv");
    let vs_module =
        device.create_shader_module(&read_spirv(std::io::Cursor::new(&vs[..])).unwrap());

    let fs = include_bytes!("shader.frag.spv");
    let fs_module =
        device.create_shader_module(&read_spirv(std::io::Cursor::new(&fs[..])).unwrap());

    let bind_group_layout =
        device.create_bind_group_layout(&BindGroupLayoutDescriptor { bindings: &[] });
    let bind_group = device.create_bind_group(&BindGroupDescriptor {
        layout: &bind_group_layout,
        bindings: &[],
    });
    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        bind_group_layouts: &[&bind_group_layout],
    });

    let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        layout: &pipeline_layout,
        vertex_stage: ProgrammableStageDescriptor {
            module: &vs_module,
            entry_point: "main",
        },
        fragment_stage: Some(ProgrammableStageDescriptor {
            module: &fs_module,
            entry_point: "main",
        }),
        rasterization_state: Some(RasterizationStateDescriptor {
            front_face: FrontFace::Ccw,
            cull_mode: CullMode::None,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
        }),
        primitive_topology: PrimitiveTopology::TriangleList,
        color_states: &[ColorStateDescriptor {
            format: TextureFormat::Bgra8UnormSrgb,
            color_blend: BlendDescriptor::REPLACE,
            alpha_blend: BlendDescriptor::REPLACE,
            write_mask: ColorWrite::ALL,
        }],
        depth_stencil_state: None,
        index_format: IndexFormat::Uint16,
        vertex_buffers: &[],
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    });

    let mut swap_chain = create_swap_chain(640, 480, &surface, &device);

    let font: &[u8] = include_bytes!("DejaVuSansMono.ttf");
    let mut glyph_brush = GlyphBrushBuilder::using_font_bytes(font).unwrap().build(&device, TextureFormat::Bgra8UnormSrgb);
    let section = Section { 
        text: "Hello wgpu text rendering",
        screen_position: (100.0, 50.0),
        scale: Scale::uniform(32.0),
        ..Section::default()
    };

    let mut event_pump = context.event_pump().expect("Could not create sdl event pump");
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => break 'running,
                Event::Window { win_event: WindowEvent::SizeChanged(width, height), .. } => {
                    swap_chain = create_swap_chain(width as u32, height as u32, &surface, &device);
                },
                _ => {}
            }
        }

        let frame = swap_chain
            .get_next_texture();

        let mut encoder =
            device.create_command_encoder(&CommandEncoderDescriptor { todo: 0 });
        {
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: LoadOp::Clear,
                    store_op: StoreOp::Store,
                    clear_color: Color::GREEN,
                }],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&render_pipeline);
            rpass.set_bind_group(0, &bind_group, &[]);
            rpass.draw(0 .. 3, 0 .. 1);
        }

        let (width, height) = window.drawable_size();

        glyph_brush.queue(section);
        glyph_brush.draw_queued(
            &device, &mut encoder, 
            &frame.view, width, 
            height);

        queue.submit(&[encoder.finish()]);
    }
}
