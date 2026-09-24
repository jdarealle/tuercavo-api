use super::dto::*;
use auth::authorization;
use common::{error::AppError, validation};
use entity::{permissions, role_permissions, roles, sessions, users};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, PaginatorTrait, QueryFilter, QueryOrder, Set,
    sea_query::{Expr, ExprTrait, Query},
};
use std::collections::{BTreeSet, HashMap};
use uuid::Uuid;

pub(super) fn code(value: String) -> Result<String, AppError> {
    if value.is_empty()
        || value.len() > 32
        || !value.as_bytes()[0].is_ascii_lowercase()
        || !value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
    {
        return Err(AppError::bad(
            "code debe empezar por a-z y contener hasta 32 caracteres a-z, 0-9 o _",
        ));
    }
    Ok(value)
}

async fn find<C: ConnectionTrait>(db: &C, code: &str) -> Result<roles::Model, AppError> {
    roles::Entity::find()
        .filter(roles::Column::Code.eq(code))
        .one(db)
        .await?
        .ok_or(AppError::NotFound)
}

async fn grants<C: ConnectionTrait>(db: &C, id: i16) -> Result<Vec<String>, AppError> {
    let query = Query::select()
        .column(role_permissions::Column::PermissionId)
        .from(role_permissions::Entity)
        .and_where(Expr::col(role_permissions::Column::RoleId).eq(id))
        .to_owned();
    Ok(permissions::Entity::find()
        .filter(permissions::Column::Id.in_subquery(query))
        .order_by_asc(permissions::Column::Code)
        .all(db)
        .await?
        .into_iter()
        .map(|p| p.code)
        .collect())
}

fn response(role: roles::Model, permissions: Vec<String>) -> RoleResponse {
    RoleResponse {
        is_system: authorization::is_system_role(&role.code),
        code: role.code,
        name: role.name,
        is_active: role.is_active,
        permissions,
    }
}

pub async fn get(db: &DatabaseConnection, code: &str) -> Result<RoleResponse, AppError> {
    let role = find(db, code).await?;
    let permissions = grants(db, role.id).await?;
    Ok(response(role, permissions))
}

pub async fn list(db: &DatabaseConnection) -> Result<Vec<RoleResponse>, AppError> {
    let roles = roles::Entity::find()
        .order_by_asc(roles::Column::Code)
        .all(db)
        .await?;
    let catalog: HashMap<_, _> = permissions::Entity::find()
        .all(db)
        .await?
        .into_iter()
        .map(|p| (p.id, p.code))
        .collect();
    let mut assignments: HashMap<i16, Vec<String>> = HashMap::new();
    for grant in role_permissions::Entity::find().all(db).await? {
        let code = catalog
            .get(&grant.permission_id)
            .ok_or(AppError::Internal)?;
        assignments
            .entry(grant.role_id)
            .or_default()
            .push(code.clone());
    }
    Ok(roles
        .into_iter()
        .map(|role| {
            let mut grants = assignments.remove(&role.id).unwrap_or_default();
            grants.sort();
            response(role, grants)
        })
        .collect())
}

pub async fn catalog(db: &DatabaseConnection) -> Result<Vec<PermissionResponse>, AppError> {
    Ok(permissions::Entity::find()
        .order_by_asc(permissions::Column::Code)
        .all(db)
        .await?
        .into_iter()
        .map(|p| PermissionResponse {
            code: p.code,
            description: p.description,
        })
        .collect())
}

pub async fn create(
    db: &DatabaseConnection,
    actor: i64,
    tenant: Uuid,
    body: CreateRole,
) -> Result<RoleResponse, AppError> {
    let code = code(body.code)?;
    let name = validation::text(body.name, "name", 80)?;
    let (tx, _) = crate::authorization::begin(db, actor, tenant, "roles.create").await?;
    let role = roles::ActiveModel {
        code: Set(code),
        name: Set(name),
        is_active: Set(true),
        ..Default::default()
    }
    .insert(&tx)
    .await?;
    tx.commit().await?;
    Ok(response(role, vec![]))
}

pub async fn update(
    db: &DatabaseConnection,
    actor: i64,
    tenant: Uuid,
    code: &str,
    body: UpdateRole,
) -> Result<RoleResponse, AppError> {
    let name = body
        .name
        .required("name")?
        .map(|v| validation::text(v, "name", 80))
        .transpose()?;
    let active = body.is_active.required("is_active")?;
    if name.is_none() && active.is_none() {
        return Err(AppError::bad("Indica name o is_active"));
    }
    let (tx, _) = crate::authorization::begin(db, actor, tenant, "roles.update").await?;
    let role = find(&tx, code).await?;
    if active == Some(false) {
        if authorization::is_system_role(code) {
            return Err(AppError::conflict("No se puede retirar un rol del sistema"));
        }
        if users::Entity::find()
            .filter(users::Column::RoleId.eq(role.id))
            .count(&tx)
            .await?
            > 0
        {
            return Err(AppError::conflict(
                "Reasigna todos los usuarios antes de retirar el rol",
            ));
        }
    }
    let mut update = role.into_active_model();
    if let Some(name) = name {
        update.name = Set(name);
    }
    if let Some(active) = active {
        update.is_active = Set(active);
    }
    let role = update.update(&tx).await?;
    let permissions = grants(&tx, role.id).await?;
    tx.commit().await?;
    Ok(response(role, permissions))
}

pub async fn set_permissions(
    db: &DatabaseConnection,
    actor: i64,
    tenant: Uuid,
    code: &str,
    body: SetPermissions,
) -> Result<RoleResponse, AppError> {
    let distinct: BTreeSet<_> = body.permissions.iter().cloned().collect();
    if distinct.len() != body.permissions.len() {
        return Err(AppError::bad("No repitas códigos de permiso"));
    }
    let requested: Vec<_> = distinct.into_iter().collect();
    authorization::validate_grants(code, &requested).map_err(AppError::conflict)?;
    let (tx, _) =
        crate::authorization::begin(db, actor, tenant, "roles.assign_permissions").await?;
    let role = find(&tx, code).await?;
    let available = permissions::Entity::find()
        .filter(permissions::Column::Code.is_in(requested.clone()))
        .all(&tx)
        .await?;
    if available.len() != requested.len() {
        return Err(AppError::bad("Uno o más permisos no existen"));
    }
    let before = grants(&tx, role.id).await?;
    if before != requested {
        role_permissions::Entity::delete_many()
            .filter(role_permissions::Column::RoleId.eq(role.id))
            .exec(&tx)
            .await?;
        if !available.is_empty() {
            role_permissions::Entity::insert_many(available.into_iter().map(|p| {
                role_permissions::ActiveModel {
                    role_id: Set(role.id),
                    permission_id: Set(p.id),
                }
            }))
            .exec(&tx)
            .await?;
        }
        if before.iter().any(|p| !requested.contains(p)) {
            let users = Query::select()
                .column(users::Column::Id)
                .from(users::Entity)
                .and_where(Expr::col(users::Column::RoleId).eq(role.id))
                .to_owned();
            sessions::Entity::update_many()
                .col_expr(
                    sessions::Column::RevokedAt,
                    Expr::cust("statement_timestamp()"),
                )
                .filter(sessions::Column::UserId.in_subquery(users))
                .filter(sessions::Column::RevokedAt.is_null())
                .exec(&tx)
                .await?;
        }
    }
    tx.commit().await?;
    Ok(response(role, requested))
}
