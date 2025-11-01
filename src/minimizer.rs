
use crate::utils::*;

pub fn minimize_gcode(gcode: &mut GCodeData) -> GCodeData {

    // For each layer
    for i in 0..gcode.layers.len() {

        // Check if it is the first layer, use the first segment as default
        if i == 0 {
            // Do nothing for the first layer
        } else {
            // For other layers, the first segment is the one closest to the last segment of the previous layer
            let previous_layer = &gcode.layers[i - 1];
            if let Some(last_segment_prev_layer) = previous_layer.segments.last() {
                let mut closest_segment_index = 0;
                let mut closest_distance = std::f64::MAX;

                for (i, segment) in gcode.layers[i].segments.iter().enumerate() {
                    let dx = (segment.x1 - last_segment_prev_layer.x2) as f64;
                    let dy = (segment.y1 - last_segment_prev_layer.y2) as f64;
                    let distance = (dx * dx + dy * dy).sqrt();
                    if distance < closest_distance {
                        closest_distance = distance;
                        closest_segment_index = i;
                    }
                }

                // Rotate the segments so that the closest segment is first
                gcode.layers[i].segments.rotate_left(closest_segment_index);
            }
        }

        // From the first segment, find the next closest segment and add it to the ordered list
        let mut ordered_segments = Vec::new();
        if let Some(first_segment) = gcode.layers[i].segments.first() {

            // Add the first segment to the ordered list
            ordered_segments.push(first_segment.clone());
            let mut remaining_segments: Vec<Segment> = gcode.layers[i].segments[2..].to_vec();

            for seg in &remaining_segments {
                println!("Remaining Segment: ({}, {}) -> ({}, {})", seg.x1, seg.y1, seg.x2, seg.y2);
            }

            while !remaining_segments.is_empty() {
                let last_segment = ordered_segments.last().unwrap();
                let mut closest_segment_index = 0;
                let mut closest_distance = std::f64::MAX;

                for (i, segment) in remaining_segments.iter().enumerate() {
                    let dx = (segment.x1 - last_segment.x2) as f64;
                    let dy = (segment.y1 - last_segment.y2) as f64;
                    let distance = (dx * dx + dy * dy).sqrt();

                    /*let dx2 = (segment.x2 - last_segment.x2) as f64;
                    let dy2 = (segment.y2 - last_segment.y2) as f64;
                    let distance2 = (dx2 * dx2 + dy2 * dy2).sqrt();

                    if distance2 < distance {
                        distance = distance2;

                        // Reverse the segment to minimize distance
                        remaining_segments[i] = Segment {
                            x1: segment.x2,
                            y1: segment.y2,
                            x2: segment.x1,
                            y2: segment.y1,
                        };
                    }*/

                    if distance < closest_distance {
                        closest_distance = distance;
                        closest_segment_index = i;
                    }
                }

                // Add the closest segment to the ordered list and remove it from remaining
                ordered_segments.push(remaining_segments.remove(closest_segment_index));
            }
        } else {
            // Layer has no segments, do nothing
        }

        // Print the ordered segments for debugging
        for segment in &ordered_segments {
            println!("Segment: ({}, {}) -> ({}, {})", segment.x1, segment.y1, segment.x2, segment.y2);
        }

        // Replace the layer's segments with the ordered segments
        gcode.layers[i].segments = ordered_segments;
    }

    GCodeData {
        num_layers: gcode.num_layers,
        layers: gcode.layers.clone(),
    }
}