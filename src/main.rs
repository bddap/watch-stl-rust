mod file_watcher;

use file_watcher::FileRevisions;
use kiss3d::light::Light;
use kiss3d::nalgebra as na;
use kiss3d::resource::Mesh;
use kiss3d::scene::SceneNode;
use kiss3d::window::Window;
use na::{Point3, Vector3};
use std::cell::RefCell;
use std::convert::TryInto;
use std::fs::File;
use std::path::Path;
use std::rc::Rc;
use stl_io::IndexedMesh;

fn load_stl(filename: &Path) -> stl_io::IndexedMesh {
    let mut f = File::open(filename).expect("file not found");
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
            // kiss3d apparently can't handle very large meshes. It uses u16 for indices.
            // A future workaround could be to split into multiple meshes.
            let err_msg = "This mesh is too large, consider pestering bddap for \
                           large mesh support: \
                           https://github.com/bddap/watch-stl-rust/issues";
            Point3::new(
                it.vertices[0].try_into().expect(err_msg),
                it.vertices[1].try_into().expect(err_msg),
                it.vertices[2].try_into().expect(err_msg),
            )
        })
        .collect();
    Mesh::new(vertices, indices, None, None, false)
}

fn get_bounds(mesh: &stl_io::IndexedMesh) -> (Vector3<f32>, Vector3<f32>) {
    let mut min = Vector3::new(std::f32::MAX, std::f32::MAX, std::f32::MAX);
    let mut max = Vector3::new(std::f32::MIN, std::f32::MIN, std::f32::MIN);
    for v in &mesh.vertices {
        min.x = min.x.min(v[0]);
        min.y = min.y.min(v[1]);
        min.z = min.z.min(v[2]);
        max.x = max.x.max(v[0]);
        max.y = max.y.max(v[1]);
        max.z = max.z.max(v[2]);
    }
    (min, max)
}

fn get_center(bounds: (Vector3<f32>, Vector3<f32>)) -> Vector3<f32> {
    let mut center = bounds.0 + bounds.1;
    center.x /= 2.0;
    center.y /= 2.0;
    center.z /= 2.0;
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
    1.0 / m
}

fn swap_mesh(w: &mut Window, c: &mut SceneNode, f: &Path) {
    let imesh = load_stl(f);
    let mesh = to_resized_kiss_mesh(&imesh);
    set_mesh(w, c, mesh)
}

fn set_mesh(w: &mut Window, c: &mut SceneNode, mesh: Mesh) {
    w.remove_node(c);
    let mut n = w.add_mesh(Rc::new(RefCell::new(mesh)), Vector3::new(0.3, 0.3, 0.3));
    n.set_color(1.0, 0.0, 0.0);
    *c = n;
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
    swap_mesh(&mut window, &mut c, filename);
    window.set_light(Light::StickToCamera);
    window.set_framerate_limit(Some(60));

    while window.render() {
        if !watch.changed()? {
            continue;
        }
        let mut file = File::open(filename)?;
        let stl = stl_io::read_stl(&mut file).unwrap_or_else(|_| unloadable_mesh());
        let mesh = to_resized_kiss_mesh(&stl);
        set_mesh(&mut window, &mut c, mesh);
    }

    Ok(())
}

/// this is what is displayed when a mesh cann't be loaded
fn unloadable_mesh() -> IndexedMesh {
    IndexedMesh {
        vertices: Vec::new(),
        faces: Vec::new(),
    }
}
