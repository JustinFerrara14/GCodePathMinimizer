use crate::utils::*;

#[test]
fn test_gcode_len(){

    let gcode = GCodeData {
        num_layers: 2,
        layers: vec![
            Layer {
                id: 0,
                segments: vec![
                    Segment { x1: 0, y1: 0, x2: 10000, y2: 0 }, // 10
                    // Travel 10
                    Segment { x1: 20000, y1: 0, x2: 30000, y2: 0 }, // 10
                ],
            },
            Layer {
                id: 1,
                segments: vec![
                    // Travel 10
                    Segment { x1: 40000, y1: 0, x2: 40000, y2: 40000 }, // travel 40
                    Segment { x1: 40000, y1: 40000, x2: 0, y2: 0 }, // travel 56.57
                ],
            },
        ],
    };

    // Total length = 10 + 10 + 10 + 10 + 40 + 56.57 = 136.57 mm
    let time = gcode.get_print_time_minutes(1);

    // Allow small floating point error
    let expected_time = 136.569 / 60.0;
    let epsilon = 0.001;
    assert!((time - expected_time).abs() < epsilon, "Expected time: {}, got: {}", expected_time, time);
}

#[test]
fn test_gcode_equality_function() {
    let gcode_a = GCodeData {
        num_layers: 1,
        layers: vec![
            Layer {
                id: 0,
                segments: vec![
                    Segment { x1: 0, y1: 0, x2: 10000, y2: 0 },
                    Segment { x1: 10000, y1: 0, x2: 20000, y2: 0 },
                ],
            },
        ],
    };

    let gcode_a_inverted = GCodeData {
        num_layers: 1,
        layers: vec![
            Layer {
                id: 0,
                segments: vec![
                    Segment { x1: 10000, y1: 0, x2: 20000, y2: 0 },
                    Segment { x1: 0, y1: 0, x2: 10000, y2: 0 },
                ],
            },
        ],
    };

    let gcode_a_other_direction = GCodeData {
        num_layers: 1,
        layers: vec![
            Layer {
                id: 0,
                segments: vec![
                    Segment { x1: 0, y1: 0, x2: 10000, y2: 0 },
                    Segment { x1: 20000, y1: 0, x2: 10000, y2: 0 }, // inverted segment
                ],
            },
        ],
    };

    // Not equal
    let gcode_a_1_segment = GCodeData {
        num_layers: 1,
        layers: vec![
            Layer {
                id: 0,
                segments: vec![
                    Segment { x1: 0, y1: 0, x2: 10000, y2: 0 },
                ],
            },
        ],
    };

    let gcode_a_3_segment = GCodeData {
        num_layers: 1,
        layers: vec![
            Layer {
                id: 0,
                segments: vec![
                    Segment { x1: 0, y1: 0, x2: 10000, y2: 0 },
                    Segment { x1: 10000, y1: 0, x2: 20000, y2: 0 },
                    Segment { x1: 20000, y1: 0, x2: 30000, y2: 0 },
                ],
            },
        ],
    };

    let gcode_b = GCodeData {
        num_layers: 1,
        layers: vec![
            Layer {
                id: 0,
                segments: vec![
                    Segment { x1: 0, y1: 0, x2: 15000, y2: 0 },
                    Segment { x1: 15000, y1: 0, x2: 20000, y2: 0 },
                ],
            },
        ],
    };

    // Equal
    assert!(gcode_a.test_gcode_equality(&gcode_a));
    assert!(gcode_a.test_gcode_equality(&gcode_a_inverted));
    assert!(gcode_a.test_gcode_equality(&gcode_a_other_direction));
    assert!(gcode_a_inverted.test_gcode_equality(&gcode_a_other_direction));

    // Not equal
    assert!(!gcode_a.test_gcode_equality(&gcode_a_1_segment));
    assert!(!gcode_a.test_gcode_equality(&gcode_a_3_segment));
    assert!(!gcode_a.test_gcode_equality(&gcode_b));
}