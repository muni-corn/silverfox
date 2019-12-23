use std::env;
use std::path::PathBuf;

mod ledger;

fn main() {
    // get the $LEDGER_FILE env var
    let ledger_path = get_ledger_path().unwrap();
    // let mvelopes_file: String = get_mvelopes_file().unwrap();

    // parse ledger
    if let Err(e) = ledger::Ledger::from_file(&ledger_path) {
        eprintln!("{}", e)
    }
}

// fn get_mvelopes_file() -> Result<String, String> {
//     // attempt to get from env variable $MVELOPES_FILE first
//     match env::var("MVELOPES_FILE") {
//         Ok(f) => Ok(f),
//         Err(e) => {
//             println!("$MVELOPES_FILE variable probably doesn't exist ('{}')", e);

//             // or attempt to get sibling file to $LEDGER_FILE
//             match get_ledger_path() {
//                 Ok(p) => {
//                     let parent_dir_path = p.parent();
//                     let mvelopes_sibling_path = parent_dir_path.join("mvelopes.journal");

//                     match mvelopes_sibling_path.to_str() {
//                         Some(p) => Ok(String::from(p)),
//                         None => Err(String::from("Getting sibling path didn't work"))
//                     }
//                 },
//                 Err(e) => {
//                     Err(format!("mvelopes file must be provided manually, due to '{}'", e))
//                 }
//             }
//         }
//     }
// }

fn get_ledger_path() -> Result<PathBuf, env::VarError> {
    match env::var("LEDGER_FILE") {
        Ok(v) => Ok(PathBuf::from(v)),
        Err(e) => Err(e)
    }
}
