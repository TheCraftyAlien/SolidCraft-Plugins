//! Timber, a SolidCraft plugin: standard sawn and planed sections (4x2, 3x2, 2x2 and more) at a
//! length you choose, on the origin or placed parallel or at 90 degrees to a selected timber.
//! `plan` is plain Rust that works out the boxes, so `cargo test` covers it; only `guest`
//! touches the wasm bindings.

mod plan;
mod units;

use plan::{shown, Bounds, Input};
use serde_json::{json, Value};

/// Runs one app command and returns its result.
pub type Run<'a> = &'a mut dyn FnMut(&str, Value) -> Result<Value, String>;

/// Material name, density in g/cm3, and look: an appearance from the app's library, or a colour.
const MATERIALS: [(&str, f64, (&str, &str)); 3] = [
    ("Softwood", 0.5, ("color", "#d9b98a")),
    ("Treated softwood", 0.55, ("color", "#a7a77a")),
    ("Oak", 0.75, ("appearance", "Wood - Oak")),
];

/// A parameter name made from a run's name: letters, digits and _, starting with a letter.
pub fn prefix(name: &str) -> String {
    let mut p: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    if !p.starts_with(|c: char| c.is_ascii_alphabetic()) {
        p.insert_str(0, "T_");
    }
    p
}

/// `name`, or with the next free number when a body or the parameters already use it:
/// 4x2, 4x2 2, 4x2 3.
pub fn free_name(name: &str, design: &Value, measure: &Value) -> String {
    let bodies: Vec<&str> = (measure["bodies"].as_array().into_iter().flatten())
        .filter_map(|b| b["name"].as_str())
        .collect();
    let taken = |name: &str| {
        let id = format!("{}_length", prefix(name));
        let params = design["params"].as_array().into_iter().flatten();
        params.into_iter().any(|p| p["name"] == id.as_str())
            || bodies
                .iter()
                .any(|b| *b == name || b.starts_with(&format!("{name} #")))
    };
    if !taken(name) {
        return name.into();
    }
    (2..)
        .map(|n| format!("{name} {n}"))
        .find(|candidate| !taken(candidate))
        .unwrap_or_default()
}

/// The selected body's name: for a face or an edge, the body it is on.
fn picked(design: &Value) -> Option<String> {
    let items = design["selection"].as_array().into_iter().flatten();
    items
        .filter_map(|item| item["name"].as_str().or(item["body"].as_str()))
        .map(String::from)
        .next()
}

fn mm(v: f64) -> String {
    format!("{} mm", shown(v))
}

/// Makes the material on first use, then gives it to the bodies.
fn assign(run: Run, material: &str, bodies: &[String]) -> Result<(), String> {
    let (name, density, look) = (MATERIALS.iter().find(|m| m.0 == material))
        .ok_or(format!("unknown material `{material}`"))?;
    let list = run("material.list", json!({}))?;
    let list = list.get("materials").unwrap_or(&list);
    let known = list
        .as_array()
        .is_some_and(|all| all.iter().any(|m| m["name"] == *name));
    if !known {
        let mut args = json!({"name": name, "density_g_cm3": density});
        args[look.0] = json!(look.1);
        run("material.create", args)?;
    }
    run(
        "material.assign",
        json!({"bodies": bodies, "material": name}),
    )?;
    Ok(())
}

pub fn execute(entry: &str, params: &Value, run: Run) -> Result<Value, String> {
    if entry != "timber" {
        return Err(format!("unknown entry `{entry}`"));
    }
    let input = Input::read(params)?;
    let design = run(
        "document.inspect",
        json!({"bodies": false, "timeline": false}),
    )?;
    let measure = run("inspect.measure", json!({}))?;
    let (selected, against) = match input.place.as_str() {
        "origin" => (None, None),
        _ => {
            let body = picked(&design).ok_or(
                "select the timber to place against first (click it), or set Place to At the origin",
            )?;
            let all = measure["bodies"].as_array().into_iter().flatten();
            let found = all.into_iter().find(|b| b["name"] == body.as_str());
            let bounds = found
                .and_then(|b| Bounds::from_json(&b["bbox"]))
                .ok_or(format!("`{body}` has no size to place against"))?;
            (Some(bounds), Some(body))
        }
    };
    let layout = plan::layout(&input, selected)?;
    let wanted = match input.name.as_str() {
        "" => input.default_name(),
        n => n.to_string(),
    };
    let name = free_name(&wanted, &design, &measure);
    let p = prefix(&name);

    // The section and length are design parameters, so a run can be made longer or swapped
    // for another section afterwards in Parameters.
    let [a, wa, ta] = layout.axes;
    let mut sizes = [String::new(), String::new(), String::new()];
    // In the chosen unit; a standard section stays in mm, the size it is sold as.
    let custom = input.section == "custom";
    for (axis, key, value, comment, imperial) in [
        (a, "length", layout.length, "length", input.inches),
        (wa, "width", input.width, "section width", input.inches && custom),
        (ta, "thickness", input.thickness, "section thickness", input.inches && custom),
    ] {
        let id = format!("{p}_{key}");
        run(
            "parameters.add",
            json!({"name": id, "expression": units::expression(value, imperial), "comment": format!("{name}: {comment}")}),
        )?;
        sizes[axis] = id;
    }
    let mut bodies = Vec::new();
    for (i, piece) in layout.pieces.iter().enumerate() {
        let body = match layout.pieces.len() {
            1 => name.clone(),
            _ => format!("{name} #{}", i + 1),
        };
        run(
            "solid.box",
            json!({
                "length": sizes[0], "width": sizes[1], "height": sizes[2],
                "corner": piece.min.map(mm), "name": body, "body_name": body,
            }),
        )?;
        bodies.push(body);
    }
    if input.material != "none" {
        assign(run, &input.material, &bodies)?;
    }
    if bodies.len() > 1 {
        run(
            "timeline.group",
            json!({"name": name, "features": bodies}),
        )?;
    }
    let section = format!("{} x {} mm", shown(input.thickness), shown(input.width));
    let mut summary = json!({
        "name": name,
        "section": section,
        "length": units::expression(layout.length, input.inches),
        "count": bodies.len(),
        "bodies": bodies,
        "parameters": format!("{p}_length, {p}_width, {p}_thickness"),
        "runs_along": (["X", "Y", "Z"][a]),
    });
    if let Some(body) = against {
        summary["placed"] = json!(format!(
            "{} {}",
            if input.place == "parallel" { "parallel to" } else { "at 90 degrees to" },
            body
        ));
    }
    Ok(summary)
}

#[cfg(target_arch = "wasm32")]
mod guest {
    wit_bindgen::generate!({ path: "wit", world: "command-plugin" });

    use craft::plugin::types::{FieldFailure, PluginError};

    struct Timber;

    impl exports::craft::plugin::entry::Guest for Timber {
        fn execute(entry: String, params: String) -> Result<String, PluginError> {
            let invalid = |reason: String| {
                PluginError::InvalidInput(FieldFailure {
                    field: "params".into(),
                    reason,
                })
            };
            let params = serde_json::from_str(&params).map_err(|e| invalid(e.to_string()))?;
            let mut run = |command: &str, args: serde_json::Value| {
                let reply = craft::plugin::commands::run(command, &args.to_string()).map_err(
                    |e| match e {
                        PluginError::CommandFailed(f) => format!("{}: {}", f.command, f.reason),
                        other => format!("{command}: {other:?}"),
                    },
                )?;
                serde_json::from_str(&reply).map_err(|e| format!("{command}: bad reply: {e}"))
            };
            let result = crate::execute(&entry, &params, &mut run).map_err(invalid)?;
            Ok(result.to_string())
        }
    }

    export!(Timber);
}
