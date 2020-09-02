use std::borrow::BorrowMut;

enum CellValue {
    FIXED(u8),
    GUESS(u8),
    UNKNOWN(Vec<u8>)
}

struct Cell(CellValue);

struct Grid<'a> {
    rows: Vec<[&'a mut Cell; 9]>, // Read from top to bottom
    columns: Vec<[&'a Cell; 9]>, // Left to right
    sections: Vec<[&'a Cell; 9]> // left to right, top to bottom
}

impl<'a> Grid<'a> {
    fn get(&self, r: usize, c: usize) -> Option<&Cell> {
        let row = self.rows.get(r)?;
        let cell = &row[c];

        return Some(&cell);
    }

    fn get_mut(&'a mut self, r: usize, c: usize) -> Option<&'a mut Cell> {
        let row : &'a mut [&'a mut Cell; 9] = self.rows.get_mut(r)?;
        let cell : &'a mut Cell = row[c].borrow_mut();

        return Some(cell);

    }

    fn print(&self) {
        for r in 0..9 {
            if (r & 3 == 0) && (r > 0) {
                println!("---+---+---");
            }

            for c in 0..9 {
                if (c % 3 == 0) && (c > 0) {
                    print!("|");
                }

                let value = &self.get(r, c).unwrap().0;
                match value {
                    CellValue::FIXED(x) => print!("{}", x),
                    _ => print!(" ")
                };

            }

            print!("\n");
        }
    }
}

fn main() {
    println!("Hello, world!");
    let grid = retrieve_grid();
    grid.print();

}

fn empty_grid<'a>() -> Grid<'a> {
    let mut placeholder_cell : Cell = Cell(CellValue::FIXED(0));

    // Rows first; we need to create cells for all of them
    let mut rows : Vec<[&'a mut Cell; 9]> = Vec::new();
    for _r in 0..9 {
        let mut new_row : [&'a mut Cell; 9] = [&mut placeholder_cell; 9];
        for i in 0..9{
            new_row[i] = &mut initial_empty_cell();
        }

        rows.push(new_row);
    }

    // Columns next; now we have to retrieve the cells from the different rows
    let mut columns : Vec<[&'a Cell; 9]> = Vec::new();
    for c in 0..9 {
        let mut new_column : [&'a Cell; 9] = [&placeholder_cell; 9];
        for r in 0..9{
            new_column[r] = rows.get(r).unwrap()[c]; // TODO - improve performance by using get_unchecked
        }
        columns.push(new_column);
    }

    // Sections next; now we have to retrieve the cells from different rows and columns
    // We read sections from left to right, top to bottom
    let mut sections : Vec<[&'a Cell; 9]> = Vec::new();
    for r in 0..3 {
        for c in 0..3 {
            let mut new_section : [&'a Cell; 9] = [&placeholder_cell; 9];

            for internal_r in 0..3 {
                let global_r = 3*r + internal_r;

                for internal_c in 0..3 {
                    let global_c = 3*c + internal_c;
                    let index = 3*internal_c + internal_r;

                    new_section[index] = rows.get(global_r).unwrap()[global_c];
                }
            }

            sections.push(new_section);

        }
    }

    return Grid {rows, columns, sections};
}

fn initial_empty_cell() -> Cell {
    return Cell(CellValue::UNKNOWN(vec![1, 2, 3, 4, 5, 6, 7, 8, 9]));
}

/**
    For now this is a stub with a pre-programmed grid; later I'll add functionality to read a CSV file
 */
fn retrieve_grid<'a>() -> Grid<'a> {
    let mut grid = empty_grid();

    grid.get_mut(0, 4).unwrap().0 = CellValue::FIXED(8);
    grid.get_mut(0, 5).unwrap().0 = CellValue::FIXED(5);
    grid.get_mut(0, 6).unwrap().0 = CellValue::FIXED(6);

    grid.get_mut(2, 3).unwrap().0 = CellValue::FIXED(9);
    grid.get_mut(2, 4).unwrap().0 = CellValue::FIXED(4);
    grid.get_mut(2, 5).unwrap().0 = CellValue::FIXED(3);
    grid.get_mut(2, 6).unwrap().0 = CellValue::FIXED(5);
    grid.get_mut(2, 7).unwrap().0 = CellValue::FIXED(7);

    grid.get_mut(3, 0).unwrap().0 = CellValue::FIXED(8);
    grid.get_mut(3, 2).unwrap().0 = CellValue::FIXED(2);
    grid.get_mut(3, 3).unwrap().0 = CellValue::FIXED(6);
    grid.get_mut(3, 4).unwrap().0 = CellValue::FIXED(7);
    grid.get_mut(3, 5).unwrap().0 = CellValue::FIXED(4);
    grid.get_mut(3, 6).unwrap().0 = CellValue::FIXED(9);

    grid.get_mut(4, 4).unwrap().0 = CellValue::FIXED(9);
    grid.get_mut(4, 8).unwrap().0 = CellValue::FIXED(5);

    grid.get_mut(5, 1).unwrap().0 = CellValue::FIXED(6);
    grid.get_mut(5, 6).unwrap().0 = CellValue::FIXED(2);

    grid.get_mut(6, 1).unwrap().0 = CellValue::FIXED(8);
    grid.get_mut(6, 8).unwrap().0 = CellValue::FIXED(2);

    grid.get_mut(7, 3).unwrap().0 = CellValue::FIXED(7);
    grid.get_mut(7, 5).unwrap().0 = CellValue::FIXED(6);
    grid.get_mut(7, 7).unwrap().0 = CellValue::FIXED(5);
    grid.get_mut(7, 8).unwrap().0 = CellValue::FIXED(4);

    grid.get_mut(8, 2).unwrap().0 = CellValue::FIXED(7);
    grid.get_mut(8, 3).unwrap().0 = CellValue::FIXED(4);


    return grid;
}
