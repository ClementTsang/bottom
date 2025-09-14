use std::sync::Arc;

use hashbrown::HashMap;

use crate::collection::error::{CollectionError, CollectionResult};

#[derive(Debug, Default)]
pub struct UserTable {
    pub uid_user_mapping: HashMap<libc::uid_t, Arc<str>>,
}

impl UserTable {
    /// Get the username associated with a UID. On first access of a name, it will
    /// be cached for future accesses.
    pub fn uid_to_username(&mut self, uid: libc::uid_t) -> CollectionResult<Arc<str>> {
        if let Some(user) = self.uid_user_mapping.get(&uid) {
            Ok(user.clone())
        } else {
            // SAFETY: getpwuid returns a null pointer if no passwd entry is found for the uid which we check.
            let passwd = unsafe { libc::getpwuid(uid) };

            if passwd.is_null() {
                Err("passwd is inaccessible".into())
            } else {
                // SAFETY: We return early if passwd is null.
                let username: Arc<str> = unsafe { std::ffi::CStr::from_ptr((*passwd).pw_name) }
                    .to_str()
                    .map_err(|err| CollectionError::General(err.into()))?
                    .into();

                self.uid_user_mapping.insert(uid, username.clone());

                Ok(username)
            }
        }
    }
}
