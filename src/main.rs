extern crate kiss3d;
extern crate stl_io;
extern crate nalgebra as na;
extern crate notify;

use notify::{RecursiveMode, Watcher, watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

use na::{Point3, Vector3, UnitQuaternion};
use std::rc::Rc;
use std::cell::RefCell;
use kiss3d::window::Window;
use kiss3d::light::Light;
use kiss3d::resource::Mesh;
use std::fs::File;

fn load_stl()-> std::rc::Rc<std::cell::RefCell<kiss3d::resource::Mesh>> {
    let mut f = File::open("bunny_99_ascii.stl").expect("file not found");
    let newm = stl_io::read_stl(&mut f).expect("can't read");
    let vertices = newm.vertices.iter().map(|v| Point3::new(v[0], v[1], v[2])).collect();
    let indices = newm.faces.iter()
        .map(|it| {
            Point3::new(it.vertices[0] as u32,it.vertices[1] as u32,it.vertices[2] as u32)
        }).collect();
    Rc::new(RefCell::new(Mesh::new(vertices, indices, None, None, false)))
}

fn main() {
    let mut window = Window::new("Kiss3d: cube");
    let mut c = window.add_cube(0.1, 0.1, 0.1);
    c.set_color(1.0, 0.0, 0.0);
    window.set_light(Light::StickToCamera);
    let rot = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.014);

    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();
    watcher.watch("bunny_99_ascii.stl", RecursiveMode::NonRecursive).unwrap();

    while window.render() {
        match rx.try_recv() {
            Ok(event) => {
                println!("{:?}", event);
                window.remove(&mut c);
                c = window.add_mesh(load_stl(), Vector3::new(1.0, 1.0, 1.0));
            },
            Err(_) => (),
        }
        c.prepend_to_local_rotation(&rot);
    }
}
