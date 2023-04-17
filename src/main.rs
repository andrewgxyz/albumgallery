use clap::Parser;
use glob::{glob_with, MatchOptions, GlobError};
use lofty::{TaggedFileExt, Accessor, Probe};
use serde::{Serialize, Deserialize};
use std::{env, fs::File, io::prelude::*};

#[derive(Serialize, Deserialize)]
struct AppConfig {
    folder: String
}

#[derive(Serialize, Deserialize)]
struct VecColor {
    color: MyRgb,
    file: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MyRgb {
    r: u8,
    g: u8,
    b: u8
}

#[derive(Debug)]
struct Hsv {
    h: f32,
    s: f32,
    v: f32
}

struct CoverFile {
    file: String,
    tags: TagStruc,
}

struct TagStruc {
    album: Option<String>,
    artist: Option<String>,
    date: Option<String>,
    genres: Option<String>,
}

impl CoverFile {
    fn new (file: String) -> Self {
        CoverFile { file, tags: Default::default() }
    }
}

impl Default for TagStruc {
    fn default () -> TagStruc {
        TagStruc {
            album: Default::default(),
            artist: Default::default(),
            genres: Default::default(),
            date: Default::default(),
        }
    }
}

// enum ColorSort {
//     RGB(String),
//     HSV(String),
//     LUM(String),
//     STEP(String),
// }

// TODO: Add flags here
//      * -g / --genres  String: Get covers by set genres, each will be seperated by semi-colon
//      * -a / --artists String: Get covers by selected artist
//      * -y / --year    String: Get covers by publish year
//      * -d / --decade  String: Collect the covers related to the decade
//      * -m / --month   String: Get covers by publish month irrespective of year if non is set
//      * -s / --asc     String: Sorting in ascending
//          * "rgb":  RGB sorting
//          * "hsv":  hsv sorting
//          * "lum":  Luminosity sorting
//          * "step": Step sorting
//      * -S / --des     String: Sorting in descending
//          * "rgb":  RGB sorting
//          * "hsv":  hsv sorting
//          * "lum":  Luminosity sorting
//          * "step": Step sorting

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // Get list if covers by specified genres 
    #[arg(short = 'g', long)]
    genres: Option<String>,

    // Get list if covers by specified genres 
    #[arg(short = 'y', long)]
    year: Option<u16>,

    // Get list if covers by specified genres 
    #[arg(short = 'm', long)]
    month: Option<u8>,

    // Get list if covers by specified genres 
    #[arg(short = 'a', long)]
    artist: Option<String>,

    // Sort albums in ascending order of color 
    #[arg(short = 's', long, value_name="rgb|hsv|lum|step", default_value="rgb")]
    asc: Option<String>,

    // Sort albums in Descending order of color 
    #[arg(short = 'S', long, value_name="rgb|hsv|lum|step")]
    desc: Option<String>,
}

// Config defaults
impl Default for AppConfig {
    fn default() -> Self {
        let home = match env::var_os("HOME") {
            Some(v) => v.into_string().unwrap(),
            None => panic!("$USER is not set")
        };

        let folder = home + "music";

        AppConfig {
            folder
        }
    }
}

fn lum (r: f32, g: f32, b: f32) -> f32 {
    r * 0.241 + g * 0.691 + b * 0.068
}

fn rgb_to_hsv (r: f32, g: f32, b: f32) -> Hsv {
    // let hsv: Hsv = Hsv::from_color(Rgb::new(r, g, b));
    let cmax = f32::max(r, f32::max(g, b));
    let cmin = f32::min(r, f32::min(g, b));
    let diff: f32 = cmax - cmin;
    let mut h = -1.00;
    let mut s = 0.00;

    if cmax == cmin {
        h = 0.00;
    } else if cmax == r {
        h = (60.00 * ((g - b) / diff) + 360.00) % 360.00;
    } else if cmax == g {
        h = (60.00 * ((b - r) / diff) + 120.00) % 360.00;
    } else if cmax == b {
        h = (60.00 * ((r - g) / diff) + 240.00) % 360.00;
    }

    if cmax != 0.00 {
        s = (diff / cmax) * 100.00;
    }

    let v = cmax * 100.00;

    return Hsv {
        h, s, v
    };
}

fn step_sort_index (r: f32, g: f32, b: f32) -> i32 {
    let mut lum: f32 = lum(r, g, b);
    let hsv: Hsv = rgb_to_hsv(r, g, b);

    lum = lum.sqrt();

    let h2 = hsv.h * 8.00;
    let mut v2 = hsv.v * 8.00;

    if h2 % 2.00 == 1.00 {
        v2 = 8.00 - v2;
        lum = 8.00 - lum;
    }

    return ((h2 * lum * v2) / 3.00) as i32;
}

fn get_list (folder: String) -> Result<Vec<CoverFile>, GlobError> {
    let mut v: Vec<CoverFile> = Vec::new();

    let patterns = "/home/andrew/music/**/**/cover.*";

    let options = MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false
    };

    let globs = glob_with(patterns, options).unwrap();

    for entry in globs {
        if let Ok(path) = entry {
            let filename = path.display().to_string();
            let mut arr_file_dir = filename.split('/').collect::<Vec<&str>>();
            arr_file_dir.pop();
            let dir = arr_file_dir.join("/");
            let tags = get_music_tags(dir + "/*01*.*");

            let mut cover = CoverFile::new(filename);

            if !tags.is_empty() {
                cover.tags = TagStruc {
                    artist: Some(tags[0].artist.clone().expect("No Artist")),
                    date: Some(tags[0].date.clone().expect("No Artist")),
                    album: Some(tags[0].album.clone().expect("No Artist")),
                    genres: Some(tags[0].genres.clone().expect("No Artist")),
                };
            }

            v.push(cover);
        }
    }

    Ok(v)
}

fn get_music_tags (folder: String) -> Vec<TagStruc> {
    let options = MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false
    };

    // let tag_item_list = vec![];

    let globs = glob_with(&folder, options).unwrap();

    for entry in globs {
        if let Ok(path) = entry {
            let tagged_file = Probe::open(path.display().to_string())
                .expect("ERROR: Bad Path")
                .read()
                .expect("ERROR: Can't read file");

            let tag = match tagged_file.primary_tag() {
                Some(primary_tag) => primary_tag,
                None => tagged_file.first_tag().expect("No Tags")
            };

            let artist = tag.artist().unwrap().to_string();
            let album = tag.album().unwrap().to_string();
            let date = tag.year().unwrap().to_string();
            let genres = tag.genre().unwrap().to_string();

            return vec![TagStruc {
                artist: Some(artist),
                album: Some(album),
                date: Some(date),
                genres: Some(genres)
            }];
        }
    }

    return vec![];
}

#[derive(Debug)]
struct HistogramItem {
    color: usize,
    count: i32
}

const HIST_SIZE: usize = (1 << 20) as usize;

fn find_dominant_color (img_pixel_vec: &[u8]) -> MyRgb {
    let mut histogram: Vec<i32> = (0..HIST_SIZE).map(|_| 0).collect();

    // Loop through each pixel and save each as an RGB
    let mut i = 0;
    let pixel_count = img_pixel_vec.len() / 3;
    while i < pixel_count {
        let pos = i * 3;

        // Convert 3 current bytes to one byte that can work with RGB
        let r = img_pixel_vec[pos + 0];
        let g = img_pixel_vec[pos + 1];
        let b = img_pixel_vec[pos + 2]; 

        let color_byte = ( ((r as i32) << (10))
            + ((g as i32) << 5)
            +   b as i32
        ) as usize;

        histogram[color_byte] += 1;

        i += 1;
    }

    let mut index: usize = 0;
    let mut count = 0;
    
    for hist in histogram.to_vec() {
        if hist > count {
            count = hist;
            index = histogram.iter().position(|x| x.eq(&hist)).unwrap();
        }
    }

    let z = i64::from_str_radix(&index.to_string(), 16).unwrap();
    let r = (z / (256*256)) as u8;
    let g = ((z / 256) % 256) as u8;
    let b = (z % 256) as u8;

    return MyRgb { r,g,b };
}

fn main () -> Result<(), confy::ConfyError> {
    // Load config file
    let cfg: AppConfig = confy::load("albumgallery", None)?;
    let args = Args::parse();
    let cover_data_file = File::create("~/.local/share/albumgallery/covers.json").unwrap();
     
    // TODO: Validate Arguments if available

    // Current a list of file directories
    let files = get_list(cfg.folder).unwrap();
    let mut cover_list: Vec<String> = vec![];

    for file in files {
        // find filename in cache 
        // if exists then return information then go to next loop
        let filename = file.file;
        cover_list.push(filename);

        // Add tag filters here
    }

    let mut colors: Vec<VecColor> = vec![];

    for cover in cover_list {
        // // Go through each image and find the dominant color on that image
        let img = image::open(&cover).unwrap().resize(256, 256, image::imageops::FilterType::Nearest);
        let img_rbg8 = img.into_bytes();
        let dominant = find_dominant_color(&img_rbg8);

        colors.push(VecColor { 
            color: dominant, 
            file: cover.to_string(),
        });
    }

    // // Sort each value by corresponding color hue
    colors.sort_by(|a, b| {
        let sort_a = step_sort_index(a.color.r as f32, a.color.g as f32, a.color.b as f32);
        let sort_b = step_sort_index(b.color.r as f32, b.color.g as f32, b.color.b as f32);
        sort_b.cmp(&sort_a)
    });

    for color in colors {
        println!("{:?}", color.file);
    }

    Ok(())
}

