use exif::{In, Tag};
use image::{imageops::FilterType, io::Reader, GenericImageView, Pixel, Rgba, RgbaImage};
use std::{error::Error, path::Path};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    img_path: String,
    padding: u32,
    size: u32,
}

pub fn get_args() -> MyResult<Config> {
    Ok(Config {
        img_path: String::from("./input.jpg"),
        padding: 32,
        size: 1920,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let mut img = Reader::open(&config.img_path)?.decode()?;

    let file = std::fs::File::open(&config.img_path)?;
    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = exif::Reader::new();
    let exif = exifreader.read_from_container(&mut bufreader)?;

    let max_size = config.size - config.padding * 2;
    img = img.resize(max_size, max_size, FilterType::Nearest);

    if let Some(orientation) = exif.get_field(Tag::Orientation, In::PRIMARY) {
        println!("orientation: {}", orientation.value.get_uint(0).unwrap());
        img = match orientation.value.get_uint(0) {
            Some(3) => img.rotate180(),
            Some(6) => img.rotate90(),
            Some(8) => img.rotate270(),
            _ => img,
        }
    };

    let (w, h) = img.dimensions();

    let (padding_x, padding_y) = if w > h {
        (config.padding, (config.size - h) / 2)
    } else {
        ((config.size - w) / 2, config.padding)
    };

    let img = RgbaImage::from_fn(config.size, config.size, |x, y| {
        if x < padding_x || x >= padding_x + w || y < padding_y || y >= padding_y + h {
            Rgba::from([255, 255, 255, 255])
        } else {
            img.get_pixel(x - padding_x, y - padding_y).to_rgba()
        }
    });

    img.save(Path::new("./output.jpg"))?;

    Ok(())
}
