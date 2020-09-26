use rand::prelude::*;
use sudoku_solver::grid::{Grid, CellValue};
use std::error::Error;
use std::io::Write;
use sudoku_solver::solver::{SolveController, SolveStatistics};
use std::str::FromStr;
use std::sync::{mpsc};
use std::process::exit;
use std::thread;

/*
We have to be very careful here because Grid contains lots of Rcs and RefCells which could enable mutability
across multiple threads (with Rcs specifically even just counting the number of active references to the object
involves mutability of the Rc itself). In my specific case with the generator here I know that all those Rcs
and RefCells are fully encapsulated in the one Grid object I'm Sending and will never be accessed again from the thread
that sent them after it's been Sent, so it's safe in this narrowly specific context.
*/
struct SafeGridWrapper(Grid);
unsafe impl Send for SafeGridWrapper {}

#[derive(Clone, Copy)] // Needed for argparse
enum Difficulty {
    Challenge,
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
            Difficulty::Challenge => {}, // Do nothing, already hard
            Difficulty::Hard => {
                controller.make_guesses = false;
            }
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
            Difficulty::Challenge => {
                (solve_statistics.guesses > 0) && (solve_statistics.possibility_groups > 20) && (solve_statistics.useful_constraints > 20)
            }
            Difficulty::Hard => {
                (solve_statistics.possibility_groups > 20) && (solve_statistics.useful_constraints > 20)
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
        } else if s.eq_ignore_ascii_case("CHALLENGE"){
            return Ok(Difficulty::Challenge);
        }

        return Err(format!("{} is not a valid difficulty", s));
    }
}

fn main() {

    let mut debug = false;
    let mut max_hints = 81;
    let mut max_attempts = 100;
    let mut filename : Option<String> = None;
    let mut difficulty = Difficulty::Challenge;
    let mut threads = 1;

    { // this block limits scope of borrows by ap.refer() method
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("Generate Sudoku puzzles");
        ap.refer(&mut debug)
            .add_option(&["--debug"], argparse::StoreTrue, "Run in debug mode");

        ap.refer(&mut max_hints)
            .add_option(&["--hints"], argparse::Store, "Only return a puzzle with less than or equal to this number of hints");

        ap.refer(&mut max_attempts)
            .add_option(&["--attempts"], argparse::Store, "Number of puzzles each thread will generate to find an appropriate puzzle; default is 100");

        ap.refer(&mut filename)
            .add_argument("filename", argparse::StoreOption, "Optional filename to store puzzle in as a CSV");

        ap.refer(&mut difficulty)
            .add_option(&["-d", "--difficulty"], argparse::Store, "Max difficulty setting; values are EASY, MEDIUM, HARD, or CHALLENGE");

        ap.refer(&mut threads)
            .add_option(&["--threads"], argparse::Store, "Number of threads to use when generating possible puzzles");

        ap.parse_args_or_exit();
    }

    let solve_controller = difficulty.map_to_solve_controller();

    let (grid, solve_statistics, num_hints) =
    if threads < 1 {
        eprintln!("--threads must be at least 1");
        exit(1);
    } else if threads == 1 {
        let mut rng = SmallRng::from_entropy();
        let result = get_puzzle_matching_conditions(&mut rng, &difficulty, &solve_controller, max_attempts, max_hints);
        match result {
            Some(x) => x,
            None => {
                eprintln!("Unable to find an appropriate puzzle in the required amount of attempts");
                exit(1);
            }
        }
    } else {
        let mut thread_rng = thread_rng();
        let (transmitter, receiver) = mpsc::channel();

        for _i in 0..threads {
            let cloned_transmitter = mpsc::Sender::clone(&transmitter);
            let mut rng = SmallRng::from_rng(&mut thread_rng).unwrap();

            thread::spawn(move || {
                if debug {
                    println!("Thread spawned");
                }

                let result = get_puzzle_matching_conditions(&mut rng, &difficulty, &solve_controller, max_attempts, max_hints);
                match result {
                    Some((grid, solve_statistics, num_hints)) => {
                        cloned_transmitter.send((SafeGridWrapper(grid), solve_statistics, num_hints)).unwrap();
                    },
                    None => {}
                };

                if debug {
                    println!("Thread terminated");
                }
            });
        }

        // TODO - fix bug where recv doesn't return if no Grid is found by any threads!
        match receiver.recv() {
            Ok((grid, solve_statistics, num_hints)) => (grid.0, solve_statistics, num_hints),
            Err(e) => {
                eprintln!("Unable to find an appropriate puzzle in the required amount of attempts");
                if debug {
                    eprintln!("Error returned: {:?}", e);
                }

                exit(1);
            }
        }
    };


    println!("{}", grid);
    println!("Puzzle has {} hints", num_hints);

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

fn get_puzzle_matching_conditions(rng: &mut SmallRng, difficulty: &Difficulty, solve_controller: &SolveController, max_attempts: i32, max_hints: i32) -> Option<(Grid, SolveStatistics, i32)>{
    let mut num_attempts = 0;

    loop {
        if num_attempts >= max_attempts {
            return None;
        }

        let (grid, num_hints, solve_statistics) = sudoku_solver::generator::generate_grid(rng, &solve_controller);
        num_attempts += 1;

        if difficulty.meets_minimum_requirements(&solve_statistics) && num_hints <= max_hints {
            return Some((grid, solve_statistics, num_hints));
        }
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
