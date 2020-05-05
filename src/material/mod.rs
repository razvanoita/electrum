use cgmath::*;

use crate::components;

pub enum Materials {
    Gold,
    RoughGold,
    Iron,
    RoughIron,
    Copper,
    RoughCopper,
    Silver,
    RoughSilver,
    Plastic,
    RoughPlastic,
    EmissiveWhite,
}

// --- F0 reflectance is sRGB; must convert to linear in shader
impl Materials {
    pub fn get(&self) -> components::PBRMaterial {
        match *self {
            Materials::Gold => components::PBRMaterial {
                albedo: cgmath::Vector3{ x:0.0, y:0.0, z:0.0 },
                f0_reflectance: cgmath::Vector3{ x:1.000, y:0.898, z:0.619 },
                roughness: 0.1,
                metalness: 1.0,
                material_type: components::PBRMaterialType::Pure,
                emissive_color: cgmath::Vector3{ x:0.0, y:0.0, z:0.0 },
            },
            Materials::RoughGold => components::PBRMaterial {
                albedo: cgmath::Vector3{ x:0.0, y:0.0, z:0.0 },
                f0_reflectance: cgmath::Vector3{ x:1.000, y:0.898, z:0.619 },
                roughness: 0.8,
                metalness: 1.0,
                material_type: components::PBRMaterialType::Pure,
                emissive_color: cgmath::Vector3{ x:0.0, y:0.0, z:0.0 },
            },
            Materials::Iron => components::PBRMaterial {
                albedo: cgmath::Vector3{ x:0.0, y:0.0, z:0.0 },
                f0_reflectance: cgmath::Vector3{ x:0.77, y:0.77, z:0.784 },
                roughness: 0.1,
                metalness: 1.0,
                material_type: components::PBRMaterialType::Pure,
                emissive_color: cgmath::Vector3{ x:0.0, y:0.0, z:0.0 },
            },
            Materials::RoughIron => components::PBRMaterial {
                albedo: cgmath::Vector3{ x:0.0, y:0.0, z:0.0 },
                f0_reflectance: cgmath::Vector3{ x:0.77, y:0.77, z:0.784 },
                roughness: 0.8,
                metalness: 1.0,
                material_type: components::PBRMaterialType::Pure,
                emissive_color: cgmath::Vector3{ x:0.0, y:0.0, z:0.0 },
            },
            Materials::Copper => components::PBRMaterial {
                albedo: cgmath::Vector3{ x:0.0, y:0.0, z:0.0 },
                f0_reflectance: cgmath::Vector3{ x:0.98, y:0.819, z:0.76 },
                roughness: 0.1,
                metalness: 1.0,
                material_type: components::PBRMaterialType::Pure,
                emissive_color: cgmath::Vector3{ x:0.0, y:0.0, z:0.0 },
            },
            Materials::RoughCopper => components::PBRMaterial {
                albedo: cgmath::Vector3{ x:0.0, y:0.0, z:0.0 },
                f0_reflectance: cgmath::Vector3{ x:0.98, y:0.819, z:0.76 },
                roughness: 0.8,
                metalness: 1.0,
                material_type: components::PBRMaterialType::Pure,
                emissive_color: cgmath::Vector3{ x:0.0, y:0.0, z:0.0 },
            },
            Materials::Silver => components::PBRMaterial {
                albedo: cgmath::Vector3{ x:0.0, y:0.0, z:0.0 },
                f0_reflectance: cgmath::Vector3{ x:0.98, y:0.97, z:0.95 },
                roughness: 0.1,
                metalness: 1.0,
                material_type: components::PBRMaterialType::Pure,
                emissive_color: cgmath::Vector3{ x:0.0, y:0.0, z:0.0 },
            },
            Materials::RoughSilver => components::PBRMaterial {
                albedo: cgmath::Vector3{ x:0.0, y:0.0, z:0.0 },
                f0_reflectance: cgmath::Vector3{ x:0.98, y:0.97, z:0.95 },
                roughness: 0.8,
                metalness: 1.0,
                material_type: components::PBRMaterialType::Pure,
                emissive_color: cgmath::Vector3{ x:0.0, y:0.0, z:0.0 },
            },
            Materials::Plastic => components::PBRMaterial {
                albedo: cgmath::Vector3{ x:0.2, y:0.2, z:0.2 },
                f0_reflectance: cgmath::Vector3{ x:0.219, y:0.219, z:0.219 },
                roughness: 0.8,
                metalness: 0.0,
                material_type: components::PBRMaterialType::Pure,
                emissive_color: cgmath::Vector3{ x:0.0, y:0.0, z:0.0 },
            },
            Materials::RoughPlastic => components::PBRMaterial {
                albedo: cgmath::Vector3{ x:0.2, y:0.2, z:0.2 },
                f0_reflectance: cgmath::Vector3{ x:0.219, y:0.219, z:0.219 },
                roughness: 0.8,
                metalness: 0.0,
                material_type: components::PBRMaterialType::Pure,
                emissive_color: cgmath::Vector3{ x:0.0, y:0.0, z:0.0 },
            },
            Materials::EmissiveWhite => components::PBRMaterial {
                albedo: cgmath::Vector3{ x:0.0, y:0.0, z:0.0 },
                f0_reflectance: cgmath::Vector3{ x:0.0, y:0.0, z:0.0 },
                roughness: 0.0,
                metalness: 0.0,
                material_type: components::PBRMaterialType::PureEmissive,
                emissive_color: cgmath::Vector3{ x:2.0, y:2.0, z:2.0 },
            },
        }
    }
}
