
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
    pub texture: Vec<u8>,
    pub texture_width: u32,
    pub texture_height: u32,
    pub bb: [[f32; 3]; 2],
}

pub fn vec3_normalize(v: [f32; 3]) -> [f32; 3] {
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
            texture_height: 0,
            texture_width: 0,
            bb: [[0.0, 0.0, 0.0], [0.0, 0.0, 0.0]],
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

                let mut simple_parse_func =
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
                                                if index == 0
                                                {
                                                    if obj_instance.bb[0][0] > v
                                                    {
                                                        obj_instance.bb[0][0] = v;
                                                    }
                                                    if obj_instance.bb[1][0] < v
                                                    {
                                                        obj_instance.bb[1][0] = v;
                                                    }
                                                }
                                                if index == 1
                                                {
                                                    if obj_instance.bb[0][1] > v
                                                    {
                                                        obj_instance.bb[0][1] = v;
                                                    }
                                                    if obj_instance.bb[1][1] < v
                                                    {
                                                        obj_instance.bb[1][1] = v;
                                                    }
                                                }
                                                if index == 2
                                                {
                                                    if obj_instance.bb[0][2] > v
                                                    {
                                                        obj_instance.bb[0][2] = v;
                                                    }
                                                    if obj_instance.bb[1][2] < v
                                                    {
                                                        obj_instance.bb[1][2] = v;
                                                    }
                                                }
                                                
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
                } else if obj_type == "sphere"
                {
                    for vertex in &mut obj_instance.vertices {
                        let x = vertex.position[0] - (obj_instance.bb[0][0] + obj_instance.bb[1][0])/2.0;
                        let y = vertex.position[1] - (obj_instance.bb[0][1] + obj_instance.bb[1][1])/2.0;
                        let z = vertex.position[2] - (obj_instance.bb[0][2] + obj_instance.bb[1][2])/2.0;
                        let y_clamp = y.clamp(-1.0, 1.0);
                        
                        vertex.uv = [(z.atan2(x) / (2.0*std::f32::consts::PI) + 0.5), y_clamp.asin() / std::f32::consts::PI + 0.5];
                    }
                }

                if !texture_path.ends_with(".ppm")
                {
                   return Err("texture must be of ppm format".to_string());
                }

                let content = std::fs::read(texture_path);
                
                match content {
                    Ok(file) => {
                        let mut a_occure: u32 = 0;
                        let mut last_a_index: usize = 0;
                        let mut width:u32 = 0;
                        let mut height:u32 = 0;
                        let mut reserve_lock = false;
                        let mut byte_index:usize = 0;
                        let mut byte_index_start:usize = 0;
                        
                        for i in 0..file.len() {
                            if file[i] == 10 && a_occure != 3
                            {
                                if a_occure == 0 && file[last_a_index..i] != [80, 54]
                                {
                                    
                                    return Err("malformed ppm 0".to_string());
                                }

                                if a_occure == 1
                                {
                                    let w_h_str = &file[last_a_index..i];
                                    
                                    
                                    match std::str::from_utf8(w_h_str) {
                                        Ok(s)=>{
                                            println!("s 1: {:?}", s);
                                            let parts = s.split_whitespace();
                                            let mut j = 0;
                                            for part in parts
                                            {
                                                let p = part.parse::<u32>();
                                                match p {
                                                    Ok(p) =>{
                                                        if j == 0{
                                                            width = p;
                                                        }
                                                        if j == 1{
                                                            height = p;
                                                        }
                                                    }
                                                    Err(e) => {
                                                        println!("malformed ppm image 1: {:?}", e);
                                                        return Err("malformed ppm 1".to_string());
                                                    }
                                                }
                                                j += 1;
                                            }
                                        }
                                        Err(e)=>{
                                            println!("malformed ppm image 1: {:?}", e);
                                            return Err("malformed ppm 1".to_string());
                                        }
                                    }
                                }
                                if a_occure == 2
                                {
                                    let max_value = &file[last_a_index..i];
                                    match std::str::from_utf8(max_value) {
                                        Ok(s) => {
                                            println!("s 2: {:?}", s);
                                            let val = s.parse::<u16>();
                                            match val {
                                                Ok(val) => {
                                                    if val == 0 || val > 255
                                                    {
                                                        println!("malformed ppm image 2");
                                                        return Err("malformed ppm 2".to_string());
                                                    }
                                                }
                                                Err(e) => {
                                                    println!("malformed ppm image 2: {:?}", e);
                                                    return Err("malformed ppm 2".to_string());
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            println!("malformed ppm image 2: {:?}", e);
                                            return Err("malformed ppm 2".to_string());
                                        }
                                    }
                                }
                                a_occure += 1;
                                last_a_index = i + 1;
                            }
                            
                            if a_occure == 3  
                            {
                                if !reserve_lock
                                {
                                    obj_instance.texture.reserve(width as usize * height as usize);
                                    reserve_lock = true;
                                    byte_index = i + 1;
                                    byte_index_start = byte_index;
                                }

                                obj_instance.texture.push(file[byte_index]);
                                byte_index+=1;
                                if(byte_index - byte_index_start) >= (width as usize * height as usize * 3)
                                {
                                    println!("successfully read the texture buffer, read: {:?}, width * height * 3: {:?}", 
                                    (byte_index - byte_index_start), 
                                    (width as usize * height as usize * 3));
                                    obj_instance.texture.shrink_to_fit();
                                    obj_instance.texture_width = width;
                                    obj_instance.texture_height = height;
                                    
                                    break;
                                }
                                // obj_instance.texture.push(value);
                            }
                        }
                    }
                    Err(e) => {
                        println!("malformed ppm image: {:?}", e);
                        return Err("malformed ppm".to_string());
                    
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
