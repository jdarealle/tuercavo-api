use crate::error::AppError;
use rust_decimal::Decimal;
use std::str::FromStr;

pub fn text(value: String, field: &str, max: usize) -> Result<String, AppError> {
    let v = value.trim();
    if v.is_empty() || v.chars().count() > max || v.chars().any(char::is_control) {
        return Err(AppError::bad(format!(
            "{field} debe tener entre 1 y {max} caracteres sin controles"
        )));
    }
    Ok(v.to_owned())
}
pub fn optional(
    value: Option<String>,
    field: &str,
    max: usize,
) -> Result<Option<String>, AppError> {
    value.map(|v| text(v, field, max)).transpose()
}
pub fn code(value: String, field: &str) -> Result<String, AppError> {
    let v = text(value, field, 64)?;
    if !v.as_bytes()[0].is_ascii_alphanumeric()
        || !v
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
    {
        return Err(AppError::bad(format!("{field} tiene un formato inválido")));
    }
    Ok(v)
}
pub fn email(value: String) -> Result<String, AppError> {
    let v = text(value, "email", 254)?;
    if v.chars().any(char::is_whitespace) || !email_address::EmailAddress::is_valid(&v) {
        return Err(AppError::bad("email inválido"));
    }
    Ok(v)
}
pub fn phone(value: String) -> Result<String, AppError> {
    let v = text(value, "phone", 32)?;
    if !v.chars().any(|c| c.is_ascii_digit())
        || !v
            .chars()
            .all(|c| c.is_ascii_digit() || "+-(). xX".contains(c))
    {
        return Err(AppError::bad("phone inválido"));
    }
    Ok(v)
}
pub fn price(value: &str) -> Result<Decimal, AppError> {
    let mut parts = value.split('.');
    let whole = parts.next().unwrap_or_default();
    let frac = parts.next();
    if whole.is_empty()
        || whole.len() > 10
        || !whole.bytes().all(|b| b.is_ascii_digit())
        || frac
            .is_some_and(|s| s.is_empty() || s.len() > 2 || !s.bytes().all(|b| b.is_ascii_digit()))
        || parts.next().is_some()
    {
        return Err(AppError::bad(
            "price debe ser una cadena decimal entre 0 y 9999999999.99, con hasta dos decimales",
        ));
    }
    Decimal::from_str(value).map_err(|_| AppError::bad("price inválido"))
}
pub fn search(value: &str) -> Result<String, AppError> {
    let value = text(value.to_owned(), "search", 150)?;
    Ok(format!(
        "%{}%",
        value
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    ))
}
