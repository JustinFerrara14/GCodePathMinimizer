mod consts;
mod utils;
mod minimizer;
mod tests;

use std::io::{self};
use inquire::{Select};

use utils::*;

fn main() -> io::Result<()> {

    let num_layers_to_print = 5;
    let num_segments_to_print = 5;

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
    let mut gcode = parse_gcode_file(file_path.clone())?;

    // Print parsed G-code data for debugging
    gcode.print_gcode(num_layers_to_print, num_segments_to_print);

    // Get the printing speed
    println!("Printing speed is set to {} mm/s", consts::PRINTING_SPEED_MM_PER_SEC);

    let printing_time = gcode.get_print_time_minutes(consts::PRINTING_SPEED_MM_PER_SEC);
    println!("Estimated printing time: {}h {}min", printing_time as u64 / 60, printing_time as u64 % 60);

    // Minimize the G-code
    minimizer::minimize_gcode(&mut gcode);

    // Print minimized G-code data for debugging
    gcode.print_gcode(num_layers_to_print, num_segments_to_print);

    // Get the printing time for minimized G-code
    let minimized_printing_time = gcode.get_print_time_minutes(consts::PRINTING_SPEED_MM_PER_SEC);
    println!("Estimated printing time after minimization: {}h {}min", minimized_printing_time as u64 / 60, minimized_printing_time as u64 % 60);
    println!("Time saved: {}min, which is approximately {:.2}%",
        (printing_time - minimized_printing_time) as u64,
        ((printing_time - minimized_printing_time) / printing_time) * 100.0
    );

    let original_gcode = parse_gcode_file(file_path)?;
    let equality = original_gcode.test_gcode_equality(&gcode);
    if equality {
        println!("The original and minimized G-code are functionally equivalent.");
    } else {
        eprintln!("The original and minimized G-code are NOT functionally equivalent !!!!!!!!!!!");
    }

    Ok(())
}
