# Tuercavo API

Backend REST para el catálogo de una ferretería: productos, categorías y proveedores. Autenticación OIDC con Microsoft Entra ID, sesiones en PostgreSQL y cookie opaca HttpOnly. Solo pueden iniciar sesión usuarios registrados previamente y activos; los permisos dependen de sus roles locales.

## Workspace

| Crate | Responsabilidad |
| --- | --- |
| [`api`](api/) | Ejecutable HTTP: arranque, composición de rutas, cabeceras y Scalar opcional. |
| [`auth`](auth/) | OIDC, cookies, sesiones, extracción del usuario autenticado y comprobación de permisos. Incluye el comando de alta del primer administrador. |
| [`common`](common/) | Configuración, estado compartido, errores, validación, paginación y telemetría. |
| [`modules`](modules/) | Rutas, DTO y lógica de productos, categorías, proveedores, usuarios y salud. |
| [`entity`](db/entity/) | Entidades de SeaORM generadas desde PostgreSQL. |
| [`migration`](db/migration/) | Migraciones versionadas del esquema y datos iniciales de roles y permisos. |

### 1. Configurar el entorno

Crea `.env` a partir de [`.env.sample`](.env.sample) si todavía no existe y sustituye los valores `REEMPLAZAR_…`. La API y `bootstrap-admin` cargan este archivo automáticamente; las variables ya exportadas en el entorno tienen prioridad.

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
podman compose up -d
```

### 3. Aplicar las migraciones

Con PostgreSQL disponible y `DATABASE_URL` configurada:

```sh
sea-orm-cli migrate up -d db/migration
```

Las migraciones crean el esquema, los roles, los permisos y la tabla de sesiones. El archivo [`db/reference/ddl/tables.sql`](db/reference/ddl/tables.sql) sirve como referencia; la instalación se realiza mediante las migraciones.

### 4. Configurar Microsoft Entra ID

Registra una aplicación para el tenant configurado, siguiendo la [documentación oficial de Microsoft](https://learn.microsoft.com/en-us/entra/identity-platform/quickstart-register-app). Añade el callback como plataforma **Web**, con la URL exacta de `OIDC_REDIRECT_URI`:

```text
http://localhost:3000/api/auth/callback
```

Configura `ENTRA_TENANT_ID`, `ENTRA_CLIENT_ID` y `ENTRA_CLIENT_SECRET`. El secreto corresponde a su **valor**, no al identificador del secreto. La API consulta el proveedor OIDC durante el arranque, por lo que necesita acceso a Microsoft Entra ID.

Si configuras `POST_LOGOUT_REDIRECT_URI`, registra esa URL exacta como otra Redirect URI **Web**. Por ejemplo, `http://localhost:3000/signed-out` debe mostrar una pantalla pública de la SPA que no inicie el login automáticamente.

### 5. Registrar el primer administrador

Además de `DATABASE_URL` y `ENTRA_TENANT_ID`, configura estas variables:

| Variable | Contenido |
| --- | --- |
| `ADMIN_OBJECT_ID` | Object ID del usuario dentro del tenant configurado; no es el client ID de la aplicación. |
| `ADMIN_EMAIL` | Correo del administrador. |
| `ADMIN_FULL_NAME` | Nombre completo del administrador. |

```sh
cargo run -p auth --bin bootstrap-admin
```

El alta es explícita: no se ejecuta al iniciar la API ni durante el login. Si esa identidad ya es un administrador activo, el comando devuelve su identificador. Si existe otro administrador activo en el tenant, las siguientes altas deben realizarse mediante la API de usuarios.

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

El login utiliza Authorization Code con PKCE, `state` y `nonce`. Tras validar la identidad de Entra, la API busca un usuario local activo por la combinación de tenant y Object ID. No crea usuarios automáticamente ni vincula cuentas por correo electrónico.

La sesión se conserva en PostgreSQL y el navegador recibe una cookie `HttpOnly`, `SameSite=Lax`, con ruta `/`. Con HTTPS se utiliza `Secure` y el prefijo `__Host-`. HTTP solo se admite para desarrollo en una dirección local de loopback. Los tokens del proveedor no se guardan en la sesión.

Las peticiones autenticadas de escritura requieren un encabezado `Origin` que coincida exactamente con el origen de `OIDC_REDIRECT_URI` —en el ejemplo, `http://localhost:3000`—. Esto también se aplica al logout.

| Rol | Permisos |
| --- | --- |
| `admin` | Gestión completa del catálogo y de usuarios; consulta de roles y permisos. |
| `capturista` | Consulta, creación y actualización del catálogo. |
| `consultor` | Consulta del catálogo. |

Los permisos se consultan en cada petición autenticada, por lo que un cambio de rol se aplica en la siguiente petición. Desactivar un usuario revoca sus sesiones. La administración impide desactivar o degradar al último administrador activo de un tenant.

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
| `GET`, `POST` | `/api/users` | Listar o registrar usuarios previamente autorizados. |
| `GET`, `PATCH` | `/api/users/{public_id}` | Consultar o actualizar un usuario. |
| `PUT` | `/api/users/{public_id}/role` | Asignar un rol. |
| `GET` | `/api/roles`, `/api/permissions` | Consultar roles y permisos disponibles. |
| `GET` | `/api/health/live` | Comprobar que el servidor responde. |
| `GET` | `/api/health/ready` | Comprobar la conexión a PostgreSQL. |

El catálogo y la administración requieren sesión y los permisos correspondientes. Los endpoints de salud son públicos. Scalar muestra los esquemas de entrada, las respuestas y los parámetros de cada operación.
