use glob::{glob_with, GlobError, MatchOptions};
use lofty::{Accessor, Probe, TaggedFileExt};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufReader, process::{exit, Command}, env, u8, time::SystemTime};

#[derive(Debug, Serialize, Deserialize)]
struct VecColor {
    color: Rgb,
    file: String,
    tags: TagStruc,
}

#[derive(Debug, Serialize, Deserialize)]
struct Rgb {
    r: u8, g: u8, b: u8,
}

struct Hsv {
    h: f32, s: f32, v: f32,
}

#[derive(Debug)]
struct CoverFile {
    file: String,
    tags: TagStruc,
}

impl CoverFile {
    fn new(file: String) -> Self {
        CoverFile {
            file,
            tags: Default::default(),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
struct TagStruc {
    album: String,
    artist: String,
    date: String,
    genres: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AppConfig {
    folder: String,
    height: u32,
}

// Config defaults
impl Default for AppConfig {
    fn default() -> Self {
        AppConfig { 
            folder: ("~/music").to_string(),
            height: 2160
        }
    }
}

fn print_docs() {
    eprintln!("Album Collage Generator\n\n");
    eprintln!("Usage: albumgallery [flags] [options]\n\n");
    eprintln!("Flags");
    eprintln!("    -h or --help\n");
    eprintln!("Options");
    eprintln!("    -g or --genres  <String> ex. \"Rock;Jazz;Dubstep\"");
    eprintln!("    -a or --artist  <String> ex. \"Green Day\" ");
    eprintln!("    -y or --year    <u8>     ex. 2012");
    eprintln!("    -d or --decade  <u8>     ex. 2010");
    eprintln!("    -s or --asc     <rgb|year|lum|step>");
    eprintln!("    -S or --desc    <rgb|year|lum|step>");
}

fn get_args() -> Vec<String> {
    let args: Vec<String> = env::args().collect();
    /*
    *
    *   0 = genre
    *   1 = artist
    *   2 = year
    *   3 = decade
    *   4 = asc
    *   5 = desc
    *
    */
    let mut args_key: Vec<String> = vec!("".to_string() ;6);

    // Print documentation
    if args.iter().any(|e| e == "-h") || args.iter().any(|e| e == "--help") {
        print_docs();
        exit(1);
    }

    // NOTE: Somehow allow non value flags
    for (key, arg) in args.iter().enumerate() {
        match arg.as_str() {
            "-g" | "--genres" => args_key[0] = get_arg_value(&args, key),
            "-a" | "--artist" => args_key[1] = get_arg_value(&args, key),
            "-y" | "--year" => args_key[2] = get_arg_value(&args, key),
            "-d" | "--decade" => args_key[3] = get_arg_value(&args, key),
            "-s" | "--asc" => args_key[4] = get_arg_value(&args, key),
            "-S" | "--desc" => args_key[5] = get_arg_value(&args, key),
            "-m" | "--mobile" => args_key[6] = "true".to_string(),
            _ => continue,
        }
    }

    args_key
}

fn get_arg_value(args: &Vec<String>, key: usize) -> String {
    // In case if the flag is at the end of the Vec
    if key == (args.len() - 1) {
        println!("{} is missing value", args[key]);
        exit(1);
    }

    // if I want to convert values into different types
    let value: String = args[key+1].to_string();

    // If the next key value is anything including a flag or empty then stop the program
    if value.is_empty() || value.contains('-') || value.contains("--") {
        println!("{} is missing value", args[key]);
        exit(1);
    }

    value
}

fn get_cover_list(folder: String) -> Result<Vec<CoverFile>, GlobError> {
    let mut v: Vec<CoverFile> = Vec::new();
    let home = env::var_os("HOME").unwrap().into_string().unwrap();
    let s_str = &home[..];

    let patterns = folder.replace('~', s_str) + "/**/**/cover.*";

    let globs = glob_with(&patterns, MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    }).unwrap();

    for entry in globs.flatten() {
        let filename = entry.display().to_string();
        let mut arr_file_dir = filename.split('/').collect::<Vec<&str>>();
        arr_file_dir.pop();
        let dir = arr_file_dir.join("/");
        let tags = get_music_tags(dir + "/*01*.*");

        let mut cover = CoverFile::new(filename);

        if !tags.is_empty() {
            cover.tags = TagStruc {
                artist: tags[0].artist.clone(),
                date: tags[0].date.clone(),
                album: tags[0].album.clone(),
                genres: tags[0].genres.clone(),
            };
        }

        v.push(cover);
    }

    Ok(v)
}

fn get_music_tags(folder: String) -> Vec<TagStruc> {
    let globs = glob_with(&folder, MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    }).unwrap();

    if let Some(entry) = globs.flatten().next() {
        let tagged_file = Probe::open(entry.display().to_string())
            .expect("ERROR: Bad Path")
            .read()
            .expect("ERROR: Can't read file");

        let tag = match tagged_file.primary_tag() {
            Some(primary_tag) => primary_tag,
            None => tagged_file.first_tag().expect("No Tags"),
        };

        let artist = tag.artist().unwrap_or_default().to_string();
        let album = tag.album().unwrap_or_default().to_string();
        let date = tag.year().unwrap_or_default().to_string();
        let genres = tag.genre().unwrap_or_default().to_string();

        return vec![TagStruc {
            artist,
            album,
            date,
            genres,
        }];
    }

    vec![]
}

const HIST_SIZE: usize = (1 << 24) as usize;

fn find_dominant_color(img_pixel_vec: &[u8]) -> Rgb {
    let mut histogram: Vec<i32> = (0..HIST_SIZE).map(|_| 0).collect();

    // Loop through each pixel and save each as an RGB
    let mut i = 0;
    let pixel_count = img_pixel_vec.len() / 3;
    while i < pixel_count {
        let pos = i * 3;

        // Convert 3 current bytes to one byte that can work with RGB
        let r = img_pixel_vec[pos] + 1;
        let g = img_pixel_vec[pos + 1] - 1;
        let b = img_pixel_vec[pos + 2] - 1;

        let hex: String = format!("{:#02x}{:#02x}{:#02x}", r, g, b).replace("0x", "");

        let z = i64::from_str_radix(&hex.to_string(), 16).unwrap();

        // let color_byte = (((r as i32) << 10) + ((g as i32) << 5) + b as i32) as usize;

        histogram[z as usize] += 1;

        i += 1;
    }

    // Count how many color per-pixel shows up
    let mut index: usize = 0;
    let mut count = 0;

    for hist in histogram.iter().copied() {
        if hist >= count {
            count = hist;
            index = histogram.iter().position(|x| x.eq(&hist)).unwrap();
        }
    }

    // Then converts everything back to RGB individually
    let r = (index / (256 * 256)) as u8;
    let g = ((index / 256) % 256) as u8;
    let b = (index % 256) as u8;

    Rgb { r, g, b }
}

fn open_json_file(dir: &str) -> Result<Vec<VecColor>,serde_json::Error> {
    let cover_data_file = File::open(dir).unwrap();
    let reader = BufReader::new(cover_data_file);
    let data = serde_json::from_reader(reader)?;

    Ok(data)
}

fn lum(r: f32, g: f32, b: f32) -> f32 {
    f32::sqrt(r * 0.241 + g * 0.691 + b * 0.068)
}

fn rgb_to_hsv(r: f32, g: f32, b: f32) -> Hsv {
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

    Hsv { h, s, v }
}

// WARNING: Might need to redo the step-sorting, the ordering seems to follow the artist order for
// the most part
fn sort_step_index(r: f32, g: f32, b: f32) -> i32 {
    let mut lum: f32 = lum(r, g, b);
    let hsv: Hsv = rgb_to_hsv(r, g, b);

    let h2 = hsv.h * 8.00;
    let mut v2 = hsv.v * 8.00;

    if h2 % 2.00 == 1.00 {
        v2 = 8.00 - v2;
        lum = 8.00 - lum;
    }

    (h2 + lum + v2) as i32
}

fn sort_rgb_index(r: f32, g: f32, b: f32) -> i32 {
    let hsv: Hsv = rgb_to_hsv(r, g, b);

    hsv.h as i32
}

fn select_sort (arr: &VecColor, arg: String) -> i32 {
    return match arg.as_str() {
        "rgb" => sort_rgb_index(arr.color.r as f32, arr.color.g as f32, arr.color.b as f32),
        "step" => sort_step_index(arr.color.r as f32, arr.color.g as f32, arr.color.b as f32),
        "year" => arr.tags.date.parse::<i32>().unwrap() as i32,
        "lum" => lum(arr.color.r as f32, arr.color.g as f32, arr.color.b as f32) as i32,
        _ => sort_rgb_index(arr.color.r as f32, arr.color.g as f32, arr.color.b as f32),
    };
}

fn cover_sort(arr: &mut Vec<VecColor>, args: Vec<String>) {
    for i in 0..arr.len() {
        for j in 0..arr.len() - i - 1 {
            let a = &arr[j];
            let b = &arr[j + 1];
            let sort_a: i32;
            let sort_b: i32;

            if !(args[5].is_empty()) {
                sort_a = select_sort(a, args[5].to_string());
                sort_b = select_sort(b, args[5].to_string());
                if sort_a < sort_b {
                    arr.swap(j + 1, j);
                }
            } else {
                if !(args[4].is_empty()) {
                    sort_a = select_sort(a, args[4].to_string());
                    sort_b = select_sort(b, args[4].to_string());
                } else {
                    sort_a = sort_step_index(a.color.r as f32, a.color.g as f32, a.color.b as f32);
                    sort_b = sort_step_index(b.color.r as f32, b.color.g as f32, b.color.b as f32);
                }

                if sort_b < sort_a {
                    arr.swap(j, j + 1);
                }
            }
        }
    }
}

struct GridTile {
    width: usize,
    height: usize
}

fn find_matching_tile(cover_len: usize) -> GridTile {
    let mut width = 1;
    let mut height = 2;

    while height > width {
        height = cover_len / width;

        width += 1;
    }

    GridTile{width, height}
}

fn find_matching_geometry(tile: &GridTile, height: u32) -> u32 {
    height / tile.height as u32
}

fn main() -> Result<(), confy::ConfyError> {
    // Load config file
    let cfg: AppConfig = confy::load("albumgallery", "config")?;
    let home = env::var_os("HOME").unwrap().into_string().unwrap();
    let args = get_args();

    let mut cover_data = open_json_file(&(home.clone() + "/.local/share/albumgallery/covers.json")).unwrap();
    
    // Current a list of file directories
    let files = get_cover_list(cfg.folder).unwrap();
    let mut cover_list: Vec<CoverFile> = vec![];

    for file in files {
        // find filename in cache
        // if exists then return information then go to next loop
        let filename = file.file;

        // Add tag filters here
        if !args[0].is_empty() {
            let genres: Vec<String> = file.tags.genres.split(';').map(|g| g.to_string()).collect();

            if !genres.contains(&args[0]) {
                continue;
            }
        }

        if !args[1].is_empty() && file.tags.artist != args[1] {
            continue;
        }

        if !args[2].is_empty() && file.tags.date != args[2] {
            continue;
        }

        if !args[3].is_empty() {
            let date: Vec<char> = file.tags.date.chars().collect();
            let arg_date: Vec<char> = args[3].chars().collect();

            if date.len() <= 1 {
                continue;
            }

            if !(date[0] == arg_date[0] && date[1] == arg_date[1] && date[2] == arg_date[2]) {
                continue;
            } 
        }

        cover_list.push(CoverFile {
            file: filename,
            tags: file.tags
        });
    }

    let mut colors: Vec<VecColor> = vec![];

    for cover in cover_list {
        // // Go through each image and find the dominant color on that image
        let vec_row: VecColor = match cover_data.iter().find(|x| x.file.eq(&cover.file.to_string())) {
            Some(e) => VecColor {
                file: e.file.clone(),
                tags: e.tags.clone(),
                color: Rgb{
                    r: e.color.r,
                    g: e.color.g,
                    b: e.color.b,
                }
            },
            None => {
                let img = image::open(&cover.file)
                    .unwrap()
                    .resize(328, 328, image::imageops::FilterType::Nearest);
                let img_rbg8 = img.into_bytes();
                let dominant = find_dominant_color(&img_rbg8);

                VecColor {
                    color: dominant,
                    tags: cover.tags,
                    file: cover.file.to_string(),
                } 
            }
        };

        colors.push(vec_row);
    }

    // // Sort each value by corresponding color hue
    cover_sort(&mut colors, args);

    let mut files: Vec<String> = vec![];
    let tile = find_matching_tile(colors.len());
    let geometry = find_matching_geometry(&tile, cfg.height);
    let mut command = Command::new("montage");
    
    command.args([
        "-tile", &format!("{}x{}", tile.width, tile.height), 
        "-background", "black", 
        "-size", &format!("{}x{}", cfg.height, cfg.height), 
        "-geometry", &format!("{}x{}+0+0", geometry, geometry)
    ]);

    // Add something to add anything new or old to the json file comparing between file data
    // and the data we get from the loop
    for color in colors {
        // Add Album and Artist name as label
        files.push(color.file.clone());

        let find_file_in_cache = cover_data.iter().find(|e| e.file == color.file);
        if find_file_in_cache.is_none() {
            cover_data.push(color);
            continue
        };
    }

    serde_json::to_writer(&File::create(&(home.clone() + "/.local/share/albumgallery/covers.json")).unwrap(), &cover_data).ok();

    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(elapsed) => {
            let secs = elapsed.as_secs().to_string();
            let collage_filename: String = format!("{}/picx/{}.jpg", home, secs);

            // Run montage command here
            let status = &command.args(&files)
                .arg(collage_filename)
                .output()
                .expect("Couldn't run command");

            if status.status.success() {
                println!("Collage has been generated");
            }
        },
        Err(e) => {
            println!("Error {e:?}")
        }
    }

    Ok(())
}
