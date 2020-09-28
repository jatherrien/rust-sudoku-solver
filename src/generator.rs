use crate::grid::{Cell, Grid, CellValue, Section};
use crate::solver::{SolveStatus, SolveController, Uniqueness, evaluate_grid_with_solve_controller, SolveStatistics};
use std::rc::Rc;
use rand::prelude::*;

pub static mut DEBUG : bool = false;

impl Grid {
    fn get_random_empty_cell(&self, rng : &mut SmallRng) -> Result<Rc<Cell>, &str> {
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
        fn eliminate_possibilities(possibilities: &mut Vec<u8>, line: &Section, cell: &Cell){
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

impl Section {
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

pub fn generate_grid(rng: &mut SmallRng, solve_controller: &SolveController) -> (Grid, i32, SolveStatistics) {

    let mut grid = generate_completed_grid(rng);
    let mut num_hints = 81;

    // We now trim down cells; first going to put them in a vector and shuffle them
    let mut non_empty_cells = Vec::new();
    for x in 0..9 {
        for y in 0..9 {
            let cell = grid.get(x, y).unwrap();
            non_empty_cells.push(Rc::clone(&cell));
        }
    }
    // Need to randomly reorder non_empty_cells
    non_empty_cells.shuffle(rng);

    let mut statistics_option = None;

    for (_index, cell) in non_empty_cells.iter().enumerate() {
        let mut grid_clone = grid.clone();
        let cell_clone = grid_clone.get(cell.x, cell.y).unwrap();
        let cell_clone = &*cell_clone;

        cell_clone.delete_value();


        let (status, statistics) = evaluate_grid_with_solve_controller(&mut grid_clone, solve_controller);
        match status {
            SolveStatus::Complete(uniqueness) => {
                let uniqueness = uniqueness.unwrap();
                match uniqueness {
                    Uniqueness::Unique => {
                        num_hints = num_hints - 1;
                        grid = grid_clone;
                    }
                    Uniqueness::NotUnique => continue // We can't remove this cell; continue onto the next one (note that grid hasn't been modified because of solve_controller)
                }
            }
            SolveStatus::Unfinished => panic!("evaluate_grid_with_solve_controller should never return UNFINISHED"),
            SolveStatus::Invalid => panic!("Removing constraints should not have set the # of solutions to zero")
        }
        statistics_option = Some(statistics);
    }

    return (grid, num_hints, statistics_option.unwrap());

}

// We generate a completed grid with no mind for difficulty; afterward generate_puzzle will take out as many fields as it can with regards to the difficulty
fn generate_completed_grid(rng: &mut SmallRng) -> Grid {
    let solve_controller = SolveController{
        determine_uniqueness: true,
        search_singles: true,
        search_hidden_singles: true,
        find_possibility_groups: true,
        search_useful_constraint: true,
        make_guesses: true
    };

    let mut grid : Grid = loop {
        // First step; randomly assign 8 different digits to different empty cells and see if there's a possible solution
        // We have to ensure that 8 of the digits appear at least once, otherwise the solution can't be unique because you could interchange the two missing digits throughout the puzzle
        // We do this in a loop so that if we are really unlucky and our guesses stop there from being any solution, we can easily re-run it
        let grid = Grid::new();

        let digit_excluded = rng.gen_range(1, 10);

        for digit in 1..10 {
            if digit != digit_excluded {
                let cell = grid.get_random_empty_cell(rng);
                cell.unwrap().set(digit);
            }
        }

        let (status, _statistics) = evaluate_grid_with_solve_controller(&grid, &solve_controller);
        match status {
            SolveStatus::Complete(uniqueness) => {
                let uniqueness = uniqueness.unwrap();
                match uniqueness {
                    Uniqueness::Unique => {
                        eprintln!("Wow! A puzzle with only 8 guesses have been found");
                        return grid;
                    }
                    Uniqueness::NotUnique => {break grid;} // What we expect
                }
            }
            SolveStatus::Unfinished => {panic!("evaluate_grid_with_solve_controller should never return UNFINISHED if we are making guesses")}
            SolveStatus::Invalid => {continue;} // unlucky; try again
        }
    };

    // Alright, we now have a grid that we can start adding more guesses onto until we find a unique solution
    grid =
        'outer: loop {
            let cell = grid.get_random_empty_cell(rng).unwrap(); // We unwrap because if somehow we're filled each cell without finding a solution, that's reason for a panic
            let cell = &*cell;
            let mut cell_possibilities = cell.get_value_possibilities().expect("An empty cell has no possibilities");

            // Let's scramble the order
            cell_possibilities.shuffle(rng);

            for (_index, digit) in cell_possibilities.iter().enumerate() {

                let mut grid_clone = grid.clone();
                let cell = &*grid_clone.get(cell.x, cell.y).unwrap();

                cell.set(*digit);

                let (status, _statistics) = evaluate_grid_with_solve_controller(&mut grid_clone, &solve_controller);
                match status {
                    SolveStatus::Complete(uniqueness) => {
                        let uniqueness = uniqueness.unwrap();
                        match uniqueness {
                            Uniqueness::Unique => {break 'outer grid_clone;} // We're done!
                            Uniqueness::NotUnique => {// We need more guesses
                                grid = grid_clone;
                                continue 'outer;
                            }
                        }
                    }
                    SolveStatus::Unfinished => panic!("evaluate_grid_with_solve_controller should never return UNFINISHED if making guesses"),
                    SolveStatus::Invalid => continue // Try another guess
                }

            };

            // If we reach this point in the loop, then none of the possibilities for cell provided any solution
            // Which means something serious happened before in the solving process - reason for panic
            eprint!("No valid hints were found for puzzle\n{} at cell ({}, {})", grid, cell.x, cell.y);
            panic!("Unable to continue as puzzle is invalid");

        };

    crate::solver::solve_grid(&mut grid);

    return grid;
}


#[cfg(test)]
mod tests {
    use crate::grid::*;
    use crate::solver::{solve_grid_with_solve_controller, SolveController, Uniqueness, SolveStatus, SolveStatistics};
    use crate::generator::generate_grid;
    use rand_chacha::SmallRng;
    use rand_chacha::rand_core::SeedableRng;

    #[test]
    fn test_unique_detection() {
        // A puzzle was generated that didn't actually have a unique solution; this is to make sure that the
        // modified solving code can actually detect this case
        let mut grid = Grid::new();

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

        let status = solve_grid_with_solve_controller(&mut grid, &SolveController{
            determine_uniqueness: true,
            search_singles: true,
            search_hidden_singles: true,
            find_possibility_groups: true,
            search_useful_constraint: true,
            make_guesses: true
        }, &mut SolveStatistics::new());

        assert_eq!(status, SolveStatus::Complete(Some(Uniqueness::NotUnique)));

    }

    // There was a bug where even though mutate_grid was set to false, the end result was still solved
    #[test]
    fn ensure_grid_not_complete(){
        let solve_controller = SolveController{
            determine_uniqueness: true,
            search_singles: true,
            search_hidden_singles: true,
            find_possibility_groups: true,
            search_useful_constraint: true,
            make_guesses: true
        };

        // Note that the puzzle itself doesn't matter
        let (grid, _num_hints, _statistics) = generate_grid(&mut SmallRng::seed_from_u64(123), &solve_controller);

        let mut observed_empty_cell = false;
        'outer : for x in 0..9 {
            for y in 0..9 {
                let cell = grid.get(x, y).unwrap();
                let value = cell.get_value_copy();

                match value {
                    CellValue::Fixed(_) => {}
                    CellValue::Unknown(_) => {
                        observed_empty_cell = true;
                        break 'outer;
                    }
                }
            }
        }

        assert!(observed_empty_cell);

    }
}