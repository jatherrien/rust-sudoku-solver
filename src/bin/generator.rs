use rand_chacha::ChaCha8Rng;
use rand::prelude::*;
use sudoku_solver::grid::{Grid, CellValue};
use std::error::Error;
use std::io::Write;

fn main() {

    let mut debug = false;
    // Starting default seed will just be based on time
    let mut seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("Time went backwards").as_secs();

    let mut max_hints = 81;
    let mut max_attempts = 100;
    let mut filename : Option<String> = None;

    { // this block limits scope of borrows by ap.refer() method
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("Generate Sudoku puzzles");
        ap.refer(&mut debug)
            .add_option(&["--debug"], argparse::StoreTrue, "Run in debug mode");

        ap.refer(&mut seed)
            .add_option(&["--seed"], argparse::Store, "Provide seed for puzzle generation");

        ap.refer(&mut max_hints)
            .add_option(&["--hints"], argparse::Store, "Only return a puzzle with less than or equal to this number of hints");

        ap.refer(&mut max_attempts)
            .add_option(&["--attempts"], argparse::Store, "Number of attempts that will be tried to generate such a puzzle; default is 100");

        ap.refer(&mut filename)
            .add_argument("filename", argparse::StoreOption, "Optional filename to store puzzle in as a CSV");

        ap.parse_args_or_exit();
    }

    if debug {
        unsafe {
            sudoku_solver::grid::DEBUG = true;
            sudoku_solver::solver::DEBUG = true;
            sudoku_solver::generator::DEBUG = true;
        }
    }

    if debug {
        println!("Using seed {}", seed);
    }
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    let mut num_attempts = 0;

    let grid = loop {
        if num_attempts >= max_attempts{
            println!("Unable to find a puzzle with only {} hints in {} attempts", max_hints, max_attempts);
            return;
        }

        let (grid, num_hints) = sudoku_solver::generator::generate_grid(&mut rng);
        num_attempts = num_attempts + 1;

        if num_hints <= max_hints {
            println!("{}", grid);
            println!("Puzzle has {} hints", num_hints);
            if num_attempts > 1 {
                println!("It took {} attempts to find this puzzle.", num_attempts);
            }
            break grid;
        }
    };

    match filename {
        Some(filename) => {
            save_grid(&grid, &filename).unwrap();
            println!("Grid saved to {}", filename);
        },
        None => {}
    }

}

fn save_grid(grid: &Grid, filename: &str) -> Result<(), Box<dyn Error>>{
    // Not using the csv crate for writing because it's being difficult and won't accept raw integers
    let mut file = std::fs::File::create(filename)?;

    for x in 0..9 {
        for y in 0..9 {
            let cell = grid.get(x, y).unwrap();
            let value = &*cell.value.borrow();
            let digit =
            match value {
                CellValue::Fixed(digit) => {*digit}
                CellValue::Unknown(_) => {0}
            };

            let mut text = digit.to_string();
            if y < 8 {
                text.push(',');
            }
            file.write(text.as_bytes())?;

        }
        file.write(b"\n")?;
    }

    Ok(())
}