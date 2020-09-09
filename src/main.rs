use std::rc::{Rc, Weak};
use std::cell::{RefCell, Ref};

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

    fn process_possibilities(line: &Line, digit: u8){
        for (_index, cell) in line.iter().enumerate() {
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

                        if new_possibilities.len() == 1 {
                            let remaining_digit = new_possibilities.first().unwrap().clone();
                            Some(CellValue::FIXED(remaining_digit))
                        } else if new_possibilities.len() == 0 {
                            None
                        } else {
                            Some(CellValue::UNKNOWN(new_possibilities))
                        }
                    },
                    _ => {None}
                }
            };

            match new_value_option {
                Some(new_value) => {
                    match new_value {
                        CellValue::UNKNOWN(_) => {
                            cell.value.replace(new_value);
                        },
                        CellValue::FIXED(new_digit) => {
                            cell.set(new_digit); // Recursive
                        }

                    }
                },
                None => {}
            }

        }
    }
}

type Line = Vec<Rc<Cell>>;
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

        for _i in 0..9 {
            rows.push(Rc::new(RefCell::new(Line::new())));
            columns.push(Rc::new(RefCell::new(Line::new())));
            sections.push(Rc::new(RefCell::new(Line::new())));
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
}

