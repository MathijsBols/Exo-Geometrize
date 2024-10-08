use exolvl::{Colour, Exolvl, Object, ObjectProperty, Read as _, Vec2, Write as _};
use flate2::{write::GzEncoder, Compression};
use std::{
    error::Error, io::{BufReader, Write}, path::PathBuf
};
use image::Rgba;
use std::fs::File;
use std::io::Result as IoResult;
//use serde_json::Value;
use std::path::Path;
use uuid::Uuid;
use serde::Deserialize;
use serde_json;
use rfd::FileDialog;
//use std::path::PathBuf;
const LEVELFILE: &[u8; 432] = include_bytes!("default.exolvl");

#[derive(Debug, Deserialize)]
pub struct Shape {
    #[serde(rename = "type")]
    shape_type: u16,
    data: Vec<i32>,
    color: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub struct Shapes {
    shapes: Vec<Shape>,
}




fn main() -> IoResult<()> {
    //file dialog window
    let files = FileDialog::new()
    .add_filter("text", &["Json file", "json"])
    .set_directory("/")
    .pick_file();
    // initailizing varibles
    let mut result: Result<Vec<u8>, _> = Ok(vec![1, 2, 3]);//random 
    // converting
    match files {
        Some(path_buf) => {
            let path: PathBuf = path_buf;
            let path: &Path = path.as_path();  // Convert PathBuf to &Path
            // open the file
            let file = File::open(path)?;
            // read the file
            let reader = BufReader::new(file);
            // import into shape_collection
            let shapes_collection: Shapes = serde_json::from_reader(reader)?;
            // convert
            result = convert(&shapes_collection, "Converted level by Exo-Geometrize");
            // set end file path
        } 
        None => println!("No file selected."),  // Handle if no file is selected
    }
    

    // write data to final file
    match result {
        Ok(data) => {
            // save file dialog
            if let Some(file_path) = rfd::FileDialog::new()
                .set_title("Save File")
                .set_directory("/")
                .set_file_name("converted_level.exolvl")
                .save_file() 
            {
                // write data
                let mut file = File::create(&file_path)?;
                file.write_all(&data)?;
                println!("Conversion successful!");
                println!("Wrote file to: {}", file_path.display());
            } else {
                println!("Save file dialog was canceled.");
            }
        }
        Err(error) => {
            eprintln!("Conversion failed: {}", error);
        }
    }

    Ok(())
}
// send convert reqeust to: fn convert_inner
pub fn convert(
    shapes_collection: &Shapes,
    level_name: &str,
) -> Result<Vec<u8>, String> {
    convert_inner(shapes_collection, level_name) 
        .map_err(|e| e.to_string())
}
// convert
fn convert_inner(
    shapes_collection: &Shapes,
    level_name: &str,
) -> Result<Vec<u8>, Box<dyn Error>> {
    // read the base level file
    let mut level = read_level()?;
    // process the incoming data 
    process_image(&mut level, shapes_collection)?;
    // set the level propeties
    update_level_properties(&mut level, level_name);

    set_theme(&mut level);
    // write the final level
    write_level(&level)
}

fn read_level() -> Result<Exolvl, Box<dyn Error>> {
    Ok(Exolvl::read(&mut BufReader::new(&LEVELFILE[..]))?)
}


fn process_image(
    level: &mut Exolvl,
    shapes_collection: &Shapes,
) -> Result<(), Box<dyn Error>> {


    let layer = level
        .level_data
        .layers
        .get_mut(0)
        .ok_or("level file doesn't have any layers")?;

    for (entity_id, shape) in shapes_collection.shapes.iter().enumerate() {
        let entity_id = entity_id.try_into()?;
        let x_coordinate = shape.data.get(0).ok_or("Missing x-coordinate")? as &i32;
        let y_coordinate = shape.data.get(1).ok_or("Missing y-coordinate")? as &i32;
        let x_coordinate2 = shape.data.get(2).ok_or("Missing x_coordinate2")? as &i32;
        let y_coordinate2 = shape.data.get(3).unwrap_or(&0) as &i32;
        
        let mut tile_id: i32 = 113491821;
        let mut rotation: f32 = 0.0;
        let mut circle: bool = false;
        let mut scale = Vec2 {
            x: (*x_coordinate2 as f32 - *x_coordinate as f32).abs(),
            y: (*y_coordinate2 as f32 - *y_coordinate as f32).abs(),
        };
        let mut position = Vec2 {
            x: (*x_coordinate as f32 + *x_coordinate2 as f32) / 2.0,
            y: (*y_coordinate as f32 + *y_coordinate2 as f32) / 2.0,
        };
        let color = shape.color.as_slice();
        if color.len() != 4 {
            return Err("Color array must have 4 elements".into());
        }
        let rgba_color = Rgba([
            color[0], // Red
            color[1], // Green
            color[2], // Blue
            color[3], // Alpha
        ]); 
        let pixel: Rgba<u8> = rgba_color;

        if shape.shape_type == 1 {
        } 
        else if shape.shape_type == 2 {
            let rotation_object = shape.data.get(4).ok_or("Missing rotation")?;
            rotation = *rotation_object as f32;
        } else if shape.shape_type == 8 {
            circle = true;
            tile_id = -284493993;
            position = Vec2 {
                x: (*x_coordinate as f32),
                y: (*y_coordinate as f32),
            };
            scale = Vec2 {
                x: (*x_coordinate2 as f32) * 2.0,
                y: (*y_coordinate2 as f32) * 2.0, 
            };
        } else if shape.shape_type == 16 {
            circle = true;
            tile_id = -284493993;
            position = Vec2 {
                x: (*x_coordinate as f32),
                y: (*y_coordinate as f32),
            };
            scale = Vec2 {
                x: (*x_coordinate2 as f32) * 2.0,
                y: (*y_coordinate2 as f32) * 2.0, 
            };
            let rotation_object = shape.data.get(4).ok_or("Missing rotation")?;
            rotation = *rotation_object as f32;
        }

        else if shape.shape_type == 32 {
            circle = true;
            tile_id = -284493993;
            position = Vec2 {
                x: (*x_coordinate as f32),
                y: (*y_coordinate as f32),
            };
            scale = Vec2 {
                x: (*x_coordinate2 as f32) * 2.0,
                y: (*x_coordinate2 as f32) * 2.0, 
            };

        } 
        else {
            println!("Unsupported shape {}", shape.shape_type);
            continue;
        }
        

        

        

        


        let obj = get_object(entity_id, tile_id, position, scale, rotation, pixel, circle);

        level.level_data.objects.push(obj);

        layer.children.push(entity_id);
    }

    Ok(())
}





fn get_object(entity_id: i32, tile_id: i32, position: Vec2, scale: Vec2, rotation: f32, pixel: Rgba<u8>, circle: bool) -> Object {
    let mut properties = vec![
        ObjectProperty::Colour(Colour {
            r: pixel.0[0] as f32 / 255.,
            g: pixel.0[1] as f32 / 255.,
            b: pixel.0[2] as f32 / 255.,
            a: pixel.0[3] as f32 / 255.,
        }),
    ];

    if circle {
        properties.push(ObjectProperty::Resolution(64));
        properties.push(ObjectProperty::TotalAngle(360.0));
    }
    Object {
        entity_id,
        tile_id,
        prefab_entity_id: 0,
        prefab_id: 0,
        position,
        scale,
        rotation,
        tag: String::new(),
        properties,
        in_layer: 1,
        in_group: 0,
        group_members: vec![],
    }
}




fn set_theme(level: &mut Exolvl) {
    level.level_data.theme = "custom".to_string();

    level.level_data.custom_terrain_colour = Colour {
        r: 1.,
        g: 1.,
        b: 1.,
        a: 1.,
    };

    level.level_data.custom_terrain_border_colour = Colour {
        r: 1.,
        g: 1.,
        b: 1.,
        a: 1.,
    };

    level.level_data.custom_background_colour = Colour {
        r: 0.,
        g: 0.,
        b: 0.,
        a: 1.,
    };
}

fn update_level_properties(level: &mut Exolvl, level_name: &str) {
    let created_time = chrono::Utc::now();

    level.local_level.level_id = Uuid::new_v4().to_string();
    level.local_level.level_name = level_name.to_string();
    level.local_level.creation_date = created_time;
    level.local_level.update_date = created_time;
}

fn write_level(level: &Exolvl) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut output = Vec::new();

    level.write(&mut output)?;

    let mut e = GzEncoder::new(Vec::new(), Compression::default());

    e.write_all(&output)?;

    Ok(e.finish()?)
}
