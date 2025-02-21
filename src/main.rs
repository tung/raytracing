use miniquad::{
    Bindings, BufferSource, BufferType, BufferUsage, EventHandler, GlContext, Pipeline,
    RenderingBackend,
};

const WIDTH: usize = 4;
const HEIGHT: usize = 4;

fn rasterize(data: &mut [u8; WIDTH * HEIGHT * 4]) {
    data.copy_from_slice(&[
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00,
        0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0xFF, 0x00, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF,
    ]);
}

#[repr(C)]
struct Vertex {
    pos: [f32; 2],
    uv: [f32; 2],
}

fn vertex(x: f32, y: f32, u: f32, v: f32) -> Vertex {
    Vertex {
        pos: [x, y],
        uv: [u, v],
    }
}

struct App {
    gfx: GlContext,
    pipeline: Pipeline,
    bindings: Bindings,
}

impl App {
    fn new() -> Self {
        let mut gfx = GlContext::new();

        let pipeline = shader::pipeline(&mut gfx);

        let quad_vbuf_data: [Vertex; 4] = [
            vertex(-1.0, 1.0, 0.0, 0.0),
            vertex(1.0, 1.0, 1.0, 0.0),
            vertex(1.0, -1.0, 1.0, 1.0),
            vertex(-1.0, -1.0, 0.0, 1.0),
        ];
        let quad_vbuf = gfx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&quad_vbuf_data),
        );

        let quad_ibuf_data: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let quad_ibuf = gfx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&quad_ibuf_data),
        );

        let mut pixels: [u8; WIDTH * HEIGHT * 4] = [0; WIDTH * HEIGHT * 4];
        rasterize(&mut pixels);
        let texture = gfx.new_texture_from_rgba8(WIDTH as u16, HEIGHT as u16, &pixels);

        let bindings = Bindings {
            vertex_buffers: vec![quad_vbuf],
            index_buffer: quad_ibuf,
            images: vec![texture],
        };

        Self { gfx, pipeline, bindings }
    }
}

impl EventHandler for App {
    fn draw(&mut self) {
        self.gfx.begin_default_pass(Default::default());
        self.gfx.apply_pipeline(&self.pipeline);
        self.gfx.apply_bindings(&self.bindings);
        self.gfx.draw(0, 6, 1);
        self.gfx.end_render_pass();

        self.gfx.commit_frame();
    }

    fn update(&mut self) {}
}

fn main() {
    miniquad::start(
        miniquad::conf::Conf {
            window_title: String::from("raytracing"),
            window_width: 640,
            window_height: 480,
            ..Default::default()
        },
        || Box::new(App::new()),
    );
}

mod shader {
    use miniquad::{
        BufferLayout, GlContext, Pipeline, PipelineParams, RenderingBackend, ShaderMeta,
        ShaderSource, UniformBlockLayout, VertexAttribute, VertexFormat,
    };

    const VERTEX: &str = r#"#version 100
    precision mediump float;

    attribute vec2 in_pos;
    attribute vec2 in_uv;

    varying vec2 tex_coord;

    void main() {
        gl_Position = vec4(in_pos, 0, 1);
        tex_coord = in_uv;
    }
    "#;

    const FRAGMENT: &str = r#"#version 100
    precision mediump float;

    varying vec2 tex_coord;

    uniform sampler2D tex;

    void main() {
        gl_FragColor = texture2D(tex, tex_coord);
    }
    "#;

    pub fn pipeline(gfx: &mut GlContext) -> Pipeline {
        let shader = gfx
            .new_shader(
                ShaderSource::Glsl {
                    vertex: VERTEX,
                    fragment: FRAGMENT,
                },
                ShaderMeta {
                    images: vec![String::from("tex")],
                    uniforms: UniformBlockLayout { uniforms: vec![] },
                },
            )
            .unwrap();

        gfx.new_pipeline(
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("in_pos", VertexFormat::Float2),
                VertexAttribute::new("in_uv", VertexFormat::Float2),
            ],
            shader,
            PipelineParams::default(),
        )
    }
}
