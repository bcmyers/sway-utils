#[macro_use]
extern crate clap;

use std::process::{Command, Stdio};

use failure::Error;
use structopt::StructOpt;
use sway_tree::get_tree;

arg_enum! {
    #[derive(Debug)]
    enum Instruction {
        Focus,
        Kill,
    }
}

#[derive(Debug, StructOpt)]
struct Opt {
    /// Command that launches the application.
    cmd: String,

    /// Window class
    #[structopt(short = "c", long = "class")]
    class: Option<String>,

    /// What to do if the application is already open.
    #[structopt(
        raw(possible_values = r#"&["focus", "kill"]"#),
        raw(case_insensitive = "true")
    )]
    instruction: Instruction,

    /// Window title
    #[structopt(short = "t", long = "title")]
    title: Option<String>,
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();

    if opt.class.is_none() && opt.title.is_none() {
        failure::bail!("TODO")
    }

    let nodes = get_tree()?;

    let maybe_node = nodes
        .iter()
        .filter(|node| match (&opt.class, &opt.title) {
            (Some(class), Some(title)) => node.class() == class && node.title() == title,
            (Some(class), None) => node.class() == class,
            (None, Some(title)) => node.title() == title,
            (None, None) => unreachable!(),
        })
        .nth(0);

    match maybe_node {
        Some(node) => match opt.instruction {
            Instruction::Kill => {
                let output = Command::new("kill").arg(&node.pid().to_string()).output()?;
                if !output.status.success() {
                    failure::bail!("TODO");
                }
            }
            Instruction::Focus => {
                let arg = format!(r#"[con_id="{}"] focus"#, node.id());
                let output = Command::new("swaymsg")
                    .stderr(Stdio::inherit())
                    .stdout(Stdio::inherit())
                    .arg(&arg)
                    .output()?;
                if !output.status.success() {
                    failure::bail!("TODO");
                }
            }
        },
        None => {
            Command::new("bash").arg("-c").arg(&opt.cmd).spawn()?;
        }
    }

    Ok(())
}
