use std::rc::{Rc, Weak};
use std::cell::{RefCell};
use std::collections::HashSet;

const DEBUG: bool = false;

#[derive(Clone, Debug, Eq, PartialEq)]
enum CellValue {
    FIXED(u8),
    UNKNOWN(Vec<u8>)
}

struct Cell {
    x: usize,
    y: usize,
    value: RefCell<CellValue>,
    row: Weak<RefCell<Line>>,
    column: Weak<RefCell<Line>>,
    section: Weak<RefCell<Line>>,
}

impl Cell {
    fn set(&self, digit: u8){
        if DEBUG {
            println!("Cell {}, {} was set with digit {}", self.x, self.y, digit);
        }

        self.value.replace(CellValue::FIXED(digit));

        // We fully expect our row, column, and section to still be here even though the Rust compiler won't guarantee it
        // Panic-ing if they're not present is perfectly reasonable

        let row = &*self.row.upgrade().unwrap();
        let row = &*row.borrow();

        let column = &*self.column.upgrade().unwrap();
        let column = &*column.borrow();

        let section = &*self.section.upgrade().unwrap();
        let section = &*section.borrow();

        Cell::process_possibilities(row, digit);
        Cell::process_possibilities(column, digit);
        Cell::process_possibilities(section, digit);
    }

    fn get_value_copy(&self) -> CellValue {
        let value = &*self.value.borrow();
        return value.clone();
    }

    fn set_value(&self, value: CellValue){
        match value {
            CellValue::FIXED(digit) => {
                self.set(digit);
                return;
            },
            CellValue::UNKNOWN(_) => {
                self.set_value_exact(value);
            } // continue on
        }
    }

    fn set_value_exact(&self, value: CellValue){
        if DEBUG {
            println!("Cell {}, {} was set with CellValue exact {:?}", self.x, self.y, value);
        }

        self.value.replace(value);
        self.mark_updates();
    }

    fn get_value_possibilities(&self) -> Option<Vec<u8>> {
        let value = &*self.value.borrow();
        match value {
            CellValue::FIXED(_) => None,
            CellValue::UNKNOWN(x) => Some(x.clone())
        }
    }

    fn mark_updates(&self){
        {
            let row = &*self.row.upgrade().unwrap();
            let row = &*row.borrow();
            row.do_update.replace(true);
        }
        {
            let column = &*self.column.upgrade().unwrap();
            let column = &*column.borrow();
            column.do_update.replace(true);
        }
        {
            let section = &*self.section.upgrade().unwrap();
            let section = &*section.borrow();
            section.do_update.replace(true);
        }
    }

    fn process_possibilities(line: &Line, digit: u8){
        for (_index, cell) in line.vec.iter().enumerate() {
            let cell = &**cell;

            // Find the new CellValue to set; may be None if the cell was already fixed or had no possibilities remaining
            let new_value_option : Option<CellValue> = {
                let value = &*cell.value.borrow();

                match value {
                    CellValue::UNKNOWN(possibilities) => {
                        let mut new_possibilities = possibilities.clone();

                        match new_possibilities.binary_search(&digit) {
                            Ok(index_remove) => {new_possibilities.remove(index_remove);},
                            _ => {}
                        };

                        Some(CellValue::UNKNOWN(new_possibilities))
                        /*
                        if new_possibilities.len() == 1 {
                            let remaining_digit = new_possibilities.first().unwrap().clone();
                            Some(CellValue::FIXED(remaining_digit))
                        } else if new_possibilities.len() == 0 {
                            None
                        } else {
                            Some(CellValue::UNKNOWN(new_possibilities))
                        }*/
                    },
                    CellValue::FIXED(_) => {None}
                }
            };

            match new_value_option {
                Some(new_value) => {
                    cell.set_value(new_value);
                },
                None => {}
            }

        }
    }
}

struct Line {
    vec: Vec<Rc<Cell>>,
    do_update: RefCell<bool>,
    index: usize,
    line_type: LineType
}

#[derive(Debug)]
enum LineType {
    ROW,
    COLUMN,
    SECTION
}

impl Line {
    fn push(&mut self, x: Rc<Cell>){
        self.vec.push(x);
    }

    fn get(&self, index: usize) -> Option<&Rc<Cell>>{
        self.vec.get(index)
    }

    fn new(index: usize, line_type: LineType) -> Line {
        Line {
            vec: Vec::new(),
            do_update: RefCell::new(false),
            index,
            line_type
        }
    }

    fn do_update(&self) -> bool {
        let do_update = &self.do_update.borrow();
        let do_update = &**do_update;

        return do_update.clone();
    }
}

type MultiMut<T> = Rc<RefCell<T>>;

struct Grid {
    rows: Vec<MultiMut<Line>>, // Read from top to bottom
    columns: Vec<MultiMut<Line>>,
    sections: Vec<MultiMut<Line>>,
}

impl Grid {
    fn new() -> Grid {

        let mut rows: Vec<MultiMut<Line>> = Vec::new();
        let mut columns: Vec<MultiMut<Line>> = Vec::new();
        let mut sections: Vec<MultiMut<Line>> = Vec::new();

        for i in 0..9 {
            rows.push(Rc::new(RefCell::new(Line::new(i, LineType::ROW))));
            columns.push(Rc::new(RefCell::new(Line::new(i, LineType::COLUMN))));
            sections.push(Rc::new(RefCell::new(Line::new(i, LineType::SECTION))));
        }

        for row_index in 0..9 {
            let row_rc = unsafe {
                rows.get_unchecked(row_index)
            };

            let row_ref = &mut *row_rc.borrow_mut();

            for column_index in 0..9 {
                let section_index = (row_index / 3) * 3 + column_index / 3;
                let (column_rc, section_rc) = unsafe {
                    (columns.get_unchecked_mut(column_index),
                    sections.get_unchecked_mut(section_index))
                };

                let column_weak = Rc::downgrade(column_rc);
                let column_ref = &mut *column_rc.borrow_mut();

                let section_weak = Rc::downgrade(section_rc);
                let section_ref = &mut *section_rc.borrow_mut();

                let row_weak = Rc::downgrade(row_rc);

                let cell = Cell {
                    x: row_index,
                    y: column_index,
                    value: RefCell::new(CellValue::UNKNOWN(vec![1, 2, 3, 4, 5, 6, 7, 8, 9])),
                    row: row_weak,
                    column: column_weak,
                    section: section_weak
                };

                let ref1 = Rc::new(cell);
                let ref2 = Rc::clone(&ref1);
                let ref3 = Rc::clone(&ref1);

                row_ref.push(ref1);
                column_ref.push(ref2);
                section_ref.push(ref3);
            }
        }

        return Grid { rows, columns, sections };
    }

    fn get(&self, r: usize, c: usize) -> Result<Rc<Cell>, &str> {
        if (r > 9) | (c > 9) {
            return Err("Row or column indices are out of bounds");
        }

        let row = match self.rows.get(r) {
            Some(x) => x,
            None => {return Err("Row index is out of bounds")}
        };

        let row = &*(&**row).borrow();

        let cell = match row.get(c) {
            Some(x) => x,
            None => {return Err("Column index is out of bounds")}
        };

        return Ok(Rc::clone(cell));
    }


    fn print(&self) {
        for r in 0..9 {

            // Each row corresponds to 3 rows since we leave room for guesses
            let mut row1 = String::new();
            let mut row2 = String::new();
            let mut row3 = String::new();

            for c in 0..9 {

                let cell = &*self.get(r, c).unwrap();
                let value = &*cell.value.borrow();


                match value {
                    CellValue::FIXED(x) => {
                        row1.push_str("   ");
                        row2.push(' '); row2.push_str(&x.to_string()); row2.push(' ');
                        row3.push_str("   ");
                    },
                    CellValue::UNKNOWN(x) => {
                        Grid::process_unknown(&x, 1, &mut row1);
                        Grid::process_unknown(&x, 2, &mut row1);
                        Grid::process_unknown(&x, 3, &mut row1);

                        Grid::process_unknown(&x, 4, &mut row2);
                        Grid::process_unknown(&x, 5, &mut row2);
                        Grid::process_unknown(&x, 6, &mut row2);

                        Grid::process_unknown(&x, 7, &mut row3);
                        Grid::process_unknown(&x, 8, &mut row3);
                        Grid::process_unknown(&x, 9, &mut row3);
                    }
                };

                if (c % 3 == 2) && (c < 8){
                    row1.push('\u{2503}');
                    row2.push('\u{2503}');
                    row3.push('\u{2503}');
                } else if c < 8{
                    row1.push('┆');
                    row2.push('┆');
                    row3.push('┆');
                }


            }

            println!("{}", row1);
            println!("{}", row2);
            println!("{}", row3);

            if (r % 3 == 2) && (r < 8) {
                //println!("███████████████████████████████████");
                println!("━━━┿━━━┿━━━╋━━━┿━━━┿━━━╋━━━┿━━━┿━━━")
            } else if r < 8{
                //println!("───┼───┼───╂───┼───┼───╂───┼───┼───")
                println!("┄┄┄┼┄┄┄┼┄┄┄╂┄┄┄┼┄┄┄┼┄┄┄╂┄┄┄┼┄┄┄┼┄┄┄")
            }
        }
    }

    fn process_unknown(x: &Vec<u8>, digit: u8, row: &mut String){
        if x.contains(&digit) {
            row.push('*');
        } else{
            row.push(' ');
        }
    }
}








struct FauxCell{
    index: usize,
    real_cell: Rc<Cell>,
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
    fn get_mut(&mut self, index: usize) -> Option<&mut FauxCell>{
        return self.0.get_mut(index);
    }

    fn num_in_group(&self) -> usize {
        self.0.iter().filter(|fauxcell| fauxcell.in_group).count()
    }

    fn num_out_group(&self) -> usize {
        self.0.len() - self.num_in_group()
    }
}

// See if there's a set of cells with possibilities that exclude those possibilities from other cells.
// Runs recursively on each group to identify all groups in case there's more than 2.
fn identify_and_process_possibility_groups(line: &Line){
    if DEBUG {
        println!("Looking for possibility groups on line {:?} {}", line.line_type, line.index);
    }

    bisect_possibility_groups(line, vec![0, 1, 2, 3, 4, 5, 6, 7, 8]);
}

fn bisect_possibility_groups(line: &Line, cells_of_interest: Vec<usize>){

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
                    CellValue::UNKNOWN(possibilities) => {
                        let mut set = HashSet::new();
                        for (_index, digit) in possibilities.iter().enumerate() {
                            set.insert(digit.clone());
                        }
                        set
                    },
                    CellValue::FIXED(_) => { continue }
                }
            };

            let faux_cell = FauxCell {
                index: i,
                real_cell: cell,
                possibilities: faux_possibilities,
                in_group: false
            };

            faux_line.0.push(faux_cell);
        }
        // </Setup>

        // No point in continuing.
        if faux_line.num_out_group() <= 2 {
            return;
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
                        CellValue::UNKNOWN(possibilities) => possibilities.clone(),
                        CellValue::FIXED(_) => {panic!("Faux_cell shouldn't have linked to fixed cell")}
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
                    let new_value = {
                        if possibilities.len() == 1 {
                            CellValue::FIXED(possibilities.pop().unwrap())
                        } else {
                            CellValue::UNKNOWN(possibilities)
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
}

// Search for a cell with only one possibility so that we can set it to FIXED
fn search_single_possibility(line: &Line){
    if DEBUG {
        println!("search_single_possibility on line {:?} {}", line.line_type, line.index);
    }

    for (_index, cell) in line.vec.iter().enumerate(){
        match cell.get_value_possibilities(){
            Some(x) => {
                if x.len() == 1 {
                    let new_value = CellValue::FIXED(x.first().unwrap().clone());
                    cell.set_value(new_value);
                }
            },
            None => {}
        }
    }
}

enum PossibilityLines {
    UNIQUE(usize),
    INVALID,
    NONE
}

impl PossibilityLines {
    fn is_invalid(&self) -> bool {
        match &self {
            PossibilityLines::INVALID => true,
            _ => false
        }
    }
}

// If all the cells for a particular possibility share a same other Line, they can remove that possibility from other cells in the main line.
// I.e. If possibility '1' only occurs in the first row for section 0, then you can remove that possibility
// from row 0 across the other sections. Conversely, if the possibility only occurs in the first section
// for row 0, then you can remove the possibility from the rest of section 0.
fn search_useful_constraint(grid: &Grid, line: &Line){
    if DEBUG {
        println!("Searching for a useful constraint on line {:?} {}", line.line_type, line.index);
    }

    let (check_row, check_column, check_section) = match line.line_type {
        LineType::ROW => {(false, false, true)},
        LineType::COLUMN => {(false, false, true)},
        LineType::SECTION => {(true, true, false)},
    };

    for possibility in 0..9 {
        let mut rows = match check_row {true => PossibilityLines::NONE, false => PossibilityLines::INVALID};
        let mut columns = match check_column {true => PossibilityLines::NONE, false => PossibilityLines::INVALID};
        let mut sections = match check_section {true => PossibilityLines::NONE, false => PossibilityLines::INVALID};

        for cell_id in 0..9 {
            let cell_ref = line.get(cell_id).unwrap();
            let cell_ref = Rc::clone(cell_ref);
            let cell_ref = &*cell_ref;
            let value = &*cell_ref.value.borrow();

            match value {
                CellValue::FIXED(x) => { // We can deduce this possibility won't occur elsewhere in our row, so leave for-loop
                    if possibility.eq(x) {
                        rows = process_possibility_line(rows, &cell_ref.row);
                        columns = process_possibility_line(columns, &cell_ref.column);
                        sections = process_possibility_line(sections, &cell_ref.section);
                        break;
                    }
                }
                CellValue::UNKNOWN(digits) => {
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
            PossibilityLines::UNIQUE(index) => {
                remove_possibilities_line(grid.rows.get(index).unwrap(), possibility, &line.line_type, line.index);
            },
            _ => {}
        }
        match columns {
            PossibilityLines::UNIQUE(index) => {
                remove_possibilities_line(grid.columns.get(index).unwrap(), possibility, &line.line_type, line.index);
            },
            _ => {}
        }
        match sections {
            PossibilityLines::UNIQUE(index) => {
                remove_possibilities_line(grid.sections.get(index).unwrap(), possibility, &line.line_type, line.index);
            },
            _ => {}
        }
    }

}

// initial_line_type and initial_line_index are to identify the cells that should NOT have their possibilities removed
fn remove_possibilities_line(line: &Rc<RefCell<Line>>, digit_to_remove: u8, initial_line_type: &LineType, initial_line_index: usize) {
    let line = &*(&**line).borrow();

    for (_index, cell) in line.vec.iter().enumerate() {
        let new_value = {
            let value = &*cell.value.borrow();
            match value {
                CellValue::UNKNOWN(possibilities) => {
                    let parent_line = match initial_line_type {
                        LineType::ROW => &cell.row,
                        LineType::COLUMN => &cell.column,
                        LineType::SECTION => &cell.section
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
                        new_value = CellValue::FIXED(new_possibilities.first().unwrap().clone());
                    } else {
                        new_value = CellValue::UNKNOWN(new_possibilities);
                    }

                    new_value
                },
                _ => { continue; }
            }
        };

        cell.set_value(new_value);

    }
}

// We detected
fn process_possibility_line(possibility_line: PossibilityLines, line: &Weak<RefCell<Line>>) -> PossibilityLines {
    let line = line.upgrade().unwrap();
    let line = &*(&*line).borrow();

    match possibility_line {
        PossibilityLines::NONE => {PossibilityLines::UNIQUE(line.index)},
        PossibilityLines::INVALID => {possibility_line},
        PossibilityLines::UNIQUE(x) => {
            if line.index.eq(&x) {
                possibility_line
            } else {
                PossibilityLines::INVALID
            }
        }
    }
}


fn solve_line(grid: &Grid, line: &Line){
    if DEBUG {
        println!("Solving {:?} {}", line.line_type, line.index);
    }

    search_single_possibility(line);
    if DEBUG {
        grid.print();
    }

    identify_and_process_possibility_groups(line);
    if DEBUG {
        grid.print();
    }

    search_useful_constraint(grid, line);
    if DEBUG {
        grid.print();
    }

    line.do_update.replace(false);
}

fn solve_grid(grid: &Grid) {
    'outer: loop {
        for (_index, line_ref) in grid.rows.iter().enumerate() {
            //println!("Processing row {}", _index);
            let line_ref = &*(&**line_ref).borrow();
            if line_ref.do_update() {
                solve_line(&grid, line_ref);
            }
        }
        for (_index, line_ref) in grid.columns.iter().enumerate() {
            //println!("Processing column {}", _index);
            let line_ref = &*(&**line_ref).borrow();
            if line_ref.do_update() {
                solve_line(&grid, line_ref);
            }
        }
        for (_index, line_ref) in grid.sections.iter().enumerate() {
            //println!("Processing section {}", _index);
            let line_ref = &*(&**line_ref).borrow();
            if line_ref.do_update() {
                solve_line(&grid, line_ref);
            }
        }

        // Check if complete or invalid
        let mut appears_complete = true;
        for x in 0..9 {
            for y in 0..9 {
                let cell = grid.get(x, y).unwrap();
                let cell = &*cell;
                let value = &**(&cell.value.borrow());

                match value {
                    CellValue::UNKNOWN(possibilities) => {
                        appears_complete = false;
                        if possibilities.len() == 0 {
                            println!("Unable to find a solution");
                            break 'outer;
                        }
                    },
                    CellValue::FIXED(_) => {}
                }
            }
        }

        if appears_complete {
            break 'outer;
        }

    }
}


fn main() {
    let grid = Grid::new();

    println!("Now setting some values");

    
    grid.get(0, 4).unwrap().set(8);
    grid.get(0, 5).unwrap().set(5);
    grid.get(0, 6).unwrap().set(6);

    grid.get(2, 3).unwrap().set(9);
    grid.get(2, 4).unwrap().set(4);
    grid.get(2, 5).unwrap().set(3);
    grid.get(2, 6).unwrap().set(5);
    grid.get(2, 7).unwrap().set(7);

    grid.get(3, 0).unwrap().set(8);
    grid.get(3, 2).unwrap().set(2);
    grid.get(3, 3).unwrap().set(6);
    grid.get(3, 4).unwrap().set(7);
    grid.get(3, 5).unwrap().set(4);
    grid.get(3, 6).unwrap().set(9);

    grid.get(4, 4).unwrap().set(9);
    grid.get(4, 8).unwrap().set(5);

    grid.get(5, 1).unwrap().set(6);
    grid.get(5, 6).unwrap().set(2);

    grid.get(6, 1).unwrap().set(8);
    grid.get(6, 8).unwrap().set(2);

    grid.get(7, 3).unwrap().set(7);
    grid.get(7, 5).unwrap().set(6);
    grid.get(7, 7).unwrap().set(5);
    grid.get(7, 8).unwrap().set(4);

    grid.get(8, 2).unwrap().set(7);
    grid.get(8, 3).unwrap().set(4);

    grid.print();

    println!("Now going to start a solver on it");

    solve_grid(&grid);
    grid.print();
    println!("\n");



}

#[test]
fn test_search_single_possibility(){
    let grid = Grid::new();

    for i in 0..8 {
        grid.get(0, i).unwrap().set(i as u8 +1);
    }

    assert_eq!(CellValue::UNKNOWN(vec![9]), grid.get(0, 8).unwrap().get_value_copy());

    let line = grid.rows.first().unwrap();
    let line = &*(**line).borrow();

    search_single_possibility(line);

    assert_eq!(CellValue::FIXED(9), grid.get(0, 8).unwrap().get_value_copy());
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

    assert_eq!(CellValue::UNKNOWN(vec![1, 2, 3, 7, 8, 9]), grid.get(0, 0).unwrap().get_value_copy());

    let line = grid.rows.first().unwrap();
    let line = &*(**line).borrow();

    identify_and_process_possibility_groups(line);

    assert_eq!(CellValue::UNKNOWN(vec![1, 2, 3]), grid.get(0, 0).unwrap().get_value_copy());
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



    assert_eq!(CellValue::UNKNOWN(vec![1, 2, 3, 4, 5, 6, 7, 8, 9]), grid.get(2, 0).unwrap().get_value_copy());

    let line = grid.rows.first().unwrap();
    let line = &*(**line).borrow();

    search_useful_constraint(&grid, line);

    assert_eq!(CellValue::UNKNOWN(vec![4, 5, 6, 7, 8, 9]), grid.get(2, 0).unwrap().get_value_copy());
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

    grid.get(0, 1).unwrap().set_value(CellValue::UNKNOWN(vec![1, 3, 4, 7, 9]));
    grid.get(1, 1).unwrap().set_value(CellValue::UNKNOWN(vec![1, 3, 4, 5, 9]));
    grid.get(2, 1).unwrap().set_value(CellValue::UNKNOWN(vec![1, 2]));
    grid.get(4, 1).unwrap().set_value(CellValue::UNKNOWN(vec![1, 3, 4, 7]));
    grid.get(7, 1).unwrap().set_value(CellValue::UNKNOWN(vec![1, 2, 3, 9]));
    grid.get(8, 1).unwrap().set_value(CellValue::UNKNOWN(vec![1, 2, 3, 5, 9]));

    // 5 is wrongly removed here
    grid.get(1, 0).unwrap().set_value(CellValue::UNKNOWN(vec![1, 3, 4, 5, 9]));

    grid.print();

    let line = grid.columns.get(1).unwrap();
    let line = &*(**line).borrow();

    search_useful_constraint(&grid, line);

    assert_eq!(CellValue::UNKNOWN(vec![1, 3, 4, 5, 9]), grid.get(1, 0).unwrap().get_value_copy());



}