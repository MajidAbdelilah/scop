#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
}
glium::implement_vertex!(Vertex, position);
pub struct Obj {
    pub vertices: Vec<Vertex>,
    pub faces: Vec<[u32; 4]>,
    pub indices: Vec<u32>,
    pub file_name: String,
}

impl Obj {
    fn new() -> Self {
        Obj {
            vertices: Vec::new(),
            faces: Vec::new(),
            indices: Vec::new(),
            file_name: String::new(),
        }
    }

    pub fn read_file(path: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(path);
        match content {
            Ok(data) => {
                let mut obj_instance = Obj::new();
                obj_instance.file_name = String::from(path);
                let lines = data.lines();
                let mut vertex_parce_failed = false;

                let simple_parse_func =
                    |line: &str,
                     arr: Option<&mut Vec<Vertex>>,
                     arr_faces: Option<&mut Vec<[u32; 4]>>,
                     failed_bool: &mut bool| {
                        let parts = line[2..].split_ascii_whitespace();
                        let mut index = 0;
                        let mut vertex = Vertex {
                            position: [0.0; 3],
                        };
                        let mut vertex_face: [u32; 4] = [0; 4];
                        parts.for_each(|part| {
                            match &arr {
                                Some(_) => {
                                    let val = part.parse::<f32>();
                                    match val {
                                        Ok(v) => {
                                            if index < 3 {
                                                vertex.position[index] = v;
                                            }
                                            index += 1;
                                        }
                                        Err(e) => {
                                            println!("Failed to parse vertex value: {}", e);
                                            *failed_bool = true;
                                        }
                                    }
                                }
                                None => {}
                            }
                            match &arr_faces {
                                Some(_) => {
                                    let val = part.parse::<u32>();
                                    match val {
                                        Ok(v) => {
                                            if index < 4 {
                                                vertex_face[index] = v;
                                            }
                                            index += 1;
                                        }
                                        Err(e) => {
                                            println!("Failed to parse face value: {}", e);
                                            *failed_bool = true;
                                        }
                                    }
                                }
                                None => {}
                            }
                        });
                        if index == 3 {
                            match arr {
                                Some(v_arr) => {
                                    (v_arr).push(vertex);
                                }
                                None => {}
                            }
                            match arr_faces {
                                Some(f_arr) => {
                                    f_arr.push(vertex_face);
                                }
                                None => {}
                            }
                        } else if index == 4 {
                            match arr {
                                Some(_) => {
                                    println!(
                                        "incorrect vertex or face data: {}, index = {}",
                                        line, index
                                    );
                                    *failed_bool = true;
                                }
                                None => {}
                            }
                            match arr_faces {
                                Some(f_arr) => {
                                    f_arr.push(vertex_face);
                                }
                                None => {}
                            }
                        } else {
                            println!("incorrect vertex or face data: {}, index = {}", line, index);
                            *failed_bool = true;
                        }
                    };

                let mut face_parce_failed = false;
                obj_instance.vertices.reserve(data.lines().count());
                obj_instance.faces.reserve(data.lines().count());
                let mut vertices_count: u32 = 0;
                let mut faces_count: u32 = 0;
                lines.for_each(|line| {
                    if line.len() > 0 && &line[0..2] == "v " {
                        simple_parse_func(
                            line,
                            Some(&mut obj_instance.vertices),
                            None,
                            &mut vertex_parce_failed,
                        );
                        vertices_count += 1;
                    }
                    if line.len() > 0 && &line[0..2] == "f " {
                        simple_parse_func(
                            line,
                            None,
                            Some(&mut obj_instance.faces),
                            &mut face_parce_failed,
                        );
                        faces_count += 1;
                    }
                });
                println!("befor resize obj_instance.vertices.capacity() = {}", obj_instance.vertices.capacity());
                println!("befor resize obj_instance.faces.capacity() = {}", obj_instance.faces.capacity());
                obj_instance.vertices.shrink_to_fit();
                obj_instance.faces.shrink_to_fit();
                println!("after resize obj_instance.vertices.capacity() = {}", obj_instance.vertices.capacity());
                println!("after resize obj_instance.faces.capacity() = {}", obj_instance.faces.capacity());

                if vertex_parce_failed {
                    return Err("Failed to parse vertex data".to_string());
                }
                if face_parce_failed {
                    return Err("Failed to parse face data".to_string());
                }

                // indices
                for face in &obj_instance.faces {
                    
                    if face[3] == 0
                    {
                        for index in *face {
                            if index != 0  {
                                obj_instance.indices.push(index - 1);
                            }
                        }   
                    } else {
                        obj_instance.indices.push(face[0] - 1);
                        obj_instance.indices.push(face[1] - 1);
                        obj_instance.indices.push(face[2] - 1);

                        obj_instance.indices.push(face[0] - 1);
                        obj_instance.indices.push(face[2] - 1);
                        obj_instance.indices.push(face[3] - 1);
                    }
                }

                return Ok(obj_instance);
            }

            Err(e) => {
                println!("Failed to read file: {}", e);
                // let e = ;
                return Err(format!("Failed to read file: {}", path));
            }
        }
    }
}
