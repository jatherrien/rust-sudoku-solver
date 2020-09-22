use std::str::FromStr;

use sudoku_solver::grid::Grid;
use sudoku_solver::solver::solve_grid;


fn main() {
    let mut debug = false;
    let mut filename = String::new();
    { // this block limits scope of borrows by ap.refer() method
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("Solve Sudoku puzzles");
        ap.refer(&mut debug)
            .add_option(&["--debug"], argparse::StoreTrue, "Run in debug mode");

        ap.refer(&mut filename)
            .required()
            .add_argument("filename", argparse::Store, "Path to puzzle CSV file");

        ap.parse_args_or_exit();
    }

    if debug {
        unsafe {
            sudoku_solver::grid::DEBUG = true;
            sudoku_solver::solver::DEBUG = true;
        }
    }


    let mut grid = match read_grid(&filename) {
        Ok(grid) => grid,
        Err(e) => {
            eprintln!("Error while reading grid: \"{}\"", e);
            std::process::exit(1);
        }
    };

    println!("Grid to be solved:\n{}", grid);

    println!("Solving grid");
    solve_grid(&mut grid);

    println!("Solved grid:\n{}", grid);


}

fn read_grid(filename: &str) -> Result<Grid, String> {
    let reader_result = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(filename);

    let mut reader = match reader_result {
        Ok(reader) => reader,
        Err(e) => {
            let e = e.to_string();
            return Err(e);
        }
    };

    let grid = Grid::new();
    let mut row = 0;
    for result in reader.records() {
        if row > 8 {
            return Err("Hit row limit".to_string());
        }

        let record = result.unwrap();

        for column in 0..9 {
            let value = record.get(column);
            match value {
                Some(x) => {
                    let digit_result = u8::from_str(x);
                    match digit_result {
                        Ok(digit) => {
                            if digit > 0 {
                                grid.get(row, column).unwrap().set(digit);
                            }

                        },
                        Err(_error) => {return Err("Invalid cell value".to_string())}
                    };

                },
                None => {}
            }
        }

        row = row + 1;

    }

    return Ok(grid);
}
