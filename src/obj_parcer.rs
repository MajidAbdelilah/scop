use std::ops::Index;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
}
glium::implement_vertex!(Vertex, position, uv);

pub struct Obj {
    pub vertices: Vec<Vertex>,
    pub faces: Vec<[u32; 4]>,
    pub indices: Vec<u32>,
    pub file_name: String,
    pub texture: Vec<[u8; 3]>,

}

fn vec3_normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len > 0.0 {
        [v[0] / len, v[1] / len, v[2] / len]
    } else {
        v
    }
}

impl Obj {
    fn new() -> Self {
        Obj {
            vertices: Vec::new(),
            faces: Vec::new(),
            indices: Vec::new(),
            file_name: String::new(),
            texture: Vec::new(),
        }
    }

    pub fn read_file(path: &str, obj_type: &str, texture_path: &str) -> Result<Self, String> {
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
                            uv: [0.0; 2],
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

                if obj_type == "42"
                {
                    for vertex in &mut obj_instance.vertices {
                        let normalized_vertex = vec3_normalize(vertex.position);
                        if normalized_vertex[0] > normalized_vertex[1] && normalized_vertex[0] > normalized_vertex[2]
                        {
                            vertex.uv = [normalized_vertex[2], normalized_vertex[1]];
                        } 
                        if normalized_vertex[1] > normalized_vertex[0] && normalized_vertex[1] > normalized_vertex[2]
                        {
                            vertex.uv = [normalized_vertex[0], normalized_vertex[2]];
                        } 
                        if normalized_vertex[2] > normalized_vertex[0] && normalized_vertex[2] > normalized_vertex[1]
                        {
                            vertex.uv = [normalized_vertex[0], normalized_vertex[1]];
                        } 
                        
                    }
                }

                if !texture_path.ends_with(".ppm")
                {
                   return Err("texture must be of ppm format".to_string());
                }

                let content = std::fs::read(texture_path);
                
                match content {
                    Ok(file) => {
                        let a_occure: u32 = 0;
                        let i: usize = 0;
                        let last_a_index: usize = 0;
                        for byte in &file {
                            if *byte == 10
                            {
                                if a_occure == 0 && file[last_a_index..i] != [80, 54]
                                {
                                    return Err("malformed ppm".to_string());
                                }

                                if a_occure == 1
                                {
                                    let w_h_str = &file[last_a_index..i];
                                    let width:u32;
                                    let height:u32;
                                    
                                    match std::str::from_utf8(w_h_str) {
                                        Ok(s)=>{
                                            let parts = s.split_whitespace();
                                            let j = 0;
                                            for part in parts
                                            {

                                                j++;
                                            }
                                        }
                                        Err(e)=>{
                                            return Err("malformed ppm".to_string());
                                        }
                                    }
                                }
                                a_occure += 1;
                                last_a_index = i;
                            }
                            i += 1;
                        }
                        
                         
                        match index {
                            Some(index) => {
                                

                            }
                            None => {
                                println!("malformed ppm");
                                return Err("malformed ppm".to_string());
                            }
                        }
                    }
                    Err(e) => {
                        println!("error while reading texture: {:?}", e);
                        return Err("error reading texture from filesystem".to_string());
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
