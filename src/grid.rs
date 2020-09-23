use std::rc::{Rc, Weak};
use std::cell::{RefCell};
use std::fmt::Formatter;

pub static mut DEBUG: bool = false;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CellValue {
    Fixed(u8),
    Unknown(Vec<u8>)
}

pub struct Cell {
    pub x: usize,
    pub y: usize,
    pub value: RefCell<CellValue>,
    pub row: Weak<RefCell<Line>>,
    pub column: Weak<RefCell<Line>>,
    pub section: Weak<RefCell<Line>>,
}

impl Cell {
    pub fn set(&self, digit: u8){
        unsafe {
            if DEBUG {
                println!("Cell {}, {} was set with digit {}", self.x, self.y, digit);
            }
        }

        self.value.replace(CellValue::Fixed(digit));

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

    pub fn get_value_copy(&self) -> CellValue {
        let value = &*self.value.borrow();
        return value.clone();
    }

    pub fn set_value(&self, value: CellValue){
        match value {
            CellValue::Fixed(digit) => {
                self.set(digit);
                return;
            },
            CellValue::Unknown(_) => {
                self.set_value_exact(value);
            } // continue on
        }
    }

    pub fn set_value_exact(&self, value: CellValue){
        unsafe {
            if DEBUG {
                println!("Cell {}, {} was set with CellValue exact {:?}", self.x, self.y, value);
            }
        }

        self.value.replace(value);
        self.mark_updates();
    }

    pub fn get_value_possibilities(&self) -> Option<Vec<u8>> {
        let value = &*self.value.borrow();
        match value {
            CellValue::Fixed(_) => None,
            CellValue::Unknown(x) => Some(x.clone())
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
                    CellValue::Unknown(possibilities) => {
                        let mut new_possibilities = possibilities.clone();

                        match new_possibilities.binary_search(&digit) {
                            Ok(index_remove) => {new_possibilities.remove(index_remove);},
                            _ => {}
                        };

                        Some(CellValue::Unknown(new_possibilities))
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
                    CellValue::Fixed(_) => {None}
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

pub struct Line {
    pub vec: Vec<Rc<Cell>>,
    pub do_update: RefCell<bool>,
    pub index: usize,
    pub line_type: LineType
}

#[derive(Debug)]
pub enum LineType {
    Row,
    Column,
    Section
}

impl Line {
    fn push(&mut self, x: Rc<Cell>){
        self.vec.push(x);
    }

    pub fn get(&self, index: usize) -> Option<&Rc<Cell>>{
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

    pub fn do_update(&self) -> bool {
        let do_update = &self.do_update.borrow();
        let do_update = &**do_update;

        return do_update.clone();
    }
}

type MultiMut<T> = Rc<RefCell<T>>;

pub struct Grid {
    pub rows: Vec<MultiMut<Line>>, // Read from top to bottom
    pub columns: Vec<MultiMut<Line>>,
    pub sections: Vec<MultiMut<Line>>,
}

impl Grid {
    pub fn new() -> Grid {

        let mut rows: Vec<MultiMut<Line>> = Vec::new();
        let mut columns: Vec<MultiMut<Line>> = Vec::new();
        let mut sections: Vec<MultiMut<Line>> = Vec::new();

        for i in 0..9 {
            rows.push(Rc::new(RefCell::new(Line::new(i, LineType::Row))));
            columns.push(Rc::new(RefCell::new(Line::new(i, LineType::Column))));
            sections.push(Rc::new(RefCell::new(Line::new(i, LineType::Section))));
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
                    value: RefCell::new(CellValue::Unknown(vec![1, 2, 3, 4, 5, 6, 7, 8, 9])),
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

    pub fn get(&self, r: usize, c: usize) -> Result<Rc<Cell>, &str> {
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

    fn process_unknown(x: &Vec<u8>, digit: u8, row: &mut String){
        if x.contains(&digit) {
            row.push('*');
        } else{
            row.push(' ');
        }
    }
}

impl Clone for Grid {
    fn clone(&self) -> Self {
        let mut new = Grid::new();
        new.clone_from(&self);

        return new;
    }

    fn clone_from(&mut self, source: &Self) {
        for x in 0..9 {
            for y in 0..9 {
                let source_value = source.get(x, y).unwrap().get_value_copy();
                self.get(x, y).unwrap().set_value_exact(source_value);
            }
        }

        for i in 0..9 {
            let new_row = &*self.rows.get(i).unwrap().borrow();
            let source_row = &*source.rows.get(i).unwrap().borrow();
            new_row.do_update.replace(source_row.do_update());

            let new_column = &*self.columns.get(i).unwrap().borrow();
            let source_column = &*source.columns.get(i).unwrap().borrow();
            new_column.do_update.replace(source_column.do_update());

            let new_section = &*self.sections.get(i).unwrap().borrow();
            let source_section = &*source.sections.get(i).unwrap().borrow();
            new_section.do_update.replace(source_section.do_update());
        }
    }
}

impl std::fmt::Display for Grid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for r in 0..9 {

            // Each row corresponds to 3 rows since we leave room for guesses
            let mut row1 = String::new();
            let mut row2 = String::new();
            let mut row3 = String::new();

            for c in 0..9 {

                let cell = &*self.get(r, c).unwrap();
                let value = &*cell.value.borrow();


                match value {
                    CellValue::Fixed(x) => {
                        row1.push_str("   ");
                        row2.push(' '); row2.push_str(&x.to_string()); row2.push(' ');
                        row3.push_str("   ");
                    },
                    CellValue::Unknown(x) => {
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

            write!(f, "{}", row1)?;
            write!(f, "\n")?;
            write!(f, "{}", row2)?;
            write!(f, "\n")?;
            write!(f, "{}", row3)?;
            write!(f, "\n")?;

            if (r % 3 == 2) && (r < 8) {
                write!(f, "━━━┿━━━┿━━━╋━━━┿━━━┿━━━╋━━━┿━━━┿━━━\n")?;
            } else if r < 8{
                write!(f, "┄┄┄┼┄┄┄┼┄┄┄╂┄┄┄┼┄┄┄┼┄┄┄╂┄┄┄┼┄┄┄┼┄┄┄\n")?;
            }
        }

        return Result::Ok(());
    }
}