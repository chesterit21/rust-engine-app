//! Utility functions

use uuid::Uuid;

pub fn is_valid_uuid(s: &str) -> bool {
    Uuid::parse_str(s).is_ok()
}

pub fn mask_email(email: &str) -> String {
    if let Some(at_pos) = email.find('@') {
        let (local, domain) = email.split_at(at_pos);
        if local.len() <= 2 {
            format!("{}***{}", &local[..1], domain)
        } else {
            format!("{}***{}", &local[..2], domain)
        }
    } else {
        "***".to_string()
    }
}
