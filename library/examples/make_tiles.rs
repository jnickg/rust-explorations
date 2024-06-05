use clap::Parser;
use jnickg_imaging::ipr::*;

#[derive(Parser, Debug)]
#[command(
    version,
    about,
    long_about = "Break an iamge into tiles of a given size"
)]
struct Args {
    /// Path to an image for which to compute a pyramid
    #[arg(long, value_name = "STR")]
    input: String,

    /// Path to a directory where result files will be saved
    #[arg(long, value_name = "STR")]
    output: String,

    /// The (max) height of the tiles
    #[arg(long, value_name = "INT")]
    tile_height: u32,

    /// The (max) width of the tiles
    #[arg(long, value_name = "INT")]
    tile_width: u32,
}

fn main() {
    let args = Args::parse();
    let image = match image::open(&args.input) {
        Ok(image) => image,
        Err(_e) => {
            eprintln!("Error opening image: {}", _e);
            return;
        }
    };
    let image_extension = match std::path::Path::new(&args.input).extension() {
        Some(ext) => ext.to_str().unwrap(),
        None => {
            eprintln!("Error getting image extension");
            return;
        }
    };
    let ipr = IprImage(&image);
    let tiles = match ipr.make_tiles(args.tile_width, args.tile_height) {
        Ok(tiles) => tiles,
        Err(_e) => {
            eprintln!("Error making tiles: {}", _e);
            return;
        }
    };

    for (t_idx, tile) in tiles.tiles.iter().enumerate() {
        let tile_x = (t_idx % tiles.count_across as usize) * args.tile_width as usize;
        let tile_y = (t_idx / tiles.count_across as usize) * args.tile_height as usize;
        let filename = std::path::Path::new(&args.output).join(format!(
            "Tile-{}_X{}_Y{}.{}",
            t_idx, tile_x, tile_y, image_extension
        ));
        match tile.save(&filename) {
            Ok(_) => println!("Saved tile to {}", filename.to_str().unwrap()),
            Err(_e) => eprintln!("Error saving tile: {}", _e),
        }
    }
}
