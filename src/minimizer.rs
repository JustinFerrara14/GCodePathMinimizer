use indicatif::{ProgressBar, ProgressStyle};
use crate::utils::*;

pub fn minimize_gcode(gcode: &mut GCodeData) -> () {

    // Initialize progress bar
    let bar = ProgressBar::new(gcode.layers.len() as u64);

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
                    let distance = last_segment_prev_layer.length_with_other(segment);

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
            let mut remaining_segments: Vec<Segment> = gcode.layers[i].segments[1..].to_vec();

            while !remaining_segments.is_empty() {
                let last_segment = ordered_segments.last().unwrap();
                let mut closest_segment_index = 0;
                let mut closest_segment_reversed = false;
                let mut closest_distance = std::f64::MAX;

                for (i, segment) in remaining_segments.iter().enumerate() {
                    let distance = last_segment.length_with_other(segment);
                    let distance_reverse = last_segment.length_with_other(&segment.reversed());

                    let min_distance = distance.min(distance_reverse);

                    if min_distance < closest_distance {
                        closest_distance = min_distance;
                        closest_segment_index = i;
                        if distance_reverse < distance {
                            closest_segment_reversed = true;
                        }
                    }
                }

                // Add the closest segment to the ordered list and remove it from remaining
                let mut next_segment = remaining_segments.remove(closest_segment_index);
                if closest_segment_reversed {
                    next_segment = next_segment.reversed();
                }
                ordered_segments.push(next_segment);
            }
        } else {
            // Layer has no segments, do nothing
        }

        // Replace the layer's segments with the ordered segments
        gcode.layers[i].segments = ordered_segments;

        // Update progress bar
        bar.inc(1);
    }

    // Finish progress bar
    bar.finish_with_message("Minimization complete");
}