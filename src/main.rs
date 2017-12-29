extern crate kiss3d;
extern crate stl_io;
extern crate nalgebra as na;
extern crate notify;

use notify::{RecursiveMode, Watcher, watcher};
use std::sync::mpsc::channel;
use std::time::{Duration, SystemTime};

use na::{Point3, Vector3, UnitQuaternion};
use std::rc::Rc;
use std::cell::RefCell;
use kiss3d::window::Window;
use kiss3d::light::Light;
use kiss3d::resource::Mesh;
use kiss3d::scene::SceneNode;
use std::fs::File;
use std::path::Path;

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

fn file_watcher(filename: &Path) -> std::sync::mpsc::Receiver<notify::DebouncedEvent> {
    let (tx, rx) = channel();
    watcher(tx, Duration::from_secs(1)).unwrap()
        .watch(filename, RecursiveMode::NonRecursive).unwrap();
    rx
}

fn swap_mesh(w: &mut Window, mut c: &mut SceneNode, f: &Path) -> SceneNode {
    w.remove(&mut c);
    let mut n = w.add_mesh(load_stl(f), Vector3::new(1.0, 1.0, 1.0));
    n.set_color(1.0, 0.0, 0.0);
    n
}

fn main() {
    use std::env;
    if let Some(flns) = env::args().nth(1) {
        let filename = Path::new(&flns);
        let start = SystemTime::now();
        let rx = file_watcher(filename);
        let mut window = Window::new("Kiss3d: cube");
        let mut c = window.add_cube(0.1, 0.1, 0.1);
        c = swap_mesh(&mut window, &mut c, &filename);
        window.set_light(Light::StickToCamera);
        window.set_framerate_limit(Some(60));
        while window.render() {
            if rx.try_recv().is_ok() {
                c = swap_mesh(&mut window, &mut c, &filename);
            }
            c.set_local_rotation(UnitQuaternion::from_axis_angle(&Vector3::y_axis(), elapsed_seconds(start)));
        }
    }
}
