# AlbumGallery

albumgalley is a tiny program written with Rust that goes and generates a collage of album covers based on set parameters and filter.

## Installation

Some prerequisites, you need to install rust on your system and one external program that generates the collage image **ImageMagick**.

```bash
git clone https://github.com/andrewgxyz/albumgallery.git
cd albumgallery
cargo build --release && cp ./target/release/albumgallery ~/.local/bin
```


## Usage 

```
Album Collage Generator
Usage: albumgallery [flags] [options]
Flags");
    -h or --help
Options");
    -g or --genres  <String> ex. "Rock;Jazz;Dubstep"
    -a or --artist  <String> ex. "Green Day"
    -y or --year    <u8>     ex. 2012");
    -d or --decade  <u8>     ex. 2010");
    -s or --asc     <rgb|year|lum|step>");
    -S or --desc    <rgb|year|lum|step>");
```

The default config is mainly the folder location for your music collection and the height which default being the 4K height for 16:9. The config file is stored in your `.config` folder on Linux under the same name, this will be updated once I have a better idea of the Windows and MacOS environments

```
music_folder = "~/Music"
output_folder = "~/Pictues"
height = 2160
```

Also some **warnings**:

- I only accept the `<artist>/<album>/01-<title>.<extention>` for what the program ends up finding
- Similarly the cover has to be named `cover.<extention>` and has to be placed with the respective album
- The first run will be the longest to generate the collage depending on collection size, the next runs will be much quicker since I cache everything afterwards
- This program will likely not work on Windows/MacOS, mostly been focusing on Linux development, might need to take some time on that

