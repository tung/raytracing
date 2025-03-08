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
use std::sync::Arc;
use std::time::{Duration, Instant};

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
    bindings: Vec<Bindings>,
    zoom: [f32; 2],
    camera: Camera,
}

impl App {
    fn new() -> Self {
        let mut rng = Rng::new(miniquad::date::now() as _);

        // Scene

        let mut scene = Scene::new();

        let ground_material = Arc::new(Material::lambertian(Color::new(0.5, 0.5, 0.5)));
        scene.add(Sphere::new(
            Vec3::new(0.0, -1000.0, 0.0),
            1000.0,
            ground_material,
        ));

        for a in -11..11 {
            for b in -11..11 {
                let center = Vec3::new(
                    a as f64 + 0.9 * rng.random_f64(),
                    0.2,
                    b as f64 + 0.9 * rng.random_f64(),
                );

                if (center - Vec3::new(4.0, 0.2, 0.0)).length() <= 0.9 {
                    continue;
                }

                let choose_mat = rng.random_f64();
                let sphere_material: Arc<Material> = if choose_mat < 0.8 {
                    // diffuse
                    let albedo = Color::from_vec3(Vec3::random(&mut rng))
                        * Color::from_vec3(Vec3::random(&mut rng));
                    Arc::new(Material::lambertian(albedo))
                } else if choose_mat < 0.95 {
                    // metal
                    let albedo = Color::from_vec3(Vec3::random_range(&mut rng, 0.5, 1.0));
                    let fuzz = rng.random_f64_range(0.0, 0.5);
                    Arc::new(Material::metal(albedo, fuzz))
                } else {
                    // glass
                    Arc::new(Material::dielectric(1.5))
                };

                scene.add(Sphere::new(center, 0.2, sphere_material));
            }
        }

        let material1 = Arc::new(Material::dielectric(1.5));
        scene.add(Sphere::new(Vec3::new(0.0, 1.0, 0.0), 1.0, material1));

        let material2 = Arc::new(Material::lambertian(Color::new(0.4, 0.2, 0.1)));
        scene.add(Sphere::new(Vec3::new(-4.0, 1.0, 0.0), 1.0, material2));

        let material3 = Arc::new(Material::metal(Color::new(0.7, 0.6, 0.5), 0.0));
        scene.add(Sphere::new(Vec3::new(4.0, 1.0, 0.0), 1.0, material3));

        // Camera

        let threads = std::env::args()
            .nth(1)
            .and_then(|a| a.parse::<u8>().ok())
            .unwrap_or(4);

        let image_width: u16 = 1200;

        let camera = Camera::new(
            &Arc::new(scene),
            miniquad::date::now() as _,
            threads,
            CameraOptions {
                aspect_ratio: 16.0 / 9.0,
                image_width,
                max_depth: 50,
                vfov: 20.0,
                lookfrom: Vec3::new(13.0, 2.0, 3.0),
                lookat: Vec3::new(0.0, 0.0, 0.0),
                vup: Vec3::new(0.0, 1.0, 0.0),
                defocus_angle: 0.6,
                focus_dist: 10.0,
            },
        );

        let image_height = camera.get_height() as u16;

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

        let mut bindings: Vec<Bindings> = vec![];

        camera.for_each_view(|_, _, view_width, pixel_buf| {
            let texture = gfx.new_texture_from_rgba8(view_width as u16, image_height, pixel_buf);
            gfx.texture_set_mag_filter(texture, FilterMode::Nearest);

            bindings.push(Bindings {
                vertex_buffers: vec![quad_vbuf],
                index_buffer: quad_ibuf,
                images: vec![texture],
            });
        });

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
        }
    }
}

impl EventHandler for App {
    fn draw(&mut self) {
        self.gfx.begin_default_pass(Default::default());
        self.gfx.apply_pipeline(&self.pipeline);

        self.camera.for_each_view(|i, view_x, view_width, _| {
            self.gfx.apply_bindings(&self.bindings[i]);
            self.gfx
                .apply_uniforms(UniformsSource::table(&shader::Uniforms {
                    x_offset: view_x as f32,
                    view_width: view_width as f32,
                    max_width: self.camera.get_width() as f32,
                    zoom: self.zoom,
                }));

            self.gfx.draw(0, 6, 1);
        });

        self.gfx.end_render_pass();

        self.gfx.commit_frame();
    }

    fn update(&mut self) {
        let until = Instant::now() + Duration::from_micros(950_000 / 60);
        self.camera.render(until);
        self.camera.for_each_view(|i, _, _, pixel_buf| {
            self.gfx
                .texture_update(self.bindings[i].images[0], pixel_buf);
        });
    }

    fn key_down_event(&mut self, keycode: KeyCode, _keymods: KeyMods, _repeat: bool) {
        if keycode == KeyCode::Escape {
            miniquad::window::request_quit();
        }
    }

    fn resize_event(&mut self, width: f32, height: f32) {
        self.zoom = calc_zoom(
            self.camera.get_width() as f32,
            self.camera.get_height() as f32,
            width,
            height,
        );
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

    uniform float x_offset;
    uniform float view_width;
    uniform float max_width;
    uniform vec2 zoom;

    varying vec2 tex_coord;

    void main() {
        gl_Position = vec4(
            (((in_pos.x + 1.0) * 0.5 * view_width + x_offset) * 2.0 / max_width - 1.0) * zoom.x,
            in_pos.y * zoom.y, 0.0, 1.0);
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
        pub x_offset: f32,
        pub view_width: f32,
        pub max_width: f32,
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
                        uniforms: vec![
                            UniformDesc::new("x_offset", UniformType::Float1),
                            UniformDesc::new("view_width", UniformType::Float1),
                            UniformDesc::new("max_width", UniformType::Float1),
                            UniformDesc::new("zoom", UniformType::Float2),
                        ],
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
