
fn main() {

    let mut debug = false;
    // Starting default seed will just be based on time
    let mut seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("Time went backwards").as_secs();

    { // this block limits scope of borrows by ap.refer() method
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("Generate Sudoku puzzles");
        ap.refer(&mut debug)
            .add_option(&["--debug"], argparse::StoreTrue, "Run in debug mode");

        ap.refer(&mut seed)
            .add_option(&["--seed"], argparse::Store, "Provide seed for puzzle generation");

        ap.parse_args_or_exit();
    }

    if debug {
        unsafe {
            sudoku_solver::grid::DEBUG = true;
            sudoku_solver::solver::DEBUG = true;
        }
    }

    if debug {
        println!("Using seed {}", seed);
    }

    let (grid, num_hints) = sudoku_solver::generator::generate_grid(seed);

    println!("{}", grid);
    println!("Puzzle has {} hints", num_hints);
}