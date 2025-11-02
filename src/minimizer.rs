use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashSet;
// use kiddo::KdTree;
use kiddo::float::kdtree::KdTree;
use kiddo::SquaredEuclidean;
use kiddo::NearestNeighbour;

use crate::utils::*;
use crate::consts::*;

pub fn minimize_gcode(gcode: &mut GCodeData) -> () {

    // Initialize progress bar
    let bar = ProgressBar::new(gcode.layers.len() as u64);

    // For each layer
    for i in 0..gcode.layers.len() {

        // Skip the first layer (it will start from the first segment)
        if i > 0 {
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

            ordered_segments.push(first_segment.clone());
            let mut remaining_segments: Vec<Segment> = gcode.layers[i].segments[1..].to_vec();
            let mut remaining_indices: HashSet<usize> = (0..remaining_segments.len()).collect();

            // Create kd-tree for the segments in the layer
            let mut kdtree: KdTree<_, _, 2, KDTREE_BUCKET_SIZE, u32> = KdTree::with_capacity(remaining_segments.len() * 2);

            for (i, s) in remaining_segments.iter().enumerate() {
                kdtree.add(&[s.x1 as f64, s.y1 as f64], i as u64);
                kdtree.add(&[s.x2 as f64, s.y2 as f64], i as u64);
            }

            let mut number_of_nearests_to_check = NUMBER_OF_NEARESTS_TO_CHECK;
            while !remaining_indices.is_empty() {
                let last_segment = ordered_segments.last().unwrap();
                let last_point = [last_segment.x2 as f64, last_segment.y2 as f64];

                // Find the nearest neighbor in the kd-tree
                let nearests = kdtree.nearest_n::<SquaredEuclidean>(&last_point, number_of_nearests_to_check);


                // Check if already used
                for i in 0..nearests.len() {
                    let nearest = &nearests[i];
                    let nearest_index = nearest.item as usize;
                    if remaining_indices.contains(&nearest_index) {
                        // Found the nearest unused segment
                        let mut next_segment = remaining_segments[nearest_index].clone();

                        // Determine if we need to reverse the segment
                        if last_segment.length_with_other(&next_segment.reversed()) < last_segment.length_with_other(&next_segment) {
                            next_segment = next_segment.reversed();
                        }

                        // Add the closest segment to the ordered list
                        ordered_segments.push(next_segment.clone());
                        remaining_indices.remove(&nearest_index);

                        // Remove the segment from remaining and kd-tree
                        kdtree.remove(&[next_segment.x1 as f64, next_segment.y1 as f64], nearest.item);
                        kdtree.remove(&[next_segment.x2 as f64, next_segment.y2 as f64], nearest.item);
                        break;
                    } else {
                        // Check if this was the last nearest to check
                        if i == nearests.len() - 1 {
                            // All nearests are used, need to increase the number to check
                            number_of_nearests_to_check += 3;
                        }

                        // Continue to next nearest
                        continue;
                    }
                }
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