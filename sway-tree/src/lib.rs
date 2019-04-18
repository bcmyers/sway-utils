#[macro_use]
extern crate serde;

use std::process::Command;

use failure::Error;

type Map = serde_json::Map<String, serde_json::Value>;

pub fn get_tree() -> Result<Vec<Node>, failure::Error> {
    let output = Command::new("swaymsg").arg("-t").arg("get_tree").output()?;
    let s = std::str::from_utf8(&output.stdout)?;
    let o: Map = serde_json::from_str(s)?;
    let mut output = Vec::new();
    walk_tree(&o, &mut output)?;
    Ok(output)
}

fn walk_tree(m: &Map, output: &mut Vec<Node>) -> Result<(), Error> {
    let f = m.get("focused").ok_or_else(|| failure::err_msg("TODO"))?;
    let f = match f {
        serde_json::Value::Bool(b) => *b,
        _ => failure::bail!("TODO"),
    };
    let w = m.get("window").ok_or_else(|| failure::err_msg("TODO"))?;
    match w {
        serde_json::Value::Null => (),
        serde_json::Value::Number(n) => {
            let n = n.as_u64().ok_or_else(|| failure::err_msg("TODO"))?;
            let pid = m.get("pid").ok_or_else(|| failure::err_msg("TODO"))?;
            let pid = match pid {
                serde_json::Value::Number(n) => {
                    n.as_u64().ok_or_else(|| failure::err_msg("TODO"))?
                }
                _ => failure::bail!("TODO"),
            };
            let t = m.get("type").ok_or_else(|| failure::err_msg("TODO"))?;
            let t = match t {
                serde_json::Value::String(s) => s.clone(),
                _ => failure::bail!("TODO"),
            };

            let app_id = m.get("app_id").ok_or_else(|| failure::err_msg("TODO"))?;
            let app_id = match app_id {
                serde_json::Value::Null => None,
                serde_json::Value::String(s) => Some(s.clone()),
                _ => failure::bail!("TODO"),
            };

            let wp = m
                .get("window_properties")
                .ok_or_else(|| failure::err_msg("TODO"))?
                .clone();
            let wp: WindowProperties = serde_json::from_value(wp)?;
            let n = Node::new(app_id, f, n, pid, wp, t);
            output.push(n);
        }
        _ => failure::bail!("TODO"),
    }
    if let Some(ns) = m.get("nodes") {
        let ns = match ns {
            serde_json::Value::Array(v) => v,
            _ => failure::bail!("TODO"),
        };
        for n in ns {
            match n {
                serde_json::Value::Object(o) => walk_tree(o, output)?,
                _ => failure::bail!("TODO"),
            }
        }
    }
    if let Some(ns) = m.get("floating_nodes") {
        let ns = match ns {
            serde_json::Value::Array(v) => v,
            _ => failure::bail!("TODO"),
        };
        for n in ns {
            match n {
                serde_json::Value::Object(o) => walk_tree(o, output)?,
                _ => failure::bail!("TODO"),
            }
        }
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
struct WindowProperties {
    class: String,
    instance: String,
    title: String,
    transient_for: Option<String>,
    window_role: Option<String>,
}

#[derive(Debug)]
pub struct Node {
    app_id: Option<String>,
    class: String,
    id: u64,
    is_focused: bool,
    instance: String,
    transient_for: Option<String>,
    window_role: Option<String>,
    title: String,
    type_: String,
    pid: u64,
}

impl Node {
    pub fn class(&self) -> &str {
        &self.class
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn pid(&self) -> u64 {
        self.pid
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    fn new(
        app_id: Option<String>,
        is_focused: bool,
        id: u64,
        pid: u64,
        properties: WindowProperties,
        type_: String,
    ) -> Self {
        Self {
            app_id,
            class: properties.class,
            id,
            type_,
            is_focused,
            pid,
            instance: properties.instance,
            title: properties.title,
            transient_for: properties.transient_for,
            window_role: properties.window_role,
        }
    }
}
