use chrono::Duration;
use entity::{roles, sessions, users};
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::Set,
    ColumnTrait, ConnectionTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QuerySelect,
    TransactionTrait,
    entity::prelude::DateTimeWithTimeZone,
    sea_query::{Alias, Expr, ExprTrait, Func, OnConflict, Query},
};
use uuid::Uuid;

use crate::{AuthError, Principal, identity::load_user, oidc::VerifiedIdentity};

pub struct SessionStore {
    db: DatabaseConnection,
    tenant_id: Uuid,
    issuer: String,
    absolute_ttl: i64,
    idle_ttl: i64,
}

impl SessionStore {
    pub async fn new(
        db: DatabaseConnection,
        tenant_id: Uuid,
        issuer: String,
        absolute_ttl: i64,
        idle_ttl: i64,
    ) -> Result<Self, AuthError> {
        sessions::Entity::find().limit(0).all(&db).await?;
        users::Entity::find().limit(0).all(&db).await?;
        Ok(Self {
            db,
            tenant_id,
            issuer,
            absolute_ttl,
            idle_ttl,
        })
    }

    pub(crate) async fn replace(
        &self,
        previous: Option<&[u8]>,
        hash: &[u8],
        identity: &VerifiedIdentity,
    ) -> Result<(), AuthError> {
        if identity.tenant_id != self.tenant_id || identity.issuer != self.issuer {
            return Err(AuthError::Forbidden);
        }
        let tx = self.db.begin().await?;
        let role = roles::Entity::find()
            .filter(roles::Column::Code.eq(&identity.role))
            .one(&tx)
            .await?
            .ok_or(AuthError::Forbidden)?;
        let now = database_now(&tx).await?;
        // The unique tenant/object key makes concurrent first logins idempotent.
        users::Entity::insert(users::ActiveModel {
            role_id: Set(role.id),
            entra_tenant_id: Set(identity.tenant_id),
            entra_object_id: Set(identity.object_id),
            // Entra's email claim is not an addressable contact field.
            email: Set(None),
            full_name: Set(identity.full_name.clone()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        })
        .on_conflict(
            OnConflict::columns([users::Column::EntraTenantId, users::Column::EntraObjectId])
                .do_nothing()
                .to_owned(),
        )
        .try_insert()
        .exec(&tx)
        .await?;
        let user = users::Entity::find()
            .filter(users::Column::EntraTenantId.eq(identity.tenant_id))
            .filter(users::Column::EntraObjectId.eq(identity.object_id))
            .lock_exclusive()
            .one(&tx)
            .await?
            .ok_or(AuthError::Forbidden)?;
        if !user.is_active {
            return Err(AuthError::Forbidden);
        }
        if user.role_id != role.id || user.full_name != identity.full_name {
            let mut update: users::ActiveModel = user.clone().into();
            update.role_id = Set(role.id);
            update.full_name = Set(identity.full_name.clone());
            update.updated_at = Set(now);
            update.update(&tx).await?;
        }
        if let Some(previous) = previous {
            revoke(&tx, previous).await?;
        }
        sessions::ActiveModel {
            session_id_hash: Set(hash.to_vec()),
            user_id: Set(user.id),
            issuer: Set(identity.issuer.clone()),
            subject: Set(identity.subject.clone()),
            tenant_id: Set(identity.tenant_id),
            created_at: Set(now),
            last_seen_at: Set(now),
            expires_at: Set(now + Duration::seconds(self.absolute_ttl)),
            ..Default::default()
        }
        .insert(&tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn authenticate(&self, hash: &[u8]) -> Result<Principal, AuthError> {
        let now = Expr::cust("statement_timestamp()");
        let cutoff = now
            .clone()
            .sub(Expr::val(self.idle_ttl).mul(Expr::cust("INTERVAL '1 second'")));
        // Conditional UPDATE avoids resurrecting expired/revoked sessions. GREATEST
        // prevents concurrent requests from moving last_seen_at backwards.
        let rows = sessions::Entity::update_many()
            .col_expr(
                sessions::Column::LastSeenAt,
                Expr::expr(Func::greatest([
                    Expr::col(sessions::Column::LastSeenAt),
                    now.clone(),
                ])),
            )
            .filter(sessions::Column::SessionIdHash.eq(hash.to_vec()))
            .filter(sessions::Column::TenantId.eq(self.tenant_id))
            .filter(sessions::Column::Issuer.eq(&self.issuer))
            .filter(sessions::Column::RevokedAt.is_null())
            .filter(Expr::col(sessions::Column::ExpiresAt).gt(now))
            .filter(Expr::col(sessions::Column::LastSeenAt).gt(cutoff))
            .exec_with_returning(&self.db)
            .await?;
        let session = rows.into_iter().next().ok_or(AuthError::Unauthorized)?;
        let principal = load_user(&self.db, session.user_id).await?;
        if principal.tenant_id != session.tenant_id {
            return Err(AuthError::Forbidden);
        }
        Ok(principal)
    }

    pub async fn revoke(&self, hash: &[u8]) -> Result<(), AuthError> {
        revoke(&self.db, hash).await?;
        Ok(())
    }

    pub async fn cleanup(&self) -> Result<(), AuthError> {
        let now = Expr::cust("statement_timestamp()");
        let cutoff = now
            .clone()
            .sub(Expr::val(self.idle_ttl).mul(Expr::cust("INTERVAL '1 second'")));
        sessions::Entity::delete_many()
            .filter(sessions::Column::TenantId.eq(self.tenant_id))
            .filter(sessions::Column::Issuer.eq(&self.issuer))
            .filter(
                sea_orm::Condition::any()
                    .add(Expr::col(sessions::Column::ExpiresAt).lte(now))
                    .add(Expr::col(sessions::Column::LastSeenAt).lte(cutoff))
                    .add(sessions::Column::RevokedAt.is_not_null()),
            )
            .exec(&self.db)
            .await?;
        Ok(())
    }
}

pub async fn revoke_user<C: ConnectionTrait>(db: &C, user_id: i64) -> Result<(), DbErr> {
    sessions::Entity::update_many()
        .col_expr(
            sessions::Column::RevokedAt,
            Expr::cust("statement_timestamp()"),
        )
        .filter(sessions::Column::UserId.eq(user_id))
        .filter(sessions::Column::RevokedAt.is_null())
        .exec(db)
        .await?;
    Ok(())
}

async fn revoke<C: ConnectionTrait>(db: &C, hash: &[u8]) -> Result<(), DbErr> {
    sessions::Entity::update_many()
        .col_expr(
            sessions::Column::RevokedAt,
            Expr::cust("statement_timestamp()"),
        )
        .filter(sessions::Column::SessionIdHash.eq(hash.to_vec()))
        .filter(sessions::Column::RevokedAt.is_null())
        .exec(db)
        .await?;
    Ok(())
}

async fn database_now<C: ConnectionTrait>(db: &C) -> Result<DateTimeWithTimeZone, DbErr> {
    let query = Query::select()
        .expr_as(Expr::cust("statement_timestamp()"), Alias::new("db_now"))
        .to_owned();
    db.query_one(&query)
        .await?
        .ok_or_else(|| DbErr::Custom("Database clock unavailable".into()))?
        .try_get("", "db_now")
}
