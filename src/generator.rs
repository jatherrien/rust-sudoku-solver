use crate::grid::{Cell, Grid, CellValue};
use crate::solver::{solve_grid_no_guess, SolveStatus, find_smallest_cell};
use std::rc::Rc;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

// Extension of SolveStatus
pub enum GenerateStatus {
    UniqueSolution,
    Unfinished,
    NoSolution,
    NotUniqueSolution
}

impl GenerateStatus {
    fn increment(self, new_status : GenerateStatus) -> GenerateStatus {
        match self {
            GenerateStatus::UniqueSolution => {
                match new_status {
                    GenerateStatus::UniqueSolution => GenerateStatus::NotUniqueSolution, // We now have two completes, so the solutions are not unique
                    GenerateStatus::NoSolution => GenerateStatus::UniqueSolution, // We already have a complete, so no issue with another guess being invalid
                    GenerateStatus::Unfinished => {panic!("Should not have encountered an UNFINISHED status")},
                    GenerateStatus::NotUniqueSolution => GenerateStatus::NotUniqueSolution // That solver found multiple solutions so no need to keep checking
                }
            },
            GenerateStatus::Unfinished => {
                match new_status {
                    GenerateStatus::UniqueSolution => GenerateStatus::UniqueSolution,
                    GenerateStatus::NoSolution => GenerateStatus::Unfinished,
                    GenerateStatus::Unfinished => {panic!("Should not have encountered an UNFINISHED status")},
                    GenerateStatus::NotUniqueSolution => GenerateStatus::NotUniqueSolution // That solver found multiple solutions so no need to keep checking
                }
            },
            GenerateStatus::NotUniqueSolution => GenerateStatus::NotUniqueSolution,
            GenerateStatus::NoSolution => GenerateStatus::NoSolution // This guess didn't pan out
        }

    }
}

impl SolveStatus {
    fn map_to_generate_status(self) -> GenerateStatus {
        match self {
            SolveStatus::Complete => {GenerateStatus::UniqueSolution }
            SolveStatus::Unfinished => {GenerateStatus::Unfinished }
            SolveStatus::Invalid => {GenerateStatus::NoSolution }
        }
    }
}

impl Grid {
    fn get_random_empty_cell(&self, rng : &mut ChaCha8Rng) -> Result<Rc<Cell>, &str> {
        // Idea - put all empty cells into a vector and choose one at random
        // If vector is empty we return an error

        let mut empty_cells = Vec::new();
        for x in 0..9 {
            for y in 0..9 {
                let cell = self.get(x, y).unwrap();
                let add_cell = {
                    let cell_value = &*cell.value.borrow();
                    match cell_value { // May cause issues with borrow rules
                        CellValue::Fixed(_) => {false}
                        CellValue::Unknown(_) => {
                            true
                        }
                    }
                };
                if add_cell {
                    empty_cells.push(cell);
                }
            }
        }

        match empty_cells.iter().choose(rng) {
            Some(cell) => Ok(Rc::clone(cell)),
            None => Err("Unable to find an empty cell")
        }
    }
}

pub fn generate_grid(seed: u64) -> (Grid, i32) {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    let digit_excluded = rng.gen_range(1, 10);

    let mut num_hints = 0;
    let mut grid : Grid = loop {
        // First step; randomly assign 8 different digits to different empty cells and see if there's a possible solution
        // We have to ensure that 8 of the digits appear at least once, otherwise the solution can't be unique because you could interchange the two missing digits throughout the puzzle
        // We do this in a loop so that if we are really unlucky and our guesses stop there from being any solution, we can easily re-run it
        let mut grid = Grid::new();

        for digit in 1..10 {
            if digit != digit_excluded {
                let cell = grid.get_random_empty_cell(&mut rng);
                cell.unwrap().set(digit);
                num_hints = num_hints + 1;
            }
        }

        let status = solve_grid(&mut grid);
        match status {
            GenerateStatus::UniqueSolution => { // very surprising result, given that the smallest puzzles found have 14 guesses
                eprintln!("Wow! A puzzle with only 8 guesses have been found");
                return (grid, num_hints);
            }
            GenerateStatus::Unfinished => {panic!("solve_grid should never return UNFINISHED")}
            GenerateStatus::NoSolution => {continue;}
            GenerateStatus::NotUniqueSolution => {break grid;}
        };
    };

    // Alright, we now have a grid that we can start adding more guesses onto until we find a unique solution
    'outer: loop {
        num_hints = num_hints + 1;
        let cell = grid.get_random_empty_cell(&mut rng).unwrap(); // We unwrap because if somehow we're filled each cell without finding a solution, that's reason for a panic
        let cell = &*cell;
        let cell_possibilities = cell.get_value_possibilities().expect("An empty cell has no possibilities");

        // Let's scramble the order
        let cell_possibilities = cell_possibilities.iter().choose_multiple(&mut rng, cell_possibilities.len());

        for (_index, digit) in cell_possibilities.iter().enumerate() {
            if **digit == digit_excluded {
                continue;
            }

            let grid_clone = grid.clone();
            let cell = &*grid_clone.get(cell.x, cell.y).unwrap();

            cell.set(**digit);

            let status = solve_grid(&grid_clone);
            match status {
                GenerateStatus::UniqueSolution => { // We're done!
                    return (grid_clone, num_hints);
                }
                GenerateStatus::Unfinished => {
                    panic!("solve_grid should never return UNFINISHED")
                }
                GenerateStatus::NoSolution => { // Try another guess
                    continue;
                }
                GenerateStatus::NotUniqueSolution => { // We need more guesses
                    grid = grid_clone;
                    continue 'outer;
                }
            }

        }

        // If we reach this point in the loop, then none of the possibilities for cell provided any solution
        // Which means something serious happened before in the solving process - reason for panic
        //eprint!("No valid hints were found for puzzle\n{} at cell ({}, {})", grid, cell.x, cell.y);
        //panic!("Unable to continue as puzzle is invalid");
        num_hints = num_hints - 1;

    }

}

fn solve_grid(grid: &Grid) -> GenerateStatus{
    // Code is kind of messy so here it goes - solve_grid first tries to solve without any guesses
    // If that's not enough and a guess is required, then solve_grid_guess is called
    // solve_grid_guess runs through all the possibilities for the smallest cell, trying to solve them
    // through calling this function.
    // solve_grid_no_guess tries to solve without any guesses.

    let mut grid = grid.clone(); // We're generating a result and don't want to make changes to our input

    let mut status = solve_grid_no_guess(&mut grid).map_to_generate_status();
    status = match status {
        GenerateStatus::Unfinished => {
            solve_grid_guess(&mut grid)
        },
        _ => {status}
    };

    match status {
        GenerateStatus::Unfinished => panic!("solve_grid_guess should never return UNFINISHED"),
        _ => return status
    }
}

fn solve_grid_guess(grid: &Grid) -> GenerateStatus{
    let smallest_cell = find_smallest_cell(grid);
    let smallest_cell = match smallest_cell {
        Some(cell) => cell,
        None => return GenerateStatus::NoSolution
    };

    let possibilities = smallest_cell.get_value_possibilities().unwrap();

    let mut current_status = GenerateStatus::Unfinished;

    for (_index, &digit) in possibilities.iter().enumerate() {
        let mut grid_copy = grid.clone();
        grid_copy.get(smallest_cell.x, smallest_cell.y).unwrap().set(digit);
        let status = solve_grid(&mut grid_copy);
        current_status = current_status.increment(status);

        match current_status {
            GenerateStatus::NotUniqueSolution => return GenerateStatus::NotUniqueSolution, // We have our answer; return it
            GenerateStatus::UniqueSolution => {continue}, // Still looking to see if solution is unique
            GenerateStatus::NoSolution => {panic!("current_status should not be NO_SOLUTION at this point")},
            GenerateStatus::Unfinished => {continue} // Still looking for a solution
        }
    }

    // We've tried all the possibilities for this guess
    match current_status {
        GenerateStatus::NotUniqueSolution => return current_status,
        GenerateStatus::Unfinished => return GenerateStatus::NoSolution, // nothing panned out; last guess is a bust
        GenerateStatus::UniqueSolution => return current_status, // Hey! Looks good!
        GenerateStatus::NoSolution => {panic!("current_status should not be NO_SOLUTION at this point")}
    }

}