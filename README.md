# Tuercavo API

Backend REST para el catálogo de una ferretería: productos, categorías y proveedores. Autenticación OIDC con Microsoft Entra ID, sesiones en PostgreSQL y cookie opaca HttpOnly. Entra autentica a las personas asignadas a la aplicación; Tuercavo administra sus roles y permisos y crea el registro local como consultor en el primer login válido.

## Workspace

| Crate | Responsabilidad |
| --- | --- |
| [`api`](api/) | Ejecutable HTTP: arranque, composición de rutas, cabeceras y Scalar opcional. |
| [`auth`](auth/) | OIDC, alta local al primer login, cookies, sesiones, extracción del usuario autenticado y comprobación de permisos. |
| [`bootstrap-admin`](bootstrap-admin/) | Comando operativo para asignar el primer administrador tras su primer login. |
| [`common`](common/) | Configuración, estado compartido, errores, validación, paginación y telemetría. |
| [`modules`](modules/) | Rutas, DTO y lógica de productos, categorías, proveedores, usuarios, roles y salud. |
| [`entity`](db/entity/) | Entidades de SeaORM generadas desde PostgreSQL. |
| [`migration`](db/migration/) | Migraciones versionadas del esquema y datos iniciales de roles y permisos. |

### 1. Configurar el entorno

Crea `.env` a partir de [`.env.sample`](.env.sample) si todavía no existe y sustituye los valores `REEMPLAZAR_…`. La API carga este archivo automáticamente; las variables ya exportadas en el entorno tienen prioridad.

| Variable | Uso / valor predeterminado |
| --- | --- |
| `DATABASE_URL` | Obligatoria. Conexión a PostgreSQL; la muestra apunta a `127.0.0.1:55433/tuercavo_dev`. |
| `HOST` | Dirección de escucha: `127.0.0.1`. |
| `PORT` | Puerto HTTP: `3000`. |
| `RUST_LOG` | Nivel de registro; la muestra usa `info`. |
| `ENTRA_TENANT_ID` | UUID del tenant admitido. |
| `ENTRA_CLIENT_ID` | UUID de la aplicación registrada en Entra. |
| `ENTRA_CLIENT_SECRET` | Valor del secreto de la aplicación. |
| `OIDC_REDIRECT_URI` | Callback obligatorio; en local: `http://localhost:3000/api/auth/callback`. |
| `POST_LOGIN_REDIRECT_PATH` | Ruta de regreso tras un login exitoso; predeterminado `/api/auth/me`. Cuando exista la SPA, configura una ruta suya, por ejemplo `/app`. |
| `POST_LOGOUT_REDIRECT_URI` | Opcional. URL de la pantalla pública de salida de la SPA, del mismo origen que `OIDC_REDIRECT_URI`. Debe estar registrada como Redirect URI **Web** en Entra. Sin ella, Entra muestra su propia pantalla de salida. |
| `SESSION_TTL_SECS` | Duración absoluta de la sesión: `28800` segundos (8 horas). |
| `SESSION_IDLE_TTL_SECS` | Tiempo máximo sin actividad: `1800` segundos (30 minutos). |

### 2. Iniciar PostgreSQL

```sh
podman-compose up -d
```

### 3. Aplicar las migraciones

Con PostgreSQL disponible y `DATABASE_URL` configurada:

```sh
sea-orm-cli migrate up -d db/migration
```

Las migraciones crean el esquema, los roles, los permisos y las sesiones. El archivo [`db/reference/ddl/tables.sql`](db/reference/ddl/tables.sql) sirve como referencia; la instalación se realiza mediante las migraciones.

Para poblar opcionalmente una base de desarrollo con categorías, proveedores y productos de ejemplo, aplica el archivo después de las migraciones.

Con `DATABASE_URL` exportada en la terminal:

```sh
psql "$DATABASE_URL" --set ON_ERROR_STOP=1 --file db/seeder/catalog.sql
```

Con los valores predeterminados de `compose.yml`:

```sh
podman-compose exec -T postgres \
  psql --username tuercavo --dbname tuercavo_dev --set ON_ERROR_STOP=1 \
  < db/seeder/catalog.sql
```

### 4. Configurar Microsoft Entra ID

Registra una aplicación para el tenant configurado, siguiendo la [documentación oficial de Microsoft](https://learn.microsoft.com/en-us/entra/identity-platform/quickstart-register-app). Añade el callback como plataforma **Web**, con la URL exacta de `OIDC_REDIRECT_URI`:

```text
http://localhost:3000/api/auth/callback
```

Configura `ENTRA_TENANT_ID`, `ENTRA_CLIENT_ID` y `ENTRA_CLIENT_SECRET`. El secreto corresponde a su **valor**, no al identificador del secreto. La API consulta el proveedor OIDC durante el arranque, por lo que necesita acceso a Microsoft Entra ID.

Si configuras `POST_LOGOUT_REDIRECT_URI`, registra esa URL exacta como otra Redirect URI **Web**. Por ejemplo, `http://localhost:3000/signed-out` debe mostrar una pantalla pública de la SPA que no inicie el login automáticamente.

En **Entra ID → Enterprise applications → All applications → tuercavo-api → Properties**, configura **Assignment required? = Yes** y guarda. Para asignar acceso:

1. Abre **Users and groups → Add user/group**.
2. Selecciona a la persona que ya existe en ese tenant.
3. Utiliza **Default Access** y pulsa **Assign**. La App registration se configura sin App Roles de negocio; los roles se administran en Tuercavo.

Consulta la [asignación de usuarios de Microsoft](https://learn.microsoft.com/en-us/entra/identity/enterprise-apps/assign-user-or-group-access-portal). La asignación mediante grupos requiere Entra ID P1 o P2.

### 5. Alta de usuarios y primer administrador

Cada persona asignada entra mediante `/api/auth/login`. En su primer login la API crea el usuario con el rol local `consultor`. Hasta ese momento no aparece en `GET /api/users`. Los siguientes logins conservan el rol que le haya asignado Tuercavo; un usuario desactivado permanece bloqueado.

Para el primer administrador:

1. Asigna acceso a la persona en Entra y pídele completar su primer login.
2. Copia su **Object ID** desde **Entra ID → Users → usuario → Overview**. Debe ser el objeto del usuario dentro del tenant de Tuercavo, no el Object ID de la aplicación.
3. Ejecuta el bootstrap con acceso a la misma base de datos y el mismo `ENTRA_TENANT_ID` de la API:

```sh
BOOTSTRAP_ADMIN_OBJECT_ID=REEMPLAZAR_CON_UUID_DEL_USUARIO \
  cargo run -p bootstrap-admin -- --env-file .env
```

`--env-file` es opcional y carga la configuración indicada sin sobrescribir variables ya presentes en el entorno. El comando no busca `.env` automáticamente. `DATABASE_URL` y `ENTRA_TENANT_ID` se reutilizan; `BOOTSTRAP_ADMIN_OBJECT_ID` se proporciona solo a esta ejecución. El comando verifica que el usuario esté registrado y activo, que no exista un administrador, lo promueve y revoca sus sesiones en una transacción; después debe iniciar sesión nuevamente.

En producción, construye `bootstrap-admin` como ejecutable independiente y ejecútalo como un job temporal con esas tres variables. Incluye únicamente el ejecutable `api` en su imagen habitual y distribuye el bootstrap como artefacto operativo aparte. La configuración de la API no utiliza `BOOTSTRAP_ADMIN_OBJECT_ID`.

El [README de bootstrap-admin](bootstrap-admin/README.md) describe su funcionamiento; el [README de auth](auth/README.md) describe las transacciones y la protección de roles.

### 6. Ejecutar la API y abrir Scalar

```sh
cargo run -p api --features scalar
```

1. Abre [Iniciar sesión](http://localhost:3000/api/auth/login) y autentícate con el usuario registrado.
2. El callback crea la sesión y redirige a `POST_LOGIN_REDIRECT_PATH` (`/api/auth/me` por defecto).
3. Abre [Scalar](http://localhost:3000/scalar) en el mismo navegador y host.

Scalar utiliza la cookie de sesión que envía el navegador. No hay que copiar tokens. Usa siempre `localhost` en este ejemplo: alternarlo con `127.0.0.1` cambia el host de la cookie y el origen de las peticiones.

La integración utiliza el crate oficial `scalar_api_reference`. Las rutas `/scalar` y `/api/openapi.json` solo se incluyen al compilar con la feature `scalar`; el proyecto impide habilitarla en una compilación `release`.

Para ejecutar la API sin Scalar:

```sh
cargo run -p api
```

## Autenticación y permisos

El login utiliza Authorization Code con PKCE, `state` y `nonce`. Tras validar la identidad de Entra, la API crea al usuario como consultor en su primer ingreso y conserva su rol local al volver a entrar. Lo identifica por tenant y Object ID; nunca vincula cuentas por correo electrónico. Un usuario local desactivado continúa bloqueado aunque conserve la asignación en Entra.

La sesión se conserva en PostgreSQL y el navegador recibe una cookie `HttpOnly`, `SameSite=Lax`, con ruta `/`. Con HTTPS se utiliza `Secure` y el prefijo `__Host-`. HTTP solo se admite para desarrollo en una dirección local de loopback. Los tokens del proveedor no se guardan en la sesión.

Las peticiones autenticadas de escritura requieren un encabezado `Origin` que coincida exactamente con el origen de `OIDC_REDIRECT_URI` —en el ejemplo, `http://localhost:3000`—. Esto también se aplica al logout.

| Rol | Permisos |
| --- | --- |
| `admin` | Carga inicial con todos los permisos; conserva obligatoriamente los de administración. |
| `capturista` | Consulta, creación y actualización del catálogo. |
| `consultor` | Consulta del catálogo. |

Los permisos se consultan en cada petición autenticada. Los roles personalizados pueden crearse y cambiarse desde la API; cada usuario tiene un solo rol. Los códigos de permiso identifican capacidades implementadas por el backend. Las migraciones proporcionan la matriz inicial; su administración posterior se realiza en Tuercavo.

`PUT /api/users/{public_id}/role` recibe `{"role":"capturista"}` y revoca las sesiones si cambia el rol. `PUT /api/roles/{code}/permissions` recibe la lista completa `{"permissions":["products.read"]}`; una reducción revoca las sesiones de todos los usuarios del rol. `admin` conserva sus permisos de administración; `consultor` admite únicamente permisos de lectura del catálogo. Ambos roles permanecen activos. Los roles personalizados nacen sin permisos y pueden retirarse mediante `PATCH` con `{"is_active":false}` después de reasignar a todos sus usuarios, incluidos los inactivos.

Para dar de baja a una persona, llama primero a `POST /api/users/{public_id}/deactivate`. La API desactiva al usuario y revoca sus sesiones en una transacción; después se retira su asignación en Entra. Para devolverle el acceso, restablece la asignación en Entra y llama a `POST /api/users/{public_id}/reactivate`; deberá iniciar sesión de nuevo. Ambas operaciones requieren `users.update`. La API impide desactivar o cambiar el rol del último administrador activo.

Los cambios de autorización y la revocación de sesiones se confirman en una misma transacción. Las peticiones que se autentiquen después del cambio usarán el estado actualizado; una petición ya autorizada puede estar ejecutándose.

`POST /api/auth/logout` revoca la sesión local y elimina la cookie. Para cerrar también la sesión de Microsoft, la SPA debe esperar el `204` y después navegar con `window.location.assign('/api/auth/entra-logout')`. Esta ruta redirige el navegador al `end_session_endpoint` descubierto en Entra. Tras la salida, Entra redirige a `POST_LOGOUT_REDIRECT_URI` si está configurada; en caso contrario muestra su propia pantalla. Una llamada `fetch` a la ruta de Entra no sustituye la navegación del navegador.

`GET /api/auth/login?prompt=select_account` muestra el selector de cuentas y `GET /api/auth/login?prompt=login` solicita nueva autenticación. El login sin `prompt` conserva el inicio de sesión único. Redirigir directamente al login después del logout local puede crear otra sesión sin interacción. La actividad renueva el límite de inactividad, pero nunca extiende el vencimiento absoluto.

Los intentos de login pendientes permanecen en memoria durante cinco minutos. Reiniciar la API los invalida; varias instancias necesitan afinidad durante ese flujo. Las sesiones ya creadas se conservan en PostgreSQL y siguen sujetas a vencimiento, inactividad y revocación.

## Rutas principales

| Método | Ruta | Uso |
| --- | --- | --- |
| `GET` | `/api/auth/login` | Iniciar el flujo OIDC; admite `prompt=select_account` o `prompt=login`. |
| `GET` | `/api/auth/callback` | Recibir la respuesta del proveedor. |
| `GET` | `/api/auth/me` | Consultar el usuario y sus permisos. |
| `POST` | `/api/auth/logout` | Revocar la sesión local. |
| `GET` | `/api/auth/entra-logout` | Redirigir el navegador al cierre de sesión de Entra después del logout local. |
| `GET`, `POST` | `/api/products`, `/api/categories`, `/api/suppliers` | Listar o crear registros. |
| `GET`, `PATCH`, `DELETE` | `/api/products/{public_id}`, `/api/categories/{public_id}`, `/api/suppliers/{public_id}` | Consultar, actualizar o eliminar un registro. |
| `GET` | `/api/users` | Listar usuarios que ya iniciaron sesión por primera vez. |
| `GET` | `/api/users/{public_id}` | Consultar un usuario. |
| `POST` | `/api/users/{public_id}/deactivate` | Desactivar al usuario y revocar todas sus sesiones locales. |
| `POST` | `/api/users/{public_id}/reactivate` | Reactivar el acceso local; requiere un nuevo login. |
| `PUT` | `/api/users/{public_id}/role` | Asignar un rol local activo. |
| `GET`, `POST` | `/api/roles` | Listar o crear roles. |
| `GET`, `PATCH` | `/api/roles/{code}` | Consultar, renombrar o retirar un rol. |
| `PUT` | `/api/roles/{code}/permissions` | Reemplazar sus permisos. |
| `GET` | `/api/permissions` | Consultar el catálogo de permisos. |
| `GET` | `/api/health/live` | Comprobar que el servidor responde. |
| `GET` | `/api/health/ready` | Comprobar la conexión a PostgreSQL. |

El catálogo y la administración requieren sesión y los permisos correspondientes. Los endpoints de salud son públicos. Scalar muestra los esquemas de entrada, las respuestas y los parámetros de cada operación.
