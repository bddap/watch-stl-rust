mod file_watcher;

use file_watcher::FileRevisions;
use kiss3d::light::Light;
use kiss3d::nalgebra as na;
use kiss3d::resource::Mesh;
use kiss3d::scene::SceneNode;
use kiss3d::window::Window;
use na::{Point3, Vector3};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::rc::Rc;
use stl_io::IndexedMesh;

fn load_stl(filename: &Path) -> stl_io::IndexedMesh {
    let mut f = File::open(filename).expect("file not found");
    stl_io::read_stl(&mut f).expect("can't read")
}

/// Convert an STL mesh into one or more kiss3d meshes, centered and scaled to fit.
///
/// kiss3d uses u16 vertex indices, so a single mesh can reference at most
/// `u16::MAX + 1` distinct vertices. Large models exceed that and used to panic.
/// We greedily pack faces into sub-meshes, remapping global vertex indices to
/// local u16 indices and starting a fresh sub-mesh whenever the next face would
/// overflow the limit. Rendered together, the sub-meshes form the whole model.
fn to_resized_kiss_meshes(imesh: &stl_io::IndexedMesh) -> Vec<Mesh> {
    split_into_buffers(imesh)
        .into_iter()
        .map(|(vertices, indices)| Mesh::new(vertices, indices, None, None, false))
        .collect()
}

/// A single sub-mesh's geometry: local vertex positions plus u16 face indices into them.
type MeshBuffers = (Vec<Point3<f32>>, Vec<Point3<u16>>);

/// Center, scale, and split an STL mesh into kiss3d-sized buffers.
///
/// kiss3d uses u16 vertex indices, so a sub-mesh can reference at most
/// `u16::MAX + 1` distinct vertices. Faces are packed greedily; when the next
/// face would overflow the current sub-mesh, a new one is started. Global vertex
/// indices are remapped to local u16 indices per sub-mesh. Kept separate from
/// kiss3d's `Mesh` so it can be unit-tested without a graphics context.
fn split_into_buffers(imesh: &stl_io::IndexedMesh) -> Vec<MeshBuffers> {
    let bounds = get_bounds(imesh);
    let center = get_center(bounds);
    let scale = get_appropriate_scale(bounds);

    let transform = |gi: usize| -> Point3<f32> {
        let v = imesh.vertices[gi];
        Point3::new(
            (v[0] - center.x) * scale,
            (v[1] - center.y) * scale,
            (v[2] - center.z) * scale,
        )
    };

    // Max distinct vertices per sub-mesh: highest u16 index is 65535, so 65536 verts.
    const MAX_VERTS: usize = u16::MAX as usize + 1;

    let mut buffers: Vec<MeshBuffers> = Vec::new();
    let mut vertices: Vec<Point3<f32>> = Vec::new();
    let mut indices: Vec<Point3<u16>> = Vec::new();
    let mut remap: HashMap<usize, u16> = HashMap::new();

    for face in &imesh.faces {
        // How many of this face's verts aren't yet in the current sub-mesh.
        // (May overcount duplicates within a face; that only flushes early, which is safe.)
        let new_verts = face
            .vertices
            .iter()
            .filter(|&&gi| !remap.contains_key(&gi))
            .count();
        if !indices.is_empty() && vertices.len() + new_verts > MAX_VERTS {
            buffers.push((std::mem::take(&mut vertices), std::mem::take(&mut indices)));
            remap.clear();
        }
        let mut local = [0u16; 3];
        for (k, &gi) in face.vertices.iter().enumerate() {
            local[k] = *remap.entry(gi).or_insert_with(|| {
                let li = vertices.len() as u16;
                vertices.push(transform(gi));
                li
            });
        }
        indices.push(Point3::new(local[0], local[1], local[2]));
    }
    if !indices.is_empty() {
        buffers.push((vertices, indices));
    }
    buffers
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
    let meshes = to_resized_kiss_meshes(&imesh);
    set_meshes(w, c, meshes)
}

fn set_meshes(w: &mut Window, c: &mut SceneNode, meshes: Vec<Mesh>) {
    w.remove_node(c);
    // Hold all sub-meshes under one group node so a single SceneNode handle
    // represents the whole model (and can be removed wholesale on reload).
    let mut group = w.add_group();
    for mesh in meshes {
        let mut n = group.add_mesh(Rc::new(RefCell::new(mesh)), Vector3::new(0.3, 0.3, 0.3));
        n.set_color(1.0, 0.0, 0.0);
    }
    *c = group;
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
        let meshes = to_resized_kiss_meshes(&stl);
        set_meshes(&mut window, &mut c, meshes);
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

#[cfg(test)]
mod tests {
    use super::*;
    use stl_io::{IndexedMesh, IndexedTriangle, Normal, Vertex};

    /// Build a mesh of `n_faces` triangles where every vertex is distinct,
    /// so the mesh has `3 * n_faces` vertices.
    fn mesh_with_unique_verts(n_faces: usize) -> IndexedMesh {
        let mut vertices = Vec::with_capacity(n_faces * 3);
        let mut faces = Vec::with_capacity(n_faces);
        for i in 0..n_faces {
            let base = i * 3;
            let x = i as f32 * 0.001;
            vertices.push(Vertex::new([x, 0.0, 0.0]));
            vertices.push(Vertex::new([x, 1.0, 0.0]));
            vertices.push(Vertex::new([x, 0.0, 1.0]));
            faces.push(IndexedTriangle {
                normal: Normal::new([0.0, 0.0, 1.0]),
                vertices: [base, base + 1, base + 2],
            });
        }
        IndexedMesh { vertices, faces }
    }

    /// Every sub-mesh must fit u16 indices, and indices must stay in range.
    fn assert_buffers_valid(buffers: &[MeshBuffers], expected_faces: usize) {
        let mut total_faces = 0;
        for (verts, indices) in buffers {
            assert!(
                verts.len() <= u16::MAX as usize + 1,
                "sub-mesh has {} verts, exceeds u16 capacity",
                verts.len()
            );
            for tri in indices {
                for &idx in &[tri.x, tri.y, tri.z] {
                    assert!(
                        (idx as usize) < verts.len(),
                        "index {} out of range for {} verts",
                        idx,
                        verts.len()
                    );
                }
            }
            total_faces += indices.len();
        }
        assert_eq!(total_faces, expected_faces, "faces lost during split");
    }

    #[test]
    fn small_mesh_single_submesh() {
        let mesh = mesh_with_unique_verts(10);
        let buffers = split_into_buffers(&mesh);
        assert_eq!(buffers.len(), 1);
        assert_buffers_valid(&buffers, 10);
    }

    #[test]
    fn large_mesh_splits_without_overflow() {
        // 30000 faces * 3 unique verts = 90000 verts > u16::MAX, so this used to panic.
        let n_faces = 30000;
        let mesh = mesh_with_unique_verts(n_faces);
        let buffers = split_into_buffers(&mesh);
        assert!(buffers.len() > 1, "expected mesh to split into multiple parts");
        assert_buffers_valid(&buffers, n_faces);
    }

    #[test]
    fn empty_mesh_yields_no_buffers() {
        let mesh = unloadable_mesh();
        assert!(split_into_buffers(&mesh).is_empty());
    }
}
