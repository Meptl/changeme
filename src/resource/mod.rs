use ::core::{Vertex, Normal, Index};
use ::collada::document::ColladaDocument;

pub trait Resource {}

pub trait ModelData: Resource {
    fn vertices(&self) -> Box<Vec<Vertex>>;
    fn normals(&self) -> Box<Vec<Normal>>;
    fn indices(&self) -> Box<Vec<u16>>;
}

impl Resource for ColladaDocument {}

impl ModelData for ColladaDocument {
    fn vertices(&self) -> Box<Vec<Vertex>> {
        let obj_set = self.get_obj_set().expect("ObjectSet in Collada file not found.");

        // Map collada lib vertices to vulkano/our vertices
        // We have to collect these iterators because from_iter requires the ExactSizeIterator trait
        let vertex_buffer = obj_set.objects.iter()
                                           .flat_map(|obj| obj.vertices.iter())
                                           .map(|vert| ::cgmath::Vector3::new(vert.x as f32,
                                                                              vert.y as f32,
                                                                              vert.z as f32))
                                           .map(|vec3| Vertex(vec3))
                                           .collect::<Vec<Vertex>>();

        return Box::new(vertex_buffer);
    }
    fn normals(&self) -> Box<Vec<Normal>> {
        let obj_set = self.get_obj_set().expect("ObjectSet in Collada file not found.");

        // Map collada lib vertices to vulkano/our vertices
        // We have to collect these iterators because from_iter requires the ExactSizeIterator trait
        let normal_buffer = obj_set.objects.iter()
                                           .flat_map(|obj| obj.normals.iter())
                                           .map(|norm| ::cgmath::Vector3::new(norm.x as f32,
                                                                              norm.y as f32,
                                                                              norm.z as f32))
                                           .map(|vec3| Normal(vec3))
                                           .collect::<Vec<Normal>>();

        return Box::new(normal_buffer);
    }
    fn indices(&self) -> Box<Vec<u16>> {
        let obj_set = self.get_obj_set().expect("ObjectSet in Collada file not found.");

        // No iterators for tuples, so add to a new array.
        let mut index_buffer = Vec::new();
        for obj in obj_set.objects.iter() {
            for geo in obj.geometry.iter() {
                for &shape in geo.shapes.iter() {
                    match shape {
                        ::collada::Shape::Triangle(u0, u1, u2) => {
                            let (vertex_index, _, _) = u0;
                            index_buffer.push(vertex_index as u16);

                            let (vertex_index, _, _) = u1;
                            index_buffer.push(vertex_index as u16);

                            let (vertex_index, _, _) = u2;
                            index_buffer.push(vertex_index as u16);
                        },
                        _ => panic!("Non-triangle shape!"),
                    }
                }
            }
        }
        return Box::new(index_buffer);
    }
}
