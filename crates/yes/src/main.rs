// yes - output a string repeatedly until killed.
//
// Goal: print "y" on its own line forever. If arguments are given,
// print them joined with spaces instead.
// Hint: loop
// Try it: cargo run -p yes -- hello | head -3

fn main() {
    loop {
        print!("y\n");
    }
}
