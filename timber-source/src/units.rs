//! Reads a length typed in the dialog: a plain number in the chosen unit (mm, or inches with the
//! Inches box ticked), or a value with its own unit, converted: 2400 mm, 2.4 m, 24 cm, 96", 96 in,
//! 8', 8 ft, 8' 6", 8ft 6in, 8' 6 (inches after feet), 1 1/2", 1-1/2", 3/4".

const MM_PER_IN: f64 = 25.4;

/// The length in mm. `inches`: a number without a unit is inches, else mm.
pub fn parse(text: &str, inches: bool) -> Result<f64, String> {
    let s = text.trim().to_lowercase().replace(['\u{2032}', '\u{2019}'], "'").replace(['\u{2033}', '\u{201d}'], "\"");
    if s.is_empty() {
        return Ok(0.0);
    }
    let bad = || format!("`{}` is not a length: try 2400, 2400 mm, 8', 96\" or 8' 6\"", text.trim());
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    let mut total = 0.0;
    let mut after_feet = false;
    let mut parts = 0;
    while i < chars.len() {
        while i < chars.len() && (chars[i] == ' ' || chars[i] == ',') {
            i += 1;
        }
        if i >= chars.len() {
            break;
        }
        // The number: 12, 1.5, 3/4, 1 1/2, 1-1/2.
        let (mut value, next) = number(&chars, i).ok_or_else(bad)?;
        i = next;
        let mut j = i;
        while j < chars.len() && (chars[j] == ' ' || chars[j] == '-') {
            j += 1;
        }
        if let Some((frac, next)) = fraction(&chars, j) {
            if value.fract() == 0.0 && j > i {
                value += frac;
                i = next;
            }
        }
        while i < chars.len() && chars[i] == ' ' {
            i += 1;
        }
        // The unit, if any.
        let word: String = chars[i..]
            .iter()
            .take_while(|c| c.is_ascii_alphabetic() || **c == '\'' || **c == '"')
            .collect();
        let (unit, len) = unit(&word).ok_or_else(bad)?;
        i += len;
        let mm = match unit {
            Some(per) => {
                after_feet = unit == Some(MM_PER_IN * 12.0);
                value * per
            }
            // a bare number after feet is inches: 8' 6
            None if after_feet => {
                after_feet = false;
                value * MM_PER_IN
            }
            None if parts > 0 => return Err(bad()),
            None if inches => value * MM_PER_IN,
            None => value,
        };
        total += mm;
        parts += 1;
    }
    Ok((total * 1e6).round() / 1e6)
}

/// A decimal or a fraction starting at `i`, and where it ends.
fn number(c: &[char], i: usize) -> Option<(f64, usize)> {
    if let Some(found) = fraction(c, i) {
        return Some(found);
    }
    let mut j = i;
    while j < c.len() && (c[j].is_ascii_digit() || c[j] == '.') {
        j += 1;
    }
    let text: String = c[i..j].iter().collect();
    text.parse::<f64>().ok().map(|v| (v, j))
}

/// a/b starting at `i`.
fn fraction(c: &[char], i: usize) -> Option<(f64, usize)> {
    let digits = |from: usize| {
        let mut j = from;
        while j < c.len() && c[j].is_ascii_digit() {
            j += 1;
        }
        j
    };
    let a_end = digits(i);
    if a_end == i || c.get(a_end) != Some(&'/') {
        return None;
    }
    let b_end = digits(a_end + 1);
    let a: f64 = c[i..a_end].iter().collect::<String>().parse().ok()?;
    let b: f64 = c[a_end + 1..b_end].iter().collect::<String>().parse().ok()?;
    (b > 0.0).then_some((a / b, b_end))
}

/// mm per unit for a unit word (None: no unit), and how many characters it took.
fn unit(word: &str) -> Option<(Option<f64>, usize)> {
    let per = match word {
        "" => return Some((None, 0)),
        "mm" => 1.0,
        "cm" => 10.0,
        "m" => 1000.0,
        "\"" | "''" | "in" | "inch" | "inches" => MM_PER_IN,
        "'" | "ft" | "foot" | "feet" => MM_PER_IN * 12.0,
        _ => return None,
    };
    Some((Some(per), word.chars().count()))
}

/// A length in mm written in the chosen unit, as a design parameter takes it: "2400 mm", "96 in".
/// In inches only when that is exact to a thousandth of an inch, so 1200 mm stays 1200 mm.
pub fn expression(mm: f64, inches: bool) -> String {
    let inch = (mm / MM_PER_IN * 1000.0).round() / 1000.0;
    match inches && (inch * MM_PER_IN - mm).abs() < 1e-6 {
        true => format!("{} in", crate::plan::shown(inch)),
        false => format!("{} mm", crate::plan::shown(mm)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mm(text: &str, inches: bool) -> f64 {
        (parse(text, inches).unwrap() * 1000.0).round() / 1000.0
    }

    #[test]
    fn plain_numbers_follow_the_box() {
        assert_eq!(mm("2400", false), 2400.0);
        assert_eq!(mm("96", true), 2438.4);
        assert_eq!(mm("1.5", true), 38.1);
        assert_eq!(mm("", true), 0.0);
        assert_eq!(mm("0", true), 0.0);
    }

    #[test]
    fn own_units_are_converted_either_way() {
        for inches in [false, true] {
            assert_eq!(mm("2400 mm", inches), 2400.0);
            assert_eq!(mm("2400mm", inches), 2400.0);
            assert_eq!(mm("2.4 m", inches), 2400.0);
            assert_eq!(mm("24cm", inches), 240.0);
            assert_eq!(mm("96\"", inches), 2438.4);
            assert_eq!(mm("96 in", inches), 2438.4);
            assert_eq!(mm("8'", inches), 2438.4);
            assert_eq!(mm("8 ft", inches), 2438.4);
            assert_eq!(mm("8' 6\"", inches), 2590.8);
            assert_eq!(mm("8'6\"", inches), 2590.8);
            assert_eq!(mm("8ft 6in", inches), 2590.8);
            assert_eq!(mm("8' 6", inches), 2590.8);
            assert_eq!(mm("1 1/2\"", inches), 38.1);
            assert_eq!(mm("1-1/2\"", inches), 38.1);
            assert_eq!(mm("3/4\"", inches), 19.05);
            assert_eq!(mm("8\u{2032} 6\u{2033}", inches), 2590.8);
        }
        assert_eq!(mm("1 1/2", true), 38.1);
    }

    #[test]
    fn refusals() {
        for text in ["abc", "12 yards", "8 6", "1/0\"", "2400 mm 5"] {
            assert!(parse(text, false).is_err(), "{text}");
        }
    }

    #[test]
    fn expressions() {
        assert_eq!(expression(2438.4, true), "96 in");
        assert_eq!(expression(2400.0, false), "2400 mm");
        assert_eq!(expression(1200.0, true), "1200 mm");
        assert_eq!(expression(2590.8, true), "102 in");
        assert_eq!(expression(19.05, true), "0.75 in");
    }
}
