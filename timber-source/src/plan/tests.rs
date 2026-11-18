use super::*;
use serde_json::json;

fn input(extra: Value) -> Input {
    let mut p = json!({
        "size": "4x2", "custom_thickness": 47, "custom_width": 100, "length": "2400 mm",
        "place": "origin", "at": "side", "along": "start", "count": 1, "spacing": 0,
        "direction": "auto", "lie": "auto", "material": "Softwood", "name": ""
    });
    for (k, v) in extra.as_object().unwrap() {
        p[k] = v.clone();
    }
    Input::read(&p).unwrap()
}

/// A 4x2 joist on edge along X, 3000 long, on the floor at the origin.
fn joist() -> Bounds {
    Bounds { min: [0.0, 0.0, 0.0], max: [3000.0, 47.0, 100.0] }
}

/// A 4x2 stud standing up, 2400 tall, thin along X.
fn stud() -> Bounds {
    Bounds { min: [0.0, 0.0, 0.0], max: [47.0, 100.0, 2400.0] }
}

fn boxes(l: &Layout) -> Vec<[f64; 6]> {
    l.pieces
        .iter()
        .map(|p| [p.min[0], p.min[1], p.min[2], p.min[0] + p.size[0], p.min[1] + p.size[1], p.min[2] + p.size[2]])
        .collect()
}

#[test]
fn origin_on_edge_along_x() {
    let l = layout(&input(json!({})), None).unwrap();
    assert_eq!(boxes(&l), vec![[0.0, 0.0, 0.0, 2400.0, 47.0, 100.0]]);
    assert_eq!(l.axes, [0, 2, 1]);
}

#[test]
fn origin_flat_upright_and_custom() {
    let l = layout(&input(json!({"lie": "flat"})), None).unwrap();
    assert_eq!(boxes(&l), vec![[0.0, 0.0, 0.0, 2400.0, 100.0, 47.0]]);
    let l = layout(&input(json!({"direction": "z"})), None).unwrap();
    assert_eq!(boxes(&l), vec![[0.0, 0.0, 0.0, 47.0, 100.0, 2400.0]]);
    let l = layout(&input(json!({"size": "custom", "custom_thickness": 150, "custom_width": 22, "length": 1000})), None).unwrap();
    assert_eq!(boxes(&l), vec![[0.0, 0.0, 0.0, 1000.0, 22.0, 150.0]]);
}

#[test]
fn origin_copies_at_spacing() {
    let l = layout(&input(json!({"count": 3, "spacing": 400})), None).unwrap();
    let ys: Vec<f64> = l.pieces.iter().map(|p| p.min[1]).collect();
    assert_eq!(ys, vec![0.0, 400.0, 800.0]);
}

#[test]
fn parallel_joists_at_400_centres() {
    let l = layout(&input(json!({"place": "parallel", "count": 3, "spacing": 400, "length": 0})), Some(joist())).unwrap();
    assert_eq!(
        boxes(&l),
        vec![
            [0.0, 400.0, 0.0, 3000.0, 447.0, 100.0],
            [0.0, 800.0, 0.0, 3000.0, 847.0, 100.0],
            [0.0, 1200.0, 0.0, 3000.0, 1247.0, 100.0],
        ]
    );
}

#[test]
fn parallel_touching_other_side_keeps_its_lie() {
    // a flat batten beside a flat batten
    let flat = Bounds { min: [0.0, 0.0, 0.0], max: [2000.0, 50.0, 25.0] };
    let l = layout(&input(json!({"size": "batten-50", "place": "parallel", "at": "other-side", "length": 0})), Some(flat)).unwrap();
    assert_eq!(boxes(&l), vec![[0.0, -50.0, 0.0, 2000.0, 0.0, 25.0]]);
}

#[test]
fn parallel_stud_beside_goes_along_the_wall() {
    let l = layout(&input(json!({"place": "parallel", "spacing": 600, "count": 2, "length": 0})), Some(stud())).unwrap();
    assert_eq!(
        boxes(&l),
        vec![[600.0, 0.0, 0.0, 647.0, 100.0, 2400.0], [1200.0, 0.0, 0.0, 1247.0, 100.0, 2400.0]]
    );
}

#[test]
fn parallel_on_top_and_in_line() {
    let l = layout(&input(json!({"place": "parallel", "at": "top", "length": 0})), Some(joist())).unwrap();
    assert_eq!(boxes(&l), vec![[0.0, 0.0, 100.0, 3000.0, 47.0, 200.0]]);
    let l = layout(&input(json!({"place": "parallel", "at": "end", "length": 1000})), Some(joist())).unwrap();
    assert_eq!(boxes(&l), vec![[3000.0, 0.0, 0.0, 4000.0, 47.0, 100.0]]);
}

#[test]
fn square_joists_on_top_of_a_bearer() {
    let l = layout(&input(json!({"place": "square", "at": "top", "count": 3, "spacing": 400, "length": 1200})), Some(joist())).unwrap();
    // across Y, centred on the bearer, on edge, the first flush with its start
    assert_eq!(
        boxes(&l),
        vec![
            [0.0, -576.5, 100.0, 47.0, 623.5, 200.0],
            [400.0, -576.5, 100.0, 447.0, 623.5, 200.0],
            [800.0, -576.5, 100.0, 847.0, 623.5, 200.0],
        ]
    );
    assert_eq!(l.axes[0], 1);
}

#[test]
fn square_from_the_end_and_middle() {
    let l = layout(&input(json!({"place": "square", "at": "top", "along": "end", "length": 600})), Some(joist())).unwrap();
    assert_eq!(boxes(&l)[0][3], 3000.0);
    let l = layout(&input(json!({"place": "square", "at": "top", "along": "middle", "count": 2, "spacing": 400, "length": 600})), Some(joist())).unwrap();
    let xs: Vec<f64> = l.pieces.iter().map(|p| p.min[0] + p.size[0] / 2.0).collect();
    assert_eq!(xs, vec![1300.0, 1700.0]);
}

#[test]
fn square_rim_across_the_end() {
    let l = layout(&input(json!({"place": "square", "at": "end", "length": 1200})), Some(joist())).unwrap();
    assert_eq!(boxes(&l), vec![[3000.0, -576.5, 0.0, 3047.0, 623.5, 100.0]]);
}

#[test]
fn square_rail_on_the_side_of_a_post() {
    let post = Bounds { min: [0.0, 0.0, 0.0], max: [100.0, 100.0, 2000.0] };
    let l = layout(&input(json!({"place": "square", "at": "side", "along": "end", "length": 1800})), Some(post)).unwrap();
    // along X, centred on the post, against its +Y face, its top flush with the post's
    assert_eq!(boxes(&l), vec![[-850.0, 100.0, 1900.0, 950.0, 147.0, 2000.0]]);
}

#[test]
fn square_upright_on_a_beam() {
    let l = layout(&input(json!({"place": "square", "at": "top", "direction": "z", "along": "middle", "length": 500})), Some(joist())).unwrap();
    assert_eq!(boxes(&l), vec![[1476.5, -26.5, 100.0, 1523.5, 73.5, 600.0]]);
}

#[test]
fn refusals() {
    assert!(layout(&input(json!({"place": "parallel"})), None).is_err());
    assert!(layout(&input(json!({"place": "square", "direction": "x"})), Some(joist())).is_err());
    assert!(layout(&input(json!({"length": 0})), None).is_err());
    let mut p = json!({"size": "2x4"});
    assert!(Input::read(&p).is_err());
    p = json!({"count": 2.5});
    assert!(Input::read(&p).is_err());
}

#[test]
fn names() {
    assert_eq!(input(json!({})).default_name(), "4x2");
    assert_eq!(input(json!({"size": "3x2-cls"})).default_name(), "3x2 CLS");
    assert_eq!(input(json!({"size": "batten-38"})).default_name(), "Batten 25x38");
    assert_eq!(input(json!({"size": "custom", "custom_thickness": 22, "custom_width": 150})).default_name(), "Timber 22x150");
}

#[test]
fn typed_lengths_and_inches() {
    let l = layout(&input(json!({"length": "8'"})), None).unwrap();
    assert_eq!(l.length, 2438.4);
    let l = layout(&input(json!({"inches": true, "length": "96", "count": 2, "spacing": "16"})), None).unwrap();
    assert_eq!(l.length, 2438.4);
    assert_eq!(l.pieces[1].min[1], 406.4);
    let l = layout(&input(json!({"inches": true, "length": "2400 mm"})), None).unwrap();
    assert_eq!(l.length, 2400.0);
    let i = input(json!({"inches": true, "size": "custom", "custom_thickness": "1 1/2", "custom_width": "3 1/2\""}));
    assert_eq!((i.thickness, i.width), (38.1, 88.9));
    assert!(Input::read(&json!({"length": "8 yards"})).is_err());
    assert!(Input::read(&json!({"length": "100'"})).is_err());
}
