//! Permission codes implemented by this version of the API.
//!
//! The migration keeps its own snapshot of the catalog. A test compares that
//! snapshot with `ALL` so runtime changes cannot silently drift from the DB.

macro_rules! codes {
    ($($name:ident = $code:literal;)+) => {
        $(pub const $name: &str = $code;)+
        pub const ALL: &[&str] = &[$($name),+];
    };
}

codes! {
    PRODUCTS_READ = "products.read";
    PRODUCTS_CREATE = "products.create";
    PRODUCTS_UPDATE = "products.update";
    PRODUCTS_DELETE = "products.delete";
    CATEGORIES_READ = "categories.read";
    CATEGORIES_CREATE = "categories.create";
    CATEGORIES_UPDATE = "categories.update";
    CATEGORIES_DELETE = "categories.delete";
    SUPPLIERS_READ = "suppliers.read";
    SUPPLIERS_CREATE = "suppliers.create";
    SUPPLIERS_UPDATE = "suppliers.update";
    SUPPLIERS_DELETE = "suppliers.delete";
    USERS_READ = "users.read";
    USERS_UPDATE = "users.update";
    DEPARTMENTS_READ = "departments.read";
    DEPARTMENTS_CREATE = "departments.create";
    USERS_ASSIGN_DEPARTMENT = "users.assign_department";
    ROLES_READ = "roles.read";
    PERMISSIONS_READ = "permissions.read";
    USERS_ASSIGN_ROLE = "users.assign_role";
    ROLES_CREATE = "roles.create";
    ROLES_UPDATE = "roles.update";
    ROLES_ASSIGN_PERMISSIONS = "roles.assign_permissions";
}

pub const ADMIN_REQUIRED: &[&str] = &[
    USERS_READ,
    USERS_UPDATE,
    USERS_ASSIGN_ROLE,
    DEPARTMENTS_READ,
    DEPARTMENTS_CREATE,
    USERS_ASSIGN_DEPARTMENT,
    ROLES_READ,
    ROLES_CREATE,
    ROLES_UPDATE,
    ROLES_ASSIGN_PERMISSIONS,
    PERMISSIONS_READ,
];

pub const DEFAULT_ALLOWED: &[&str] = &[PRODUCTS_READ, CATEGORIES_READ, SUPPLIERS_READ];

pub const ADMIN_ONLY: &[&str] = &[
    DEPARTMENTS_READ,
    DEPARTMENTS_CREATE,
    USERS_ASSIGN_DEPARTMENT,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grant_policies_reference_catalog_codes() {
        for code in ADMIN_REQUIRED
            .iter()
            .chain(DEFAULT_ALLOWED)
            .chain(ADMIN_ONLY)
        {
            assert!(ALL.contains(code), "unknown policy permission {code}");
        }
        for code in ADMIN_ONLY {
            assert!(ADMIN_REQUIRED.contains(code), "admin must retain {code}");
        }
    }
}
