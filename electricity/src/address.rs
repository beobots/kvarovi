use nom::{
  IResult,
  bytes::complete::{tag, take_while_m_n},
  combinator::map_res,
  sequence::tuple
};

#[derive(Debug)]
pub struct Address {
    pub city: String,
    pub municipality: String,
    pub settlement: String,
    pub street: String,
    pub building: String,
}

impl Address {
    // pub fn from_str(value: &str) -> Self {
    //     Address {}
    // }
}



// parse A to Vec<Address>
 
