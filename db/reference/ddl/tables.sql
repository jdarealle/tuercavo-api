-- Tuercavo: documentación del esquema; NO es la vía operativa de instalación.
-- Aplicar db/migration con SeaORM. Este archivo describe una base ya provisionada.
-- PostgreSQL 18, UTF8, locale de la base C.UTF-8 (provider libc).
-- Unicidad de SKU/código/nombre de categoría mediante índices sobre lower(...).
-- lower usa el locale de la base; no elimina acentos. Consultar con lower(col) = lower(valor).
-- Fechas TIMESTAMPTZ; conexiones en UTC. La API mantiene updated_at, sin triggers.
-- Textos con trim, sin controles; opcionales vacíos se rechazan, ausencia = NULL.
-- Descripciones: hasta 4000 caracteres, una línea. Nombres: hasta 150.
-- SKU/código: 1..64 caracteres ASCII alfanuméricos, punto, guion y guion bajo.
-- Emails: hasta 254 caracteres; su sintaxis se valida en la API, no con regex SQL.

CREATE TYPE catalog_status AS ENUM ('active', 'inactive', 'archived');

-- Roles locales administrables; admin y consultor son roles protegidos del sistema.
CREATE TABLE roles (
    id SMALLINT GENERATED ALWAYS AS IDENTITY,
    code VARCHAR(32) NOT NULL,
    name VARCHAR(80) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    CONSTRAINT pk_roles PRIMARY KEY (id),
    CONSTRAINT uq_roles_code UNIQUE (code),
    CONSTRAINT ck_roles_code CHECK (code ~ '^[a-z][a-z0-9_]{0,31}$'),
    CONSTRAINT ck_roles_system_active CHECK (code NOT IN ('admin', 'consultor') OR is_active),
    CONSTRAINT ck_roles_name CHECK (char_length(name) BETWEEN 1 AND 80 AND name = btrim(name) AND name !~ '[[:cntrl:]]')
);

-- Permisos de referencia, sin comodines ni permisos particulares por usuario.
CREATE TABLE permissions (
    id SMALLINT GENERATED ALWAYS AS IDENTITY,
    code VARCHAR(80) NOT NULL,
    description VARCHAR(200) NOT NULL,
    CONSTRAINT pk_permissions PRIMARY KEY (id),
    CONSTRAINT uq_permissions_code UNIQUE (code),
    CONSTRAINT ck_permissions_code CHECK (code ~ '^[a-z]+\.[a-z]+(_[a-z]+)*$'),
    CONSTRAINT ck_permissions_description CHECK (char_length(description) BETWEEN 1 AND 200 AND description = btrim(description) AND description !~ '[[:cntrl:]]')
);

-- PK compuesta; las referencias estables no se borran en cascada.
CREATE TABLE role_permissions (
    role_id SMALLINT NOT NULL,
    permission_id SMALLINT NOT NULL,
    CONSTRAINT pk_role_permissions PRIMARY KEY (role_id, permission_id),
    CONSTRAINT fk_role_permissions_role FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE RESTRICT,
    CONSTRAINT fk_role_permissions_permission FOREIGN KEY (permission_id) REFERENCES permissions(id) ON DELETE RESTRICT
);

-- Departamentos de la empresa; UUID público y nombre único sin distinguir mayúsculas.
CREATE TABLE departments (
    id INTEGER GENERATED ALWAYS AS IDENTITY,
    public_id UUID NOT NULL DEFAULT gen_random_uuid(),
    name VARCHAR(150) NOT NULL,
    CONSTRAINT pk_departments PRIMARY KEY (id),
    CONSTRAINT uq_departments_public_id UNIQUE (public_id),
    CONSTRAINT ck_departments_name CHECK (char_length(name) BETWEEN 1 AND 150 AND name = btrim(name) AND name !~ '[[:cntrl:]]')
);
CREATE UNIQUE INDEX uq_departments_name ON departments (lower(name));

-- Identidad por tenant/object ID. Email y nombre son metadatos opcionales, no únicos.
-- Un único rol por usuario; los usuarios se desactivan, no se borran por API.
-- El departamento es opcional y se administra localmente, sin afectar los permisos.
CREATE TABLE users (
    id BIGINT GENERATED ALWAYS AS IDENTITY,
    public_id UUID NOT NULL DEFAULT gen_random_uuid(),
    role_id SMALLINT NOT NULL,
    department_id INTEGER,
    entra_tenant_id UUID NOT NULL,
    entra_object_id UUID NOT NULL,
    email VARCHAR(254),
    full_name VARCHAR(150),
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT pk_users PRIMARY KEY (id),
    CONSTRAINT uq_users_public_id UNIQUE (public_id),
    CONSTRAINT uq_users_entra_identity UNIQUE (entra_tenant_id, entra_object_id),
    CONSTRAINT fk_users_role FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE RESTRICT,
    CONSTRAINT fk_users_department FOREIGN KEY (department_id) REFERENCES departments(id) ON DELETE RESTRICT,
    CONSTRAINT ck_users_email CHECK (char_length(email) BETWEEN 1 AND 254 AND email = btrim(email) AND email !~ '[[:space:][:cntrl:]]'),
    CONSTRAINT ck_users_full_name CHECK (char_length(full_name) BETWEEN 1 AND 150 AND full_name = btrim(full_name) AND full_name !~ '[[:cntrl:]]')
);

-- Sesión local: la cookie contiene 32 bytes aleatorios codificados en Base64url.
-- Solo se persiste su hash SHA-256; nunca tokens OIDC ni la cookie en claro.
-- issuer/subject/tenant_id son la identidad verificada al iniciar esta sesión.
-- La aplicación comprueba tenant_id contra users.entra_tenant_id al autenticar.
CREATE TABLE sessions (
    session_id_hash BYTEA NOT NULL,
    user_id BIGINT NOT NULL,
    issuer VARCHAR(512) NOT NULL,
    subject VARCHAR(255) NOT NULL,
    tenant_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    CONSTRAINT pk_sessions PRIMARY KEY (session_id_hash),
    CONSTRAINT fk_sessions_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE RESTRICT,
    CONSTRAINT ck_sessions_hash_length CHECK (octet_length(session_id_hash) = 32),
    CONSTRAINT ck_sessions_issuer CHECK (issuer <> ''),
    CONSTRAINT ck_sessions_subject CHECK (subject <> ''),
    CONSTRAINT ck_sessions_expiration CHECK (expires_at > created_at),
    CONSTRAINT ck_sessions_activity CHECK (last_seen_at >= created_at),
    CONSTRAINT ck_sessions_revocation CHECK (revoked_at IS NULL OR revoked_at >= created_at)
);
CREATE INDEX idx_sessions_user_id ON sessions (user_id);
CREATE INDEX idx_sessions_expires_at ON sessions (expires_at);
CREATE INDEX idx_sessions_last_seen_at ON sessions (last_seen_at);

-- Categorías planas; nombre único sin distinguir mayúsculas. Sin jerarquías.
CREATE TABLE categories (
    id INTEGER GENERATED ALWAYS AS IDENTITY,
    public_id UUID NOT NULL DEFAULT gen_random_uuid(),
    name VARCHAR(150) NOT NULL,
    description TEXT,
    status catalog_status NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT pk_categories PRIMARY KEY (id),
    CONSTRAINT uq_categories_public_id UNIQUE (public_id),
    CONSTRAINT ck_categories_name CHECK (char_length(name) BETWEEN 1 AND 150 AND name = btrim(name) AND name !~ '[[:cntrl:]]'),
    CONSTRAINT ck_categories_description CHECK (char_length(description) BETWEEN 1 AND 4000 AND description = btrim(description) AND description !~ '[[:cntrl:]]')
);
CREATE UNIQUE INDEX uq_categories_name ON categories (lower(name));

-- Proveedor principal opcional de un producto; contactos opcionales anulables.
CREATE TABLE suppliers (
    id INTEGER GENERATED ALWAYS AS IDENTITY,
    public_id UUID NOT NULL DEFAULT gen_random_uuid(),
    code VARCHAR(64) NOT NULL,
    name VARCHAR(150) NOT NULL,
    contact_name VARCHAR(150),
    email VARCHAR(254),
    phone VARCHAR(32),
    status catalog_status NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT pk_suppliers PRIMARY KEY (id),
    CONSTRAINT uq_suppliers_public_id UNIQUE (public_id),
    CONSTRAINT ck_suppliers_code CHECK (char_length(code) BETWEEN 1 AND 64 AND code ~ '^[A-Za-z0-9][A-Za-z0-9._-]*$'),
    CONSTRAINT ck_suppliers_name CHECK (char_length(name) BETWEEN 1 AND 150 AND name = btrim(name) AND name !~ '[[:cntrl:]]'),
    CONSTRAINT ck_suppliers_contact_name CHECK (char_length(contact_name) BETWEEN 1 AND 150 AND contact_name = btrim(contact_name) AND contact_name !~ '[[:cntrl:]]'),
    CONSTRAINT ck_suppliers_email CHECK (char_length(email) BETWEEN 1 AND 254 AND email = btrim(email) AND email !~ '[[:space:][:cntrl:]]'),
    CONSTRAINT ck_suppliers_phone CHECK (char_length(phone) BETWEEN 1 AND 32 AND phone = btrim(phone) AND phone !~ '[[:cntrl:]]')
);
CREATE UNIQUE INDEX uq_suppliers_code ON suppliers (lower(code));

-- Precio de catálogo por unidad, MXN, sin impuestos ni conversiones.
-- La API debe rechazar >2 decimales ANTES de persistir (NUMERIC redondea).
-- También se excluye NaN explícitamente; NUMERIC compara NaN mayor que números.
-- Referencias RESTRICT: nunca eliminar productos ni limpiar supplier_id en cascada.
-- La actividad de referencias se valida al crear/cambiar en el servicio.
CREATE TABLE products (
    id BIGINT GENERATED ALWAYS AS IDENTITY,
    public_id UUID NOT NULL DEFAULT gen_random_uuid(),
    sku VARCHAR(64) NOT NULL,
    name VARCHAR(150) NOT NULL,
    description TEXT,
    brand VARCHAR(100),
    category_id INTEGER NOT NULL,
    supplier_id INTEGER,
    unit VARCHAR(16) NOT NULL,
    price NUMERIC(12,2) NOT NULL,
    status catalog_status NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT pk_products PRIMARY KEY (id),
    CONSTRAINT uq_products_public_id UNIQUE (public_id),
    CONSTRAINT fk_products_category FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE RESTRICT,
    CONSTRAINT fk_products_supplier FOREIGN KEY (supplier_id) REFERENCES suppliers(id) ON DELETE RESTRICT,
    CONSTRAINT ck_products_sku CHECK (char_length(sku) BETWEEN 1 AND 64 AND sku ~ '^[A-Za-z0-9][A-Za-z0-9._-]*$'),
    CONSTRAINT ck_products_name CHECK (char_length(name) BETWEEN 1 AND 150 AND name = btrim(name) AND name !~ '[[:cntrl:]]'),
    CONSTRAINT ck_products_description CHECK (char_length(description) BETWEEN 1 AND 4000 AND description = btrim(description) AND description !~ '[[:cntrl:]]'),
    CONSTRAINT ck_products_brand CHECK (char_length(brand) BETWEEN 1 AND 100 AND brand = btrim(brand) AND brand !~ '[[:cntrl:]]'),
    CONSTRAINT ck_products_unit CHECK (unit IN ('piece', 'box', 'pack', 'meter', 'liter', 'kg')),
    CONSTRAINT ck_products_price CHECK (price >= 0 AND price <> 'NaN'::numeric)
);
CREATE UNIQUE INDEX uq_products_sku ON products (lower(sku));

-- PK y UNIQUE ya generan índices. Estos cubren FKs sin repetirlos.
CREATE INDEX idx_role_permissions_permission_id ON role_permissions (permission_id);
CREATE INDEX idx_users_role_id ON users (role_id);
CREATE INDEX idx_users_department_id ON users (department_id);
CREATE INDEX idx_products_category_id ON products (category_id);
CREATE INDEX idx_products_supplier_id ON products (supplier_id);

-- DML de referencia. IDs resueltos por código; sin usuarios ni datos demo.
INSERT INTO roles (code, name) VALUES
    ('admin', 'Administrador'), ('capturista', 'Capturista'), ('consultor', 'Consultor');

INSERT INTO permissions (code, description) VALUES
    ('products.read', 'Consultar productos'),
    ('products.create', 'Crear productos'),
    ('products.update', 'Editar productos'),
    ('products.delete', 'Borrar productos'),
    ('categories.read', 'Consultar categorías'),
    ('categories.create', 'Crear categorías'),
    ('categories.update', 'Editar categorías'),
    ('categories.delete', 'Borrar categorías'),
    ('suppliers.read', 'Consultar proveedores'),
    ('suppliers.create', 'Crear proveedores'),
    ('suppliers.update', 'Editar proveedores'),
    ('suppliers.delete', 'Borrar proveedores'),
    ('users.read', 'Consultar usuarios'),
    ('users.update', 'Editar y desactivar usuarios'),
    ('departments.read', 'Consultar departamentos'),
    ('departments.create', 'Crear departamentos'),
    ('users.assign_department', 'Asignar departamentos a usuarios'),
    ('roles.read', 'Consultar roles'),
    ('permissions.read', 'Consultar permisos'),
    ('users.assign_role', 'Asignar roles a usuarios'),
    ('roles.create', 'Crear roles'),
    ('roles.update', 'Editar y retirar roles'),
    ('roles.assign_permissions', 'Asignar permisos a roles');

INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM (VALUES
    ('admin', 'products.read'), ('admin', 'products.create'),
    ('admin', 'products.update'), ('admin', 'products.delete'),
    ('admin', 'categories.read'), ('admin', 'categories.create'),
    ('admin', 'categories.update'), ('admin', 'categories.delete'),
    ('admin', 'suppliers.read'), ('admin', 'suppliers.create'),
    ('admin', 'suppliers.update'), ('admin', 'suppliers.delete'),
    ('admin', 'users.read'), ('admin', 'users.update'),
    ('admin', 'departments.read'), ('admin', 'departments.create'),
    ('admin', 'users.assign_department'),
    ('admin', 'roles.read'), ('admin', 'permissions.read'),
    ('admin', 'users.assign_role'), ('admin', 'roles.create'),
    ('admin', 'roles.update'), ('admin', 'roles.assign_permissions'),
    ('capturista', 'products.read'), ('capturista', 'products.create'),
    ('capturista', 'products.update'), ('capturista', 'categories.read'),
    ('capturista', 'categories.create'), ('capturista', 'categories.update'),
    ('capturista', 'suppliers.read'), ('capturista', 'suppliers.create'),
    ('capturista', 'suppliers.update'),
    ('consultor', 'products.read'), ('consultor', 'categories.read'),
    ('consultor', 'suppliers.read')
) AS grants(role_code, permission_code)
JOIN roles r ON r.code = grants.role_code
JOIN permissions p ON p.code = grants.permission_code;
