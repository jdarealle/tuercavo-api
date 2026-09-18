use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const ROLES: [(&str, &str); 3] = [
    ("admin", "Administrador"),
    ("capturista", "Capturista"),
    ("consultor", "Consultor"),
];

const PERMISSIONS: [(&str, &str); 18] = [
    ("products.read", "Consultar productos"),
    ("products.create", "Crear productos"),
    ("products.update", "Editar productos"),
    ("products.delete", "Borrar productos"),
    ("categories.read", "Consultar categorías"),
    ("categories.create", "Crear categorías"),
    ("categories.update", "Editar categorías"),
    ("categories.delete", "Borrar categorías"),
    ("suppliers.read", "Consultar proveedores"),
    ("suppliers.create", "Crear proveedores"),
    ("suppliers.update", "Editar proveedores"),
    ("suppliers.delete", "Borrar proveedores"),
    ("users.read", "Consultar usuarios"),
    ("users.create", "Crear usuarios"),
    ("users.update", "Editar y desactivar usuarios"),
    ("users.assign_role", "Asignar rol a usuarios"),
    ("roles.read", "Consultar roles"),
    ("permissions.read", "Consultar permisos"),
];

const CAPTURISTA: [&str; 9] = [
    "products.read",
    "products.create",
    "products.update",
    "categories.read",
    "categories.create",
    "categories.update",
    "suppliers.read",
    "suppliers.create",
    "suppliers.update",
];
const CONSULTOR: [&str; 3] = ["products.read", "categories.read", "suppliers.read"];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // El migrador envuelve cada migración PostgreSQL en una transacción.
        let mut roles = Query::insert();
        roles
            .into_table(Roles::Table)
            .columns([Roles::Code, Roles::Name]);
        for (code, name) in ROLES {
            roles.values_panic([code.into(), name.into()]);
        }
        manager.execute(roles).await?;
        let mut permissions = Query::insert();
        permissions
            .into_table(Permissions::Table)
            .columns([Permissions::Code, Permissions::Description]);
        for (code, description) in PERMISSIONS {
            permissions.values_panic([code.into(), description.into()]);
        }
        manager.execute(permissions).await?;

        for (role, _) in ROLES {
            let codes: Vec<&str> = match role {
                "admin" => PERMISSIONS.iter().map(|(code, _)| *code).collect(),
                "capturista" => CAPTURISTA.to_vec(),
                "consultor" => CONSULTOR.to_vec(),
                _ => unreachable!(),
            };
            let selected = Query::select()
                .column((Roles::Table, Roles::Id))
                .column((Permissions::Table, Permissions::Id))
                .from(Roles::Table)
                .from(Permissions::Table)
                .and_where(Expr::col((Roles::Table, Roles::Code)).eq(role))
                .and_where(Expr::col((Permissions::Table, Permissions::Code)).is_in(codes))
                .to_owned();
            manager
                .execute(
                    Query::insert()
                        .into_table(RolePermissions::Table)
                        .columns([RolePermissions::RoleId, RolePermissions::PermissionId])
                        .select_from(selected)
                        .map_err(|e| DbErr::Custom(e.to_string()))?
                        .to_owned(),
                )
                .await?;
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let role_ids = Query::select()
            .column(Roles::Id)
            .from(Roles::Table)
            .and_where(Expr::col(Roles::Code).is_in(ROLES.map(|(code, _)| code)))
            .to_owned();
        let permission_ids = Query::select()
            .column(Permissions::Id)
            .from(Permissions::Table)
            .and_where(Expr::col(Permissions::Code).is_in(PERMISSIONS.map(|(code, _)| code)))
            .to_owned();
        manager
            .execute(
                Query::delete()
                    .from_table(RolePermissions::Table)
                    .and_where(Expr::col(RolePermissions::RoleId).in_subquery(role_ids))
                    .and_where(Expr::col(RolePermissions::PermissionId).in_subquery(permission_ids))
                    .to_owned(),
            )
            .await?;
        manager
            .execute(
                Query::delete()
                    .from_table(Permissions::Table)
                    .and_where(
                        Expr::col(Permissions::Code).is_in(PERMISSIONS.map(|(code, _)| code)),
                    )
                    .to_owned(),
            )
            .await?;
        // RESTRICT impide borrar roles con usuarios. El rollback es atómico y no borra usuarios.
        manager
            .execute(
                Query::delete()
                    .from_table(Roles::Table)
                    .and_where(Expr::col(Roles::Code).is_in(ROLES.map(|(code, _)| code)))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Permissions {
    Table,
    Id,
    Code,
    Description,
}

#[derive(DeriveIden)]
enum RolePermissions {
    Table,
    RoleId,
    PermissionId,
}

#[derive(DeriveIden)]
enum Roles {
    Table,
    Id,
    Code,
    Name,
}
