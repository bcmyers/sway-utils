use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use failure::Error;
use structopt::StructOpt;

const EXTENSIONS: [&str; 3] = ["jpg", "jpeg", "png"];
const MAX_DEPTH: usize = 2;

#[derive(Clone, Debug, StructOpt)]
#[structopt(name = "background")]
/// Command line utility for setting the background in Sway.
pub struct Opt {
    /// Directory
    #[structopt(
        short = "d",
        long = "dir",
        parse(from_os_str),
        default_value = "/home/bcmyers/wallpaper"
    )]
    dir: PathBuf,

    /// Absolute path to an image file
    #[structopt(short = "p", long = "path", parse(from_os_str))]
    path: Option<PathBuf>,

    /// For use in sway config
    #[structopt(short = "s", long = "startup")]
    startup: bool,
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();
    set_background(&opt.dir, &opt.path, opt.startup)?;
    Ok(())
}

pub fn set_background(dir: &Path, path: &Option<PathBuf>, startup: bool) -> Result<(), Error> {
    let path = copy_image(dir, path)?;

    let image = match path {
        Some(path) => path,
        None => {
            let images = gather_images(dir, 0)?;
            if images.is_empty() {
                failure::bail!("No background images in folder.")
            }
            let choice = rand::random::<usize>() % images.len();
            (&images[choice]).clone()
        }
    };

    if startup {
        print!("{}", image.display());
    } else {
        let arg = format!("output \"*\" background {} fill", image.display());
        let output = Command::new("swaymsg").arg(arg).output()?;
        if !output.status.success() {
            let e = std::str::from_utf8(&output.stderr)?;
            failure::bail!("Sway error: {}", e);
        }
    }

    Ok(())
}

fn copy_image(dir: &Path, path: &Option<PathBuf>) -> Result<Option<PathBuf>, Error> {
    if let Some(ref path) = path {
        let outpath = dir.join(path.file_name().unwrap());
        if outpath.exists() {
            return Ok(Some(outpath));
        }
        let mut infile = fs::File::open(path)?;
        let mut outfile = fs::File::create(&outpath)?;
        io::copy(&mut infile, &mut outfile)?;
        return Ok(Some(outpath));
    }
    Ok(None)
}

fn gather_images<P>(dir: P, depth: usize) -> Result<Vec<PathBuf>, Error>
where
    P: AsRef<Path>,
{
    if depth > MAX_DEPTH {
        return Ok(Vec::new());
    }
    let dir = dir.as_ref();
    let mut images = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            let mut paths = gather_images(&entry.path(), depth + 1)?;
            images.append(&mut paths);
        } else if file_type.is_file() || file_type.is_symlink() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if let Some(ref ext) = ext.to_str() {
                    if (&EXTENSIONS).contains(ext) {
                        images.push(path.into());
                    }
                }
            }
        }
    }
    Ok(images)
}
