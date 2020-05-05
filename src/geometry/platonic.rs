
use cgmath::prelude::InnerSpace;

use crate::geometry::GeometryData;
use crate::geometry::Vertex;

pub fn tetrahedron() -> GeometryData {
    let mut data = GeometryData {
        vertices: Vec::default(),
        indices: Vec::default()
    };

    let sq2over3: f32 = 1.41421356237309504880 / 3.0;
    let sq6over3: f32 = 2.44948974278317809820 / 3.0;
    let third: f32 = 1.0 / 3.0;

    let vertex_data = vec![
        cgmath::Vector3 { x: 0.0, y: 0.0, z: 1.0 },
        cgmath::Vector3 { x: sq2over3, y: 0.0, z: -third, },
        cgmath::Vector3 { x: -sq2over3, y: sq6over3, z: -third, },
        cgmath::Vector3 { x: -sq2over3, y: -sq6over3, z: -third, }
    ];

    let faces = vec![
        [0, 1, 2],
        [0, 2, 3],
        [0, 3, 1],
        [1, 3, 2]
    ];

    let face_colors: Vec<[f32; 3]> = vec![
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0]
    ];

    for (i, face) in faces.iter().enumerate() {
        let normal = (vertex_data[face[1] as usize] - vertex_data[face[0] as usize])
            .cross(vertex_data[face[2] as usize] - vertex_data[face[0] as usize])
            .normalize();
        
        let base = data.vertices.len() as u32;
        data.indices.push(base + 2);
        data.indices.push(base + 1);
        data.indices.push(base);

        data.vertices.push(
            Vertex {
                position: [vertex_data[face[0] as usize].x, vertex_data[face[0] as usize].y, vertex_data[face[0] as usize].z],
                normal: [normal.x, normal.y, normal.z],
                color: face_colors[i as usize] 
            }
        );
        data.vertices.push(
            Vertex {
                position: [vertex_data[face[1] as usize].x, vertex_data[face[1] as usize].y, vertex_data[face[1] as usize].z],
                normal: [normal.x, normal.y, normal.z],
                color: face_colors[i as usize]
            }
        );
        data.vertices.push(
            Vertex {
                position: [vertex_data[face[2] as usize].x, vertex_data[face[2] as usize].y, vertex_data[face[2] as usize].z],
                normal: [normal.x, normal.y, normal.z],
                color: face_colors[i as usize]
            }
        );
    }

    data
}

pub fn cube() -> GeometryData {
    let mut data = GeometryData {
        vertices: Vec::default(),
        indices: Vec::default()
    };

    let face_normals_and_tangents = vec![
        (
            cgmath::Vector3 { x: 0.0, y: 0.0, z: 1.0 },
            cgmath::Vector3 { x: 0.0, y: 1.0, z: 0.0 }
        ),
        (
            cgmath::Vector3 { x: 0.0, y: 0.0, z: -1.0 },
            cgmath::Vector3 { x: 0.0, y: 1.0, z: 0.0 }
        ),
        (
            cgmath::Vector3 { x: 1.0, y: 0.0, z: 0.0 },
            cgmath::Vector3 { x: 0.0, y: 1.0, z: 0.0 }
        ),
        (
            cgmath::Vector3 { x: -1.0, y: 0.0, z: 0.0 },
            cgmath::Vector3 { x: 0.0, y: 1.0, z: 0.0 }
        ),
        (
            cgmath::Vector3 { x: 0.0, y: 1.0, z: 0.0 },
            cgmath::Vector3 { x: 0.0, y: 0.0, z: 1.0 }
        ),
        (
            cgmath::Vector3 { x: 0.0, y: -1.0, z: 0.0 },
            cgmath::Vector3 { x: 0.0, y: 0.0, z: 1.0 }
        )
    ];

    let face_colors: Vec<[f32; 3]> = vec![
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 1.0]
    ];

    for (i, face) in face_normals_and_tangents.iter().enumerate() {
        let (normal, tangent) = face;
        let edge0 = normal.cross(*tangent);
        let edge1 = normal.cross(edge0);

        let base = data.vertices.len() as u32;
        data.indices.push(base);
        data.indices.push(base + 1);
        data.indices.push(base + 2);
        data.indices.push(base);
        data.indices.push(base + 2);
        data.indices.push(base + 3);

        let vertex_positions = vec![
            normal - edge0 - edge1,
            normal - edge0 + edge1,
            normal + edge0 + edge1,
            normal + edge0 - edge1
        ];

        for j in 0..4 {
            data.vertices.push(
                Vertex {
                    position: [vertex_positions[j].x, vertex_positions[j].y, vertex_positions[j].z],
                    normal: [normal.x, normal.y, normal.z],
                    color: face_colors[i as usize] 
                }
            );    
        }
    }

    data
}

pub fn octahedron() -> GeometryData {
    let mut data = GeometryData {
        vertices: Vec::default(),
        indices: Vec::default()
    };

    let vertex_data = vec![
        cgmath::Vector3 { x: 1.0, y: 0.0, z: 0.0 },
        cgmath::Vector3 { x: -1.0, y: 0.0, z: 0.0, },
        cgmath::Vector3 { x: 0.0, y: 1.0, z: 0.0, },
        cgmath::Vector3 { x: 0.0, y: -1.0, z: 0.0, },
        cgmath::Vector3 { x: 0.0, y: 0.0, z: 1.0, },
        cgmath::Vector3 { x: 0.0, y: 0.0, z: -1.0, }
    ];

    let faces = vec![
        [4, 0, 2],
        [4, 2, 1],
        [4, 1, 3],
        [4, 3, 0],
        [5, 2, 0],
        [5, 1, 2],
        [5, 3, 1],
        [5, 0, 3]
    ];

    let face_colors: Vec<[f32; 3]> = vec![
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [0.5, 0.0, 0.0]
    ];

    for (i, face) in faces.iter().enumerate() {
        let normal = (vertex_data[face[1] as usize] - vertex_data[face[0] as usize])
            .cross(vertex_data[face[2] as usize] - vertex_data[face[0] as usize])
            .normalize();
        
        let base = data.vertices.len() as u32;
        data.indices.push(base);
        data.indices.push(base + 1);
        data.indices.push(base + 2);

        data.vertices.push(
            Vertex {
                position: [vertex_data[face[0] as usize].x, vertex_data[face[0] as usize].y, vertex_data[face[0] as usize].z],
                normal: [normal.x, normal.y, normal.z],
                color: face_colors[i as usize] 
            }
        );
        data.vertices.push(
            Vertex {
                position: [vertex_data[face[1] as usize].x, vertex_data[face[1] as usize].y, vertex_data[face[1] as usize].z],
                normal: [normal.x, normal.y, normal.z],
                color: face_colors[i as usize]
            }
        );
        data.vertices.push(
            Vertex {
                position: [vertex_data[face[2] as usize].x, vertex_data[face[2] as usize].y, vertex_data[face[2] as usize].z],
                normal: [normal.x, normal.y, normal.z],
                color: face_colors[i as usize]
            }
        );
    }

    data
}

pub fn dodecahedron() -> GeometryData {
    let sq3: f32 = 1.73205080757;
    let a = 1.0 / sq3;
    let b: f32 = 0.35682208977;
    let c: f32 = 0.93417235896;

    let mut data = GeometryData {
        vertices: Vec::default(),
        indices: Vec::default()
    };

    let vertex_data = vec![
        cgmath::Vector3 { x: a, y: a, z: a },
        cgmath::Vector3 { x: a, y: a, z: -a },
        cgmath::Vector3 { x: a, y: -a, z: a },
        cgmath::Vector3 { x: a, y: -a, z: -a },
        cgmath::Vector3 { x: -a, y: a, z: a },
        cgmath::Vector3 { x: -a, y: a, z: -a },
        cgmath::Vector3 { x: -a, y: -a, z: a },
        cgmath::Vector3 { x: -a, y: -a, z: -a },
        cgmath::Vector3 { x: b, y: c, z: 0.0 },
        cgmath::Vector3 { x: -b, y: c, z: 0.0 },
        cgmath::Vector3 { x: b, y: -c, z: 0.0 },
        cgmath::Vector3 { x: -b, y: -c, z: 0.0 },
        cgmath::Vector3 { x: c, y: 0.0, z: b },
        cgmath::Vector3 { x: c, y: 0.0, z: -b },
        cgmath::Vector3 { x: -c, y: 0.0, z: b },
        cgmath::Vector3 { x: -c, y: 0.0, z: -b },
        cgmath::Vector3 { x: 0.0, y: b, z: c },
        cgmath::Vector3 { x: 0.0, y: -b, z: c },
        cgmath::Vector3 { x: 0.0, y: b, z: -c },
        cgmath::Vector3 { x: 0.0, y: -b, z: -c },
    ];

    let faces: Vec<[u32; 5]> = vec![
        [0, 8, 9, 4, 16],
        [0, 16, 17, 2, 12],
        [12, 2, 10, 3, 13],
        [9, 5, 15, 14, 4],
        [3, 19, 18, 1, 13],
        [7, 11, 6, 14, 15],
        [0, 12, 13, 1, 8],
        [8, 1, 18, 5, 9],
        [16, 4, 14, 6, 17],
        [6, 11, 10, 2, 17],
        [7, 15, 5, 18, 19],
        [7, 19, 3, 10, 11],
    ];

    let face_colors: Vec<[f32; 3]> = vec![
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [0.5, 0.0, 0.0],
        [0.0, 0.5, 0.0],
        [0.0, 0.0, 0.5],
        [0.5, 0.5, 0.0],
        [0.5, 0.0, 0.5],
    ];

    for (i, face) in faces.iter().enumerate() {
        let normal = (vertex_data[face[1] as usize] - vertex_data[face[0] as usize])
            .cross(vertex_data[face[2] as usize] - vertex_data[face[0] as usize])
            .normalize();
        
        let base = data.vertices.len() as u32;
        data.indices.push(base + 2);
        data.indices.push(base + 1);
        data.indices.push(base);

        data.indices.push(base + 3);
        data.indices.push(base + 2);
        data.indices.push(base);

        data.indices.push(base + 4);
        data.indices.push(base + 3);
        data.indices.push(base);

        data.vertices.push(
            Vertex {
                position: [vertex_data[face[0] as usize].x, vertex_data[face[0] as usize].y, vertex_data[face[0] as usize].z],
                normal: [normal.x, normal.y, normal.z],
                color: face_colors[i as usize] 
            }
        );
        data.vertices.push(
            Vertex {
                position: [vertex_data[face[1] as usize].x, vertex_data[face[1] as usize].y, vertex_data[face[1] as usize].z],
                normal: [normal.x, normal.y, normal.z],
                color: face_colors[i as usize]
            }
        );
        data.vertices.push(
            Vertex {
                position: [vertex_data[face[2] as usize].x, vertex_data[face[2] as usize].y, vertex_data[face[2] as usize].z],
                normal: [normal.x, normal.y, normal.z],
                color: face_colors[i as usize]
            }
        );
        data.vertices.push(
            Vertex {
                position: [vertex_data[face[3] as usize].x, vertex_data[face[3] as usize].y, vertex_data[face[3] as usize].z],
                normal: [normal.x, normal.y, normal.z],
                color: face_colors[i as usize]
            }
        );
        data.vertices.push(
            Vertex {
                position: [vertex_data[face[4] as usize].x, vertex_data[face[4] as usize].y, vertex_data[face[4] as usize].z],
                normal: [normal.x, normal.y, normal.z],
                color: face_colors[i as usize]
            }
        );
    }

    data
}

pub fn icosahedron() -> GeometryData {
    let t0: f32 = 1.6180339887;
    let t1: f32 = 1.5195449958;

    let mut data = GeometryData {
        vertices: Vec::default(),
        indices: Vec::default()
    };

    let vertex_data = vec![
        cgmath::Vector3 { x: t0 / t1, y: 1.0 / t1, z: 0.0 },
        cgmath::Vector3 { x: -t0 / t1, y: 1.0 / t1, z: 0.0 },
        cgmath::Vector3 { x: t0 / t1, y: -1.0 / t1, z: 0.0 },
        cgmath::Vector3 { x: -t0 / t1, y: -1.0 / t1, z: -0.0 },
        cgmath::Vector3 { x: 1.0 / t1, y: 0.0, z: t0 / t1 },
        cgmath::Vector3 { x: 1.0 / t1, y: 0.0, z: -t0 / t1 },
        cgmath::Vector3 { x: -1.0 / t1, y: 0.0, z: t0 / t1 },
        cgmath::Vector3 { x: -1.0 / t1, y: 0.0, z: -t0 / t1 },
        cgmath::Vector3 { x: 0.0, y: t0 / t1, z: 1.0 / t1 },
        cgmath::Vector3 { x: 0.0, y: -t0 / t1, z: 1.0 / t1 },
        cgmath::Vector3 { x: 0.0, y: t0 / t1, z: -1.0 / t1 },
        cgmath::Vector3 { x: 0.0, y: -t0 / t1, z: -1.0 / t1 },
    ];

    let faces: Vec<[u32; 3]> = vec![
        [0, 8, 4],
        [0, 5, 10],
        [2, 4, 9],
        [2, 11, 5],
        [1, 6, 8],
        [1, 10, 7],
        [3, 9, 6],
        [3, 7, 11],
        [0, 10, 8],
        [1, 8, 10],
        [2, 9, 11],
        [3, 11, 9],
        [4, 2, 0],
        [5, 0, 2],
        [6, 1, 3],
        [7, 3, 1],
        [8, 6, 4],
        [9, 4, 6],
        [10, 5, 7],
        [11, 7, 5]
    ];

    let face_colors: Vec<[f32; 3]> = vec![
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [0.5, 0.0, 0.0],
        [0.0, 0.5, 0.0],
        [0.0, 0.0, 0.5],
        [0.5, 0.5, 0.0],
        [0.0, 0.5, 0.5],
        [0.5, 0.0, 0.5],
        [0.5, 0.0, 1.0],
        [1.0, 0.0, 0.5],
        [0.5, 1.0, 0.0],
        [1.0, 0.5, 0.0],
        [0.0, 1.0, 0.5],
        [0.0, 0.5, 1.0],
        [0.5, 0.5, 0.5],
    ];

    for (i, face) in faces.iter().enumerate() {
        let normal = (vertex_data[face[1] as usize] - vertex_data[face[0] as usize])
            .cross(vertex_data[face[2] as usize] - vertex_data[face[0] as usize])
            .normalize();
        
        let base = data.vertices.len() as u32;
        data.indices.push(base + 2);
        data.indices.push(base + 1);
        data.indices.push(base);

        data.vertices.push(
            Vertex {
                position: [vertex_data[face[0] as usize].x, vertex_data[face[0] as usize].y, vertex_data[face[0] as usize].z],
                normal: [normal.x, normal.y, normal.z],
                color: face_colors[i as usize] 
            }
        );
        data.vertices.push(
            Vertex {
                position: [vertex_data[face[1] as usize].x, vertex_data[face[1] as usize].y, vertex_data[face[1] as usize].z],
                normal: [normal.x, normal.y, normal.z],
                color: face_colors[i as usize]
            }
        );
        data.vertices.push(
            Vertex {
                position: [vertex_data[face[2] as usize].x, vertex_data[face[2] as usize].y, vertex_data[face[2] as usize].z],
                normal: [normal.x, normal.y, normal.z],
                color: face_colors[i as usize]
            }
        );
    }

    data
}