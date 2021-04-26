#[cfg(feature = "emulate")]
use crate::CURRENT_TEST_NAME;
#[cfg(not(feature = "emulate"))]
use scale::{Channel, Reveal};
use scale::{Player, SecretModp};
use scale_std::slice::Slice;

#[cfg(feature = "emulate")]
use once_cell::sync::Lazy;
#[cfg(feature = "emulate")]
use std::{collections::HashMap, sync::Mutex};

#[cfg(feature = "emulate")]
// Use lazy static to iterate over the data inside
static PLAYERS_DATA: Lazy<Mutex<HashMap<u32, Vec<Vec<serde_json::Value>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Struct to read row coming from input
pub struct InputRow<const P: u32> {
    #[allow(dead_code)]
    player: Player<P>,
    nb_col: u64,
    cursor: i64,
    #[cfg(feature = "emulate")]
    line_consumed_deleted: bool,
}

/// Describe a column and what it could be in a row
pub enum Column {
    Slice(Slice<SecretModp>),
    SecretModp(SecretModp),
}

impl Column {
    /// Convert to SecretModp if possible
    pub fn into_secret_modp(self) -> Option<SecretModp> {
        match self {
            Column::Slice(_) => None,
            Column::SecretModp(secret) => Some(secret),
        }
    }

    /// Convert to Slice<SecretModp> if possible
    pub fn into_slice(self) -> Option<Slice<SecretModp>> {
        match self {
            Column::Slice(slice) => Some(slice),
            Column::SecretModp(_) => None,
        }
    }
}

impl<const P: u32> InputRow<P> {
    /// Read input row from given player
    #[cfg(not(feature = "emulate"))]
    pub fn read(player: Player<P>) -> Self {
        let nb_col = SecretModp::private_input(player, Channel::<0>).reveal();

        Self {
            player,
            nb_col: i64::from(nb_col) as u64,
            cursor: 0,
        }
    }

    /// Read input row from given player
    #[cfg(feature = "emulate")]
    pub fn read(player: Player<P>) -> Self {
        use std::fs;

        let mut players_data = PLAYERS_DATA.lock().unwrap();
        let nb_col = if !players_data.contains_key(&P) {
            let current_test_name = CURRENT_TEST_NAME.with(|current_test_name| {
                let val = current_test_name.borrow();
                *val
            });
            let filepath = match current_test_name {
                Some(test_name) => {
                    format!("./fixtures/{}/inputs/player_{}.json", test_name, P)
                }
                None => {
                    format!("./data/inputs/player_{}.json", P)
                }
            };
            let res = fs::read(filepath)
                .unwrap_or_else(|_| panic!("cannot read emulate input data for player {}", P));
            let data: Vec<Vec<serde_json::Value>> = serde_json::from_slice(&res)
                .expect("data input json must be of type Vec<Vec<i64>>");
            let nb_col = data.get(0).map(|col| col.len()).unwrap_or_default();

            players_data.insert(P, data);

            nb_col
        } else {
            players_data
                .get(&P)
                .unwrap()
                .first()
                .map(|d| d.len())
                .unwrap_or_default()
        } as u64;

        Self {
            player,
            nb_col,
            cursor: -1,
            line_consumed_deleted: false,
        }
    }

    /// Get next column in this row
    #[cfg(not(feature = "emulate"))]
    pub fn next_col(&mut self) -> Option<Column> {
        if self.cursor >= self.nb_col as i64 {
            return None;
        }

        let col_len = SecretModp::private_input(self.player, Channel::<1>).reveal();
        let col_len = i64::from(col_len) as u64;
        self.cursor += 1;
        if col_len == 0 {
            None
        } else if col_len == 1 {
            Some(Column::SecretModp(SecretModp::private_input(
                self.player,
                Channel::<2>,
            )))
        } else {
            let mut data = Slice::uninitialized(col_len);

            for row_nb in 0..col_len {
                data.set(
                    row_nb,
                    &SecretModp::private_input(self.player, Channel::<2>),
                );
            }
            Some(Column::Slice(data))
        }
    }

    /// Get next column in this row
    #[cfg(feature = "emulate")]
    pub fn next_col(&mut self) -> Option<Column> {
        self.cursor += 1;
        if self.cursor > self.nb_col as i64 {
            self.delete_consumed_line();

            return None;
        }
        let current_row = {
            let players_data = PLAYERS_DATA.lock().unwrap();
            players_data.get(&P)?.first()?.clone()
        };

        let current_col = current_row.get(self.cursor as usize)?;

        match current_col {
            serde_json::Value::Number(num) => Some(Column::SecretModp(SecretModp::from(
                num.as_i64().expect("element must be an integer 64"),
            ))),
            serde_json::Value::Array(array) => {
                if array.len() == 1 {
                    return Some(Column::SecretModp(SecretModp::from(
                        array
                            .first()
                            .unwrap()
                            .clone()
                            .as_i64()
                            .expect("element must be an integer 64"),
                    )));
                }
                let mut slice = Slice::uninitialized(array.len() as u64);
                for (idx, elt) in array.iter().enumerate() {
                    if let serde_json::Value::Number(num) = elt {
                        let secret_modp =
                            SecretModp::from(num.as_i64().expect("element must be an integer 64"));
                        slice.set(idx as u64, &secret_modp);
                    } else {
                        panic!("element in array should be a number");
                    }
                }

                Some(Column::Slice(slice))
            }
            _ => {
                panic!("element in column should be a number or an array");
            }
        }
    }

    /// Flush the row to pass on the next row in a future iteration or creation of a row
    #[cfg(feature = "emulate")]
    pub fn flush(mut self) {
        self.delete_consumed_line();
    }

    /// Flush the row to pass on the next row in a future iteration or creation of a row
    #[cfg(not(feature = "emulate"))]
    pub fn flush(self) {}

    #[cfg(feature = "emulate")]
    fn delete_consumed_line(&mut self) {
        self.line_consumed_deleted = true;
        let mut players_data = PLAYERS_DATA.lock().unwrap();
        // Delete the line
        players_data
            .get_mut(&P)
            .expect("should have data for this player")
            .remove(0);
    }
}

impl<const P: u32> Iterator for InputRow<P> {
    type Item = Column;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_col()
    }
}

#[cfg(feature = "emulate")]
impl<const P: u32> Drop for InputRow<P> {
    fn drop(&mut self) {
        if self.cursor < self.nb_col as i64 && !self.line_consumed_deleted {
            let mut players_data = PLAYERS_DATA.lock().unwrap();
            // Delete the line
            players_data
                .get_mut(&P)
                .expect("should have data for this player")
                .remove(0);
        }
    }
}
