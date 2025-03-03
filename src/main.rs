mod camera;
mod color;
mod ray;
mod vec3;

use camera::*;

use miniquad::{
    Bindings, BufferSource, BufferType, BufferUsage, EventHandler, FilterMode, GlContext, KeyCode,
    KeyMods, Pipeline, RenderingBackend,
};

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
    camera: Camera,
}

impl App {
    fn new() -> Self {
        // Image

        let aspect_ratio = 16.0 / 9.0;
        let image_width: u16 = 400;

        // Calculate image height, and ensure it's at least 1.
        let image_height: u16 = u16::max(1, (image_width as f64 / aspect_ratio) as u16);

        // Camera

        let camera = Camera::new(image_width as _, image_height as _);

        // App Setup

        let mut gfx = GlContext::new();

        let pipeline = shader::pipeline(&mut gfx);

        let quad_vbuf_data: [Vertex; 4] = [
            vertex(-1.0, 1.0, 0.0, 1.0),
            vertex(1.0, 1.0, 1.0, 1.0),
            vertex(1.0, -1.0, 1.0, 0.0),
            vertex(-1.0, -1.0, 0.0, 0.0),
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

        let texture = gfx.new_texture_from_rgba8(image_width, image_height, camera.get_pixels());
        gfx.texture_set_mag_filter(texture, FilterMode::Nearest);

        let bindings = Bindings {
            vertex_buffers: vec![quad_vbuf],
            index_buffer: quad_ibuf,
            images: vec![texture],
        };

        Self {
            gfx,
            pipeline,
            bindings,
            camera,
        }
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

    fn update(&mut self) {
        self.camera.render();
        self.gfx
            .texture_update(self.bindings.images[0], self.camera.get_pixels());
    }

    fn key_down_event(&mut self, keycode: KeyCode, _keymods: KeyMods, _repeat: bool) {
        if keycode == KeyCode::Escape {
            miniquad::window::request_quit();
        }
    }
}

fn main() {
    miniquad::start(
        miniquad::conf::Conf {
            window_title: String::from("raytracing"),
            window_width: 1200,
            window_height: 675,
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
