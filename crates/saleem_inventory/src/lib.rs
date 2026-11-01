pub mod apps;
pub mod games;
pub mod leftovers;
pub mod models;

pub use apps::discover_installed_registry_apps;
pub use games::discover_installed_games;
pub use leftovers::{remove_verified_leftovers, scan_inventory_and_leftovers};
pub use models::{
    InstalledAppEntry, InventoryScanRequest, InventoryScanResult, LeftoverCandidate,
    LeftoverRemovalItemRequest, LeftoverRemovalOutcome, LeftoverRemovalReport,
    LeftoverRemovalRequest,
};

use serde::{Deserialize, Serialize};

pub const MODULE_NAME: &str = "SALEEM Inventory & Startup";
pub const MODULE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InventoryHealth {
    pub module_name: String,
    pub module_version: String,
    pub status: String,
}

pub fn get_module_health() -> InventoryHealth {
    InventoryHealth {
        module_name: MODULE_NAME.to_string(),
        module_version: MODULE_VERSION.to_string(),
        status: "READY".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_module_health() {
        let h = get_module_health();
        assert_eq!(h.module_name, "SALEEM Inventory & Startup");
        assert_eq!(h.status, "READY");
    }
}
