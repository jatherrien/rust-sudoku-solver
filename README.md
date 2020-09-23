# Sudoku Solver

This project started as a way for me to learn Rust. 
I'd been trying to learn it for a few years, and I've built a Sudoku solver in the past for other languages when I wanted to learn them since it's a small but complex enough task that you get to experience enough of the language; so I thought I'd try the same with Rust.

To build this you'll need to install [Cargo](https://www.rust-lang.org/). 
Afterwards, and after you've cloned this project, navigate to the project root and run `cargo build --release`. 
Two binaries, `solver` and `generator` will be generated in `target/release/`.

Try running both of them, first with the `-h` flag to see what other arguments they take. 
* `solver` reads a CSV file for a puzzle, prints it, solves it, and then prints the solved version. Some example CSV files are in the `puzzle` folder.
* `generator` tries to generate a new puzzle from scratch. You can set a maximum number of hints that it will allow and it will try to generate a puzzle that meets that requirement. You can also optionally write it to a CSV file. Be warned, however, that the puzzles that it generates aren't particularly difficult; most puzzles have at least 25 hints, usually much more. If I work on this in the future I'll want to find ways to improve the difficulty.

Regarding code quality, I could probably have commented more and I certainly should have written more unit tests. 
I also wish that I didn't rely so heavily on `Rc` & `RefCell`, which provide ways to get around (sometimes necessarily) the compiler's strict rules on references and ownership. 
That said, for a project that was really designed for me to learn a language I'm pretty happy with how it turned out.

If you have any questions about my project feel free to [email me](mailto:joel@joeltherrien.ca).   