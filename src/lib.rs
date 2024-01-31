use clap::{value_parser, Arg, ArgAction, Command, ValueHint};
use exif::{In, Tag};
use image::{
    imageops::FilterType, io::Reader, GenericImageView, ImageFormat, Pixel, Rgba, RgbaImage,
};
use std::{
    error::Error,
    fs,
    io::{BufReader, Read},
    path::Path,
};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    padding: u32,
    size: u32,
    outdir: String,
    format: ImageFormat,
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
                .action(ArgAction::Append)
                .value_hint(ValueHint::AnyPath),
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
        .arg(
            Arg::new("outdir")
                .short('o')
                .long("outdir")
                .value_name("OUTDIR")
                .help("Output directory")
                .default_value(".")
                .action(ArgAction::Set)
                .value_hint(ValueHint::DirPath),
        )
        .arg(
            Arg::new("format")
                .short('f')
                .long("format")
                .value_name("FORMAT")
                .help("Output format (jpeg, png, gif, webp, tiff)")
                .default_value("jpeg")
                .action(ArgAction::Set),
        )
        .get_matches();

    let filenames: Vec<String> = matches
        .get_many::<String>("file")
        .unwrap()
        .map(|s| s.to_string())
        .collect();
    let padding = *matches.get_one::<u32>("padding").unwrap();
    let size = *matches.get_one::<u32>("size").unwrap();
    let mut outdir = matches.get_one::<String>("outdir").unwrap().to_string();
    if !outdir.ends_with('/') {
        outdir.push('/');
    }
    let format = match matches
        .get_one::<String>("format")
        .unwrap()
        .to_lowercase()
        .as_str()
    {
        "png" => ImageFormat::Png,
        "jpg" => ImageFormat::Jpeg, // "jpg" is for backward compatibility
        "jpeg" => ImageFormat::Jpeg,
        "gif" => ImageFormat::Gif,
        "webp" => ImageFormat::WebP,
        "tiff" => ImageFormat::Tiff,
        unknown => {
            eprintln!("Unknown format {}. Use png instead.", unknown);
            ImageFormat::Png
        }
    };

    Ok((
        filenames,
        Config {
            padding,
            size,
            outdir,
            format,
        },
    ))
}

fn process_img(img_path: &str, config: &Config) -> MyResult<()> {
    let mut img = Reader::open(img_path)
        .map_err(|e| format!("{}: {}", img_path, e))?
        .decode()?;

    let file = fs::File::open(img_path)?;
    let mut bufreader = BufReader::new(&file);
    let exifreader = exif::Reader::new();
    let exif = exifreader.read_from_container(&mut bufreader)?;

    let max_size = config.size - config.padding * 2;
    img = img.resize(max_size, max_size, FilterType::Lanczos3);

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
    let output_name = format!("{}_framed.png", img_name);
    let output_path = format!("{}{}", config.outdir, output_name);

    fs::create_dir_all(&config.outdir)?;

    img.save_with_format(Path::new(output_path.as_str()), config.format)?;
    println!("{} done", img_path);

    Ok(())
}

pub fn run(img_paths: Vec<String>, config: Config) -> MyResult<()> {
    for img_path in img_paths {
        if img_path == "-" {
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf).ok();
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
