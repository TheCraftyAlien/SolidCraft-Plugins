//! Turns the dialog's values, and the box of the selected timber, into the boxes to make.
//! Plain Rust with no app calls, so `cargo test` covers it.

use serde_json::Value;

/// Standard sections: id, thickness, width in mm. The manifest lists the same ids.
pub const SIZES: [(&str, f64, f64); 12] = [
    ("4x2", 47.0, 100.0),
    ("4x2-cls", 38.0, 89.0),
    ("3x2", 47.0, 75.0),
    ("3x2-cls", 38.0, 63.0),
    ("2x2", 47.0, 50.0),
    ("2x2-par", 44.0, 44.0),
    ("6x2", 47.0, 150.0),
    ("8x2", 47.0, 200.0),
    ("3x3", 75.0, 75.0),
    ("4x4", 100.0, 100.0),
    ("batten-50", 25.0, 50.0),
    ("batten-38", 25.0, 38.0),
];

const X: usize = 0;
const Z: usize = 2;

/// A world box in mm.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    pub min: [f64; 3],
    pub max: [f64; 3],
}

impl Bounds {
    pub fn from_json(bbox: &Value) -> Option<Bounds> {
        let read = |end: &str| -> Option<[f64; 3]> {
            let v = &bbox[end];
            Some([v[0].as_f64()?, v[1].as_f64()?, v[2].as_f64()?])
        };
        Some(Bounds {
            min: read("min")?,
            max: read("max")?,
        })
    }
    fn size(&self, axis: usize) -> f64 {
        self.max[axis] - self.min[axis]
    }
    fn mid(&self, axis: usize) -> f64 {
        (self.min[axis] + self.max[axis]) / 2.0
    }
    /// The axis it is longest along.
    fn long(&self) -> usize {
        (0..3)
            .max_by(|&a, &b| self.size(a).total_cmp(&self.size(b)))
            .unwrap_or(X)
    }
}

/// What the dialog asked for, checked.
#[derive(Debug, Clone)]
pub struct Input {
    pub section: String,
    pub thickness: f64,
    pub width: f64,
    /// 0: as long as the selected timber.
    pub length: f64,
    pub place: String,
    pub at: String,
    pub along: String,
    pub count: usize,
    pub spacing: f64,
    pub direction: String,
    pub lie: String,
    pub material: String,
    pub name: String,
    /// Plain numbers are inches, and the run's parameters are written in inches.
    pub inches: bool,
}

fn text<'a>(params: &'a Value, key: &str, default: &'a str, allowed: &[&str]) -> Result<&'a str, String> {
    let value = params[key].as_str().unwrap_or(default);
    if allowed.contains(&value) {
        Ok(value)
    } else {
        Err(format!("{key} must be one of: {}", allowed.join(", ")))
    }
}

/// A length field in mm: typed text with or without a unit (see `units`), or a plain number in
/// the chosen unit.
fn size(params: &Value, key: &str, default_mm: f64, min: f64, max: f64, inches: bool) -> Result<f64, String> {
    let value = match &params[key] {
        Value::Null => default_mm,
        Value::String(text) => crate::units::parse(text, inches).map_err(|e| format!("{key}: {e}"))?,
        v => {
            let n = v.as_f64().ok_or(format!("{key} must be a length"))?;
            if inches { n * 25.4 } else { n }
        }
    };
    if value < min || value > max || !value.is_finite() {
        return Err(format!("{key} must be from {} to {} mm", shown(min), shown(max)));
    }
    Ok(value)
}

fn number(params: &Value, key: &str, default: f64, min: f64, max: f64) -> Result<f64, String> {
    let value = match &params[key] {
        Value::Null => default,
        v => v.as_f64().ok_or(format!("{key} must be a number"))?,
    };
    if value < min || value > max || !value.is_finite() {
        return Err(format!("{key} must be from {min} to {max}"));
    }
    Ok(value)
}

impl Input {
    pub fn read(params: &Value) -> Result<Input, String> {
        let inches = params["inches"].as_bool().unwrap_or(false);
        let mut ids: Vec<&str> = SIZES.iter().map(|s| s.0).collect();
        ids.push("custom");
        let section = text(params, "size", "4x2", &ids)?.to_string();
        let (thickness, width) = match SIZES.iter().find(|s| s.0 == section) {
            Some(&(_, t, w)) => (t, w),
            None => {
                let a = size(params, "custom_thickness", 47.0, 1.0, 500.0, inches)?;
                let b = size(params, "custom_width", 100.0, 1.0, 500.0, inches)?;
                (a.min(b), a.max(b))
            }
        };
        let count = number(params, "count", 1.0, 1.0, 100.0)?;
        if count.fract() != 0.0 {
            return Err("count must be a whole number".into());
        }
        let name = params["name"].as_str().unwrap_or("").trim().to_string();
        if name.len() > 40 {
            return Err("name must be at most 40 characters".into());
        }
        Ok(Input {
            section,
            thickness,
            width,
            length: size(params, "length", 2400.0, 0.0, 20000.0, inches)?,
            place: text(params, "place", "origin", &["origin", "parallel", "square"])?.into(),
            at: text(
                params,
                "at",
                "side",
                &["side", "other-side", "top", "below", "end", "start"],
            )?
            .into(),
            along: text(params, "along", "start", &["start", "middle", "end"])?.into(),
            count: count as usize,
            spacing: size(params, "spacing", 0.0, 0.0, 20000.0, inches)?,
            direction: text(params, "direction", "auto", &["auto", "x", "y", "z"])?.into(),
            lie: text(params, "lie", "auto", &["auto", "edge", "flat"])?.into(),
            material: text(
                params,
                "material",
                "Softwood",
                &["Softwood", "Treated softwood", "Oak", "none"],
            )?
            .into(),
            name,
            inches,
        })
    }

    /// The name a run gets when the Name field is left empty: 4x2, or 47x100 for a custom size.
    pub fn default_name(&self) -> String {
        if self.section == "custom" {
            format!("Timber {}x{}", shown(self.thickness), shown(self.width))
        } else {
            let short = self.section.trim_end_matches("-par").trim_end_matches("-cls");
            match self.section.as_str() {
                s if s.starts_with("batten") => format!("Batten {}x{}", shown(self.thickness), shown(self.width)),
                s if s.ends_with("-cls") => format!("{short} CLS"),
                s if s.ends_with("-par") => format!("{short} PAR"),
                _ => short.to_string(),
            }
        }
    }
}

/// A number as short as it can be written: 47, 22.5.
pub fn shown(v: f64) -> String {
    let r = (v * 1000.0).round() / 1000.0;
    if r.fract() == 0.0 {
        format!("{}", r as i64)
    } else {
        format!("{r}")
    }
}

/// One piece to make: its world box, and which axis each of its three sizes runs along.
#[derive(Debug, Clone, PartialEq)]
pub struct Piece {
    pub min: [f64; 3],
    pub size: [f64; 3],
}

/// The whole run: every piece, and which axis is the length, the width and the thickness.
#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    pub pieces: Vec<Piece>,
    /// Axis of the length, the width and the thickness: 0 X, 1 Y, 2 Z.
    pub axes: [usize; 3],
    pub length: f64,
}

fn axis_named(name: &str) -> Option<usize> {
    match name {
        "x" => Some(0),
        "y" => Some(1),
        "z" => Some(2),
        _ => None,
    }
}

/// The two axes square to `a`, horizontal first.
fn others(a: usize) -> [usize; 2] {
    match a {
        0 => [1, 2],
        1 => [0, 2],
        _ => [0, 1],
    }
}

/// Which way the width and the thickness run for a piece whose length runs along `a`.
/// On edge: the width stands up (a joist), or for an upright piece runs along Y.
/// Flat: the width lies down, or for an upright piece runs along X.
fn section_axes(a: usize, edge: bool) -> (usize, usize) {
    let [h, v] = others(a);
    match edge {
        true => (v, h),
        false => (h, v),
    }
}

/// Centres for `n` pieces of size `s` laid along `lo..hi`, from the start, the middle or the
/// end, `pitch` apart centre to centre (0: touching).
fn walk(lo: f64, hi: f64, s: f64, n: usize, pitch: f64, from: &str) -> Vec<f64> {
    let p = if pitch > 0.0 { pitch } else { s };
    (0..n)
        .map(|i| {
            let i = i as f64;
            match from {
                "end" => hi - s / 2.0 - i * p,
                "middle" => (lo + hi) / 2.0 + (i - (n as f64 - 1.0) / 2.0) * p,
                _ => lo + s / 2.0 + i * p,
            }
        })
        .collect()
}

/// The boxes for the run. `selected` is the world box of the selected timber, needed for
/// parallel and square placing.
// ponytail: the selected timber is read from its world bounding box, so it has to lie square to
// the axes; a timber turned at another angle is placed as if it were its box.
pub fn layout(input: &Input, selected: Option<Bounds>) -> Result<Layout, String> {
    let sel = match (input.place.as_str(), selected) {
        ("origin", _) => None,
        (_, Some(b)) => Some(b),
        (_, None) => return Err("select a timber (a body) first, or set Place to At the origin".into()),
    };
    let length = match (input.length, sel) {
        (l, _) if l > 0.0 => l,
        (_, Some(b)) => b.size(b.long()),
        _ => return Err("length 0 means as long as the selected timber; with nothing to match, enter a length".into()),
    };
    let asked = axis_named(&input.direction);
    let a = match (input.place.as_str(), sel) {
        ("parallel", Some(b)) => b.long(),
        ("square", Some(b)) => {
            let d = b.long();
            match asked {
                Some(a) if a == d => {
                    return Err(format!(
                        "the selected timber already runs along {}: pick another direction for one at 90 degrees",
                        ["X", "Y", "Z"][a]
                    ))
                }
                Some(a) => a,
                // the other horizontal one; a post gets X
                None if d == Z => X,
                None if d == X => 1,
                None => X,
            }
        }
        _ => asked.unwrap_or(X),
    };
    // On edge unless asked; parallel to a timber, the way that one lies.
    let edge = match (input.lie.as_str(), input.place.as_str(), sel) {
        ("edge", ..) => true,
        ("flat", ..) => false,
        (_, "parallel", Some(b)) => {
            let (w, t) = section_axes(a, true);
            b.size(w) >= b.size(t)
        }
        _ => true,
    };
    let (wa, ta) = section_axes(a, edge);
    let mut size = [0.0; 3];
    size[a] = length;
    size[wa] = input.width;
    size[ta] = input.thickness;
    let n = input.count;
    let mut centres: Vec<[f64; 3]> = Vec::new();
    match (input.place.as_str(), sel) {
        (_, None) => {
            // On the origin, the corner at 0: copies side by side across the horizontal
            // (for an upright piece, along X).
            let o = if a == Z { X } else { others(a)[0] };
            let mut base = size.map(|s| s / 2.0);
            for c in walk(0.0, f64::INFINITY, size[o], n, input.spacing, "start") {
                base[o] = c;
                centres.push(base);
            }
        }
        (place, Some(b)) => {
            let d = b.long();
            // Defaults: centred on the selected timber, its bottom on the same level.
            let mut base = [b.mid(0), b.mid(1), b.min[Z] + size[Z] / 2.0];
            let up_or_down = |top: bool, i: usize| match top {
                true => b.max[Z] + size[Z] * (i as f64 + 0.5),
                false => b.min[Z] - size[Z] * (i as f64 + 0.5),
            };
            match (place, input.at.as_str()) {
                ("parallel", "side" | "other-side") => {
                    // Beside it: across the horizontal, or for an upright one, the way it is
                    // thinner (along the wall for a stud).
                    let o = match a {
                        Z => {
                            if b.size(0) <= b.size(1) {
                                0
                            } else {
                                1
                            }
                        }
                        _ => others(a)[0],
                    };
                    let sign = if input.at == "side" { 1.0 } else { -1.0 };
                    base[a] = b.min[a] + length / 2.0;
                    let first = if input.spacing > 0.0 {
                        input.spacing
                    } else {
                        (b.size(o) + size[o]) / 2.0
                    };
                    let step = if input.spacing > 0.0 { input.spacing } else { size[o] };
                    for i in 0..n {
                        let mut c = base;
                        c[o] = b.mid(o) + sign * (first + i as f64 * step);
                        centres.push(c);
                    }
                }
                ("parallel", "top" | "below") => {
                    base[a] = b.min[a] + length / 2.0;
                    for i in 0..n {
                        let mut c = base;
                        c[Z] = up_or_down(input.at == "top", i);
                        centres.push(c);
                    }
                }
                ("parallel", _) => {
                    // In line, butted on its end.
                    for i in 0..n {
                        let mut c = base;
                        let i = i as f64;
                        c[a] = match input.at.as_str() {
                            "end" => b.max[a] + length * (i + 0.5),
                            _ => b.min[a] - length * (i + 0.5),
                        };
                        centres.push(c);
                    }
                }
                (_, "end" | "start") => {
                    // Across its end, like a rim joist: touching the end face, centred on it.
                    for i in 0..n {
                        let mut c = base;
                        let i = i as f64;
                        c[d] = match input.at.as_str() {
                            "end" => b.max[d] + size[d] * (i + 0.5),
                            _ => b.min[d] - size[d] * (i + 0.5),
                        };
                        centres.push(c);
                    }
                }
                (_, at) => {
                    // Crossing it on top or underneath, or against one side: walked along it.
                    let k = (0..3).find(|&k| k != a && k != d).unwrap_or(Z);
                    let top = match at {
                        "top" => Some(true),
                        "below" => Some(false),
                        // a side that faces up or down is the top or the underneath
                        "side" if k == Z => Some(true),
                        "other-side" if k == Z => Some(false),
                        _ => None,
                    };
                    if d == Z && n > 1 && top.is_some() {
                        return Err("several timbers across an upright one: place them on its side instead, or make one".into());
                    }
                    let along = walk(b.min[d], b.max[d], size[d], n, input.spacing, &input.along);
                    for at_d in along {
                        let mut c = base;
                        match top {
                            Some(up) => c[Z] = up_or_down(up, 0),
                            None => {
                                let sign = if at == "side" { 1.0 } else { -1.0 };
                                c[k] = b.mid(k) + sign * (b.size(k) + size[k]) / 2.0;
                            }
                        }
                        if d != Z || top.is_none() {
                            c[d] = at_d;
                        }
                        centres.push(c);
                    }
                }
            }
        }
    }
    let pieces = centres
        .into_iter()
        .map(|c| Piece {
            min: [0, 1, 2].map(|i| round(c[i] - size[i] / 2.0)),
            size,
        })
        .collect();
    Ok(Layout {
        pieces,
        axes: [a, wa, ta],
        length,
    })
}

fn round(v: f64) -> f64 {
    let r = (v * 1e6).round() / 1e6;
    if r == 0.0 {
        0.0
    } else {
        r
    }
}

#[cfg(test)]
mod tests;
