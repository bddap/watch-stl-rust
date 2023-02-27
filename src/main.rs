mod file_watcher;

use file_watcher::FileRevisions;
use kiss3d::light::Light;
use kiss3d::nalgebra as na;
use kiss3d::resource::Mesh;
use kiss3d::scene::SceneNode;
use kiss3d::window::Window;
use na::{Point3, Vector3};
use std::cell::RefCell;
use std::fs::File;
use std::path::Path;
use std::rc::Rc;

fn load_stl(filename: &Path) -> stl_io::IndexedMesh {
    let mut f = File::open(&filename).expect("file not found");
    stl_io::read_stl(&mut f).expect("can't read")
}

fn to_resized_kiss_mesh(imesh: &stl_io::IndexedMesh) -> Mesh {
    let bounds = get_bounds(imesh);
    let center = get_center(bounds);
    let scale = get_appropriate_scale(bounds);
    let vertices: Vec<Point3<f32>> = imesh
        .vertices
        .iter()
        .map(|v| {
            Point3::new(
                (v[0] - center.x) * scale,
                (v[1] - center.y) * scale,
                (v[2] - center.z) * scale,
            )
        })
        .collect();
    let indices: Vec<Point3<u16>> = imesh
        .faces
        .iter()
        .map(|it| {
            Point3::new(
                it.vertices[0] as u16, // may panic on very large meshes
                it.vertices[1] as u16,
                it.vertices[2] as u16,
            )
        })
        .collect();
    Mesh::new(vertices, indices, None, None, false)
}

fn get_bounds(mesh: &stl_io::IndexedMesh) -> (Vector3<f32>, Vector3<f32>) {
    let max_x = mesh
        .vertices
        .iter()
        .map(|v| v[0])
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .expect("zero length mesh");
    let max_y = mesh
        .vertices
        .iter()
        .map(|v| v[1])
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .expect("zero length mesh");
    let max_z = mesh
        .vertices
        .iter()
        .map(|v| v[2])
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .expect("zero length mesh");
    let min_x = mesh
        .vertices
        .iter()
        .map(|v| v[0])
        .max_by(|b, a| a.partial_cmp(b).unwrap())
        .expect("zero length mesh");
    let min_y = mesh
        .vertices
        .iter()
        .map(|v| v[1])
        .max_by(|b, a| a.partial_cmp(b).unwrap())
        .expect("zero length mesh");
    let min_z = mesh
        .vertices
        .iter()
        .map(|v| v[2])
        .max_by(|b, a| a.partial_cmp(b).unwrap())
        .expect("zero length mesh");
    (
        Vector3::new(min_x, min_y, min_z),
        Vector3::new(max_x, max_y, max_z),
    )
}

fn get_center(bounds: (Vector3<f32>, Vector3<f32>)) -> Vector3<f32> {
    let mut center = bounds.0 + bounds.1;
    center.x = center.x / 2.0;
    center.y = center.y / 2.0;
    center.z = center.z / 2.0;
    center
}

fn get_appropriate_scale(bounds: (Vector3<f32>, Vector3<f32>)) -> f32 {
    let diff = bounds.0 - bounds.1;
    let mut m = diff.x.abs();
    if m > diff.y.abs() {
        m = diff.y.abs();
    }
    if m > diff.z.abs() {
        m = diff.z.abs();
    }
    return 1.0 / m;
}

fn swap_mesh(w: &mut Window, c: &mut SceneNode, f: &Path) -> SceneNode {
    let imesh = load_stl(f);
    let mesh = to_resized_kiss_mesh(&imesh);
    set_mesh(w, c, mesh)
}

fn set_mesh(w: &mut Window, mut c: &mut SceneNode, mesh: Mesh) -> SceneNode {
    w.remove_node(&mut c);
    let mut n = w.add_mesh(Rc::new(RefCell::new(mesh)), Vector3::new(0.3, 0.3, 0.3));
    n.set_color(1.0, 0.0, 0.0);
    n
}

fn main() -> anyhow::Result<()> {
    use std::env;
    let Some(flns) = env::args().nth(1) else {
        return Err(anyhow::anyhow!("no file name given"));
    };

    let filename = Path::new(&flns);
    let mut watch = FileRevisions::from_path(filename)?;
    let mut window = Window::new(&flns);
    let mut c = window.add_cube(0.1, 0.1, 0.1);
    c = swap_mesh(&mut window, &mut c, filename);
    window.set_light(Light::StickToCamera);
    window.set_framerate_limit(Some(60));

    while window.render() {
        if !watch.changed()? {
            continue;
        }
        let mut file = File::open(filename)?;
        let stl = stl_io::read_stl(&mut file)?;
        let mesh = to_resized_kiss_mesh(&stl);
        set_mesh(&mut window, &mut c, mesh);
    }

    Ok(())
}
