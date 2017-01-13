#[derive(Copy, Clone)]
pub struct Vertex(pub ::cgmath::Vector3<f32>);
#[derive(Copy, Clone)]
pub struct Normal(pub ::cgmath::Vector3<f32>);
pub type Index = u16;

/// This is the expansion of impl_vertex! macro which implements the needed traits to
/// use in the vulkano pipeline. It was modified to support newtypes.
/// Using this allows us to freely communicate between vulkan and the cgmath crates
unsafe impl ::vulkano::pipeline::vertex::Vertex for Vertex {
    fn member(name: &str) -> Option<::vulkano::pipeline::vertex::VertexMemberInfo> {
        use ::vulkano::format::Format;
        use ::vulkano::pipeline::vertex::VertexMemberInfo;
        use ::vulkano::pipeline::vertex::VertexMemberTy;
        use ::vulkano::pipeline::vertex::VertexMember;

        match name {
            "x" => {
                let (ty, array_size) = unsafe {
                    fn f<S: VertexMember>(_: &S) -> (VertexMemberTy, usize) { S::format() }
                    let dummy = 0usize as *const Vertex;
                    f(&(&*dummy).0.x)
                };

                Some(VertexMemberInfo {
                    offset: unsafe {
                        let dummy = 0usize as *const Vertex;
                        let member = (&(&*dummy).0.x) as *const _;
                        member as usize
                    },

                    ty: ty,
                    array_size: array_size,
                })
            },
            "y" => {
                let (ty, array_size) = unsafe {
                    fn f<S: VertexMember>(_: &S) -> (VertexMemberTy, usize) { S::format() }
                    let dummy = 0usize as *const Vertex;
                    f(&(&*dummy).0.y)
                };

                Some(VertexMemberInfo {
                    offset: unsafe {
                        let dummy = 0usize as *const Vertex;
                        let member = (&(&*dummy).0.y) as *const _;
                        member as usize
                    },

                    ty: ty,
                    array_size: array_size,
                })
            },
            "z" => {
                let (ty, array_size) = unsafe {
                    fn f<S: VertexMember>(_: &S) -> (VertexMemberTy, usize) { S::format() }
                    let dummy = 0usize as *const Vertex;
                    f(&(&*dummy).0.z)
                };

                Some(VertexMemberInfo {
                    offset: unsafe {
                        let dummy = 0usize as *const Vertex;
                        let member = (&(&*dummy).0.z) as *const _;
                        member as usize
                    },

                    ty: ty,
                    array_size: array_size,
                })
            },
            _ => None,
        }
    }
}

unsafe impl ::vulkano::pipeline::vertex::Vertex for Normal {
    fn member(name: &str) -> Option<::vulkano::pipeline::vertex::VertexMemberInfo> {
        use ::vulkano::format::Format;
        use ::vulkano::pipeline::vertex::VertexMemberInfo;
        use ::vulkano::pipeline::vertex::VertexMemberTy;
        use ::vulkano::pipeline::vertex::VertexMember;

        match name {
            "x" => {
                let (ty, array_size) = unsafe {
                    fn f<S: VertexMember>(_: &S) -> (VertexMemberTy, usize) { S::format() }
                    let dummy = 0usize as *const Normal;
                    f(&(&*dummy).0.x)
                };

                Some(VertexMemberInfo {
                    offset: unsafe {
                        let dummy = 0usize as *const Normal;
                        let member = (&(&*dummy).0.x) as *const _;
                        member as usize
                    },

                    ty: ty,
                    array_size: array_size,
                })
            },
            "y" => {
                let (ty, array_size) = unsafe {
                    fn f<S: VertexMember>(_: &S) -> (VertexMemberTy, usize) { S::format() }
                    let dummy = 0usize as *const Normal;
                    f(&(&*dummy).0.y)
                };

                Some(VertexMemberInfo {
                    offset: unsafe {
                        let dummy = 0usize as *const Normal;
                        let member = (&(&*dummy).0.y) as *const _;
                        member as usize
                    },

                    ty: ty,
                    array_size: array_size,
                })
            },
            "z" => {
                let (ty, array_size) = unsafe {
                    fn f<S: VertexMember>(_: &S) -> (VertexMemberTy, usize) { S::format() }
                    let dummy = 0usize as *const Normal;
                    f(&(&*dummy).0.z)
                };

                Some(VertexMemberInfo {
                    offset: unsafe {
                        let dummy = 0usize as *const Normal;
                        let member = (&(&*dummy).0.z) as *const _;
                        member as usize
                    },

                    ty: ty,
                    array_size: array_size,
                })
            },
            _ => None,
        }
    }
}
