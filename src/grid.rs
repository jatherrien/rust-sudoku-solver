use std::rc::{Rc, Weak};
use std::cell::{RefCell};
use std::fmt::Formatter;

pub static mut DEBUG: bool = false;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CellValue {
    Fixed(u8),
    Unknown(Vec<u8>)
}

/// A representation of a single cell in a Sudoku grid. Don't make this directly; make a Grid.
pub struct Cell {
    pub x: usize,
    pub y: usize,
    pub value: RefCell<CellValue>,
    pub row: Weak<RefCell<Section>>,
    pub column: Weak<RefCell<Section>>,
    pub section: Weak<RefCell<Section>>,
}

impl Cell {
    /// Set the `Cell`'s value to be a fixed digit. This method also removes the digit from any
    /// affected cells in the same row, column, or square.
    ///
    /// # Examples
    ///
    /// ```
    /// use sudoku_solver::grid::{Grid, CellValue};
    /// let grid = Grid::new();
    ///
    /// let cell1 = grid.get(0,0).unwrap();
    /// let cell2 = grid.get(0,1).unwrap();
    ///
    /// assert_eq!(cell1.get_value_copy(), CellValue::Unknown(vec![1,2,3,4,5,6,7,8,9]));
    /// assert_eq!(cell2.get_value_copy(), CellValue::Unknown(vec![1,2,3,4,5,6,7,8,9]));
    ///
    /// cell1.set(1);
    ///
    /// assert_eq!(cell1.get_value_copy(), CellValue::Fixed(1));
    /// assert_eq!(cell2.get_value_copy(), CellValue::Unknown(vec![2,3,4,5,6,7,8,9]));
    ///
    /// ```
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

    /// Get a copy of the `CellValue`
    pub fn get_value_copy(&self) -> CellValue {
        let value = &*self.value.borrow();
        return value.clone();
    }

    /// Set the cell value with a provided `CellValue`; if `value` is Fixed then the related cell's
    /// possibilities are adjusted like in `set`.
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

    /// Set the `Cell`'s value to be a value **without** adjusting any of the nearby cells.
    ///
    /// # Examples
    ///
    /// ```
    /// use sudoku_solver::grid::{Grid, CellValue};
    /// let grid = Grid::new();
    ///
    /// let cell1 = grid.get(0,0).unwrap();
    /// let cell2 = grid.get(0,1).unwrap();
    ///
    /// assert_eq!(cell1.get_value_copy(), CellValue::Unknown(vec![1,2,3,4,5,6,7,8,9]));
    /// assert_eq!(cell2.get_value_copy(), CellValue::Unknown(vec![1,2,3,4,5,6,7,8,9]));
    ///
    /// cell1.set_value_exact(CellValue::Fixed(1));
    ///
    /// assert_eq!(cell1.get_value_copy(), CellValue::Fixed(1));
    /// assert_eq!(cell2.get_value_copy(), CellValue::Unknown(vec![1,2,3,4,5,6,7,8,9])); // still contains 1
    ///
    /// ```
    pub fn set_value_exact(&self, value: CellValue){
        unsafe {
            if DEBUG {
                println!("Cell {}, {} was set with CellValue exact {:?}", self.x, self.y, value);
            }
        }

        self.value.replace(value);
        self.mark_updates();
    }

    /// Return a copy of the cell's possibilities if it has them.
    pub fn get_value_possibilities(&self) -> Option<Vec<u8>> {
        let value = &*self.value.borrow();
        match value {
            CellValue::Fixed(_) => None,
            CellValue::Unknown(x) => Some(x.clone())
        }
    }

    // Internal function - mark all the Sections the cell belongs to as having had a change
    // so that the solver will look at it later
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

    // Go through and remove digit from the Section's Cells' possibilities
    fn process_possibilities(line: &Section, digit: u8){
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

/// A representation of either a Row, Column, or Square in a Sudoku grid. Don't make this directly; make a Grid.
pub struct Section {
    /// A vector of `Rc`s of the `Cell`s inside this Section. We use `Rc` because one of the
    /// Sections needs to have ownership of the Cells but then the others have to have a different
    /// signature.
    pub vec: Vec<Rc<Cell>>,
    pub do_update: RefCell<bool>,
    pub index: usize,
    pub section_type: SectionType
}

#[derive(Debug)]
pub enum SectionType {
    Row,
    Column,
    Square
}

impl Section {
    fn push(&mut self, x: Rc<Cell>){
        self.vec.push(x);
    }

    /// Short-hand for accessing `vec` and calling it's `get` method.
    pub fn get(&self, index: usize) -> Option<&Rc<Cell>>{
        self.vec.get(index)
    }

    fn new(index: usize, line_type: SectionType) -> Section {
        Section {
            vec: Vec::new(),
            do_update: RefCell::new(false),
            index,
            section_type: line_type
        }
    }

    /// Return a copy of whether this `Section` has been marked for the solver to work on it or not.
    pub fn do_update(&self) -> bool {
        let do_update = &self.do_update.borrow();
        let do_update = &**do_update;

        return do_update.clone();
    }
}

type MultiMut<T> = Rc<RefCell<T>>;

/// A representation of a Sudoku grid.
pub struct Grid {
    pub rows: Vec<MultiMut<Section>>, // Read from top to bottom
    pub columns: Vec<MultiMut<Section>>,
    pub sections: Vec<MultiMut<Section>>,
}

impl Grid {
    /// Generate a new empty `Grid` with full empty possibilities for each `Cell`
    pub fn new() -> Grid {

        let mut rows: Vec<MultiMut<Section>> = Vec::new();
        let mut columns: Vec<MultiMut<Section>> = Vec::new();
        let mut sections: Vec<MultiMut<Section>> = Vec::new();

        for i in 0..9 {
            rows.push(Rc::new(RefCell::new(Section::new(i, SectionType::Row))));
            columns.push(Rc::new(RefCell::new(Section::new(i, SectionType::Column))));
            sections.push(Rc::new(RefCell::new(Section::new(i, SectionType::Square))));
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

    /// Returns the `Cell` (in an `Rc`) at the specified coordinates.
    /// * `r` is the row coordinate (first row starting at 0)
    /// * `c` is the column coordinate (first column starting at 0)
    ///
    /// Returns None if the coordinates are out of bounds.
    pub fn get(&self, r: usize, c: usize) -> Option<Rc<Cell>> {

        let row = match self.rows.get(r) {
            Some(x) => x,
            None => return None
        };

        let row = &*(&**row).borrow();

        let cell = match row.get(c) {
            Some(x) => x,
            None => return None
        };

        return Some(Rc::clone(cell));
    }

    fn process_unknown(x: &Vec<u8>, digit: u8, row: &mut String){
        if x.contains(&digit) {
            row.push('*');
        } else{
            row.push(' ');
        }
    }

    /// Find the smallest empty `Cell` in terms of possibilities; returns `None` if all Cells have
    /// `Fixed` `CellValue`s.
    pub fn find_smallest_cell(&self) -> Option<Rc<Cell>>{
        let mut smallest_cell : Option<Rc<Cell>> = None;
        let mut smallest_size = usize::MAX;

        for x in 0..9 {
            for y in 0..9 {
                let cell_rc = self.get(x, y).unwrap();
                let cell = &*self.get(x, y).unwrap();
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
            }
        }
        smallest_cell
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