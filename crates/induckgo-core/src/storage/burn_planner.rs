//! Works out what to delete when the person burns a profile's data (the Fire Button).

use crate::profile::profile_directories::ProfileLocation;
use crate::storage::burn_plan::BurnPlan;
use crate::storage::burn_scope::BurnScope;

pub struct BurnPlanner {
    location: ProfileLocation,
}

impl BurnPlanner {
    const STORAGE_DIRECTORIES: [&'static str; 4] = ["Local Storage", "IndexedDB", "Service Worker", "Session Storage"];

    #[must_use]
    pub fn new(location: ProfileLocation) -> Self {
        Self { location }
    }

    #[must_use]
    pub fn plan(&self, scope: &BurnScope) -> BurnPlan {
        match scope {
            BurnScope::Everything => BurnPlan {
                delete_all_cookies: true,
                clear_cache: true,
                storage_directories: Self::STORAGE_DIRECTORIES
                    .iter()
                    .map(|name| self.location.data.join(name))
                    .collect(),
                ..BurnPlan::default()
            },
            BurnScope::Site(site) => BurnPlan { cookie_sites: vec![site.to_ascii_lowercase()], ..BurnPlan::default() },
        }
    }
}
