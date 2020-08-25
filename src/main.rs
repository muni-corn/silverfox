pub mod account;
pub mod amount;
pub mod entry;
pub mod envelope;
pub mod errors;
pub mod flags;
pub mod importer;
pub mod ledger;
pub mod posting;
pub mod utils;

fn main() {
    match flags::CommandFlags::parse_from_env() {
        Ok(f) => if let Err(e) = f.execute() {
            eprintln!("{}", e)
        },
        Err(e) => eprintln!("{}", e),
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
