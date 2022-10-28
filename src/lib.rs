use itertools::Itertools;
use wasm_bindgen::prelude::*;

#[derive(PartialEq, Clone, Copy)]
struct Location {
    x: usize,
    y: usize,
}

impl Location {
    pub fn new(x: usize, y: usize) -> Self {
        Location {x, y}
    }
    
    fn manhattan_distance(&self, &other: &Location) -> usize {
        (self.x).abs_diff(other.x) + (self.y).abs_diff(other.y)
    }
}

struct PathStep {
    location: Location,
    parent: Option<Location>,
    distance: usize,
}

impl PathStep {
    pub fn new(location: Location, parent: Option<Location>, distance: usize) -> Self {
        PathStep {
            location,
            parent,
            distance,
        }
    }
}

#[wasm_bindgen]
pub fn validate_text_circle(s: &str) -> String {
    if s.len() == 0 {
        return "Invalid. The input is empty.".to_string();
    }

    if !square(s) {
        return "Invalid. The input is not square.".to_string();
    }
    
    if !odd(s) {
        return "Invalid. The side length of the square is not odd.".to_string();
    }
    
    if distinct_characters(s).len() != 2 {
        return "Invalid. The input does not contain 2 distinct characters.".to_string();
    }
    
    let missing_background = missing_background_characters(s);
    
    if missing_background.len() > 0 {
        let background = background_character(s);
        let formatted_missing_background = br_separated_tuples(&missing_background);
        return format!("Invalid. The following positions (x, y) from (0, 0) at left top should be background character \"{background}\":<br>{formatted_missing_background}");
    }
    
    match path_out_of_circle(s) {
        Some(path) => return format!("Invalid. There should not be a path from inside the circle to outside:<br><br><code>{path}</code>"),
        None => {let r = radius(s); return format!("This is a valid text circle of radius {r}.")}
    }
}

fn lines(s: &str) -> Vec<&str> {
    s.lines().collect::<Vec<&str>>()
}

fn height(s: &str) -> usize {
    lines(s).len()
}

fn square(s: &str) -> bool {
    let widths = s.lines().map(|l| l.chars().collect::<Vec<char>>().len());
    let max_width = widths.clone().max().unwrap();
    let min_width = widths.min().unwrap();
    
    height(s) == max_width && min_width == max_width
}

fn odd(s: &str) -> bool {
    height(s) % 2 == 1
}

fn distinct_characters(s: &str) -> Vec<char> {
    s.replace("\n", "").chars().collect::<Vec<char>>().into_iter().unique().collect()
}

fn radius(s: &str) -> usize {
    height(s) / 2
}

fn background_character(s: &str) -> char {
    let r = radius(s);
    
    character_at(r, r, s)
}

fn character_at(x: usize, y: usize, s: &str) -> char {
    let line = lines(s)[y];
    
    line.chars().collect::<Vec<char>>()[x]
}

fn missing_background_characters(s: &str) -> Vec<(usize, usize)> {
    let background = background_character(s);
    let mut missing_characters: Vec<(usize, usize)> = vec!();
    let r = radius(s);
    
    for (y, line) in s.lines().enumerate() {
        for (x, character) in line.chars().enumerate() {
            if character != background && required_background(x, y, r) {
                missing_characters.push((x, y))
            }
        }
    }
    
    missing_characters
}

fn br_separated_tuples(v: &Vec<(usize, usize)>) -> String {
    v.iter().map(|t| format!("({}, {})", t.0, t.1)).join("<br>")
}

fn required_background(x: usize, y: usize, r: usize) -> bool {
    let (x_offset, y_offset) = (x.abs_diff(r), y.abs_diff(r));
    let distance = ((x_offset * x_offset + y_offset * y_offset) as f64).sqrt();
    
    distance <= (r - 1) as f64 || distance >= (r + 1) as f64
}

fn path_out_of_circle(s: &str) -> Option<String> {
    let r = radius(s);
    let centre = Location::new(r, r);
    let start = PathStep::new(centre, None, 0);
    let background = background_character(s);
    
    let mut unfound = vec!();
    let h = height(s);
        
    for y in 0..h {
        for x in 0..h {
            if character_at(x, y, s) == background {
                let l = Location::new(x, y);
                
                if l != centre {
                    unfound.push(l)
                }
            }
        }
    }
    
    let mut found_to_check = vec!();
    found_to_check.push(start);
    
    let mut checked = vec!();
    
    loop {
        if found_to_check.len() == 0 {
            return None;
        }
        
        found_to_check.sort_by(|a, b| b.distance.cmp(&a.distance));
        let candidate = found_to_check.pop().unwrap();
        
        if edge_square(&candidate.location, h) {
            return Some(path_diagram(&candidate, &checked, s, h));
        }
        
        let unfound_cloned = unfound.clone();
        let unfound_neighbours = neighbours_in_unfound(&candidate, &unfound_cloned);
        
        for neighbour in unfound_neighbours {
            unfound.swap_remove(unfound.iter().position(|u| u == neighbour).unwrap());      
            found_to_check.push(PathStep::new(*neighbour, Some(candidate.location), candidate.distance + 1));
        }
        
        checked.push(candidate);
    }
}

fn edge_square(l: &Location, height: usize) -> bool {
    let Location { x, y } = l;
    
    *x == 0 || *y == 0 || *x == height - 1 || *y == height - 1
}

fn path_diagram(last_step: &PathStep, checked_squares: &Vec<PathStep>, s: &str, h: usize) -> String {
    let paving = character_to_pave_with(s);
    let mut path_squares = vec!();
    
    let mut current_step = last_step;
    
    loop {
        let current_location = current_step.location;
        path_squares.push(current_location);
        
        current_step = match current_step.parent {
            Some(parent_location) => checked_squares.iter().filter(|s| s.location == parent_location).collect::<Vec<&PathStep>>()[0],
            None => break,
        }
    }
    
    let mut diagram_rows = vec!();
    
    for y in 0..h {
        let mut row = "".to_string();
        
        for x in 0..h {
            row = format!("{}{}", row, if path_squares.iter().any(|&s| s == Location{x, y}) { paving } else { character_at(x, y, s) });
        }
        
        diagram_rows.push(row);
    }
    
    diagram_rows.join("<br>")    
}

fn neighbours_in_unfound<'a>(candidate: &PathStep, unfound: &'a Vec<Location>) -> Vec<&'a Location> {
    let c = candidate.location;
    
    unfound.iter().filter(|l| l.manhattan_distance(&c) == 1).collect()
}

fn character_to_pave_with(s: &str) -> char {
    let used_characters = distinct_characters(s);
    let potential_paving = vec!['#', 'X', '.'];
    
    *(potential_paving.iter().filter(|&c| used_characters.iter().all(|u| u != c)).collect::<Vec<&char>>()[0])
}

