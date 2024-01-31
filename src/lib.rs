use clap::{value_parser, Arg, ArgAction, Command};
use exif::{In, Tag};
use image::{imageops::FilterType, io::Reader, GenericImageView, Pixel, Rgba, RgbaImage};
use std::{error::Error, fs::File, io::BufReader, path::Path};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    padding: u32,
    size: u32,
}

pub fn get_args() -> MyResult<(Vec<String>, Config)> {
    let matches = Command::new("framy")
        .version("0.1.0")
        .author("Hikaru Takakura <takakurahikaru@gmail.com>")
        .about("add padding to image")
        .arg(
            Arg::new("file")
                .value_name("FILE")
                .help("Input file(s)")
                .default_value("-")
                .action(ArgAction::Append),
        )
        .arg(
            Arg::new("padding")
                .short('p')
                .long("padding")
                .value_name("PADDING")
                .help("Padding pixels")
                .default_value("32")
                .action(ArgAction::Set)
                .value_parser(value_parser!(u32)),
        )
        .arg(
            Arg::new("size")
                .short('s')
                .long("size")
                .value_name("SIZE")
                .help("Output size")
                .default_value("1920")
                .action(ArgAction::Set)
                .value_parser(value_parser!(u32)),
        )
        .get_matches();

    let filenames: Vec<String> = matches
        .get_many::<String>("file")
        .unwrap()
        .map(|s| s.to_string())
        .collect();
    let padding = *matches.get_one::<u32>("padding").unwrap();
    let size = *matches.get_one::<u32>("size").unwrap();

    Ok((filenames, Config { padding, size }))
}

fn process_img(img_path: &str, config: &Config) -> MyResult<()> {
    let mut img = Reader::open(img_path)
        .map_err(|e| format!("{}: {}", img_path, e))?
        .decode()?;

    let file = File::open(img_path)?;
    let mut bufreader = BufReader::new(&file);
    let exifreader = exif::Reader::new();
    let exif = exifreader.read_from_container(&mut bufreader)?;

    let max_size = config.size - config.padding * 2;
    img = img.resize(max_size, max_size, FilterType::Nearest);

    if let Some(orientation) = exif.get_field(Tag::Orientation, In::PRIMARY) {
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

    let img_name = img_path
        .split('/')
        .last()
        .unwrap()
        .split('.')
        .next()
        .unwrap();
    let output_path = format!("{}_framed.png", img_name);
    img.save(Path::new(output_path.as_str()))?;
    println!("{} done", img_path);

    Ok(())
}

pub fn run(img_paths: Vec<String>, config: Config) -> MyResult<()> {
    for img_path in img_paths {
        if img_path == "-" {
            let mut buf = String::new();
            std::io::stdin().read_line(&mut buf).ok();
            let paths: Vec<String> = buf
                .trim()
                .parse::<String>()
                .ok()
                .unwrap()
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();
            return run(paths, config);
        }
        process_img(&img_path, &config)?;
    }
    Ok(())
}
