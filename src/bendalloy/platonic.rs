
use cgmath::prelude::InnerSpace;

#[derive(Clone, Debug, Copy)]
pub struct Vertex {
    pub position: [f32; 4],
    pub normal: [f32; 4],
    pub color: [f32; 4]
}

pub struct GeometryData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>
}

const SQRT_OF_2: f32 = 1.41421356237309504880;
const SQRT_OF_6: f32 = 2.44948974278317809820;

pub fn tetrahedron() -> GeometryData {
    let mut data = GeometryData {
        vertices: Vec::default(),
        indices: Vec::default()
    };

    let vertex_data = vec![
        cgmath::Vector3 {
            x: 0.0f32,
            y: 0.0,
            z: 1.0
        },
        cgmath::Vector3 {
            x: SQRT_OF_2 / 3.0,
            y: 0.0,
            z: -1.0 / 3.0,
        },
        cgmath::Vector3 {
            x: -SQRT_OF_2 / 3.0,
            y: SQRT_OF_6 / 3.0,
            z: -1.0 / 3.0,
        },
        cgmath::Vector3 {
            x: -SQRT_OF_2 / 3.0,
            y: -SQRT_OF_6 / 3.0,
            z: -1.0 / 3.0,
        }
    ];

    let faces = vec![
        [2u32, 1, 0],
        [3u32, 2, 0],
        [1u32, 3, 0],
        [2u32, 3, 1]
    ];

    let face_colors: Vec<[f32; 4]> = vec![
        [1.0, 0.0, 0.0, 1.0],
        [0.0, 1.0, 0.0, 1.0],
        [0.0, 0.0, 1.0, 1.0],
        [1.0, 0.0, 1.0, 1.0]
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
                position: [vertex_data[face[0] as usize].x, vertex_data[face[0] as usize].y, vertex_data[face[0] as usize].z, 1.0],
                normal: [normal.x, normal.y, normal.z, 0.0],
                color: face_colors[i as usize] 
            }
        );
        data.vertices.push(
            Vertex {
                position: [vertex_data[face[1] as usize].x, vertex_data[face[1] as usize].y, vertex_data[face[1] as usize].z, 1.0],
                normal: [normal.x, normal.y, normal.z, 0.0],
                color: face_colors[i as usize]
            }
        );
        data.vertices.push(
            Vertex {
                position: [vertex_data[face[2] as usize].x, vertex_data[face[2] as usize].y, vertex_data[face[2] as usize].z, 1.0],
                normal: [normal.x, normal.y, normal.z, 0.0],
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

    let face_colors: Vec<[f32; 4]> = vec![
        [1.0, 0.0, 0.0, 1.0],
        [0.0, 1.0, 0.0, 1.0],
        [0.0, 0.0, 1.0, 1.0],
        [1.0, 0.0, 1.0, 1.0],
        [1.0, 1.0, 0.0, 1.0],
        [0.0, 1.0, 1.0, 1.0]
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
                    position: [vertex_positions[j].x, vertex_positions[j].y, vertex_positions[j].z, 1.0],
                    normal: [normal.x, normal.y, normal.z, 0.0],
                    color: face_colors[i as usize] 
                }
            );    
        }
    }

    data
}