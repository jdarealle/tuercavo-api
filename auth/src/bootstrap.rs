use entity::{roles, users};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, Set,
    TransactionTrait,
};
use uuid::Uuid;

/// Explicit, one-time enrollment. Never called during API startup or OIDC login.
pub async fn first_admin(
    db: &DatabaseConnection,
    tenant: Uuid,
    object: Uuid,
    email: String,
    full_name: String,
) -> Result<Uuid, String> {
    if tenant.is_nil() || object.is_nil() {
        return Err("La identidad Entra no puede ser nil".into());
    }
    let email = email.trim().to_owned();
    let full_name = full_name.trim().to_owned();
    if email.len() > 254
        || email.chars().any(|c| c.is_whitespace() || c.is_control())
        || !email_address::EmailAddress::is_valid(&email)
    {
        return Err("ADMIN_EMAIL inválido".into());
    }
    if full_name.is_empty()
        || full_name.chars().count() > 150
        || full_name.chars().any(char::is_control)
    {
        return Err("ADMIN_FULL_NAME debe tener entre 1 y 150 caracteres sin controles".into());
    }
    let db_error = |_| "No se pudo registrar al administrador en PostgreSQL".to_owned();
    let tx = db.begin().await.map_err(db_error)?;
    // The same lock serializes administrative changes that protect the last admin.
    let admin = roles::Entity::find()
        .filter(roles::Column::Code.eq("admin"))
        .lock_exclusive()
        .one(&tx)
        .await
        .map_err(db_error)?
        .ok_or("Falta el rol admin; aplica las migraciones")?;
    let existing = users::Entity::find()
        .filter(users::Column::EntraTenantId.eq(tenant))
        .filter(users::Column::EntraObjectId.eq(object))
        .one(&tx)
        .await
        .map_err(db_error)?;
    if let Some(user) = existing {
        if user.is_active && user.role_id == admin.id {
            tx.commit().await.map_err(db_error)?;
            return Ok(user.public_id);
        }
        return Err("La identidad ya existe con otro rol o está desactivada".into());
    }
    if users::Entity::find()
        .filter(users::Column::RoleId.eq(admin.id))
        .filter(users::Column::EntraTenantId.eq(tenant))
        .filter(users::Column::IsActive.eq(true))
        .one(&tx)
        .await
        .map_err(db_error)?
        .is_some()
    {
        return Err("Ya existe un administrador activo; registra usuarios desde la API".into());
    }
    let user = users::ActiveModel {
        role_id: Set(admin.id),
        entra_tenant_id: Set(tenant),
        entra_object_id: Set(object),
        email: Set(email),
        full_name: Set(full_name),
        ..Default::default()
    }
    .insert(&tx)
    .await
    .map_err(db_error)?;
    tx.commit().await.map_err(db_error)?;
    Ok(user.public_id)
}
