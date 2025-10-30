mod consts;
mod utils;
mod minimizer;

use std::io::{self};
use inquire::{Select};

fn main() -> io::Result<()> {

    // List files in the GCODE_FOLDER directory
    let paths = std::fs::read_dir(consts::GCODE_FOLDER)?;
    let mut file_names = Vec::new();
    for path in paths {
        let path = path?;
        if let Some(ext) = path.path().extension() {
            if ext == "txt" {
                if let Some(name) = path.path().file_name() {
                    file_names.push(name.to_string_lossy().to_string());
                }
            }
        }
    }

    // Prompt user to select a file
    let file_name = Select::new("Select a G-code file:", file_names).prompt().unwrap();
    let file_path = format!("{}/{}", consts::GCODE_FOLDER, file_name);
    let gcode = utils::parse_gcode_file(file_path)?;

    // Print parsed G-code data for debugging
    println!("Parsed G-code {} with {} layers", file_name, gcode.num_layers);

    // Print the first segment of each 10th layer as a sample
    for layer in gcode.layers.iter().step_by(50) {
        println!("Layer ID: {}, First Segment: {:?}", layer.id, layer.segments.first());
    }

    // Get the printing speed
    println!("Printing speed is set to {} mm/s", consts::PRINTING_SPEED_MM_PER_SEC);

    let printing_time = utils::get_print_time_minutes(&gcode, consts::PRINTING_SPEED_MM_PER_SEC);
    println!("Estimated printing time: {}h {}min", printing_time as u64 / 60, printing_time as u64 % 60);

    // Minimize the G-code
    let minimized_gcode = minimizer::minimize_gcode(&gcode);

    // Print minimized G-code data for debugging
    println!("Minimized G-code has {} layers", minimized_gcode.num_layers);
    for layer in minimized_gcode.layers.iter().step_by(50) {
        println!("Layer ID: {}, First Segment: {:?}", layer.id, layer.segments.first());
    }

    // Get the printing time for minimized G-code
    let minimized_printing_time = utils::get_print_time_minutes(&minimized_gcode, consts::PRINTING_SPEED_MM_PER_SEC);
    println!("Estimated printing time after minimization: {}h {}min", minimized_printing_time as u64 / 60, minimized_printing_time as u64 % 60);
    println!("Time saved: {}min, which is approximately {:.2}%",
        (printing_time - minimized_printing_time) as u64,
        ((printing_time - minimized_printing_time) / printing_time) * 100.0
    );

    Ok(())
}
