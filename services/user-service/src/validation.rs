use regex::Regex;
use crate::user::CreateUserRequest;
use crate::user::UpdateUserRequest;

pub fn validate_email(email: &str) -> Result<(), String> {
     let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
     if !email_regex.is_match(email) {
          return Err("Invalid email format".to_string());
     }
     Ok(())
}

pub fn validate_password(password: &str) -> Result<(), String> {
     if password.len() < 8 {
          return Err("Password must be at least 8 characters".to_string());
     }
     
     let forbidden_chars = ['!', '*', '&', '^', '%', '$', '#', '@', '(', ')', '-', '+', '=', '[', ']', '{', '}', '|', '\\', ':', ';', '"', '\'', '<', '>', ',', '.', '?', '/', '~', '`'];
     if password.chars().any(|c| forbidden_chars.contains(&c)) {
          return Err("Password contains forbidden characters".to_string());
     }
     
     Ok(())
}

pub fn validate_username(username: &str) -> Result<(), String> {
     if username.len() < 3 || username.len() > 30 {
          return Err("Username must be between 3 and 30 characters".to_string());
     }
     if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
          return Err("Username can only contain letters, numbers and underscore".to_string());
     }
     Ok(())
}

pub fn validate_create_user_request(req: &CreateUserRequest) -> Result<(), String> {
     validate_email(&req.email)?;
     validate_password(&req.password)?;
     validate_username(&req.username)?;
     Ok(())
}

pub fn validate_update_user_request(req: &UpdateUserRequest) -> Result<(), String> {
     if let Some(email) = req.email.as_ref() {
          if !email.is_empty() {
               validate_email(email)?;
          }
     }

     if let Some(password) = req.password.as_ref() {
          if !password.is_empty() {
               validate_password(password)?;
          }
     }

     if let Some(username) = req.username.as_ref() {
          if !username.is_empty() {
               validate_username(username)?;
          }
     }

     if req.email.as_ref().map_or(true, |s| s.is_empty())
          && req.password.as_ref().map_or(true, |s| s.is_empty())
          && req.username.as_ref().map_or(true, |s| s.is_empty())
          && req.role == Some(0)
     {
          return Err("At least one field must be non-empty and role must not be 0".to_string());
     }

     Ok(())
}
