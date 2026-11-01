use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &str = "SALEEM Quarantine";
pub const MODULE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuarantineHealth {
    pub module_name: String,
    pub module_version: String,
    pub status: String,
}

pub fn get_module_health() -> QuarantineHealth {
    QuarantineHealth {
        module_name: MODULE_NAME.to_string(),
        module_version: MODULE_VERSION.to_string(),
        status: "READY".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quarantine_module_health() {
        let h = get_module_health();
        assert_eq!(h.module_name, "SALEEM Quarantine");
        assert_eq!(h.status, "READY");
    }
}
