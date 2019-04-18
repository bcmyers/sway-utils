#[macro_use]
extern crate log;

use std::borrow::Cow;
use std::fs;
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

    /// Path (or glob) to image file(s)
    #[structopt(short = "p", long = "path")]
    path: Vec<String>,

    /// For use in sway config
    #[structopt(short = "s", long = "startup")]
    startup: bool,
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let opt = Opt::from_args();
    if let Err(e) = set_background(&opt.dir, &opt.path, opt.startup) {
        error!("{}", e);
        return Err(e);
    };
    Ok(())
}

pub fn set_background(dir: &Path, glob: &Vec<String>, startup: bool) -> Result<(), Error> {
    let paths = copy_images(dir, glob)?;

    let image: Cow<Path> = match paths.len() {
        0 => {
            let images = gather_images(dir, 0)?;
            if images.is_empty() {
                failure::bail!("No background images in folder.")
            }
            let choice = rand::random::<usize>() % images.len();
            (&images[choice]).clone().into()
        }
        1 => paths.get(0).unwrap().into(),
        n => {
            let choice = rand::random::<usize>() % n;
            paths.get(choice).unwrap().into()
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

fn copy_images(dir: &Path, glob: &Vec<String>) -> Result<Vec<PathBuf>, Error> {
    use crossbeam::channel;
    use rayon::prelude::*;

    const H: u32 = 1080 * 2;
    const W: u32 = 1920 * 2;

    let (sender, receiver) = channel::bounded(glob.len());
    glob.par_iter().for_each(move |inpath| {
        let inpath = Path::new(inpath);
        let file_stem = match inpath.file_stem() {
            Some(osstr) => osstr.to_str().unwrap(),
            None => {
                sender.send(Err(failure::err_msg("TODO"))).unwrap();
                return;
            }
        };
        let outfilename = format!("{}.jpg", file_stem);
        let outpath = dir.join(outfilename);
        if outpath.exists() {
            sender.send(Ok(None)).unwrap();
            return;
        }
        let image = match image::open(inpath) {
            Ok(image) => image,
            Err(e) => {
                sender.send(Err(e.into())).unwrap();
                return;
            }
        };
        let image = image.resize_to_fill(W, H, image::FilterType::CatmullRom);
        let mut outfile = match fs::File::create(&outpath) {
            Ok(file) => file,
            Err(e) => {
                sender.send(Err(e.into())).unwrap();
                return;
            }
        };
        match image.write_to(&mut outfile, image::ImageOutputFormat::JPEG(100)) {
            Ok(()) => {
                sender.send(Ok(Some(outpath))).unwrap();
                return;
            }
            Err(e) => {
                sender.send(Err(e.into())).unwrap();
                return;
            }
        };
    });
    let mut outpaths = Vec::with_capacity(glob.len());
    for _ in 0..glob.len() {
        let maybe_outpath = receiver.recv().unwrap()?;
        if let Some(outpath) = maybe_outpath {
            outpaths.push(outpath);
        }
    }
    Ok(outpaths)
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
