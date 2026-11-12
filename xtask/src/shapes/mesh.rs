//! MakeHuman's base mesh, read from its Wavefront `.obj` file: the points, the
//! skin's faces, and where each joint is.
//!
//! The file is plain text, a line each: `v x y z` is a point, `g name` starts a
//! group, and `f a/b c/d ...` is a face of the group, by the numbers of its
//! points counting from 1 (what follows a `/` is for textures, not used here).
//! The `body` group is the skin; `joint-...` groups are small cubes whose
//! middles mark the joints; the rest (hair, skirt, teeth) is left out.

use std::collections::BTreeMap;

/// x across (towards the body's left), y up, z forwards, in the file's own
/// units (decimetres for a person).
pub type Point = [f64; 3];

/// The group holding the skin.
const SKIN: &str = "body";
/// What every joint's group starts with.
const JOINT: &str = "joint-";

#[derive(Debug, Default)]
pub struct Mesh {
    pub points: Vec<Point>,
    /// The skin's faces: each the places of its corners in `points`.
    pub skin: Vec<Vec<usize>>,
    /// The points of each joint's cube, by the joint's name without `joint-`.
    joints: BTreeMap<String, Vec<usize>>,
}

impl Mesh {
    /// Reads the text of an `.obj` file.
    pub fn read(text: &str) -> Result<Self, String> {
        let mut mesh = Self::default();
        let mut group = String::new();
        for (number, line) in text.lines().enumerate() {
            let mut words = line.split_whitespace();
            let wrong = |what: &str| format!("line {}: {what}: {line}", number + 1);
            match words.next() {
                Some("v") => {
                    let mut point = [0.0; 3];
                    for value in &mut point {
                        *value = words
                            .next()
                            .and_then(|word| word.parse().ok())
                            .ok_or_else(|| wrong("not a point"))?;
                    }
                    mesh.points.push(point);
                }
                Some("g") => group = words.next().unwrap_or_default().to_string(),
                Some("f") => {
                    let corners = words
                        .map(|word| corner(word, mesh.points.len()))
                        .collect::<Option<Vec<_>>>()
                        .ok_or_else(|| wrong("not a face"))?;
                    if group == SKIN {
                        mesh.skin.push(corners);
                    } else if let Some(joint) = group.strip_prefix(JOINT) {
                        let cube = mesh.joints.entry(joint.to_string()).or_default();
                        cube.extend(corners);
                    }
                }
                _ => {}
            }
        }
        if mesh.skin.is_empty() {
            return Err(format!("no faces in a `{SKIN}` group"));
        }
        Ok(mesh)
    }

    /// The middle of a joint's cube, such as `l-knee` or `neck`.
    pub fn joint(&self, name: &str) -> Option<Point> {
        let cube = self.joints.get(name)?;
        let mut sum = [0.0; 3];
        for &at in cube {
            for (total, value) in sum.iter_mut().zip(self.points[at]) {
                *total += value;
            }
        }
        Some(sum.map(|total| total / cube.len() as f64))
    }

    /// The lowest and highest of the skin's points along each direction.
    pub fn bounds(&self) -> (Point, Point) {
        let mut low = [f64::MAX; 3];
        let mut high = [f64::MIN; 3];
        for &at in self.skin.iter().flatten() {
            for way in 0..3 {
                low[way] = low[way].min(self.points[at][way]);
                high[way] = high[way].max(self.points[at][way]);
            }
        }
        (low, high)
    }
}

/// A face's corner (`12/7` or `12`) as a place in the points, which must exist.
fn corner(word: &str, points: usize) -> Option<usize> {
    let number: usize = word.split('/').next()?.parse().ok()?;
    (1..=points).contains(&number).then(|| number - 1)
}
