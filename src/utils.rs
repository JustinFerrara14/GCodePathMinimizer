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
    let mut total_length_um: u64 = 0;

    for layer in &gcode.layers {
        let segment_count = layer.segments.len();
        for (i, segment) in layer.segments.iter().enumerate() {
            let dx = (segment.x2 - segment.x1) as f64;
            let dy = (segment.y2 - segment.y1) as f64;
            let length = (dx * dx + dy * dy).sqrt().round() as u64;
            total_length_um += length;

            // Get the length between two segments (the travel move)
            if i >= segment_count - 1 {
                continue;
            }

            let next_segment = &layer.segments[i + 1];
            let travel_dx = (next_segment.x1 - segment.x2) as f64;
            let travel_dy = (next_segment.y1 - segment.y2) as f64;
            let travel_length = (travel_dx * travel_dx + travel_dy * travel_dy).sqrt().round() as u64;
            total_length_um += travel_length;
        }
    }

    let speed_mm_per_min = speed_mm_per_sec * 60;
    let total_length_mm = total_length_um / 1000; // Convert microns to mm
    total_length_mm as f64 / speed_mm_per_min as f64
}