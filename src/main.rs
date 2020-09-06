use std::rc::Rc;
use std::cell::RefCell;

struct Grid {

    rows: Vec<Vec<Rc<RefCell<CellValue>>>>, // Read from top to bottom
    columns: Vec<Vec<Rc<RefCell<CellValue>>>>,
    sections: Vec<Vec<Rc<RefCell<CellValue>>>>
}

enum CellValue {
    FIXED(u8),
    GUESS(u8),
    UNKNOWN(Vec<u8>)
}

impl Grid {
    fn get(&self, r: usize, c: usize) -> Result<Rc<RefCell<CellValue>>, &str> {
        if (r > 9) | (c > 9) {
            return Err("Row or column indices are out of bounds");
        }

        let row = match self.rows.get(r) {
            Some(x) => x,
            None => {return Err("Row index is out of bounds")}
        };

        let cell = match row.get(c) {
            Some(x) => x,
            None => {return Err("Column index is out of bounds")}
        };

        return Ok(Rc::clone(cell));
    }

    fn print(&self) {
        for r in 0..9 {
            if (r % 3 == 0) && (r > 0) {
                println!("---+---+---");
            }

            for c in 0..9 {
                if (c % 3 == 0) && (c > 0) {
                    print!("|");
                }

                let value = self.get(r, c).unwrap_or_else(|err| panic!());
                match *value.borrow() {
                    CellValue::FIXED(x) => print!("{}", x),
                    _ => print!(" ")
                };
            }

            print!("\n");
        }
    }

    fn new() -> Grid {
        // Rows first; we need to create cells for all of them
        let mut rows: Vec<Vec<Rc<RefCell<CellValue>>>> = Vec::new();
        for _r in 0..9 {
            let mut new_row: Vec<Rc<RefCell<CellValue>>> = Vec::new();

            for _i in 0..9 {
                let empty_cell = initial_empty_cell();
                new_row.push(Rc::new(empty_cell));

            }
            rows.push(new_row);
        }

            // Columns next; now we have to retrieve the cells from the different rows
            let mut columns : Vec<Vec<Rc<RefCell<CellValue>>>> = Vec::new();
            for c in 0..9 {
                let mut new_column : Vec<Rc<RefCell<CellValue>>> = Vec::new();
                for r in 0..9{
                    new_column.push(Rc::clone(&rows.get(r).unwrap()[c]));
                }
                columns.push(new_column);
            }

            // Sections next; now we have to retrieve the cells from different rows and columns
            // We read sections from left to right, top to bottom
            let mut sections : Vec<Vec<Rc<RefCell<CellValue>>>> = Vec::new();
            for r in 0..3 {
                for c in 0..3 {
                    let mut new_section : Vec<Rc<RefCell<CellValue>>> = Vec::new();

                    for internal_r in 0..3 {
                        let global_r = 3*r + internal_r;

                        for internal_c in 0..3 {
                            let global_c = 3*c + internal_c;

                            new_section.push(Rc::clone(&rows.get(global_r).unwrap()[global_c]));
                        }
                    }

                    sections.push(new_section);

                }
            }

        return Grid { rows, columns, sections };
    }
}

fn initial_empty_cell() -> RefCell<CellValue> {
    return RefCell::new(CellValue::UNKNOWN(vec![1, 2, 3, 4, 5, 6, 7, 8, 9]));
}


fn main() {
    println!("Hello, world!");
    let grid = retrieve_grid();
    grid.print();

}




/**
    For now this is a stub with a pre-programmed grid; later I'll add functionality to read a CSV file
 */
fn retrieve_grid() -> Grid {
    let grid = Grid::new();
    

    grid.get(0, 4).unwrap().replace(CellValue::FIXED(8));
    grid.get(0, 5).unwrap().replace(CellValue::FIXED(5));
    grid.get(0, 6).unwrap().replace(CellValue::FIXED(6));

    grid.get(2, 3).unwrap().replace(CellValue::FIXED(9));
    grid.get(2, 4).unwrap().replace(CellValue::FIXED(4));
    grid.get(2, 5).unwrap().replace(CellValue::FIXED(3));
    grid.get(2, 6).unwrap().replace(CellValue::FIXED(5));
    grid.get(2, 7).unwrap().replace(CellValue::FIXED(7));

    grid.get(3, 0).unwrap().replace(CellValue::FIXED(8));
    grid.get(3, 2).unwrap().replace(CellValue::FIXED(2));
    grid.get(3, 3).unwrap().replace(CellValue::FIXED(6));
    grid.get(3, 4).unwrap().replace(CellValue::FIXED(7));
    grid.get(3, 5).unwrap().replace(CellValue::FIXED(4));
    grid.get(3, 6).unwrap().replace(CellValue::FIXED(9));

    grid.get(4, 4).unwrap().replace(CellValue::FIXED(9));
    grid.get(4, 8).unwrap().replace(CellValue::FIXED(5));

    grid.get(5, 1).unwrap().replace(CellValue::FIXED(6));
    grid.get(5, 6).unwrap().replace(CellValue::FIXED(2));

    grid.get(6, 1).unwrap().replace(CellValue::FIXED(8));
    grid.get(6, 8).unwrap().replace(CellValue::FIXED(2));

    grid.get(7, 3).unwrap().replace(CellValue::FIXED(7));
    grid.get(7, 5).unwrap().replace(CellValue::FIXED(6));
    grid.get(7, 7).unwrap().replace(CellValue::FIXED(5));
    grid.get(7, 8).unwrap().replace(CellValue::FIXED(4));

    grid.get(8, 2).unwrap().replace(CellValue::FIXED(7));
    grid.get(8, 3).unwrap().replace(CellValue::FIXED(4));
    
    /*
    
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
    
     */


    return grid;
}
