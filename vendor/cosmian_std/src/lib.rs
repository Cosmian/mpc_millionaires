#![cfg_attr(not(feature = "emulate"), no_std)]

mod input;
mod output;

pub mod prelude;

pub use input::{Column, InputRow};
pub use output::OutputRow;
pub use scale;

#[cfg(feature = "emulate")]
use std::cell::RefCell;
#[cfg(feature = "emulate")]
#[doc(hidden)]
thread_local!(pub static CURRENT_TEST_NAME: RefCell<Option<&'static str>> = RefCell::new(None));

#[cfg(feature = "emulate")]
#[doc(hidden)]
pub use output::PLAYERS_OUTPUT_ROWS;

pub use scale::main;

#[cfg(feature = "emulate")]
#[doc(hidden)]
pub mod reexports {
    pub use serde_json;
}

#[macro_export]
macro_rules! test {
    ($test_name:expr) => {
        $crate::CURRENT_TEST_NAME.with(|current_test_name| {
            *current_test_name.borrow_mut() = Some($test_name);
        });

        main();

        let mut players_output_rows = $crate::PLAYERS_OUTPUT_ROWS
            .lock()
            .expect("players output rows poisoned");
        for (player, output_rows) in players_output_rows.iter() {
            let output: Vec<Vec<i64>> = {
                let file = std::fs::OpenOptions::new()
                    .read(true)
                    .open(format!(
                        "fixtures/{}/outputs/player_{}.json",
                        $test_name, player
                    ))
                    .unwrap_or_else(|_| {
                        std::panic!("cannot open file for output for player {}", player)
                    });
                let reader = std::io::BufReader::new(&file);

                $crate::reexports::serde_json::from_reader(reader).unwrap_or_else(|_| {
                    std::panic!("cannot read output file for player {}", player)
                })
            };

            assert_eq!(
                &output, output_rows,
                "Output rows are different for player {} (left is what is expected)",
                player
            );
        }

        *players_output_rows = std::collections::HashMap::new();
    };
}
