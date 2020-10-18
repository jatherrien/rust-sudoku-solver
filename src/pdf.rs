use crate::grid::{CellValue, Grid};
use printpdf::*;
use std::fs::File;
use std::io::BufWriter;

const BOTTOM_LEFT_X: f64 = 10.0;
const BOTTOM_LEFT_Y: f64 = 279.0 - 200.0 - 10.0;
const GRID_DIMENSION: f64 = 190.0;

const A4: (Mm, Mm) = (Mm(215.0), Mm(279.0));

pub fn draw_grid(grid: &Grid, filename: &str, print_possibilities: bool) -> Result<(), Box<dyn std::error::Error>> {
    let (doc, page1, layer1) = PdfDocument::new("Sudoku Puzzle", A4.0, A4.1, "Layer 1");
    let layer = doc.get_page(page1).get_layer(layer1);

    let font = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;
    let fixed_value_font_size = 45;
    let possibility_font_size = 12;

    draw_empty_grid(&layer);

    // x represents position on left-right scale
    // y represents position on up-down scale

    // Now need to add any values
    // One thing to note - higher y values are associated with the top of the page, while for my grid
    //   higher row values are associated with the bottom of the page.

    let x_offset = 6.1;
    let y_offset = -16.5;

    for r in 0..9 {
        let y = Mm(BOTTOM_LEFT_Y + (GRID_DIMENSION / 9.0) * (9.0 - r as f64) + y_offset);

        for c in 0..9 {
            let x = Mm(BOTTOM_LEFT_X + (GRID_DIMENSION / 9.0) * (c as f64) + x_offset);

            let cell = grid.get(r, c).unwrap();
            let value = &*cell.value.borrow();
            match value {
                CellValue::Fixed(digit) => {
                    let text = digit.to_string();
                    layer.use_text(text, fixed_value_font_size, x, y, &font);
                }
                CellValue::Unknown(possibilities) => {
                    if print_possibilities {
                        for (_, possibility) in possibilities.iter().enumerate() {

                            let sub_row = (possibility - 1) / 3;
                            let sub_column = (possibility - 1) % 3;
                            // Need to adjust x & y
                            let x = Mm(x.0 - 4.0 + (GRID_DIMENSION / 27.0) * (sub_column as f64));
                            let y = Mm(y.0 - 9.5 + (GRID_DIMENSION / 27.0) * (3.0 - sub_row as f64));

                            let text = possibility.to_string();
                            layer.use_text(text, possibility_font_size, x, y, &font);
                        }
                    }
                }
            }
        }
    }

    doc.save(&mut BufWriter::new(File::create(filename)?))?;

    return Ok(());
}

fn draw_empty_grid(layer: &PdfLayerReference) {
    // x represents position on left-right scale
    // y represents position on up-down scale

    // Thick lines first

    layer.set_outline_thickness(2.0);
    // Horizontal first
    {
        let starting_x = Mm(BOTTOM_LEFT_X);
        let ending_x = Mm(BOTTOM_LEFT_X + GRID_DIMENSION);
        let y_increment = GRID_DIMENSION / 3.0;
        for i in 0..4 {
            let y = Mm(BOTTOM_LEFT_Y + (i as f64) * y_increment);
            draw_line(layer, Point::new(starting_x, y), Point::new(ending_x, y));
        }
    }

    // Vertical lines next
    {
        let starting_y = Mm(BOTTOM_LEFT_Y);
        let ending_y = Mm(BOTTOM_LEFT_Y + GRID_DIMENSION);
        let x_increment = GRID_DIMENSION / 3.0;
        for i in 0..4 {
            let x = Mm(BOTTOM_LEFT_X + (i as f64) * x_increment);
            draw_line(layer, Point::new(x, starting_y), Point::new(x, ending_y));
        }
    }

    // Thin lines next

    layer.set_outline_thickness(0.0); // Special value to make line be 1px on all devices and zoom levels
                                      // Horizontal first
    {
        let starting_x = Mm(BOTTOM_LEFT_X);
        let ending_x = Mm(BOTTOM_LEFT_X + GRID_DIMENSION);
        let y_increment = GRID_DIMENSION / 9.0;
        for i in 1..9 {
            if i % 3 != 0 {
                let y = Mm(BOTTOM_LEFT_Y + (i as f64) * y_increment);
                draw_line(layer, Point::new(starting_x, y), Point::new(ending_x, y));
            }
        }
    }

    // Vertical lines next
    {
        let starting_y = Mm(BOTTOM_LEFT_Y);
        let ending_y = Mm(BOTTOM_LEFT_Y + GRID_DIMENSION);
        let x_increment = GRID_DIMENSION / 9.0;
        for i in 1..9 {
            if i % 3 != 0 {
                let x = Mm(BOTTOM_LEFT_X + (i as f64) * x_increment);
                draw_line(layer, Point::new(x, starting_y), Point::new(x, ending_y));
            }
        }
    }
}

fn draw_line(layer: &PdfLayerReference, point1: Point, point2: Point) {
    let points = vec![(point1, false), (point2, false)];

    let line = Line {
        points,
        is_closed: false,
        has_fill: false,
        has_stroke: true,
        is_clipping_path: false,
    };

    layer.add_shape(line);
}
