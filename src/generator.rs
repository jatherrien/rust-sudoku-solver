use crate::grid::{Cell, Grid, CellValue, Line};
use crate::solver::{solve_grid_no_guess, SolveStatus, find_smallest_cell};
use std::rc::Rc;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

pub static mut DEBUG : bool = false;

// Extension of SolveStatus
#[derive(Debug, Eq, PartialEq)]
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

impl Cell {
    fn delete_value(&self){
        unsafe {
            if DEBUG {
                println!("Cell {}, {} had its value deleted.", self.x, self.y);
            }
        }

        self.set_value_exact(CellValue::Unknown(vec![])); // placeholder

        // This will reset all the possibilities for this cell and the ones that might have been limited by this cell
        self.section.upgrade().unwrap().borrow().recalculate_and_set_possibilities();
        self.row.upgrade().unwrap().borrow().recalculate_and_set_possibilities();
        self.column.upgrade().unwrap().borrow().recalculate_and_set_possibilities();

    }

    /**
        As part of delete_value, we need to manually recalculate possibilities for not just the cell whose value we deleted,
        but also the other empty cells in the same row, column, and section.
    */
    fn calculate_possibilities(&self) -> Vec<u8> {
        // Need to calculate possibilities for this cell
        let mut possibilities = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        fn eliminate_possibilities(possibilities: &mut Vec<u8>, line: &Line, cell: &Cell){
            for (_index, other) in line.vec.iter().enumerate(){
                if other.x != cell.x || other.y != cell.y {
                    let value = &*other.value.borrow();
                    match value {
                        CellValue::Fixed(digit) => {
                            let location = possibilities.binary_search(digit);
                            match location {
                                Ok(location) => {
                                    possibilities.remove(location);
                                }
                                Err(_) => {}
                            }
                        }
                        CellValue::Unknown(_) => {}
                    }
                }
            }
        }

        eliminate_possibilities(&mut possibilities, &self.section.upgrade().unwrap().borrow(), self);
        eliminate_possibilities(&mut possibilities, &self.row.upgrade().unwrap().borrow(), self);
        eliminate_possibilities(&mut possibilities, &self.column.upgrade().unwrap().borrow(), self);

        return possibilities;
    }
}

impl Line {
    fn recalculate_and_set_possibilities(&self) {
        for (_index, cell) in self.vec.iter().enumerate() {
            let cell = &**cell;
            let new_possibilities = {
                let cell_value = &*cell.value.borrow();
                match cell_value {
                    CellValue::Fixed(_) => { continue; }
                    CellValue::Unknown(_) => {
                        cell.calculate_possibilities()
                    }
                }
            };

            cell.set_value_exact(CellValue::Unknown(new_possibilities));
        }
    }
}

pub fn generate_grid(rng: &mut ChaCha8Rng) -> (Grid, i32) {

    let mut num_hints;
    let mut grid : Grid = loop {
        // First step; randomly assign 8 different digits to different empty cells and see if there's a possible solution
        // We have to ensure that 8 of the digits appear at least once, otherwise the solution can't be unique because you could interchange the two missing digits throughout the puzzle
        // We do this in a loop so that if we are really unlucky and our guesses stop there from being any solution, we can easily re-run it
        let mut grid = Grid::new();
        num_hints = 0;

        let digit_excluded = rng.gen_range(1, 10);

        for digit in 1..10 {
            if digit != digit_excluded {
                let cell = grid.get_random_empty_cell(rng);
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
            GenerateStatus::NoSolution => {continue;} // unlucky; try again
            GenerateStatus::NotUniqueSolution => {break grid;}
        };
    };

    // Alright, we now have a grid that we can start adding more guesses onto until we find a unique solution
    grid =
    'outer: loop {
        num_hints = num_hints + 1;
        let cell = grid.get_random_empty_cell(rng).unwrap(); // We unwrap because if somehow we're filled each cell without finding a solution, that's reason for a panic
        let cell = &*cell;
        let mut cell_possibilities = cell.get_value_possibilities().expect("An empty cell has no possibilities");

        // Let's scramble the order
        cell_possibilities.shuffle(rng);

        for (_index, digit) in cell_possibilities.iter().enumerate() {

            let grid_clone = grid.clone();
            let cell = &*grid_clone.get(cell.x, cell.y).unwrap();

            cell.set(*digit);

            let status = solve_grid(&grid_clone);
            match status {
                GenerateStatus::UniqueSolution => { // We're done!
                    break 'outer grid_clone;
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

        };

        // If we reach this point in the loop, then none of the possibilities for cell provided any solution
        // Which means something serious happened before in the solving process - reason for panic
        eprint!("No valid hints were found for puzzle\n{} at cell ({}, {})", grid, cell.x, cell.y);
        panic!("Unable to continue as puzzle is invalid");


    };

    // At this point we have a valid puzzle, but from experience it has way too many guesses, and many of them
    // are likely not needed. Let's now try removing a bunch.
    let mut non_empty_cells = Vec::new();
    for x in 0..9 {
        for y in 0..9 {
            let cell = grid.get(x, y).unwrap();
            let value = &*cell.value.borrow();
            match value {
                CellValue::Fixed(_) => {non_empty_cells.push(Rc::clone(&cell))}
                CellValue::Unknown(_) => {}
            }
        }
    }
    // Need to randomly reorder non_empty_cells
    non_empty_cells.shuffle(rng);

    for (_index, cell) in non_empty_cells.iter().enumerate() {
        let mut grid_clone = grid.clone();
        let cell_clone = grid_clone.get(cell.x, cell.y).unwrap();
        let cell_clone = &*cell_clone;

        cell_clone.delete_value();

        let status = solve_grid(&mut grid_clone);
        match status {
            GenerateStatus::UniqueSolution => { // great; that cell value was not needed
                num_hints = num_hints - 1;
                grid = grid_clone;

            }
            GenerateStatus::Unfinished => {panic!("solve_grid should never return UNFINISHED")}
            GenerateStatus::NoSolution => {panic!("Removing constraints should not have set the # of solutions to zero")}
            GenerateStatus::NotUniqueSolution => {continue;} // We can't remove this cell; continue onto the next one (note that grid hasn't been modified)
        };
    }

    return (grid, num_hints);

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


#[cfg(test)]
mod tests {
    use crate::grid::*;
    use crate::generator::{solve_grid, GenerateStatus};

    #[test]
    fn test_unique_detection() {
        // A puzzle was generated that didn't actually have a unique solution; this is to make sure that the
        // modified solving code can actually detect this case
        let grid = Grid::new();

        grid.get(0, 0).unwrap().set(9);
        grid.get(0, 7).unwrap().set(4);

        grid.get(1, 3).unwrap().set(5);
        grid.get(1, 6).unwrap().set(8);

        grid.get(2, 0).unwrap().set(2);
        grid.get(2, 1).unwrap().set(4);
        grid.get(2, 4).unwrap().set(7);

        grid.get(3, 2).unwrap().set(8);
        grid.get(3, 4).unwrap().set(2);
        grid.get(3, 6).unwrap().set(9);

        grid.get(4, 3).unwrap().set(6);
        grid.get(4, 7).unwrap().set(7);

        grid.get(5, 5).unwrap().set(5);
        grid.get(5, 8).unwrap().set(1);

        grid.get(6, 0).unwrap().set(3);
        grid.get(6, 4).unwrap().set(8);
        grid.get(6, 6).unwrap().set(4);
        grid.get(6, 8).unwrap().set(7);

        grid.get(7, 0).unwrap().set(7);
        grid.get(7, 4).unwrap().set(1);
        grid.get(7, 5).unwrap().set(9);
        grid.get(7, 6).unwrap().set(2);

        grid.get(8, 2).unwrap().set(6);

        let status = solve_grid(&grid);

        assert_eq!(status, GenerateStatus::NotUniqueSolution);

    }
}