use hashbrown::HashMap;

use crate::data_collection::error::{CollectionError, CollectionResult};

#[derive(Debug, Default)]
pub struct UserTable {
    pub uid_user_mapping: HashMap<libc::uid_t, String>,
}

impl UserTable {
    pub fn get_uid_to_username_mapping(&mut self, uid: libc::uid_t) -> CollectionResult<String> {
        if let Some(user) = self.uid_user_mapping.get(&uid) {
            Ok(user.clone())
        } else {
            // SAFETY: getpwuid returns a null pointer if no passwd entry is found for the
            // uid
            let passwd = unsafe { libc::getpwuid(uid) };

            if passwd.is_null() {
                Err("passwd is inaccessible".into())
            } else {
                // SAFETY: We return early if passwd is null.
                let username = unsafe { std::ffi::CStr::from_ptr((*passwd).pw_name) }
                    .to_str()
                    .map_err(|err| CollectionError::General(err.into()))?
                    .to_string();
                self.uid_user_mapping.insert(uid, username.clone());

                Ok(username)
            }
        }
    }
}
