use failure::Error;
use sway_tree::get_tree;

fn main() -> Result<(), Error> {
    let tree = get_tree()?;
    println!("{:#?}", &tree);
    Ok(())
}
