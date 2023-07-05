#[derive(Debug, Clone)]
pub struct Street<'a> {
    pub street_name: &'a str,
    pub old_street_name: Option<&'a str>,
    pub municipality: &'a str,
    pub settlement: &'a str,
    pub settlement_part: &'a str,
    pub si_list: Option<&'a str>,
}

pub type StaticStreet = Street<'static>;

pub static STREETS: &[StaticStreet] = include!(concat!(env!("OUT_DIR"), "/streets_data.rs"));
