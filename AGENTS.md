# Repository Guidelines

- Consult `README.md` for setup and commands, and `auth/README.md` for authentication and authorization flows. Keep those details in their respective documentation.
- This project is not in production. By default, change an existing database object in the migration that defines it so a fresh database gets the final schema. Create a new migration when explicitly requested or when it better fits the change, including new independent objects. Choose the necessary migration operations case by case. State when an edited, already applied migration requires rebuilding the development database.
- Before editing migrations, consult the current official SeaORM/SeaQuery documentation for the workspace versions. Prefer SeaQuery builders when they express the change clearly; use raw SQL when needed.
- With every migration change, review and update `db/reference/ddl/tables.sql` and `db/seeder/catalog.sql` so both match a fresh migration run. Regenerate `db/entity/` using the CLI flags in `README.md` to preserve enum Serde derives and attributes.
- Entra ID authenticates users; Tuercavo's database controls roles and permissions. Preserve this boundary when changing access rules.
- Use Podman for containers and pnpm for JavaScript tooling. Use `.env.example` for configuration examples; do not read or commit real secrets.
- Before destructive file or database operations, identify the exact target.
