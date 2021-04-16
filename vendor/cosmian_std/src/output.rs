#[cfg(feature = "emulate")]
use crate::CURRENT_TEST_NAME;
#[cfg(feature = "emulate")]
use scale::Reveal;
#[cfg(not(feature = "emulate"))]
use scale::{Channel, ConstI32};
use scale::{Player, SecretModp};

#[cfg(feature = "emulate")]
use once_cell::sync::Lazy;

#[cfg(feature = "emulate")]
use std::{
    collections::{HashMap, HashSet},
    io::Write,
    sync::Mutex,
};

#[cfg(feature = "emulate")]
// Use lazy static to create output files
static PLAYERS_OUTPUT_FILES: Lazy<Mutex<HashSet<u32>>> = Lazy::new(|| Mutex::new(HashSet::new()));

#[cfg(feature = "emulate")]
#[doc(hidden)]
pub static PLAYERS_OUTPUT_ROWS: Lazy<Mutex<HashMap<u32, Vec<Vec<i64>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[cfg(feature = "emulate")]
const OUTPUT_DIR: &str = "./data/outputs";

/// Struct to send row to a player
pub struct OutputRow<const P: u32> {
    #[allow(dead_code)]
    player: Player<P>,
    #[cfg(feature = "emulate")]
    elements: Vec<SecretModp>,
    flushed: bool,
}

impl<const P: u32> OutputRow<P> {
    #[cfg(not(feature = "emulate"))]
    pub fn new(player: Player<P>) -> Self {
        Self {
            player,
            flushed: false,
        }
    }

    #[cfg(feature = "emulate")]
    pub fn new(player: Player<P>) -> Self {
        match CURRENT_TEST_NAME.with(|test_name| *test_name.borrow()) {
            None => {
                use std::fs;

                if let Err(err) = fs::create_dir_all("./data/outputs") {
                    if !matches!(err.kind(), std::io::ErrorKind::AlreadyExists) {
                        panic!("can't create ./data/outputs directory : {:?}", err);
                    }
                }

                let mut players_output_files = PLAYERS_OUTPUT_FILES
                    .lock()
                    .expect("player outputs file poisoned");

                if players_output_files.get(&P).is_none() {
                    let mut file = std::fs::OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(true)
                        .read(true)
                        .open(format!("{}/player_{}.json", OUTPUT_DIR, P))
                        .unwrap_or_else(|_| {
                            panic!("cannot create output data file for player {}", P)
                        });

                    file.write_all(b"[]")
                        .unwrap_or_else(|_| panic!("cannot write empty array for player {}", P));

                    file.flush()
                        .unwrap_or_else(|_| panic!("cannot flush writer for player {}", P));

                    players_output_files.insert(P);
                }
            }
            Some(_) => {
                let mut players_output_rows = PLAYERS_OUTPUT_ROWS
                    .lock()
                    .expect("players output rows poisoned");
                players_output_rows.entry(P).or_insert_with(Vec::new);
            }
        }

        Self {
            player,
            elements: vec![],
            flushed: false,
        }
    }

    /// Append a column into this row
    #[cfg(feature = "emulate")]
    pub fn append<T: Into<SecretModp>>(&mut self, col: T) {
        let secret_col = col.into();
        self.elements.push(secret_col);
    }

    /// Append a column into this row
    #[cfg(not(feature = "emulate"))]
    pub fn append<T: Into<SecretModp>>(&mut self, col: T) {
        let secret_col = col.into();

        secret_col.private_output(self.player, Channel::<0>);
    }

    /// Flush the row and send it (not mandatory, dropping the row will do the same)
    pub fn flush(mut self) {
        self.flushed = true;
        self._flush();
    }

    #[cfg(feature = "emulate")]
    fn _flush(&mut self) {
        match CURRENT_TEST_NAME.with(|test_name| *test_name.borrow()) {
            None => {
                let mut output: Vec<Vec<i64>> = {
                    let file = std::fs::OpenOptions::new()
                        .read(true)
                        .open(format!("{}/player_{}.json", OUTPUT_DIR, P))
                        .unwrap_or_else(|_| panic!("cannot open file for output for player {}", P));
                    let reader = std::io::BufReader::new(&file);

                    serde_json::from_reader(reader)
                        .unwrap_or_else(|_| panic!("cannot read output file for player {}", P))
                };

                output.push(
                    self.elements
                        .iter()
                        .map(|elt| i64::from(elt.reveal()))
                        .collect(),
                );

                let file = std::fs::OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .open(format!("{}/player_{}.json", OUTPUT_DIR, P))
                    .unwrap_or_else(|_| panic!("cannot open file for output for player {}", P));

                serde_json::to_writer_pretty(file, &output)
                    .unwrap_or_else(|_| panic!("cannot serialize to output file for player {}", P));
            }
            Some(_) => {
                // Set custom filepath
                let mut current_rows_cell = PLAYERS_OUTPUT_ROWS
                    .lock()
                    .expect("players output rows poisoned");
                current_rows_cell
                    .get_mut(&P)
                    .expect("should have an existing player entry in this hashmap")
                    .push(
                        self.elements
                            .iter()
                            .map(|elt| i64::from(elt.reveal()))
                            .collect(),
                    );
            }
        }
    }

    #[cfg(not(feature = "emulate"))]
    fn _flush(&mut self) {
        SecretModp::from(ConstI32::<0>).private_output(self.player, Channel::<1>);
    }
}

impl<const P: u32> Drop for OutputRow<P> {
    fn drop(&mut self) {
        if !self.flushed {
            self._flush()
        }
    }
}
