mod camera;
mod color;
mod hit_record;
mod material;
mod random;
mod ray;
mod scene;
mod sphere;
mod vec3;

use camera::*;
use color::*;
use material::*;
use random::*;
use scene::*;
use sphere::*;
use vec3::*;

use miniquad::{
    Bindings, BufferSource, BufferType, BufferUsage, EventHandler, FilterMode, GlContext, KeyCode,
    KeyMods, Pipeline, RenderingBackend, UniformsSource,
};
use std::rc::Rc;

const LAUNCH_WIDTH: i32 = 1200;
const LAUNCH_HEIGHT: i32 = 675;

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

fn calc_zoom(
    image_width: f32,
    image_height: f32,
    window_width: f32,
    window_height: f32,
) -> [f32; 2] {
    let zoom_x = window_width / image_width;
    let zoom_y = window_height / image_height;
    if zoom_x <= zoom_y {
        [1.0, zoom_x / zoom_y]
    } else {
        [zoom_y / zoom_x, 1.0]
    }
}

struct App {
    gfx: GlContext,
    pipeline: Pipeline,
    bindings: Bindings,
    zoom: [f32; 2],
    camera: Camera,
    scene: Scene,
}

impl App {
    fn new() -> Self {
        // World

        let material_ground = Rc::new(Material::lambertian(Color::new(0.8, 0.8, 0.0)));
        let material_center = Rc::new(Material::lambertian(Color::new(0.1, 0.2, 0.5)));
        let material_left = Rc::new(Material::dielectric(1.5));
        let material_bubble = Rc::new(Material::dielectric(1.0 / 1.5));
        let material_right = Rc::new(Material::metal(Color::new(0.8, 0.6, 0.2), 1.0));

        let mut scene = Scene::new();

        scene.add(Sphere::new(
            Vec3::new(0.0, -100.5, -1.0),
            100.0,
            Rc::clone(&material_ground),
        ));
        scene.add(Sphere::new(
            Vec3::new(0.0, 0.0, -1.2),
            0.5,
            Rc::clone(&material_center),
        ));
        scene.add(Sphere::new(
            Vec3::new(-1.0, 0.0, -1.0),
            0.5,
            Rc::clone(&material_left),
        ));
        scene.add(Sphere::new(
            Vec3::new(-1.0, 0.0, -1.0),
            0.4,
            Rc::clone(&material_bubble),
        ));
        scene.add(Sphere::new(
            Vec3::new(1.0, 0.0, -1.0),
            0.5,
            Rc::clone(&material_right),
        ));

        // Camera

        let image_width: u16 = 400;

        let camera = Camera::new(
            Rng::new(miniquad::date::now() as _),
            CameraOptions {
                aspect_ratio: 16.0 / 9.0,
                image_width,
                max_depth: 50,
                vfov: 20.0,
                lookfrom: Vec3::new(-2.0, 2.0, 1.0),
                lookat: Vec3::new(0.0, 0.0, -1.0),
                vup: Vec3::new(0.0, 1.0, 0.0),
            },
        );

        let image_height = camera.get_image_height() as u16;

        // App Setup

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
            zoom: calc_zoom(
                image_width as f32,
                image_height as f32,
                LAUNCH_WIDTH as f32,
                LAUNCH_HEIGHT as f32,
            ),
            camera,
            scene,
        }
    }
}

impl EventHandler for App {
    fn draw(&mut self) {
        self.gfx.begin_default_pass(Default::default());
        self.gfx.apply_pipeline(&self.pipeline);
        self.gfx.apply_bindings(&self.bindings);
        self.gfx
            .apply_uniforms(UniformsSource::table(&shader::Uniforms { zoom: self.zoom }));
        self.gfx.draw(0, 6, 1);
        self.gfx.end_render_pass();

        self.gfx.commit_frame();
    }

    fn update(&mut self) {
        self.camera.render(&self.scene);
        self.gfx
            .texture_update(self.bindings.images[0], self.camera.get_pixels());
    }

    fn key_down_event(&mut self, keycode: KeyCode, _keymods: KeyMods, _repeat: bool) {
        if keycode == KeyCode::Escape {
            miniquad::window::request_quit();
        }
    }

    fn resize_event(&mut self, width: f32, height: f32) {
        let (image_width, image_height) = self.gfx.texture_size(self.bindings.images[0]);
        self.zoom = calc_zoom(image_width as f32, image_height as f32, width, height);
    }
}

fn main() {
    miniquad::start(
        miniquad::conf::Conf {
            window_title: String::from("raytracing"),
            window_width: LAUNCH_WIDTH,
            window_height: LAUNCH_HEIGHT,
            ..Default::default()
        },
        || Box::new(App::new()),
    );
}

mod shader {
    use miniquad::{
        BufferLayout, GlContext, Pipeline, PipelineParams, RenderingBackend, ShaderMeta,
        ShaderSource, UniformBlockLayout, UniformDesc, UniformType, VertexAttribute, VertexFormat,
    };

    const VERTEX: &str = r#"#version 100
    precision mediump float;

    attribute vec2 in_pos;
    attribute vec2 in_uv;

    uniform vec2 zoom;

    varying vec2 tex_coord;

    void main() {
        gl_Position = vec4(in_pos * zoom, 0, 1);
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

    #[repr(C)]
    pub struct Uniforms {
        pub zoom: [f32; 2],
    }

    pub fn pipeline(gfx: &mut GlContext) -> Pipeline {
        let shader = gfx
            .new_shader(
                ShaderSource::Glsl {
                    vertex: VERTEX,
                    fragment: FRAGMENT,
                },
                ShaderMeta {
                    images: vec![String::from("tex")],
                    uniforms: UniformBlockLayout {
                        uniforms: vec![UniformDesc::new("zoom", UniformType::Float2)],
                    },
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
