extern crate kiss3d;
extern crate stl_io;
extern crate simple_signal;
extern crate nalgebra as na;

use std::time::SystemTime;
use na::{Point3, Vector3, UnitQuaternion};
use std::rc::Rc;
use std::cell::RefCell;
use kiss3d::window::Window;
use kiss3d::light::Light;
use kiss3d::resource::Mesh;
use kiss3d::scene::SceneNode;
use std::fs::File;
use std::path::Path;
use simple_signal::Signal;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn load_stl(filename: &Path)-> std::rc::Rc<std::cell::RefCell<kiss3d::resource::Mesh>> {
    let mut f = File::open(&filename).expect("file not found");
    let newm = stl_io::read_stl(&mut f).expect("can't read");
    let vertices = newm.vertices.iter().map(|v| Point3::new(v[0], v[1], v[2])).collect();
    let indices = newm.faces.iter()
        .map(|it| {
            Point3::new(it.vertices[0] as u32,it.vertices[1] as u32,it.vertices[2] as u32)
        }).collect();
    Rc::new(RefCell::new(Mesh::new(vertices, indices, None, None, false)))
}

fn elapsed_seconds(t: SystemTime) -> f32 {
    let d = t.elapsed().expect("wtf");
    d.as_secs() as f32 + d.subsec_nanos() as f32 / 1_000_000_000.0
}

struct Renderer {
    w: Window,
    m: SceneNode,
    e: SystemTime
}

impl Renderer {
    fn render(&mut self) -> bool {
        let rotation = UnitQuaternion::from_axis_angle(
            &Vector3::y_axis(), elapsed_seconds(self.e));
        self.m.set_local_rotation(rotation);
        self.w.render()
    }
    
    fn reload_mesh(&mut self, f: &Path) {
        self.w.remove(&mut self.m);
        self.m = self.w.add_mesh(load_stl(f), Vector3::new(1.0, 1.0, 1.0));
        self.m.set_color(1.0, 0.0, 0.0);
    }
    
    fn new() -> Renderer {
        let mut w = Window::new("watch-stl");
        let m = w.add_cube(0.1, 0.1, 0.1);
        let e = SystemTime::now();
        w.set_light(Light::StickToCamera);
        w.set_framerate_limit(Some(60));
        Renderer { w: w, m: m, e: e}
    }
}

fn watch(filename: &Path) {
    let mut r = Renderer::new();
    r.reload_mesh(filename);

    let do_reload = Arc::new(AtomicBool::new(true));
    let d = do_reload.clone();
    simple_signal::set_handler(&[Signal::Hup], move |_signals| {
        d.store(false, Ordering::SeqCst);
    });
    
    while r.render() {
        if do_reload.load(Ordering::SeqCst) {
            r.reload_mesh(filename);
        }
    }
}

fn main() {
    use std::env;
    if let Some(flns) = env::args().nth(1) {
        let filename = Path::new(&flns);
        watch(&filename);
    }
}
