use uint::construct_uint;
use serde::Serialize;
construct_uint!{
    #[derive(Serialize)]
    pub struct U256(4);
}

pub mod sha256;
pub mod types;
pub mod util;
pub mod crypto;

