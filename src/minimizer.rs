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

        // Create a list to hold the ordered segments
        let mut ordered_segments = Vec::new();
        let remaining_segments: Vec<Segment> = gcode.layers[i].segments.clone();
        let mut remaining_indices: HashSet<usize> = (0..remaining_segments.len()).collect();

        // Create kd-tree for the segments in the layer
        let mut kdtree: KdTree<_, _, 2, KDTREE_BUCKET_SIZE, u32> = KdTree::with_capacity(remaining_segments.len() * 2);

        for (i, s) in remaining_segments.iter().enumerate() {
            kdtree.add(&[s.x1 as f64, s.y1 as f64], i as u64);
            kdtree.add(&[s.x2 as f64, s.y2 as f64], i as u64);
        }

        // Default to first segment in the layer
        let mut first_segment_id: usize = 0;
        let mut reversed_first_segment = false;
        if i > 0 {
            // For other layers, the first segment is the one closest to the last segment of the previous layer
            let previous_layer = &gcode.layers[i - 1];
            if let Some(last_segment_prev_layer) = previous_layer.segments.last() {
                let last_point = [last_segment_prev_layer.x2 as f64, last_segment_prev_layer.y2 as f64];
                let nearest = kdtree.nearest_one::<SquaredEuclidean>(&last_point);

                // Find the nearest unused segment
                let nearest_index = nearest.item as usize;
                first_segment_id = nearest_index;
                if remaining_indices.contains(&nearest_index) {
                    let mut first_segment = remaining_segments[nearest_index].clone();
                    // Determine if we need to reverse the segment
                    if last_segment_prev_layer.length_with_other(&first_segment.reversed()) < last_segment_prev_layer.length_with_other(&first_segment) {
                        reversed_first_segment = true;
                    }
                }
            }
        }

        // Add the first segment to the ordered list
        if reversed_first_segment {
            ordered_segments.push(remaining_segments[first_segment_id].reversed());
        } else {
            ordered_segments.push(remaining_segments[first_segment_id].clone());
        }
        remaining_indices.remove(&first_segment_id);
        remove_segment_from_tree(&mut kdtree, &remaining_segments[first_segment_id], first_segment_id as u64);

        // From the first segment, find the next closest segment and add it to the ordered list
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
                    remove_segment_from_tree(&mut kdtree, &next_segment, nearest.item);
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

        // Replace the layer's segments with the ordered segments
        gcode.layers[i].segments = ordered_segments;

        // Update progress bar
        bar.inc(1);
    }

    // Finish progress bar
    bar.finish_with_message("Minimization complete");
}

fn remove_segment_from_tree(kdtree: &mut KdTree<f64, u64, 2, KDTREE_BUCKET_SIZE, u32>, segment: &Segment, item: u64) -> () {
    kdtree.remove(&[segment.x1 as f64, segment.y1 as f64], item);
    kdtree.remove(&[segment.x2 as f64, segment.y2 as f64], item);
}