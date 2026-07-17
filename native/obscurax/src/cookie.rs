use obscura::CookieStore;
use rustler::Resource;

pub struct CookieStoreHandle {
    pub store: CookieStore,
}

#[rustler::resource_impl]
impl Resource for CookieStoreHandle {}

impl CookieStoreHandle {
    pub fn new(store: CookieStore) -> Self {
        Self { store }
    }
}
