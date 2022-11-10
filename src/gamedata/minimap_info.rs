use image::DynamicImage;


#[derive(Debug,Copy,Clone)]
pub struct MapInfo {
    pub pos_x: f32,
    pub pos_y: f32,
    pub scale: f32,

}

pub fn load_map_info(map_name: String) -> std::result::Result<MapInfo, Box<dyn std::error::Error>> {
    let file_contents = std::fs::read_to_string(format!("./assets/maps/{}.txt", map_name))?;
    let parsed = keyvalues_parser::Vdf::parse(file_contents.as_str())?;
    //let name = parsed.value.get_obj().unwrap().keys().next().unwrap();
    let kv = parsed.value.get_obj().unwrap();

    Ok(MapInfo {
        pos_x: kv.get("pos_x").unwrap().first().unwrap().get_str().unwrap().parse::<f32>()?,
        pos_y: kv.get("pos_y").unwrap().first().unwrap().get_str().unwrap().parse::<f32>()?,
        scale: kv.get("scale").unwrap().first().unwrap().get_str().unwrap().parse::<f32>()?,
    })
}

pub fn load_map_image(map_name: String) -> std::result::Result<DynamicImage, Box<dyn std::error::Error>> {
    Ok(image::open(format!("./assets/maps/{}_radar.png", map_name))?)
}