// Copyright (c) 2021, COSIC-KU Leuven, Kasteelpark Arenberg 10, bus 2452, B-3001 Leuven-Heverlee, Belgium.
// Copyright (c) 2021, Cosmian Tech SAS, 53-55 rue La Boétie, Paris, France.

fn main() {
    println!(
        "cargo:rerun-if-changed={}",
        option_env!("SHARING_DATA").unwrap_or("./SharingData.txt")
    );
    println!("cargo:rerun-if-env-changed=SHARING_DATA");
}
