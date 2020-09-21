use std::str::FromStr;

use sudoku_solver::grid::Grid;
use sudoku_solver::solver::solve_grid;


fn main() {
    let mut grid = read_grid().unwrap();

    println!("{}", grid);

    println!("Solving grid");
    solve_grid(&mut grid);
    println!("{}", grid);
}

fn read_grid() -> Result<Grid, &'static str> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(std::io::stdin());
    let grid = Grid::new();
    let mut row = 0;
    for result in reader.records() {
        if row > 8 {
            return Err("Hit row limit");
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
                        Err(_error) => {return Err("Invalid cell value")}
                    };

                },
                None => {}
            }
        }

        row = row + 1;

    }

    return Ok(grid);
}
