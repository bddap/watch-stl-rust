// extern crate kiss3d;
// extern crate nalgebra as na;
extern crate mint;
extern crate notify;
extern crate stl_io;
extern crate three;

use mint::Point3;
use three::Object;

mod file_watcher;
use file_watcher::FileRevisions;
mod aligned_orbit;

// use notify::{watcher, RecursiveMode, Watcher};
// use std::sync::mpsc::channel;
// use std::time::Duration;

// // use kiss3d::light::Light;
// // use kiss3d::resource::Mesh;
// // use kiss3d::scene::SceneNode;
// // use kiss3d::window::Window;
// use na::{Point3, Vector3};
// use std::cell::RefCell;
use std::fs::File;
use std::io;
use std::path::Path;
// use std::rc::Rc;

fn load_stl(mut file: File) -> io::Result<three::Geometry> {
    let mesh = stl_io::read_stl(&mut file)?;
    let vertices = mesh.vertices.iter().map(|&v| Point3::from(v)).collect();
    let faces = mesh.faces
        .iter()
        .map(|t| {
            [
                t.vertices[0] as u32,
                t.vertices[1] as u32,
                t.vertices[2] as u32,
            ]
        })
        .collect();

    Ok(three::Geometry {
        faces,
        base: three::Shape {
            vertices,
            ..three::Shape::default()
        },
        ..three::Geometry::default()
    })
}

// fn to_resized_kiss_mesh(imesh: &stl_io::IndexedMesh) -> Mesh {
//     let bounds = get_bounds(imesh);
//     let center = get_center(bounds);
//     let scale = get_appropriate_scale(bounds);
//     let vertices = imesh
//         .vertices
//         .iter()
//         .map(|v| {
//             Point3::new(
//                 (v[0] - center.x) * scale,
//                 (v[1] - center.y) * scale,
//                 (v[2] - center.z) * scale,
//             )
//         })
//         .collect();
//     let indices = imesh
//         .faces
//         .iter()
//         .map(|it| {
//             Point3::new(
//                 it.vertices[0] as u32,
//                 it.vertices[1] as u32,
//                 it.vertices[2] as u32,
//             )
//         })
//         .collect();
//     Mesh::new(vertices, indices, None, None, false)
// }

// fn get_bounds(mesh: &stl_io::IndexedMesh) -> (Vector3<f32>, Vector3<f32>) {
//     let max_x = mesh.vertices
//         .iter()
//         .map(|v| v[0])
//         .max_by(|a, b| a.partial_cmp(b).unwrap())
//         .expect("zero length mesh");
//     let max_y = mesh.vertices
//         .iter()
//         .map(|v| v[1])
//         .max_by(|a, b| a.partial_cmp(b).unwrap())
//         .expect("zero length mesh");
//     let max_z = mesh.vertices
//         .iter()
//         .map(|v| v[2])
//         .max_by(|a, b| a.partial_cmp(b).unwrap())
//         .expect("zero length mesh");
//     let min_x = mesh.vertices
//         .iter()
//         .map(|v| v[0])
//         .max_by(|b, a| a.partial_cmp(b).unwrap())
//         .expect("zero length mesh");
//     let min_y = mesh.vertices
//         .iter()
//         .map(|v| v[1])
//         .max_by(|b, a| a.partial_cmp(b).unwrap())
//         .expect("zero length mesh");
//     let min_z = mesh.vertices
//         .iter()
//         .map(|v| v[2])
//         .max_by(|b, a| a.partial_cmp(b).unwrap())
//         .expect("zero length mesh");
//     (
//         Vector3::new(min_x, min_y, min_z),
//         Vector3::new(max_x, max_y, max_z),
//     )
// }

// fn get_center(bounds: (Vector3<f32>, Vector3<f32>)) -> Vector3<f32> {
//     let mut center = bounds.0 + bounds.1;
//     center.x = center.x / 2.0;
//     center.y = center.y / 2.0;
//     center.z = center.z / 2.0;
//     center
// }

// fn get_appropriate_scale(bounds: (Vector3<f32>, Vector3<f32>)) -> f32 {
//     let diff = bounds.0 - bounds.1;
//     let mut m = diff.x.abs();
//     if m > diff.y.abs() {
//         m = diff.y.abs();
//     }
//     if m > diff.z.abs() {
//         m = diff.z.abs();
//     }
//     return 1.0 / m;
// }

// fn swap_mesh(w: &mut Window, mut c: &mut SceneNode, f: &Path) -> SceneNode {
//     let imesh = load_stl(f);
//     let mesh = to_resized_kiss_mesh(&imesh);
//     set_mesh(w, c, mesh)
// }

// fn set_mesh(w: &mut Window, mut c: &mut SceneNode, mesh: Mesh) -> SceneNode {
//     w.remove(&mut c);
//     let mut n = w.add_mesh(Rc::new(RefCell::new(mesh)), Vector3::new(0.3, 0.3, 0.3));
//     n.set_color(1.0, 0.0, 0.0);
//     n
// }

// fn main() {

//         let mut mshs = FileRevisions::from_path(filename).unwrap().map(|fo| {
//             fo.and_then(|mut f| stl_io::read_stl(&mut f))
//                 .map(|mut m| to_resized_kiss_mesh(&mut m))
//         });
//         let mut window = Window::new(&flns);
//         let mut c = window.add_cube(0.1, 0.1, 0.1);
//         c = swap_mesh(&mut window, &mut c, filename);
//         window.set_light(Light::StickToCamera);
//         window.set_framerate_limit(Some(60));
//         while window.render() {
//             match mshs.next() {
//                 Some(Ok(mesh)) => {
//                     c = set_mesh(&mut window, &mut c, mesh);
//                 }
//                 _ => {}
//             }
//         }
//     }
// }

// use three::Object;

fn watch(title: String, path: &Path) -> io::Result<()> {
    if let Ok(mut rs) = FileRevisions::from_path(path) {
        let mut window = three::Window::new(title);

        // let vertices = vec![
        //     [-0.5, -0.5, -0.5].into(),
        //     [0.5, -0.5, -0.5].into(),
        //     [0.0, 0.5, -0.5].into(),
        // ];
        // let mut geometry = three::Geometry::with_vertices(vertices);
        let geometry = three::Geometry::cuboid(1.0, 1.0, 1.0);
        // let geometry = load_stl(File::open(path)?)?;
        let material = three::material::Basic {
            color: 0xFFFF00,
            ..Default::default()
        };

        let center = [0.0, 0.0];
        let yextent = 100.0;
        let zrange = 0.1..1000.0;
        // let camera = window.factory.orthographic_camera(center, yextent, zrange);
        let mut camera = window.factory.perspective_camera(yextent, zrange);
        camera.set_position([0.0, 0.0, 10.0]);

        let mesh = window.factory.mesh(geometry, material);
        window.scene.add(&mesh);

        // let mut controls = three::controls::Orbit::builder(&camera)
        //     .position([0.0, 2.0, -5.0])
        //     .target([0.0, 0.0, 0.0])
        //     .build();

        let mut controls = aligned_orbit::AlignedOrbit::new(&mesh);
        // controls.target = [0.1, 0.2, 0.3].into();

        while window.update() {
            controls.update(&window.input);
            window.render(&camera);
            match rs.next() {
                Some(Ok(file)) => {
                    // geometry = load_stl(file)?;
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn main() {
    use std::env;
    if let Some(flns) = env::args().nth(1) {
        let fl2 = flns.clone();
        watch(flns, Path::new(&fl2)).is_ok();
    }
}
