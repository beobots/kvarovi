use std::sync::OnceLock;

pub mod streets;

#[derive(Debug, Clone)]
pub struct Street {
    pub street_name: String,
    pub old_street_name: Option<String>,
    pub municipality: String,
    pub settlement: String,
    pub settlement_part: String,
    pub si_list: Option<String>,
}

static STREETS: OnceLock<Vec<Street>> = OnceLock::new();

pub fn get_streets() -> &'static Vec<Street> {
    STREETS.get_or_init(|| {
        crate::streets::get_streets_list()
    })
}
