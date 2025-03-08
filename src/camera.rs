use crate::color::*;
use crate::random::*;
use crate::ray::*;
use crate::scene::*;
use crate::vec3::*;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, SyncSender};
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub struct CameraOptions {
    pub aspect_ratio: f64,  // Ratio of image width over height
    pub image_width: u16,   // Rendered image width in pixel count
    pub max_depth: u16,     // Maximum number of ray bounces into scene
    pub vfov: f64,          // Vertical view angle (field of view)
    pub lookfrom: Vec3,     // Point camera is looking from
    pub lookat: Vec3,       // Point camera is looking at
    pub vup: Vec3,          // Camera-relative "up" direction
    pub defocus_angle: f64, // Variation angle of rays through each pixel.
    pub focus_dist: f64,    // Distance from camera lookfrom point to plane of perfect focus.
}

impl Default for CameraOptions {
    fn default() -> Self {
        Self {
            aspect_ratio: 1.0,
            image_width: 100,
            max_depth: 10,
            vfov: 90.0,
            lookfrom: Vec3::new(0.0, 0.0, 0.0),
            lookat: Vec3::new(0.0, 0.0, -1.0),
            vup: Vec3::new(0.0, 1.0, 0.0),
            defocus_angle: 0.0,
            focus_dist: 10.0,
        }
    }
}

pub struct Camera {
    image_width: usize,
    image_height: usize,
    pixel_bufs: Vec<Arc<Mutex<Vec<u8>>>>,
    view_xs: Vec<usize>,
    view_widths: Vec<usize>,
    passes_wanted: usize,
    pause: Arc<AtomicBool>,
    passes_wanted_txs: Vec<SyncSender<usize>>,
    passes_done_rxs: Vec<Receiver<usize>>,
}

impl Camera {
    pub fn new(scene: &Arc<Scene>, rng_seed: u64, num_views: u8, options: CameraOptions) -> Self {
        let i_width_usize = options.image_width as usize;
        let i_height_usize = usize::max(
            1,
            (options.image_width as f64 / options.aspect_ratio) as usize,
        );

        let image_width = i_width_usize as f64;
        let image_height = i_height_usize as f64;

        let center = options.lookfrom;

        // Determine viewport dimensions.
        let theta = options.vfov.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h * options.focus_dist;
        let viewport_width = viewport_height * image_width / image_height;

        // Calculate the u,v,w unit basis vectors for the camera coordinate frame.
        let w = (options.lookfrom - options.lookat).unit();
        let u = options.vup.cross(w).unit();
        let v = w.cross(u);

        // Calculate the vectors across the horizontal and down the vertical viewport edges.
        let viewport_u = viewport_width * u; // Vector across viewport horizontal edge
        let viewport_v = viewport_height * -v; // Vector across viewport vertical edge

        // Calculate the horizontal and vertical delta vectors from pixel to pixel.
        let pixel_delta_u = viewport_u / image_width;
        let pixel_delta_v = viewport_v / image_height;

        // Calculate the location of the upper left pixel.
        let viewport_upper_left =
            center - options.focus_dist * w - viewport_u / 2.0 - viewport_v / 2.0;
        let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        // Calculate the defocus disk basis vectors.
        let defocus_radius = options.focus_dist * (options.defocus_angle / 2.0).to_radians().tan();
        let defocus_disk_u = defocus_radius * u;
        let defocus_disk_v = defocus_radius * v;

        // Create multiple vertical views to cover the camera's full view of the scene.

        let num_views = num_views as usize;

        assert!(num_views > 0);
        assert!(num_views <= i_width_usize);

        let mut pixel_bufs: Vec<Arc<Mutex<Vec<u8>>>> = vec![];
        let mut view_xs: Vec<usize> = vec![];
        let mut view_widths: Vec<usize> = vec![];
        let pause = Arc::new(AtomicBool::new(false));
        let mut passes_wanted_txs: Vec<SyncSender<usize>> = vec![];
        let mut passes_done_rxs: Vec<Receiver<usize>> = vec![];

        for i in 0..num_views {
            let view_x = i * i_width_usize / num_views;
            let view_width = (i + 1) * i_width_usize / num_views - view_x;

            let pixel_buf = Arc::new(Mutex::new(vec![0_u8; 4 * view_width * i_height_usize]));

            pixel_bufs.push(Arc::clone(&pixel_buf));
            view_xs.push(view_x);
            view_widths.push(view_width);

            let (passes_wanted_tx, passes_wanted_rx) = std::sync::mpsc::sync_channel::<usize>(0);
            let (passes_done_tx, passes_done_rx) = std::sync::mpsc::sync_channel::<usize>(0);

            passes_wanted_txs.push(passes_wanted_tx);
            passes_done_rxs.push(passes_done_rx);

            std::thread::spawn({
                let scene = Arc::clone(scene);
                let rng_seed = rng_seed + i as u64;
                let pause = pause.clone();

                move || {
                    let mut view = View {
                        color_buf: vec![Color::new(0.0, 0.0, 0.0); view_width * i_height_usize],
                        width: view_width,
                        height: i_height_usize,
                        max_depth: options.max_depth,
                        start_row: 0,
                        render_passes: 0,
                        pause,
                        pixel00_loc: pixel00_loc + view_x as f64 * pixel_delta_u,
                        pixel_delta_u,
                        pixel_delta_v,
                        center,
                        defocus_angle: options.defocus_angle,
                        defocus_disk_u,
                        defocus_disk_v,
                    };
                    let mut rng = Rng::new(rng_seed);

                    while let Ok(passes_wanted) = passes_wanted_rx.recv() {
                        let mut pixel_buf = pixel_buf.lock().expect("pixel_buf mutex");
                        view.render(&mut rng, &scene, &mut pixel_buf[..], passes_wanted);
                        drop(pixel_buf);
                        passes_done_tx
                            .send(view.render_passes)
                            .expect("passes_done_tx");
                    }
                }
            });
        }

        Self {
            image_width: i_width_usize,
            image_height: i_height_usize,
            pixel_bufs,
            view_xs,
            view_widths,
            passes_wanted: 0,
            pause,
            passes_wanted_txs,
            passes_done_rxs,
        }
    }

    pub fn get_width(&self) -> usize {
        self.image_width
    }

    pub fn get_height(&self) -> usize {
        self.image_height
    }

    pub fn for_each_view<F: FnMut(usize, usize, usize, &[u8])>(&self, mut f: F) {
        for (i, ((view_x, view_width), pixel_buf)) in self
            .view_xs
            .iter()
            .copied()
            .zip(self.view_widths.iter().copied())
            .zip(&self.pixel_bufs)
            .enumerate()
        {
            let pixel_buf = pixel_buf.lock().expect("pixel_buf mutex");
            f(i, view_x, view_width, &pixel_buf);
        }
    }

    pub fn render(&mut self, until: Instant) {
        // Request no more than `self.passes_wanted` render passes from view threads.
        for passes_wanted_tx in &self.passes_wanted_txs {
            passes_wanted_tx
                .send(self.passes_wanted)
                .expect("passes_wanted_tx");
        }

        // Sleep up to `until`, then pause any currently-rendering view threads.
        let now = Instant::now();
        if until > now {
            std::thread::sleep(until.saturating_duration_since(now));
        }
        self.pause.store(true, Ordering::Relaxed);

        // Gather passes done by all the view threads.
        let mut all_passes_done = true;
        for passes_done_rx in &self.passes_done_rxs {
            let this_passes_done = passes_done_rx.recv().expect("passes_done_rx");
            if this_passes_done < self.passes_wanted {
                all_passes_done = false;
            }
        }

        // Increment `self.passes_wanted` if all threads have finished this pass.
        if all_passes_done {
            self.passes_wanted += 1;
        }

        // Prepare to let view threads render again.
        self.pause.store(false, Ordering::Relaxed);
    }
}

struct View {
    color_buf: Vec<Color>,
    width: usize,
    height: usize,
    max_depth: u16,
    start_row: usize,
    render_passes: usize,
    pause: Arc<AtomicBool>,
    pixel00_loc: Vec3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    center: Vec3,
    defocus_angle: f64,
    defocus_disk_u: Vec3,
    defocus_disk_v: Vec3,
}

impl View {
    fn sample_square(rng: &mut Rng) -> Vec3 {
        // Returns the vector to a random point in the [-0.5,-0.5] to [+0.5,+0.5] unit square.
        Vec3::new(rng.random_f64() - 0.5, rng.random_f64() - 0.5, 0.0)
    }

    fn defocus_disk_sample(&self, rng: &mut Rng) -> Vec3 {
        // Returns a random point in the camera defocus disk.
        let p = Vec3::random_in_unit_disk(rng);
        self.center + p.x() * self.defocus_disk_u + p.y() * self.defocus_disk_v
    }

    fn get_ray(&self, rng: &mut Rng, i: f64, j: f64) -> Ray {
        // Construct a camera ray originating from the defocus disk and directed at a
        // randomly-sampled point around the pixel location i, j.

        let offset = Self::sample_square(rng);
        let pixel_sample = self.pixel00_loc
            + ((i + offset.x()) * self.pixel_delta_u)
            + ((j + offset.y()) * self.pixel_delta_v);

        let ray_origin = if self.defocus_angle <= 0.0 {
            self.center
        } else {
            self.defocus_disk_sample(rng)
        };
        let ray_direction = pixel_sample - ray_origin;

        Ray {
            pos: ray_origin,
            dir: ray_direction,
        }
    }

    fn ray_color(rng: &mut Rng, depth: u16, r: &Ray, scene: &Scene) -> Color {
        if depth == 0 {
            return Color::new(0.0, 0.0, 0.0);
        }

        if let Some(rec) = scene.hit(r, 0.001, f64::INFINITY) {
            return if let Some(sc_rec) = rec.mat.scatter(rng, r, &rec) {
                sc_rec.attenuation * Self::ray_color(rng, depth - 1, &sc_rec.scattered, scene)
            } else {
                Color::new(0.0, 0.0, 0.0)
            };
        }

        let unit_direction = r.dir.unit();
        let a = 0.5 * (unit_direction.y() + 1.0);
        (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
    }

    pub fn render(
        &mut self,
        rng: &mut Rng,
        scene: &Scene,
        pixel_buf: &mut [u8],
        passes_wanted: usize,
    ) {
        if self.render_passes >= passes_wanted {
            return;
        }

        let passes_plus_one = self.render_passes as f64 + 1.0;
        let mut color_buf = vec![];

        std::mem::swap(&mut color_buf, &mut self.color_buf);

        let color_rows = color_buf.chunks_exact_mut(self.width);
        let pixel_rows = pixel_buf.chunks_exact_mut(self.width * 4);

        for (y, (color_row, pixel_row)) in
            color_rows.zip(pixel_rows).enumerate().skip(self.start_row)
        {
            let colors = color_row.iter_mut();
            let pixels = pixel_row.chunks_exact_mut(4);

            for (x, (c, p)) in colors.zip(pixels).enumerate() {
                let ray = self.get_ray(rng, x as f64, y as f64);
                *c += Self::ray_color(rng, self.max_depth, &ray, scene);
                p[0] = ((c.r() / passes_plus_one).sqrt() * 255.999) as u8;
                p[1] = ((c.g() / passes_plus_one).sqrt() * 255.999) as u8;
                p[2] = ((c.b() / passes_plus_one).sqrt() * 255.999) as u8;
                p[3] = 255;
            }

            self.start_row += 1;
            if self.start_row >= self.height {
                self.render_passes += 1;
                self.start_row = 0;
            }
            if self.pause.load(Ordering::Relaxed) {
                break;
            }
        }

        std::mem::swap(&mut color_buf, &mut self.color_buf);
    }
}
