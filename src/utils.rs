use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Segment {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
}

impl Segment {
    pub fn length_um(&self) -> f64 {
        let dx = (self.x2 - self.x1) as f64;
        let dy = (self.y2 - self.y1) as f64;
        (dx * dx + dy * dy).sqrt()
    }

    // Length from the end of self to the start of other
    pub fn length_with_other(&self, other: &Segment) -> f64 {
        let dx = (other.x1 - self.x2) as f64;
        let dy = (other.y1 - self.y2) as f64;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn is_equal(&self, other: &Segment) -> bool {
        (self.x1 == other.x1 && self.y1 == other.y1 && self.x2 == other.x2 && self.y2 == other.y2) ||
        (self.x1 == other.x2 && self.y1 == other.y2 && self.x2 == other.x1 && self.y2 == other.y1)
    }

    pub fn reversed(&self) -> Segment {
        Segment {
            x1: self.x2,
            y1: self.y2,
            x2: self.x1,
            y2: self.y1,
        }
    }

}

#[derive(Debug, Clone)]
pub struct Layer {
    pub id: usize,
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone)]
pub struct GCodeData {
    pub num_layers: usize,
    pub layers: Vec<Layer>,
}

pub fn parse_gcode_file<P: AsRef<Path>>(path: P) -> io::Result<GCodeData> {
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    let mut lines = reader.lines();

    // --- Parse first line: total number of layers
    let first_line = lines
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing first line"))??;
    let num_layers: usize = first_line
        .split_whitespace()
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid first line"))?
        .parse()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid layer count"))?;

    let mut layers: Vec<Layer> = Vec::new();
    let mut current_layer_id: Option<usize> = None;
    let mut current_num_segments: usize = 0;
    let mut segments: Vec<Segment> = Vec::new();

    while let Some(Ok(line)) = lines.next() {
        let parts: Vec<&str> = line.split_whitespace().collect();

        // Detect new layer header like "0 1111 Numéro de couche, nb segments"
        if parts.len() >= 5 && parts[2] == "Numéro" && parts[3] == "de" && parts[4].starts_with("couche") {
            // If we were collecting segments from a previous layer, save them first
            if let Some(layer_id) = current_layer_id {
                layers.push(Layer { id: layer_id, segments: segments.clone() });
                segments.clear();
            }

            current_layer_id = Some(
                parts[0]
                    .parse()
                    .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid layer ID"))?,
            );
            current_num_segments = parts[1]
                .parse()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid segment count"))?;
        }
        // Otherwise, if this line looks like a segment (4 integers)
        else if parts.len() == 4 {
            let x1: i32 = parts[0]
                .parse()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid x1"))?;
            let y1: i32 = parts[1]
                .parse()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid y1"))?;
            let x2: i32 = parts[2]
                .parse()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid x2"))?;
            let y2: i32 = parts[3]
                .parse()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid y2"))?;

            segments.push(Segment { x1, y1, x2, y2 });

            // If all segments for current layer collected, save it
            if segments.len() == current_num_segments {
                if let Some(layer_id) = current_layer_id {
                    layers.push(Layer { id: layer_id, segments: segments.clone() });
                    segments.clear();
                    current_layer_id = None; // reset for next layer
                }
            }
        }
    }

    // Add the last layer if not already added
    if let Some(layer_id) = current_layer_id {
        if !segments.is_empty() {
            layers.push(Layer { id: layer_id, segments });
        }
    }

    // Optional validation
    if layers.len() > num_layers {
        eprintln!(
            "Warning: file declares {} layers but {} were parsed",
            num_layers,
            layers.len()
        );
    }

    Ok(GCodeData { num_layers, layers })
}

pub fn get_print_time_minutes(gcode: &GCodeData, speed_mm_per_sec: u64) -> f64 {
    let mut total_length_um: f64 = 0.0;
    let mut total_travel_length_um: f64 = 0.0;

    let mut prev_layer_end: Option<Segment> = None;

    for layer in &gcode.layers {
        if layer.segments.is_empty() {
            continue;
        }

        // --- Add travel between previous layer end and current layer start ---
        if let Some(prev) = prev_layer_end {
            let first = &layer.segments[0];
            total_travel_length_um += prev.length_with_other(&first);
        }

        // --- Process all segments within the current layer ---
        let segment_count = layer.segments.len();
        for (i, segment) in layer.segments.iter().enumerate() {
            let length = segment.length_um();
            total_length_um += length;

            // Travel between segments in the same layer
            if i < segment_count - 1 {
                let next_segment = &layer.segments[i + 1];
                total_travel_length_um += segment.length_with_other(&next_segment);
            }
        }

        // Save the last endpoint of this layer
        let last_segment = &layer.segments[segment_count - 1];
        prev_layer_end = Some(last_segment.clone());
    }

    let speed_mm_per_min = speed_mm_per_sec * 60;
    let total_length_mm = (total_length_um + total_travel_length_um) as f64 / 1000.0;

    println!("Total length [mm]: {}", total_length_mm);

    total_length_mm / speed_mm_per_min as f64
}

pub fn test_gcode_equality(gcode_a: &GCodeData, gcode_b: &GCodeData) -> bool {
    if gcode_a.num_layers != gcode_b.num_layers {
        return false;
    }

    for (layer_a, layer_b) in gcode_a.layers.iter().zip(gcode_b.layers.iter()) {
        if layer_a.id != layer_b.id || layer_a.segments.len() != layer_b.segments.len() {
            return false;
        }

        // Check if all segment of gcode_a are in gcode_b
        for segment_a in &layer_a.segments {
            if !layer_b.segments.iter().any(|segment_b| {
                segment_a.is_equal(segment_b)
            }) {
                return false;
            }
        }
    }

    true
}