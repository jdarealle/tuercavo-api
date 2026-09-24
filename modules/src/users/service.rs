use super::dto::*;
use common::{
    error::AppError,
    pagination::{Page, Pagination},
};
use entity::{permissions, roles, users};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, DatabaseTransaction,
    EntityTrait, IntoActiveModel, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set,
    TransactionTrait,
};
use uuid::Uuid;

fn response(user: users::Model, role: String) -> UserResponse {
    UserResponse {
        public_id: user.public_id,
        entra_tenant_id: user.entra_tenant_id,
        entra_object_id: user.entra_object_id,
        email: user.email,
        full_name: user.full_name,
        role,
        is_active: user.is_active,
        created_at: user.created_at,
        updated_at: user.updated_at,
    }
}

async fn describe<C: ConnectionTrait>(
    db: &C,
    user: users::Model,
) -> Result<UserResponse, AppError> {
    let role = roles::Entity::find_by_id(user.role_id)
        .one(db)
        .await?
        .ok_or(AppError::Internal)?;
    Ok(response(user, role.code))
}

pub async fn list(
    db: &DatabaseConnection,
    tenant: Uuid,
    p: Pagination,
) -> Result<Page<UserResponse>, AppError> {
    p.validate()?;
    let query = users::Entity::find()
        .filter(users::Column::EntraTenantId.eq(tenant))
        .order_by_asc(users::Column::Id)
        .paginate(db, p.per_page);
    let total = query.num_items().await?;
    let roles: std::collections::HashMap<_, _> = roles::Entity::find()
        .all(db)
        .await?
        .into_iter()
        .map(|role| (role.id, role.code))
        .collect();
    let rows = query
        .fetch_page(p.page - 1)
        .await?
        .into_iter()
        .map(|user| {
            let role = roles.get(&user.role_id).ok_or(AppError::Internal)?.clone();
            Ok(response(user, role))
        })
        .collect::<Result<Vec<_>, AppError>>()?;
    Ok(Page::new(rows, total, &p))
}

pub async fn get(
    db: &DatabaseConnection,
    tenant: Uuid,
    public_id: Uuid,
) -> Result<UserResponse, AppError> {
    let user = users::Entity::find()
        .filter(users::Column::PublicId.eq(public_id))
        .filter(users::Column::EntraTenantId.eq(tenant))
        .one(db)
        .await?
        .ok_or(AppError::NotFound)?;
    describe(db, user).await
}

async fn actor<C: ConnectionTrait>(
    db: &C,
    actor_id: i64,
    tenant: Uuid,
    permission: &str,
) -> Result<auth::Principal, AppError> {
    let actor = auth::identity::load_user(db, actor_id).await?;
    if actor.tenant_id != tenant {
        return Err(auth::AuthError::Forbidden.into());
    }
    actor.require(permission)?;
    Ok(actor)
}

async fn target(
    tx: &DatabaseTransaction,
    tenant: Uuid,
    public_id: Uuid,
) -> Result<users::Model, AppError> {
    users::Entity::find()
        .filter(users::Column::PublicId.eq(public_id))
        .filter(users::Column::EntraTenantId.eq(tenant))
        .lock_exclusive()
        .one(tx)
        .await?
        .ok_or(AppError::NotFound)
}

async fn set_active(
    db: &DatabaseConnection,
    actor_id: i64,
    tenant: Uuid,
    public_id: Uuid,
    active: bool,
) -> Result<UserResponse, AppError> {
    let tx = db.begin().await?;
    actor(&tx, actor_id, tenant, "users.update").await?;
    let user = target(&tx, tenant, public_id).await?;
    if !active {
        // Revoking on repeated calls also covers any session created by a
        // concurrent login before this transaction acquired the user lock.
        auth::session::revoke_user(&tx, user.id).await?;
    }
    let user = if user.is_active == active {
        user
    } else {
        let mut model = user.into_active_model();
        model.is_active = Set(active);
        model.updated_at = Set(chrono::Utc::now().fixed_offset());
        model.update(&tx).await?
    };
    let result = describe(&tx, user).await?;
    tx.commit().await?;
    Ok(result)
}

pub async fn deactivate(
    db: &DatabaseConnection,
    actor_id: i64,
    tenant: Uuid,
    public_id: Uuid,
) -> Result<UserResponse, AppError> {
    set_active(db, actor_id, tenant, public_id, false).await
}

pub async fn reactivate(
    db: &DatabaseConnection,
    actor_id: i64,
    tenant: Uuid,
    public_id: Uuid,
) -> Result<UserResponse, AppError> {
    set_active(db, actor_id, tenant, public_id, true).await
}

pub async fn roles(db: &DatabaseConnection) -> Result<Vec<RoleResponse>, AppError> {
    Ok(roles::Entity::find()
        .order_by_asc(roles::Column::Code)
        .all(db)
        .await?
        .into_iter()
        .map(|role| RoleResponse {
            code: role.code,
            name: role.name,
        })
        .collect())
}

pub async fn permissions(db: &DatabaseConnection) -> Result<Vec<PermissionResponse>, AppError> {
    Ok(permissions::Entity::find()
        .order_by_asc(permissions::Column::Code)
        .all(db)
        .await?
        .into_iter()
        .map(|permission| PermissionResponse {
            code: permission.code,
            description: permission.description,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use entity::{permissions, role_permissions};
    use sea_orm::{DbBackend, DbErr, IntoMockRow, MockDatabase, MockExecResult};

    fn admin(active: bool) -> users::Model {
        let now = chrono::Utc::now().fixed_offset();
        users::Model {
            id: 1,
            public_id: Uuid::from_u128(1),
            role_id: 1,
            entra_tenant_id: Uuid::from_u128(2),
            entra_object_id: Uuid::from_u128(3),
            email: None,
            full_name: Some("Administrador".into()),
            is_active: active,
            created_at: now,
            updated_at: now,
        }
    }

    fn role() -> roles::Model {
        roles::Model {
            id: 1,
            code: "admin".into(),
            name: "Administrador".into(),
        }
    }

    fn db_for_change(initially_active: bool, result_active: bool) -> MockDatabase {
        MockDatabase::new(DbBackend::Postgres).append_query_results([
            vec![admin(true).into_mock_row()],
            vec![role().into_mock_row()],
            vec![
                role_permissions::Model {
                    role_id: 1,
                    permission_id: 1,
                }
                .into_mock_row(),
            ],
            vec![
                permissions::Model {
                    id: 1,
                    code: "users.update".into(),
                    description: "Administrar usuarios".into(),
                }
                .into_mock_row(),
            ],
            vec![admin(initially_active).into_mock_row()],
            vec![admin(result_active).into_mock_row()],
            vec![role().into_mock_row()],
        ])
    }

    #[tokio::test]
    async fn deactivation_revokes_sessions_and_commits_even_for_only_admin() {
        let db = db_for_change(true, false)
            .append_exec_results([MockExecResult {
                rows_affected: 2,
                last_insert_id: 0,
            }])
            .into_connection();

        let result = deactivate(&db, 1, Uuid::from_u128(2), Uuid::from_u128(1))
            .await
            .unwrap();
        assert!(!result.is_active);

        let log = db.into_transaction_log();
        assert_eq!(log.len(), 1);
        let sql: Vec<_> = log[0]
            .statements()
            .iter()
            .map(|statement| statement.sql.as_str())
            .collect();
        let revoke = sql
            .iter()
            .position(|sql| sql.starts_with("UPDATE \"sessions\""))
            .unwrap();
        let disable = sql
            .iter()
            .position(|sql| sql.starts_with("UPDATE \"users\""))
            .unwrap();
        assert!(revoke < disable);
        assert!(sql[revoke].contains("\"revoked_at\" IS NULL"));
        assert_eq!(sql.last(), Some(&"COMMIT"));
    }

    #[tokio::test]
    async fn failed_session_revocation_cannot_commit_deactivation() {
        let db = db_for_change(true, false)
            .append_exec_errors([DbErr::Custom("revocation failed".into())])
            .into_connection();

        assert!(
            deactivate(&db, 1, Uuid::from_u128(2), Uuid::from_u128(1))
                .await
                .is_err()
        );

        let log = db.into_transaction_log();
        assert_eq!(log.len(), 1);
        let sql: Vec<_> = log[0]
            .statements()
            .iter()
            .map(|statement| statement.sql.as_str())
            .collect();
        assert!(sql.iter().any(|sql| sql.starts_with("UPDATE \"sessions\"")));
        assert!(!sql.iter().any(|sql| sql.starts_with("UPDATE \"users\"")));
        assert_ne!(sql.last(), Some(&"COMMIT"));
    }

    #[tokio::test]
    async fn reactivation_does_not_restore_sessions() {
        let db = db_for_change(false, true).into_connection();

        let result = reactivate(&db, 1, Uuid::from_u128(2), Uuid::from_u128(1))
            .await
            .unwrap();
        assert!(result.is_active);

        let log = db.into_transaction_log();
        let sql: Vec<_> = log[0]
            .statements()
            .iter()
            .map(|statement| statement.sql.as_str())
            .collect();
        assert!(sql.iter().any(|sql| sql.starts_with("UPDATE \"users\"")));
        assert!(!sql.iter().any(|sql| sql.starts_with("UPDATE \"sessions\"")));
        assert_eq!(sql.last(), Some(&"COMMIT"));
    }
}
