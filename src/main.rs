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
// year, net_worth
const NUM_COLS: u64 = 2;

// Example program
#[cosmian_std::main(KAPPA = 40)]
#[inline(never)]
fn main() {
    println!("Program starting");
    // a simple row counter to improve debug information
    // this is a clear text value which is visible as such in the program memory
    let mut row_counter: u64 = 0;
    loop {
        // Read the rows of Alice ono by one
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
        // because `read_tabular` guarantees that we have 2 columns
        // these values are `SecretModP` i.e. secret/encrypted values
        let alice_year = *alice_next_now.get_unchecked(0);
        let alice_net_worth = *alice_next_now.get_unchecked(1);
        // now (secretly) find the same year on Bob's records
        let bob_row = match find_tabular(BOB, 0, &alice_year, NUM_COLS) {
            None => {
                //Bob does no have this year => end of program
                println!("End of Bob's rows");
                break;
            }
            Some(row) => row,
        };
        let bob_net_worth = *bob_row.get_unchecked(1);
        // same thing with Charlie
        let charlie_row = match find_tabular(CHARLIE, 0, &alice_year, NUM_COLS) {
            None => {
                //Charlie does no have this year => end of program
                println!("End of Charlie's rows");
                break;
            }
            Some(row) => row,
        };
        let charlie_net_worth = *charlie_row.get_unchecked(1);

        // we have now collected all the secret net worth for the years

        // Prepare `OutputRow`s to reveal data to the players
        let mut alice_output = OutputRow::new(ALICE);
        let mut bob_output = OutputRow::new(BOB);
        let mut charlie_output = OutputRow::new(CHARLIE);

        // The `year` is the data we want to output in each column
        alice_output.append(alice_year);
        bob_output.append(alice_year);
        charlie_output.append(alice_year);

        // let us calculate the net worth total which we are going to reveal to all participants
        let net_total: SecretModp = alice_net_worth + bob_net_worth + charlie_net_worth;
        // the total is still secret. We can selectively choose who we are
        // going to reveal it to. In this case: all participants
        alice_output.append(net_total);
        bob_output.append(net_total);
        charlie_output.append(net_total);

        // the final step is to calculate the rank for each participant
        // and reveal that to the participant only. So we must make sure
        // our ranking algorithms does not reveal/leak any information.
        // This what the algorithms below does

        // First, we prepare a Slice with the net worths
        // Since secret comparisons are performed on boolean circuits
        // we need to switch to another representation for the net worths
        let mut secret_net_worths: Slice<SecretI64> = Slice::uninitialized(3);
        secret_net_worths.set(0, &SecretI64::from(alice_net_worth));
        secret_net_worths.set(1, &SecretI64::from(bob_net_worth));
        secret_net_worths.set(2, &SecretI64::from(charlie_net_worth));

        // call the ranking algorithm which is going to output the ranks
        // in the same order as the inputs
        let ranks = secretly_rank(&secret_net_worths, true);

        // privately output its rank to each participant
        alice_output.append(SecretModp::from(*ranks.get_unchecked(0)));
        bob_output.append(SecretModp::from(*ranks.get_unchecked(1)));
        charlie_output.append(SecretModp::from(*ranks.get_unchecked(2)));

        // as the `OutputRows` are dropped, they will be automatically
        // flushed to each participant
    }
    println!(".... end of Program");
}

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
    ranks
}

#[inline(always)]
fn cmp(a: &SecretI64, b: &SecretI64) -> SecretI64 {
    let cmp: SecretI64 = a.le(*b).into();
    -2 * cmp + 1
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        // An example of test which can be run with `bash test.sh`
        cosmian_std::test!("example");
    }
}
