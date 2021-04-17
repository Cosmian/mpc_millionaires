#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

use cosmian_std::{
    prelude::*,
    scale::{Reveal, ScaleCmp, SecretI64, SecretModp},
};
use cosmian_std::{scale, scale::println, InputRow, OutputRow};
use scale_std::slice::Slice;

// Players list
const ALICE: Player<0> = Player::<0>;
const BOB: Player<1> = Player::<1>;
const CHARLIE: Player<2> = Player::<2>;

// the numbér of columns provided by the players
// i.e (year, wealth)
const NUM_COLS: u64 = 2;

#[cosmian_std::main(KAPPA = 40)]
#[inline(never)]
fn main() {
    println!("Program starting");
    // A simple row counter which is only there to improve debug information.
    // This is a clear text value which is visible as such in the program memory
    // and is therefore "leaked". Remove it with the associated `print()` lines
    // if you want to hide this information
    let mut row_counter: u64 = 0;
    loop {
        // Read the rows of Alice one by one
        let alice_next_now = match read_tabular(ALICE, NUM_COLS) {
            None => {
                //no more records => end of program
                println!("End of Alice's rows");
                break;
            }
            Some(row) => row,
        };
        row_counter += 1;
        println!("Processing Alice row ", row_counter);
        // we can use `get_unchecked` here
        // because `read_tabular` guarantees that we have 2 columns,
        // These values have type `SecretModp` i.e. they are secret/encrypted values
        // and are not visible in the players program memory
        let alice_year = *alice_next_now.get_unchecked(0);
        let alice_wealth = *alice_next_now.get_unchecked(1);
        // now (secretly) find the same year on Bob's records
        let bob_row = match find_tabular(BOB, 0, &alice_year, NUM_COLS) {
            None => {
                //Bob does no have this year => end of program
                println!("End of Bob's rows");
                break;
            }
            Some(row) => row,
        };
        let bob_wealth = *bob_row.get_unchecked(1);
        // same thing with Charlie
        let charlie_row = match find_tabular(CHARLIE, 0, &alice_year, NUM_COLS) {
            None => {
                //Charlie does no have this year => end of program
                println!("End of Charlie's rows");
                break;
            }
            Some(row) => row,
        };
        let charlie_wealth = *charlie_row.get_unchecked(1);

        // we have now collected all the secret wealth of all participants for the year

        // Prepare `OutputRow`s to reveal data to the players
        let mut alice_output = OutputRow::new(ALICE);
        let mut bob_output = OutputRow::new(BOB);
        let mut charlie_output = OutputRow::new(CHARLIE);

        // The `year` is the data we want to output in the first column of each participant
        alice_output.append(alice_year);
        bob_output.append(alice_year);
        charlie_output.append(alice_year);

        // In the second column, we will output back its **own** wealth
        // to each participant
        alice_output.append(alice_wealth);
        bob_output.append(bob_wealth);
        charlie_output.append(charlie_wealth);

        // Let us calculate the total wealth for the year
        // which we are going to reveal to all participants in the 3rd column
        // Arithmetic operations are performed over the `SecretModp` type
        // They can mix secret and clear text scalars
        let total_wealth: SecretModp = alice_wealth + bob_wealth + charlie_wealth;
        // the total is still secret. We can selectively choose who we are
        // going to reveal it to. In this case: we reveal it to all participants
        alice_output.append(total_wealth);
        bob_output.append(total_wealth);
        charlie_output.append(total_wealth);

        // the final step is to calculate the rank for each participant
        // and reveal it that to the participant only. We must therefore make sure
        // our ranking algorithms does not reveal/leak any information.
        // This is exactly what the `secretly_rank` algorithm below does

        // First, we prepare a Slice with the wealths
        // Since secret comparisons are performed on boolean circuits
        // we need to switch to another integer representation for the wealths
        let mut secret_wealths: Slice<SecretI64> = Slice::uninitialized(3);
        secret_wealths.set(0, &SecretI64::from(alice_wealth));
        secret_wealths.set(1, &SecretI64::from(bob_wealth));
        secret_wealths.set(2, &SecretI64::from(charlie_wealth));

        // call the ranking algorithm which is going to output the ranks
        // in the same order as the inputs
        let ranks = secretly_rank(&secret_wealths, true);

        // privately output its rank to each participant
        alice_output.append(SecretModp::from(*ranks.get_unchecked(0)));
        bob_output.append(SecretModp::from(*ranks.get_unchecked(1)));
        charlie_output.append(SecretModp::from(*ranks.get_unchecked(2)));

        // as the `OutputRows` are dropped, they will be automatically
        // flushed to each participant
    }
    println!(".... end of Program");
}

/// Read input data from a Player and expect it to be in a tabular format
/// with one scalar per column and a fixed number of columns
/// (e.g. a row of a CSV file)
fn read_tabular<const P: u32>(player: Player<P>, num_cols: u64) -> Option<Slice<SecretModp>> {
    let mut row = InputRow::read(player);
    let mut result = Slice::uninitialized(num_cols);
    for i in 0..num_cols {
        let next_column = match row.next_col() {
            Some(c) => c,
            None => {
                //there is no more data to be read
                if i == 0 {
                    // this is the en of the dataset
                    return None;
                }
                // this will write the entry in the logs
                println!(
                    "ERROR: player ",
                    P, ": invalid number of columns: ", num_cols, " expected but ", i, " found!"
                );
                scale::panic!(
                    "ERROR: player ",
                    P,
                    ": invalid number of columns: ",
                    num_cols,
                    " expected but ",
                    i,
                    " found!"
                );
                // trick the compiler
                return None;
            }
        };
        // we expect a single scalar value in the column
        let value = next_column
            .into_secret_modp()
            .expect("There should be a single scalar value in the column");
        result.set(i, &value);
    }
    Some(result)
}

/// Read tabular rows of data of a Player (see `read_tabular`)
/// Until it finds `value` in the column `column` number (starting from 0)
fn find_tabular<const P: u32>(
    player: Player<P>,
    column: u64,
    value: &SecretModp,
    num_cols: u64,
) -> Option<Slice<SecretModp>> {
    // secret comparisons are performed on boolean circuits
    // so we need to switch to another representation for our value
    let value_f2: SecretI64 = (*value).into();
    loop {
        match read_tabular(player, num_cols) {
            None => {
                return None;
            }
            Some(row) => {
                // convert the value read, as above
                let this_value: SecretI64 = (*row.get_unchecked(column)).into();
                // we need to reveal the equality to test it
                // in clear text with an if
                if this_value.eq(value_f2).reveal() {
                    // ok found
                    return Some(row);
                }
                // not found =>_loop
            }
        };
    }
}

/// Rank the values from 1..n provided in tre `values` slice of size `n`.
/// The output slice will contain the rank of the corresponding value in the
/// `values` input slice i.e input values of:
///
///  -  Secret([11, 33, 22]) will output Secret([1, 3, 2 ]) ascending
///  -  Secret([11, 33, 22]) will output Secret([3, 1, 2 ]) descending
///
/// The algorithm is designed in such a way all data stay secret
/// during the processing and nothing is revealed
fn secretly_rank(values: &Slice<SecretI64>, descending: bool) -> Slice<SecretI64> {
    let n = values.len();
    let mut ranks: Slice<SecretI64> = Slice::uninitialized(n);
    for left in 0..n - 1 {
        for right in left + 1..n {
            let left_value = &*values.get_unchecked(left);
            let right_value = &*values.get_unchecked(right);
            let cmp = cmp(left_value, right_value);
            if descending {
                ranks.set(left, &(*ranks.get_unchecked(left) - cmp));
                ranks.set(right, &(*ranks.get_unchecked(right) + cmp));
            } else {
                ranks.set(left, &(*ranks.get_unchecked(left) + cmp));
                ranks.set(right, &(*ranks.get_unchecked(right) - cmp));
            }
        }
    }
    rescale(&ranks)
}

/// Secretly compare 2 secret values.
///
/// This function returns a **secret**
///
/// - Secret(-1) if a <= b
/// - Secret(1) otherwise
#[inline(always)]
fn cmp(a: &SecretI64, b: &SecretI64) -> SecretI64 {
    let cmp: SecretI64 = a.le(*b).into();
    -2 * cmp + 1
}

/// Used by the `rank` function to rescale the ranks before output.
///
/// The base arithmetic algorithm will rank 3 values outputting [-2,0,2],
/// This rescales the ranks to [1,2,3]
fn rescale(indexes: &Slice<SecretI64>) -> Slice<SecretI64> {
    let n = indexes.len();
    let n_1 = SecretI64::from(n as i64 - 1);
    let mut rescaled: Slice<SecretI64> = Slice::uninitialized(n);
    for i in 0..n {
        let v = *indexes
            .get(i)
            .expect("there should be an index at that position");
        rescaled.set(i, &(((v + n_1) >> ConstU32::<1>) + 1));
    }
    rescaled
}

// To run the tests below, use the provided `test.sh` script
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        // An example of a successful test
        // which input and expected output data are located
        // in the `fixtures/success_test` folder
        cosmian_std::test!("success_test");
        // If you change any data in the input or output files,
        // the test will fail
    }
}
