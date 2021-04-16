#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

use cosmian_std::{prelude::*, scale::SecretModp};
use cosmian_std::{scale, scale::println, InputRow, OutputRow};
use scale_std::slice::Slice;

// Players list

const ALICE: Player<0> = Player::<0>;
const BOB: Player<1> = Player::<1>;
const CHARLIE: Player<2> = Player::<2>;

// Example program
#[cosmian_std::main(KAPPA = 40)]
#[inline(never)]
fn main() {
    // Read first row of each players
    let mut alice_next_now = read_tabular(ALICE, 2);

    println!("##### Reading input row from player 0");
    let mut first_row_player_0 = InputRow::read(ALICE);
    println!("##### Reading input row from player 1");
    let mut first_row_player_1 = InputRow::read(BOB);
    println!("##### Reading input row from player 2");
    let mut first_row_player_2 = InputRow::read(CHARLIE);

    // Send data to output of each player

    println!("##### Writing output row for player 0");
    let mut first_output_row_player_0 = OutputRow::new(ALICE);
    first_output_row_player_0.append(
        first_row_player_0
            .next_col() // Fetch the next column
            .unwrap()
            .into_secret_modp() // Convert the column of the row into SecretModP
            .expect("value should be i64"),
    );
    println!("##### Writing output row for player 1");
    let mut first_output_row_player_1 = OutputRow::new(BOB);
    first_output_row_player_1.append(
        first_row_player_1
            .next_col() // Fetch the next column
            .unwrap()
            .into_secret_modp() // Convert the column of the row into SecretModP
            .expect("value should be i64"),
    );
    println!("##### Writing output row for player 2");
    let mut first_output_row_player_2 = OutputRow::new(CHARLIE);
    first_output_row_player_2.append(
        first_row_player_2
            .next_col() // Fetch the next column
            .unwrap()
            .into_secret_modp() // Convert the column of the row into SecretModP
            .expect("value should be i64"),
    );

    // All rows are automatically flushed thanks to the drop implementation
    println!("If you're running in emulation mode, please check into data/outputs directory the output of each players");
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
                //  "ERROR: player",
                //     P,
                //     ": invalid number of columns: ",
                //     num_cols as i64,
                //     "expected but ",
                //     i as i64,
                //     "found!\n"
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        // An example of test which can be run with `bash test.sh`
        cosmian_std::test!("example");
    }
}
