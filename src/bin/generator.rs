use rand_chacha::ChaCha8Rng;
use rand::prelude::*;
use sudoku_solver::grid::{Grid, CellValue};
use std::error::Error;
use std::io::Write;
use sudoku_solver::solver::{SolveController, SolveStatistics};
use std::str::FromStr;

#[derive(Clone)] // Needed for argparse
enum Difficulty {
    Hard,
    Medium,
    Easy
}

impl Difficulty {
    fn map_to_solve_controller(&self) -> SolveController {
        let mut controller = SolveController{
            determine_uniqueness: true,
            search_singles: true,
            search_hidden_singles: true,
            find_possibility_groups: true,
            search_useful_constraint: true,
            make_guesses: true
        };

        match self {
            Difficulty::Hard => {} // Do nothing, already hard
            Difficulty::Medium => {
                controller.make_guesses = false;
            },
            Difficulty::Easy => {
                controller.make_guesses = false;
                controller.search_useful_constraint = false;
                controller.find_possibility_groups = false;
            }
        }

        controller
    }

    fn meets_minimum_requirements(&self, solve_statistics: &SolveStatistics) -> bool {
        match self {
            Difficulty::Hard => {
                (solve_statistics.guesses > 0) && (solve_statistics.possibility_groups > 10) && (solve_statistics.useful_constraints > 10)
            }
            Difficulty::Medium => {
                (solve_statistics.possibility_groups > 10) && (solve_statistics.useful_constraints > 10)
            }
            Difficulty::Easy => {true} // easy has no minimum
        }
    }
}

impl FromStr for Difficulty { // Needed for argparse
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {

        if s.eq_ignore_ascii_case("EASY"){
            return Ok(Difficulty::Easy);
        } else if s.eq_ignore_ascii_case("MEDIUM"){
            return Ok(Difficulty::Medium);
        } else if s.eq_ignore_ascii_case("HARD"){
            return Ok(Difficulty::Hard);
        }

        return Err(format!("{} is not a valid difficulty", s));
    }
}

fn main() {

    let mut debug = false;
    // Starting default seed will just be based on time
    let mut seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("Time went backwards").as_secs();

    let mut max_hints = 81;
    let mut max_attempts = 100;
    let mut filename : Option<String> = None;
    let mut difficulty = Difficulty::Hard;

    { // this block limits scope of borrows by ap.refer() method
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("Generate Sudoku puzzles");
        ap.refer(&mut debug)
            .add_option(&["--debug"], argparse::StoreTrue, "Run in debug mode");

        ap.refer(&mut seed)
            .add_option(&["-s", "--seed"], argparse::Store, "Provide seed for puzzle generation");

        ap.refer(&mut max_hints)
            .add_option(&["--hints"], argparse::Store, "Only return a puzzle with less than or equal to this number of hints");

        ap.refer(&mut max_attempts)
            .add_option(&["--attempts"], argparse::Store, "Number of attempts that will be tried to generate such a puzzle; default is 100");

        ap.refer(&mut filename)
            .add_argument("filename", argparse::StoreOption, "Optional filename to store puzzle in as a CSV");

        ap.refer(&mut difficulty)
            .add_option(&["-d", "--difficulty"], argparse::Store, "Max difficulty setting; values are EASY, MEDIUM, or HARD");

        ap.parse_args_or_exit();
    }

    /*
    if debug {
        unsafe {
            sudoku_solver::grid::DEBUG = true;
            sudoku_solver::solver::DEBUG = true;
            sudoku_solver::generator::DEBUG = true;
        }
    }
    */


    if debug {
        println!("Using seed {}", seed);
    }

    let solve_controller = difficulty.map_to_solve_controller();


    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    let mut num_attempts = 0;

    let (grid, solve_statistics) = loop {
        if num_attempts >= max_attempts{
            println!("Unable to find a puzzle with only {} hints in {} attempts", max_hints, max_attempts);
            return;
        }

        let (grid, num_hints, solve_statistics) = sudoku_solver::generator::generate_grid(&mut rng, &solve_controller);
        num_attempts = num_attempts + 1;

        if difficulty.meets_minimum_requirements(&solve_statistics) && num_hints <= max_hints {
            println!("{}", grid);
            println!("Puzzle has {} hints", num_hints);
            if num_attempts > 1 {
                println!("It took {} attempts to find this puzzle.", num_attempts);
            }
            break (grid, solve_statistics);
        }
    };

    if debug {
        println!("Solving this puzzle involves roughly:");
        println!("\t{} SINGLE actions", solve_statistics.singles);
        println!("\t{} HIDDEN_SINGLE actions", solve_statistics.hidden_singles);
        println!("\t{} USEFUL_CONSTRAINT actions", solve_statistics.useful_constraints);
        println!("\t{} POSSIBILITY_GROUP actions", solve_statistics.possibility_groups);
        println!("\t{} GUESS actions", solve_statistics.guesses);
    }

    match filename {
        Some(filename) => {
            // check if we save to a csv or a pdf
            if filename.ends_with(".pdf") {
                sudoku_solver::pdf::draw_grid(&grid, &filename).unwrap();
                println!("Grid saved as pdf to {}", filename);
            } else{
                save_grid_csv(&grid, &filename).unwrap();
                println!("Grid saved as CSV to {}", filename);
            }
        },
        None => {}
    }

}

fn save_grid_csv(grid: &Grid, filename: &str) -> Result<(), Box<dyn Error>>{
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
