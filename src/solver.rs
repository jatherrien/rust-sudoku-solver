
use std::rc::Rc;
use crate::grid::{Cell, Line, Grid, CellValue};

pub static mut DEBUG: bool = false;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Uniqueness {
    Unique,
    NotUnique
}

#[derive(Eq, PartialEq, Debug)]
pub enum SolveStatus {
    Complete(Option<Uniqueness>),
    Unfinished,
    Invalid
}


enum SolveAction{
    Single,
    HiddenSingle,
    PossibilityGroup,
    UsefulConstraints,
    Guess
}

impl SolveStatus {

    fn increment(self, additional_status : SolveStatus) -> SolveStatus {
        match self {
            SolveStatus::Complete(uniqueness_option) => {
                if uniqueness_option.is_none() {
                    return SolveStatus::Complete(None);
                } else {
                    match uniqueness_option.unwrap() {
                        Uniqueness::NotUnique => SolveStatus::Complete(Some(Uniqueness::NotUnique)),
                        Uniqueness::Unique => match additional_status {
                            SolveStatus::Complete(_) => SolveStatus::Complete(Some(Uniqueness::NotUnique)),
                            SolveStatus::Unfinished => SolveStatus::Complete(Some(Uniqueness::Unique)),
                            SolveStatus::Invalid => SolveStatus::Complete(Some(Uniqueness::Unique))
                        }
                    }
                }

            }
            SolveStatus::Unfinished => match additional_status {
                SolveStatus::Invalid => SolveStatus::Unfinished,
                _ => additional_status
            },
            SolveStatus::Invalid => panic!("increment() shouldn't be called on SolveStatus::Invalid")
        }
    }
}

pub struct SolveController {
    pub determine_uniqueness: bool,
    pub search_singles: bool,
    pub search_hidden_singles: bool,
    pub find_possibility_groups: bool,
    pub search_useful_constraint: bool,
    pub make_guesses: bool,
}

impl SolveController {
    fn determine_uniqueness(&self) -> bool {
        self.determine_uniqueness
    }

    fn search_singles(&self) -> bool {
        self.search_singles
    }

    fn search_hidden_singles(&self) -> bool {
        self.search_hidden_singles
    }

    fn find_possibility_groups(&self) -> bool {
        self.find_possibility_groups
    }

    fn search_useful_constraint(&self) -> bool {
        self.search_useful_constraint
    }

    fn make_guesses(&self) -> bool {
        self.make_guesses
    }
}

/**
    Tracks when we relied on a method to make progress. We'll consider 'relied on' to mean that the method make at least
    one change to the line it was originally called on, whether that be setting a value or adjusting the possibilities in a cell.
*/
#[derive(Copy, Clone)]
pub struct SolveStatistics {
    pub singles: u32,
    pub hidden_singles: u32,
    pub possibility_groups: u32,
    pub useful_constraints: u32,
    pub guesses: u32
}

impl SolveStatistics {
    pub(crate) fn new() -> SolveStatistics {
        SolveStatistics{
            singles: 0,
            hidden_singles: 0,
            possibility_groups: 0,
            useful_constraints: 0,
            guesses: 0
        }
    }

    fn increment(&mut self, action: &SolveAction) {
        match action {
            SolveAction::Single => {self.singles = self.singles + 1}
            SolveAction::HiddenSingle => {self.hidden_singles = self.hidden_singles + 1}
            SolveAction::PossibilityGroup => {self.possibility_groups = self.possibility_groups + 1}
            SolveAction::UsefulConstraints => {self.useful_constraints = self.useful_constraints + 1}
            SolveAction::Guess => {self.guesses = self.guesses + 1}
        }
    }
}

pub fn find_smallest_cell(grid: &Grid) -> Option<Rc<Cell>>{
    // Find a cell of smallest size (in terms of possibilities) and make a guess
    // Can assume that no cells of only possibility 1 exist

    let mut smallest_cell : Option<Rc<Cell>> = None;
    let mut smallest_size = usize::MAX;

    'outer: for x in 0..9 {
        for y in 0..9 {
            let cell_rc = grid.get(x, y).unwrap();
            let cell = &*grid.get(x, y).unwrap();
            let cell_value = &*cell.value.borrow();

            match cell_value {
                CellValue::Unknown(possibilities) => {
                    if (possibilities.len() < smallest_size) && (possibilities.len() > 0){
                        smallest_size = possibilities.len();
                        smallest_cell = Some(cell_rc);
                    }
                },
                _ => {}
            }

            if smallest_size <= 2 {
                break 'outer; // We aren't going to get smaller
            }

        }
    }
    smallest_cell

}


// Code for identify_and_process_possibility_groups (it uses it's own structs)
mod process_possibility_groups {
    use crate::grid::{Line, CellValue};
    use std::collections::HashSet;
    use std::rc::Rc;

    struct FauxCell{
        index: usize,
        possibilities: HashSet<u8>,
        in_group: bool
    }

    impl FauxCell {
        fn len(&self) -> usize {
            self.possibilities.len()
        }

        fn remove(&mut self, to_remove: &HashSet<u8>){
            to_remove.iter().for_each(|digit| {self.possibilities.remove(digit);});
        }
    }

    struct FauxLine (Vec<FauxCell>);

    impl FauxLine {

        fn num_in_group(&self) -> usize {
            self.0.iter().filter(|faux_cell| faux_cell.in_group).count()
        }

        fn num_out_group(&self) -> usize {
            self.0.len() - self.num_in_group()
        }
    }

    // See if there's a set of cells with possibilities that exclude those possibilities from other cells.
// Runs recursively on each group to identify all groups in case there's more than 2.
    pub fn identify_and_process_possibility_groups(line: &Line) -> bool{
        unsafe {
            if super::DEBUG {
                println!("Looking for possibility groups on line {:?} {}", line.line_type, line.index);
            }
        }

        bisect_possibility_groups(line, vec![0, 1, 2, 3, 4, 5, 6, 7, 8])
    }

    fn bisect_possibility_groups(line: &Line, cells_of_interest: Vec<usize>) -> bool{

        /*
            Algorithm -
                Setup - Let count = 0
                1. Choose cell with least number of possibilities. Add to in-group.
                2. Add to count the number of possibilities in that cell
                3. Remove the possibilities of that cell from all other out-group cells.
                4. If the number of cells in group == count, finish.
                5. Goto 1
         */
        // For later recursive calls; put here because of scope reasons
        let mut in_group_indices = Vec::new();
        let mut out_group_indices = Vec::new();
        let mut run_recursion = false;

        let mut made_change = false;

        {
            // <Setup>
            let mut count = 0;
            let mut faux_line = FauxLine(Vec::new());

            for i in 0..9 {
                if !cells_of_interest.contains(&i) {
                    continue;
                }

                let cell = line.get(i).unwrap();
                let cell = Rc::clone(cell);

                let faux_possibilities = {
                    let value = &*cell.value.borrow();
                    match value {
                        CellValue::Unknown(possibilities) => {
                            let mut set = HashSet::new();
                            for (_index, digit) in possibilities.iter().enumerate() {
                                set.insert(digit.clone());
                            }
                            set
                        },
                        CellValue::Fixed(_) => { continue }
                    }
                };

                let faux_cell = FauxCell {
                    index: i,
                    possibilities: faux_possibilities,
                    in_group: false
                };

                faux_line.0.push(faux_cell);
            }
            // </Setup>

            // No point in continuing.
            if faux_line.num_out_group() <= 2 {
                return made_change;
            }

            // A kind of do-while loop
            loop {
                if faux_line.num_out_group() == 0 {
                    break;
                }

                // Step 1
                let mut smallest_cell: Option<&mut FauxCell> = None;
                let mut smallest_size = usize::MAX;

                for (_index, cell) in faux_line.0.iter_mut().filter(|faux_cell| !faux_cell.in_group).enumerate() {
                    if cell.len() < smallest_size {
                        smallest_size = cell.len();
                        smallest_cell = Some(cell);
                    }
                }

                let smallest_cell = smallest_cell.unwrap(); // Safe because we already verified the out-group had members
                smallest_cell.in_group = true;

                // Step 2
                count = count + smallest_size;


                let possibilities_to_remove = smallest_cell.possibilities.clone(); // Necessary because of mutable borrow rules

                // Step 3
                for (_index, cell) in faux_line.0.iter_mut().filter(|faux_cell| !faux_cell.in_group).enumerate() {
                    cell.remove(&possibilities_to_remove);
                }

                // Step 4 (finish condition)
                if faux_line.num_in_group() == count {
                    break;
                }
            }

            // Now we have to see if this was worth it
            if faux_line.num_out_group() > 0 { // Worth it
                // We now have two distinct groups and can separate their possibilities
                let mut in_group_possibilities = HashSet::new();
                let mut out_group_possibilities = HashSet::new();

                // Collect the possibilities for each group
                for (_index, cell) in faux_line.0.iter().enumerate() {
                    if cell.in_group {
                        cell.possibilities.iter().for_each(|digit| {in_group_possibilities.insert(digit.clone());});
                    } else {
                        cell.possibilities.iter().for_each(|digit| {out_group_possibilities.insert(digit.clone());});
                    }
                }

                // Now to apply this to the real cells
                for (_index, faux_cell) in faux_line.0.iter().enumerate() {
                    let real_cell = line.get(faux_cell.index).unwrap();
                    let mut possibilities = {
                        let value = &*real_cell.value.borrow();
                        match value {
                            CellValue::Unknown(possibilities) => possibilities.clone(),
                            CellValue::Fixed(_) => {panic!("Faux_cell shouldn't have linked to fixed cell")}
                        }
                    };
                    let starting_possibility_size = possibilities.len();

                    let possibilities_to_remove = match faux_cell.in_group {
                        true => &out_group_possibilities,
                        false => &in_group_possibilities
                    };

                    for (_i, possibility) in possibilities_to_remove.iter().enumerate() {
                        match possibilities.binary_search(possibility) {
                            Ok(x) => {
                                possibilities.remove(x);
                            },
                            Err(_) => {}
                        };
                    }

                    if possibilities.len() < starting_possibility_size { // We have a change to make
                        made_change = true;
                        let new_value = {
                            if possibilities.len() == 1 {
                                CellValue::Fixed(possibilities.pop().unwrap())
                            } else {
                                CellValue::Unknown(possibilities)
                            }
                        };

                        real_cell.set_value(new_value);
                    }
                }

                // Now finally, it's possible that there were 3 or more groups while this algorithm only identifies 2
                // So we recursively call it but restricted to each of the groups
                run_recursion = true;
                for (index, cell) in faux_line.0.iter().enumerate() {
                    if cell.in_group {
                        in_group_indices.push(index);
                    } else {
                        out_group_indices.push(index);
                    }
                }
            }
        }

        // Out of scope of everything; we need to check again if it was worth it.
        if run_recursion {
            bisect_possibility_groups(line, in_group_indices);
            bisect_possibility_groups(line, out_group_indices);
        }

        return made_change;
    }
}

// Search for a cell with only one possibility so that we can set it to FIXED
fn search_single_possibility(line: &Line) -> bool{
    unsafe {
        if DEBUG {
            println!("search_single_possibility on line {:?} {}", line.line_type, line.index);
        }
    }

    let mut made_change = false;

    for (_index, cell) in line.vec.iter().enumerate(){
        match cell.get_value_possibilities(){
            Some(x) => {
                if x.len() == 1 {
                    let new_value = CellValue::Fixed(x.first().unwrap().clone());
                    cell.set_value(new_value);
                    made_change = true;
                }
            },
            None => {}
        }
    }

    return made_change;
}

// Count up how many times each possibility occurs in the Line. If it only occurs once, that's a hidden single that we can set
fn search_hidden_single(line: &Line) -> bool{
    enum Count {
        None,
        One(Rc<Cell>),
        Many
    };

    impl Count {
        fn increment(&self, cell: Rc<Cell>) -> Count{
            match self {
                Count::None => {Count::One(cell)}
                Count::One(_) => {Count::Many}
                Count::Many => {Count::Many}
            }
        }
    }

    let mut made_change = false;

    let mut counts = [Count::None, Count::None, Count::None, Count::None, Count::None, Count::None, Count::None, Count::None, Count::None];

    for (_index, cell) in line.vec.iter().enumerate() {
        let value = &*cell.value.borrow();
        match value {
            CellValue::Unknown(possibilities) => {
                for digit in 1..10 {
                    if possibilities.contains(&(digit as u8)){
                        counts[digit-1] = counts[digit-1].increment(Rc::clone(cell));
                    }
                }
            },
            CellValue::Fixed(_) => {} // do nothing
        }
    }

    for (digit, count) in counts.iter().enumerate() {
        match count {
            Count::One(cell) => {
                cell.set((digit + 1) as u8);
                made_change = true;
            },
            _ => {}
        }
    }

    return made_change;
}

mod search_useful_constraint{
    use crate::grid::{Grid, Line, LineType, CellValue};
    use std::rc::{Rc, Weak};
    use std::cell::RefCell;

    enum PossibilityLines {
        Unique(usize),
        Invalid,
        None
    }

    impl PossibilityLines {
        fn is_invalid(&self) -> bool {
            match &self {
                PossibilityLines::Invalid => true,
                _ => false
            }
        }
    }

    // If all the cells for a particular possibility share a same other Line, they can remove that possibility from other cells in the main line.
// I.e. If possibility '1' only occurs in the first row for section 0, then you can remove that possibility
// from row 0 across the other sections. Conversely, if the possibility only occurs in the first section
// for row 0, then you can remove the possibility from the rest of section 0.
    pub fn search_useful_constraint(grid: &Grid, line: &Line) -> bool{
        unsafe {
            if super::DEBUG {
                println!("Searching for a useful constraint on line {:?} {}", line.line_type, line.index);
            }
        }

        let mut made_change = false;

        let (check_row, check_column, check_section) = match line.line_type {
            LineType::Row => {(false, false, true)},
            LineType::Column => {(false, false, true)},
            LineType::Section => {(true, true, false)},
        };

        for possibility in 0..9 {
            let mut rows = match check_row {true => PossibilityLines::None, false => PossibilityLines::Invalid };
            let mut columns = match check_column {true => PossibilityLines::None, false => PossibilityLines::Invalid };
            let mut sections = match check_section {true => PossibilityLines::None, false => PossibilityLines::Invalid };

            for cell_id in 0..9 {
                let cell_ref = line.get(cell_id).unwrap();
                let cell_ref = Rc::clone(cell_ref);
                let cell_ref = &*cell_ref;
                let value = &*cell_ref.value.borrow();

                match value {
                    CellValue::Fixed(x) => { // We can deduce this possibility won't occur elsewhere in our row, so leave for-loop
                        if possibility.eq(x) {
                            rows = process_possibility_line(rows, &cell_ref.row);
                            columns = process_possibility_line(columns, &cell_ref.column);
                            sections = process_possibility_line(sections, &cell_ref.section);
                            break;
                        }
                    }
                    CellValue::Unknown(digits) => {
                        if digits.contains(&possibility) {
                            rows = process_possibility_line(rows, &cell_ref.row);
                            columns = process_possibility_line(columns, &cell_ref.column);
                            sections = process_possibility_line(sections, &cell_ref.section);
                        }
                    }
                }

                if rows.is_invalid() & columns.is_invalid() & sections.is_invalid() {
                    break;
                }
            }

            // Check each line and see if we can determine anything
            match rows {
                PossibilityLines::Unique(index) => {
                    made_change = made_change |
                        remove_possibilities_line(grid.rows.get(index).unwrap(), possibility, &line.line_type, line.index);
                },
                _ => {}
            }
            match columns {
                PossibilityLines::Unique(index) => {
                    made_change = made_change |
                        remove_possibilities_line(grid.columns.get(index).unwrap(), possibility, &line.line_type, line.index);
                },
                _ => {}
            }
            match sections {
                PossibilityLines::Unique(index) => {
                    made_change = made_change |
                        remove_possibilities_line(grid.sections.get(index).unwrap(), possibility, &line.line_type, line.index);
                },
                _ => {}
            }
        }

        return made_change;

    }

    // initial_line_type and initial_line_index are to identify the cells that should NOT have their possibilities removed
    fn remove_possibilities_line(line: &Rc<RefCell<Line>>, digit_to_remove: u8, initial_line_type: &LineType, initial_line_index: usize) -> bool {
        let line = &*(&**line).borrow();
        let mut made_change = false;

        for (_index, cell) in line.vec.iter().enumerate() {
            let new_value = {
                let value = &*cell.value.borrow();
                match value {
                    CellValue::Unknown(possibilities) => {
                        let parent_line = match initial_line_type {
                            LineType::Row => &cell.row,
                            LineType::Column => &cell.column,
                            LineType::Section => &cell.section
                        };
                        let parent_line = &*parent_line.upgrade().unwrap();
                        let parent_line = &*parent_line.borrow();
                        if parent_line.index == initial_line_index {
                            // Don't want to apply to this cell
                            continue;
                        }

                        let new_possibilities = match possibilities.binary_search(&digit_to_remove) {
                            Ok(x) => {
                                let mut new_possibilities = possibilities.clone();
                                new_possibilities.remove(x);
                                new_possibilities
                            },
                            Err(_) => { continue; }
                        };

                        let new_value;
                        if new_possibilities.len() == 1 {
                            new_value = CellValue::Fixed(new_possibilities.first().unwrap().clone());
                        } else {
                            new_value = CellValue::Unknown(new_possibilities);
                        }

                        new_value
                    },
                    _ => { continue; }
                }
            };

            cell.set_value(new_value);
            made_change = true;

        }

        return made_change;
    }

    // We detected a useful constraint
    fn process_possibility_line(possibility_line: PossibilityLines, line: &Weak<RefCell<Line>>) -> PossibilityLines {
        let line = line.upgrade().unwrap();
        let line = &*(&*line).borrow();

        match possibility_line {
            PossibilityLines::None => {PossibilityLines::Unique(line.index)},
            PossibilityLines::Invalid => {possibility_line},
            PossibilityLines::Unique(x) => {
                if line.index.eq(&x) {
                    possibility_line
                } else {
                    PossibilityLines::Invalid
                }
            }
        }
    }

}


fn solve_line(grid: &Grid, line: &Line, solve_controller: &SolveController, solve_statistics: &mut SolveStatistics){
    unsafe {
        if DEBUG {
            println!("Solving {:?} {}", line.line_type, line.index);
        }
    }

    line.do_update.replace(false);

    if solve_controller.search_singles() {
        unsafe {
            if DEBUG {
                println!("Searching for singles on line {:?} of {}\n{}", line.line_type, line.index, grid);
            }
        }
        if search_single_possibility(line) {
            solve_statistics.increment(&SolveAction::Single);
        }
    }

    if solve_controller.search_hidden_singles() {
        unsafe {
            if DEBUG {
                println!("Searching for hidden singles on line {:?} of {}\n{}", line.line_type, line.index, grid);
            }
        }
        if search_hidden_single(line) {
            solve_statistics.increment(&SolveAction::HiddenSingle);
        }
    }

    if solve_controller.find_possibility_groups() {
        unsafe {
            if DEBUG {
                println!("Searching for possibility groups on line {:?} of {}\n{}", line.line_type, line.index, grid);
            }
        }
        if process_possibility_groups::identify_and_process_possibility_groups(line) {
            solve_statistics.increment(&SolveAction::PossibilityGroup);
        }
    }

    if solve_controller.search_useful_constraint() {
        unsafe {
            if DEBUG {
                println!("Searching for useful constraints on line {:?} of {}\n{}", line.line_type, line.index, grid);
            }
        }
        if search_useful_constraint::search_useful_constraint(grid, line) {
            solve_statistics.increment(&SolveAction::UsefulConstraints);
        }
    }

}

pub fn solve_grid(grid: &mut Grid) -> (SolveStatus, SolveStatistics) {
    // By default we enable everything
    let solve_controller = SolveController {
        determine_uniqueness: true,
        search_singles: true,
        search_hidden_singles: true,
        find_possibility_groups: true,
        search_useful_constraint: true,
        make_guesses: true
    };

    let mut solve_statistics = SolveStatistics::new();
    let solve_status = solve_grid_with_solve_controller(grid, &solve_controller, &mut solve_statistics);

    return (solve_status, solve_statistics);
}

pub fn solve_grid_with_solve_controller(grid: &mut Grid, solve_controller: &SolveController, solve_statistics: &mut SolveStatistics) -> SolveStatus{
    // Code is kind of messy so here it goes - solve_grid first tries to solve without any guesses
    // If that's not enough and a guess is required, then solve_grid_guess is called
    // solve_grid_guess runs through all the possibilities for the smallest cell, trying to solve them
    // through calling this function.
    // solve_grid_no_guess tries to solve without any guesses.
    // Of course this is if the solve_controller lets everything be used for solving it

    let mut status = solve_grid_no_guess(grid, solve_controller, solve_statistics);
    status = match status {
        SolveStatus::Unfinished => {
            if solve_controller.make_guesses() {
                solve_grid_guess(grid, solve_controller, solve_statistics)
            } else {
                SolveStatus::Complete(Some(Uniqueness::NotUnique)) // solve_grid_no_guess couldn't finish and we can't make guesses, so it's 'not unique' in the sense that we need more guesses
            }
        },
        _ => {status}
    };

    return status;
}

// Similar to solve_grid_with_solve_controller except that we don't modify the input Grid; we only determine SolveStatus
pub fn evaluate_grid_with_solve_controller(grid: &Grid, solve_controller: &SolveController) -> (SolveStatus, SolveStatistics){
    let mut mut_grid = grid.clone();
    let mut solve_statistics = SolveStatistics::new();

    let solve_status = solve_grid_with_solve_controller(&mut mut_grid, solve_controller, &mut solve_statistics);

    return (solve_status, solve_statistics);
}

pub fn solve_grid_no_guess(grid: &mut Grid, solve_controller: &SolveController, solve_statistics: &mut SolveStatistics) -> SolveStatus{

    loop {
        let mut ran_something = false;
        for (_index, line_ref) in grid.rows.iter().enumerate() {
            //println!("Processing row {}", _index);
            let line_ref = &*(&**line_ref).borrow();
            if line_ref.do_update() {
                solve_line(&grid, line_ref, solve_controller, solve_statistics);
                ran_something = true;
            }
        }
        for (_index, line_ref) in grid.columns.iter().enumerate() {
            //println!("Processing column {}", _index);
            let line_ref = &*(&**line_ref).borrow();
            if line_ref.do_update() {
                solve_line(&grid, line_ref, solve_controller, solve_statistics);
                ran_something = true;
            }
        }
        for (_index, line_ref) in grid.sections.iter().enumerate() {
            //println!("Processing section {}", _index);
            let line_ref = &*(&**line_ref).borrow();
            if line_ref.do_update() {
                solve_line(&grid, line_ref, solve_controller, solve_statistics);
                ran_something = true;
            }
        }

        if !ran_something{ // No lines have changed since we last analyzed them
            return SolveStatus::Unfinished;
        }

        // Check if complete or invalid
        let mut appears_complete = true;
        for x in 0..9 {
            for y in 0..9 {
                let cell = grid.get(x, y).unwrap();
                let cell = &*cell;
                let value = &**(&cell.value.borrow());

                match value {
                    CellValue::Unknown(possibilities) => {
                        appears_complete = false;
                        if possibilities.len() == 0 {
                            return SolveStatus::Invalid;

                        }
                    },
                    CellValue::Fixed(_) => {}
                }
            }
        }

        if appears_complete {
            // Solving by logic rules only implies Uniqueness;
            // may be overridden if guesses were made
            return SolveStatus::Complete(Some(Uniqueness::Unique));
        }
    }

}

fn solve_grid_guess(grid: &mut Grid, solve_controller: &SolveController, solve_statistics: &mut SolveStatistics) -> SolveStatus{
    solve_statistics.increment(&SolveAction::Guess);

    let smallest_cell = find_smallest_cell(grid);
    let smallest_cell = match smallest_cell {
        Some(cell) => cell,
        None => return SolveStatus::Invalid
    };

    let possibilities = smallest_cell.get_value_possibilities().unwrap();

    let mut current_status = SolveStatus::Unfinished;
    let mut grid_solution = None;

    for (_index, &digit) in possibilities.iter().enumerate() {

        let mut grid_copy = grid.clone();
        grid_copy.get(smallest_cell.x, smallest_cell.y).unwrap().set(digit);
        let status = solve_grid_with_solve_controller(&mut grid_copy, solve_controller, solve_statistics);

        // Keep a copy of grid_copy in case we later mutate grid with it
        match status {
            SolveStatus::Complete(_) => {
                grid_solution = Some(grid_copy);
            },
            _ => {}
        }

        current_status = current_status.increment(status);

        match current_status {
            SolveStatus::Complete(uniqueness) => {
                if !solve_controller.determine_uniqueness() {
                    current_status = SolveStatus::Complete(None); // be explicit we don't know
                    break; // no point in continuing
                }

                let uniqueness = uniqueness.expect("We're looking for uniqueness and yet an earlier function didn't make a claim");
                match uniqueness {
                    Uniqueness::Unique => {continue;} // gotta keep on checking
                    Uniqueness::NotUnique => {
                        break; // We can stop looking as we already found at least two solutions
                    }
                }
            }
            SolveStatus::Unfinished => {continue} // Keep looking for a solution
            SolveStatus::Invalid => panic!("current_status should not be INVALID at this point")
        }


    }

    // We've finished the for-loop
    match current_status {
        SolveStatus::Complete(_) => {
            grid.clone_from(&grid_solution.expect("grid_solution should have value if we found a solution"));
        },
        SolveStatus::Unfinished => {
            current_status = SolveStatus::Invalid; // We can now say Invalid
        },
        SolveStatus::Invalid => {}
    }

    return current_status;

}

#[cfg(test)]
mod tests {
    use crate::grid::*;
    use crate::solver::*;

    #[test]
    fn test_search_single_possibility(){
        let grid = Grid::new();

        for i in 0..8 {
            grid.get(0, i).unwrap().set(i as u8 +1);
        }

        assert_eq!(CellValue::Unknown(vec![9]), grid.get(0, 8).unwrap().get_value_copy());

        let line = grid.rows.first().unwrap();
        let line = &*(**line).borrow();

        search_single_possibility(line);

        assert_eq!(CellValue::Fixed(9), grid.get(0, 8).unwrap().get_value_copy());
    }

    #[test]
    fn test_identify_and_process_possibility_groups(){
        let grid = Grid::new();

        // Fill up the first row with values, except for the first section
        for i in 3..6 {
            grid.get(0, i).unwrap().set(i as u8 +1);
        }
        grid.get(1, 6).unwrap().set(1);
        grid.get(1, 7).unwrap().set(2);
        grid.get(1, 8).unwrap().set(3);

        assert_eq!(CellValue::Unknown(vec![1, 2, 3, 7, 8, 9]), grid.get(0, 0).unwrap().get_value_copy());

        let line = grid.rows.first().unwrap();
        let line = &*(**line).borrow();

        process_possibility_groups::identify_and_process_possibility_groups(line);

        assert_eq!(CellValue::Unknown(vec![1, 2, 3]), grid.get(0, 0).unwrap().get_value_copy());
    }

    #[test]
    fn test_search_useful_constraint_1(){
        let grid = Grid::new();

        // Fill up the first row with values, except for the first section
        for i in 3..6 {
            grid.get(0, i).unwrap().set(i as u8 +1);
        }
        grid.get(1, 6).unwrap().set(1);
        grid.get(1, 7).unwrap().set(2);
        grid.get(1, 8).unwrap().set(3);



        assert_eq!(CellValue::Unknown(vec![1, 2, 3, 4, 5, 6, 7, 8, 9]), grid.get(2, 0).unwrap().get_value_copy());

        let line = grid.rows.first().unwrap();
        let line = &*(**line).borrow();

        search_useful_constraint::search_useful_constraint(&grid, line);

        assert_eq!(CellValue::Unknown(vec![4, 5, 6, 7, 8, 9]), grid.get(2, 0).unwrap().get_value_copy());
    }


    #[test]
    fn test_search_useful_constraint_2(){
        let grid = Grid::new();

        // These values come from a specific bug example where a constraint was incorrectly identified
        grid.get(3, 0).unwrap().set(8);
        grid.get(3, 1).unwrap().set(5);
        grid.get(3, 2).unwrap().set(2);
        grid.get(5, 1).unwrap().set(6);
        grid.get(6, 1).unwrap().set(8);
        grid.get(8, 2).unwrap().set(7);

        grid.get(0, 1).unwrap().set_value(CellValue::Unknown(vec![1, 3, 4, 7, 9]));
        grid.get(1, 1).unwrap().set_value(CellValue::Unknown(vec![1, 3, 4, 5, 9]));
        grid.get(2, 1).unwrap().set_value(CellValue::Unknown(vec![1, 2]));
        grid.get(4, 1).unwrap().set_value(CellValue::Unknown(vec![1, 3, 4, 7]));
        grid.get(7, 1).unwrap().set_value(CellValue::Unknown(vec![1, 2, 3, 9]));
        grid.get(8, 1).unwrap().set_value(CellValue::Unknown(vec![1, 2, 3, 5, 9]));

        // 5 is wrongly removed here
        grid.get(1, 0).unwrap().set_value(CellValue::Unknown(vec![1, 3, 4, 5, 9]));

        println!("{}", grid);

        let line = grid.columns.get(1).unwrap();
        let line = &*(**line).borrow();

        search_useful_constraint::search_useful_constraint(&grid, line);

        assert_eq!(CellValue::Unknown(vec![1, 3, 4, 5, 9]), grid.get(1, 0).unwrap().get_value_copy());

    }

    #[test]
    fn test_hidden_single() {
        let grid = Grid::new();

        // In Row 0 there should be only one spot for 1s and 2s to be set, even though every cell will
        // have possibilities for values 3 - 9

        grid.get(1, 5).unwrap().set(1);
        grid.get(2, 6).unwrap().set(1);
        grid.get(5, 2).unwrap().set(1);
        grid.get(6, 1).unwrap().set(1);

        grid.get(1, 6).unwrap().set(2);
        grid.get(2, 5).unwrap().set(2);
        grid.get(5, 1).unwrap().set(2);
        grid.get(6, 0).unwrap().set(2);

        let first_row = grid.rows.get(0).unwrap();
        let first_row = &*(**first_row).borrow();

        search_hidden_single(&first_row);

        assert_eq!(CellValue::Fixed(1), grid.get(0, 0).unwrap().get_value_copy());
        assert_eq!(CellValue::Fixed(2), grid.get(0, 2).unwrap().get_value_copy());

    }


}