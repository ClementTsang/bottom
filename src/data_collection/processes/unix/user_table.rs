use hashbrown::HashMap;

use crate::utils::error;

#[derive(Debug, Default)]
pub struct UserTable {
    pub uid_user_mapping: HashMap<libc::uid_t, String>,
}

impl UserTable {
    pub fn get_uid_to_username_mapping(&mut self, uid: libc::uid_t) -> error::Result<String> {
        if let Some(user) = self.uid_user_mapping.get(&uid) {
            Ok(user.clone())
        } else {
            // SAFETY: getpwuid returns a null pointer if no passwd entry is found for the uid
            let passwd = unsafe { libc::getpwuid(uid) };

            if passwd.is_null() {
                Err(error::BottomError::QueryError("Missing passwd".into()))
            } else {
                // SAFETY: We return early if passwd is null.
                let username = unsafe { std::ffi::CStr::from_ptr((*passwd).pw_name) }
                    .to_str()?
                    .to_string();
                self.uid_user_mapping.insert(uid, username.clone());

                Ok(username)
            }
        }
    }
}
