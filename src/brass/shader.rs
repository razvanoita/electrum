use vulkano as vk;

pub struct Shader {

}

#[derive(Debug, Copy, Clone)]
struct VertexInputIterator(u16);

impl Iterator for VertexInputIterator {
    
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct VertexInput;

unsafe impl vk::pipeline::shader::ShaderInterfaceDef for VertexInput {
    type Iter = VertexInputIterator;

    fn elements(&self) -> VertexInputIterator {
        VertexInputIterator(0)
    }
}

impl VertexShader {
    pub fn new(path: String, device: Arc<vk::device::Device>) {
        let f = File::open(path).expect("Can't find shader file in path!");
        let mut v = vec![];
        f.read_to_end(&mut v).unwrap();

        let shader_module = unsafe { vk::pipeline::shader::ShaderModule::new(device.clone(), &v ) }.unwrap();


    }
}