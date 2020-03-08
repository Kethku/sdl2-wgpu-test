use std::{mem, ops::Range, rc::Rc};
use zerocopy::{AsBytes, FromBytes};

use sdl2::{
    self,
    event::{
        Event,
        WindowEvent
    }
};

use wgpu::*;
use wgpu::vertex_attr_array;

// use wgpu_glyph::{Section, GlyphBrushBuilder, Scale};

#[repr(C)]
#[derive(Clone, Copy, AsBytes, FromBytes)]
struct Uniforms {
    font_size: [f32; 2],
    window_size: [f32; 2]
}

#[repr(C)]
#[derive(Clone, Copy, AsBytes, FromBytes)]
struct Vertex {
    vertex_position: [f32; 2]
}

impl Vertex {
    pub fn new(x: f32, y: f32) -> Vertex {
        Vertex {
            vertex_position: [x, y]
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, AsBytes, FromBytes)]
struct Instance {
    position: [f32; 2],
    dimensions: [f32; 2],
    color: [f32; 3]
}

impl Instance {
    pub fn new(x: f32, y: f32, width: f32, height: f32, r: f32, g: f32, b: f32) -> Instance {
        Instance {
            position: [x, y],
            dimensions: [width, height],
            color: [r, g, b]
        }
    }
}

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
        },
        BackendBit::PRIMARY).unwrap();

    let (device, mut queue) = adapter.request_device(&DeviceDescriptor {
        extensions: Extensions {
            anisotropic_filtering: false
        },
        limits: Limits::default()
    });

    // Static Data
    let vertex_buffer_data = [
        Vertex::new(0.0, 0.0), // Top Left
        Vertex::new(1.0, 0.0), // Top Right
        Vertex::new(1.0, 1.0), // Bottom Right
        Vertex::new(0.0, 1.0) // Bottom Left
    ];
    let vertex_buf = device.create_buffer_with_data(vertex_buffer_data.as_bytes(), BufferUsage::VERTEX);

    let index_buffer_data: &[u16] = &[0, 1, 2, 0, 2, 3];
    let index_buf = device.create_buffer_with_data(index_buffer_data.as_bytes(), BufferUsage::INDEX);

    // Dynamic Per Instance Data
    /*_________
     *_1_3_444_
     *_123__5__
     *_1_3_666_
     *_________
     */
    let instance_buffer_data = [
        Instance::new(1.0, 1.0, 1.0, 3.0, 1.0, 0.0, 0.0),
        Instance::new(2.0, 2.0, 1.0, 1.0, 1.0, 1.0, 0.0),
        Instance::new(3.0, 1.0, 1.0, 3.0, 0.0, 1.0, 0.0),
        Instance::new(5.0, 1.0, 3.0, 1.0, 0.0, 1.0, 1.1),
        Instance::new(6.0, 2.0, 1.0, 1.0, 0.0, 0.0, 1.0),
        Instance::new(5.0, 3.0, 3.0, 1.0, 1.0, 0.0, 1.0)
    ];
    let instance_buf = device.create_buffer_with_data(instance_buffer_data.as_bytes(), BufferUsage::VERTEX);

    // Dynamic Per Frame Data
    let uniforms = Uniforms {
        font_size: [20.0, 40.0],
        window_size: [640.0, 480.0]
    };
    let uniform_size = mem::size_of::<Uniforms>() as BufferAddress;
    let uniform_buf = device.create_buffer_with_data(
        uniforms.as_bytes(),
        BufferUsage::UNIFORM
    );

    let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        bindings: &[
            BindGroupLayoutBinding {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: BindingType::UniformBuffer { dynamic: false }
            }
        ]
    });
    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        bind_group_layouts: &[&bind_group_layout],
    });
    let bind_group = device.create_bind_group(&BindGroupDescriptor {
        layout: &bind_group_layout,
        bindings: &[
            Binding {
                binding: 0,
                resource: BindingResource::Buffer {
                    buffer: &uniform_buf,
                    range: 0 .. uniform_size
                }
            }
        ]
    });

    let vs = include_bytes!("shader.vert.spv");
    let vs_module =
        device.create_shader_module(&read_spirv(std::io::Cursor::new(&vs[..])).unwrap());

    let fs = include_bytes!("shader.frag.spv");
    let fs_module =
        device.create_shader_module(&read_spirv(std::io::Cursor::new(&fs[..])).unwrap());

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
        vertex_buffers: &[
            VertexBufferDescriptor {
                stride: mem::size_of::<Vertex>() as BufferAddress,
                step_mode: InputStepMode::Vertex,
                attributes: &vertex_attr_array![
                    0 => Float2 // Vertex Position
                ]
            },
            VertexBufferDescriptor {
                stride: mem::size_of::<Instance>() as BufferAddress,
                step_mode: InputStepMode::Instance,
                attributes: &vertex_attr_array![
                    1 => Float2, // Top Left Position
                    2 => Float2, // Dimensions
                    3 => Float3  // Color
                ]
            },
        ],
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    });

    let mut swap_chain = create_swap_chain(640, 480, &surface, &device);

    // let font: &[u8] = include_bytes!("DejaVuSansMono.ttf");
    // let mut glyph_brush = GlyphBrushBuilder::using_font_bytes(font).unwrap().build(&device, TextureFormat::Bgra8UnormSrgb);
    // let section = Section { 
    //     text: "Hello wgpu text rendering",
    //     screen_position: (100.0, 50.0),
    //     scale: Scale::uniform(32.0),
    //     ..Section::default()
    // };

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
            .get_next_texture().unwrap();

        let mut encoder =
            device.create_command_encoder(&CommandEncoderDescriptor { todo: 0 });
        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: LoadOp::Clear,
                    store_op: StoreOp::Store,
                    clear_color: Color {
                        r: 0.39,
                        g: 0.58,
                        b: 0.93,
                        a: 1.0
                    },
                }],
                depth_stencil_attachment: None,
            });
            pass.set_pipeline(&render_pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.set_index_buffer(&index_buf, 0);
            pass.set_vertex_buffers(0, &[(&vertex_buf, 0)]);



            pass.set_vertex_buffers(1, &[(&instance_buf, 0)]);
            pass.draw_indexed(0 .. 6, 0, 0 .. 6);
        }

        let (width, height) = window.drawable_size();

        // glyph_brush.queue(section);
        // glyph_brush.draw_queued(
        //     &device, &mut encoder, 
        //     &frame.view, width, 
        //     height);

        queue.submit(&[encoder.finish()]);
    }
}
