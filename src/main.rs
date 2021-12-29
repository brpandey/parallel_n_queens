// N-Queens

// Usage: $ cargo run 10
// Compiling parallel_n_queens v0.1.0 (.../parallel_n_queens)
//     Finished dev [unoptimized + debuginfo] target(s) in 3.69s
//     Running `target/debug/parallel_n_queens 10`
//     N Queens 10X10 Parallel & Recursive: 976.113125ms


// The n-queens puzzle is the problem of placing n queens on an n×n chessboard such that no two queens attack each other.
// Given an integer n, return all distinct solutions to the n-queens puzzle.
// Each solution contains a distinct board configuration of the n-queens' placement,
// where 'Q' and '.' both indicate a queen and an empty space respectively.

// Example:

// Input: 4
// Output: [
//     [".Q..",  // Solution 1
//      "...Q",
//      "Q...",
//      "..Q."],

//     ["..Q.",  // Solution 2
//      "Q...",
//      "...Q",
//      ".Q.."]
// ]

// Explanation: There exist two distinct solutions to the 4-queens puzzle as shown above.

/*
Note: Attempted standard thread
  use std::thread;

Note:
  Using threads without a threadpool breaks down at n = 10 or 724 board solutions
  Error msgs:
     ' panicked at 'failed to spawn thread: Os { code: 11, kind: WouldBlock, message: "Resource temporarily unavailable" }', /home/brpandey/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/src/libstd/thread/mod.rs:619:5
      :5::55thread '<unnamed>' panicked at 'called `Result::unwrap()` on an `Err` value: Any'
 */

use std::env;
use std::sync::{Arc, Mutex};
use std::time::{Instant};

const BOARD_SIZE: u32 = 4;

fn main() {
    // Collect args
    let args: Vec<String> = env::args().collect();

    let (size, display) = match args.len() {
        n if n <= 1 || n > 3 => (BOARD_SIZE, false),
        2 => (args[1].parse::<u32>().unwrap_or(BOARD_SIZE), false),
        3 => (args[1].parse::<u32>().unwrap_or(BOARD_SIZE), args[2].parse::<bool>().unwrap_or(false)),
        _ => panic!("Unreachable path"),
    };

    setup(size, display);

}

fn setup(size: u32, display: bool) {
/*
    // Note: Thought this would speed things up by predicted an accurate size but it looks like not
    let base: f32 = 2.54;         // an explicit type is required
    let denom = base.powf(size as f32);   
    let bounds: f64 = (factorial(size as usize) as f64) / (denom as f64);
*/

    // Setup collections for solutions and candidate board
    // Apparently solutions doesn't need to be mutable thanks to the Mutex that we wrap it in! (interior mutability)
    let mut solutions: Vec<Vec<String>> = Vec::with_capacity(size as usize);
    let mut board = vec![vec![b'.'; size as usize]; size as usize];

    // Create shared lock
    let solutions_lock: Arc<Mutex<Vec<Vec<String>>>> = Arc::new(Mutex::new(solutions));

    // Setup thread pool and thread safe structures
    let thread_count = size as usize;
    let thread_pool = rayon::ThreadPoolBuilder::new().num_threads(thread_count).build().unwrap();
    let sl = Arc::clone(&solutions_lock);

    // Setup timing
    let start = Instant::now();
    // Ensure that the threadpool has access to the context with which we want to run threads within
    thread_pool.install(|| solve_parallel(sl, &mut board, size));

    let duration = start.elapsed();

    if display {
        //Remove solutions value out of Arc and Mutex
        let lock = Arc::try_unwrap(solutions_lock).expect("Unable to shed Arc wrapping as Arc still has multiple owners");
        solutions = lock.into_inner().expect("Mutex lock can not be retrieved");
        println!("{} Board Solutions", solutions.len());
    }

    println!("N Queens {}X{} Parallel & Recursive: {:?}", size, size, duration);
}

// Parallel version which for NxN board puts N queens in N separate threads on the first row
// and then recurses with the non parallel solver 
fn solve_parallel(solutions_lock: Arc<Mutex<Vec<Vec<String>>>>, board: &mut Vec<Vec<u8>>, size: u32) {
    for col in 0..size {
        let slock = Arc::clone(&solutions_lock);
        let b = &mut board.to_vec();

        rayon::scope(move |_| {
            // For each thread place queens on each column for row 0
            b[0][col as usize] = b'Q';
            solve_helper(&slock, b, size, 1);
        });
    }
}


// Attempt to solve problem by placing queen in topmost row advancing
// to the right along all the squares until a spot is found, then
// recurse for the next row to put the next queen
fn solve_helper(solutions_lock: &Arc<Mutex<Vec<Vec<String>>>>, board: &mut Vec<Vec<u8>>, size: u32, row: u32) {

    // If we have placed all queens successfully on all rows 0..row,
    // the if row == size e.g. 4 means we have successfully finished placing 4 queens
    if row == size {
        let answer: Vec<String> = board.iter().map(|x| {String::from_utf8(x.to_vec()).unwrap()}).collect();
        {
            let mut unlocked = solutions_lock.lock().unwrap();
            unlocked.push(answer);
        }
    };

    // Attempt to successfully put Queen on every square in current row
    // When valid spot found, attempt to recurse to place remaining queens
    for col in 0..size {
        if safe(board, size as usize, row as usize, col as usize) {
            // Set queen on current square since there are no threats
            board[row as usize][col as usize] = b'Q';
            // Attempt to place the remaining queens successfully through recursive calls
            solve_helper(solutions_lock, board, size, row + 1);
            // Backtrack and undo the placement of the queen to
            // generate other solution combinations
            board[row as usize][col as usize] = b'.';
        }
    }
}


fn safe(board: &[Vec<u8>], size: usize, row: usize, column: usize) -> bool {
    // Note: We don't need to be concerned with checking rows below us (greater than the current row)

    // Range iterators
    let decr_rows = (0..row).rev();
    let dec_rows = decr_rows.clone();
    let decr_cols = (0..column).rev();
    let incr_cols = (column+1)..size;

    // Does queen exist at row and column location?
    let q = |r: usize, c: usize| {
        board[r][c] == b'Q'
    };

    // Cases
    // 1) if a queen exists on the same column
    // 2a) if a queen exists on top diagonal left \ (decreasing rows and decreasing columns)
    // 2b) if a queen exists on top diagonal right / (decreasing row and increasing columns)

    for r in 0..row { if q(r, column) { return false }; }; // 1
    for (r,c) in decr_rows.zip(decr_cols) { if q(r, c) { return false }; }; // 2a
    for (r,c) in dec_rows.zip(incr_cols) { if q(r, c) { return false }; }; // 2b

    true
}


/*
fn factorial(n: usize) -> usize {
let (mut result, mut i): (usize, usize) = (1,1);
while i <= n { result *= i; i += 1;};
result
}
*/
