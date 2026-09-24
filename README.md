# Tuercavo API

Backend REST para el catálogo de una ferretería: productos, categorías y proveedores. Autenticación OIDC con Microsoft Entra ID, sesiones en PostgreSQL y cookie opaca HttpOnly. Entra asigna quién puede entrar y con qué App Role; la API crea su registro local al primer login válido.

## Workspace

| Crate | Responsabilidad |
| --- | --- |
| [`api`](api/) | Ejecutable HTTP: arranque, composición de rutas, cabeceras y Scalar opcional. |
| [`auth`](auth/) | OIDC, alta local al primer login, cookies, sesiones, extracción del usuario autenticado y comprobación de permisos. |
| [`common`](common/) | Configuración, estado compartido, errores, validación, paginación y telemetría. |
| [`modules`](modules/) | Rutas, DTO y lógica de productos, categorías, proveedores, usuarios y salud. |
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
podman compose up -d
```

### 3. Aplicar las migraciones

Con PostgreSQL disponible y `DATABASE_URL` configurada:

```sh
sea-orm-cli migrate up -d db/migration
```

Las migraciones crean el esquema, los roles, los permisos y la tabla de sesiones. El archivo [`db/reference/ddl/tables.sql`](db/reference/ddl/tables.sql) sirve como referencia; la instalación se realiza mediante las migraciones.

Para poblar opcionalmente una base de desarrollo con categorías, proveedores y productos de ejemplo, aplica el archivo después de las migraciones.

Con `DATABASE_URL` exportada en la terminal:

```sh
psql "$DATABASE_URL" --set ON_ERROR_STOP=1 --file db/seeder/catalog.sql
```

Con los valores predeterminados de `compose.yml`:

```sh
podman compose exec -T postgres \
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

Los roles de las migraciones existen en PostgreSQL, pero **no crean los App Roles en Entra**. Configúralos en la App registration cuyo **Application (client) ID** pusiste en `ENTRA_CLIENT_ID`:

1. En el [centro de administración de Microsoft Entra](https://entra.microsoft.com/), abre **Entra ID → App registrations → tuercavo-api → App roles → Create app role**.
2. Crea un rol por cada fila de la tabla. Para cada uno, configura **Allowed member types = Users/Groups**, escribe el **Value** exactamente como aparece, añade una descripción, deja **Enable this app role** activado y pulsa **Apply**. **Display name** es el nombre visible en el portal; el backend comprueba el **Value** de la claim `roles`, no ese nombre.

   | Display name sugerido | Value obligatorio |
   | --- | --- |
   | Administrador | `admin` |
   | Capturista | `capturista` |
   | Consultor | `consultor` |

3. Abre **Entra ID → Enterprise applications → All applications → tuercavo-api → Properties**, cambia **Assignment required?** a **Yes** y pulsa **Save**. Es la aplicación empresarial asociada a esa misma App registration.

Microsoft documenta por separado [la creación de App Roles](https://learn.microsoft.com/en-us/entra/identity-platform/howto-add-app-roles-in-apps) y [la propiedad Assignment required](https://learn.microsoft.com/en-us/entra/identity/enterprise-apps/add-application-portal-configure). La API exige exactamente uno de los tres valores en el ID Token; `Default Access` no contiene un App Role válido para Tuercavo.

### 5. Dar de alta al primer administrador y a los demás usuarios

La persona debe tener una cuenta en el tenant. Para asignarle acceso:

1. En **Entra ID → Enterprise applications → All applications → tuercavo-api → Users and groups**, pulsa **Add user/group**.
2. En **Users and groups**, busca a la persona, selecciónala y pulsa **Select**.
3. En **Select a role**, elige `admin` para el primer administrador, o `capturista`/`consultor` para otro usuario, y pulsa **Select**. No elijas **Default Access**.
4. Pulsa **Assign** y comprueba que la persona y su rol aparecen en **Users and groups**. Asigna a cada persona un solo App Role de Tuercavo, ya que el login rechaza cero o varios roles.

Estos son los pasos de la [guía oficial para asignar usuarios a una aplicación empresarial](https://learn.microsoft.com/en-us/entra/identity/enterprise-apps/assign-user-or-group-access-portal). También puedes asignar grupos, pero Microsoft exige Entra ID P1 o P2 para esa modalidad y la pertenencia a grupos anidados no se propaga a la asignación.

Al primer login, la API valida la asignación recibida en el ID Token y crea el registro local en PostgreSQL. Hasta ese momento la persona no aparece en `GET /api/users`. El registro local refleja un acceso ya autorizado por Entra.

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

El login utiliza Authorization Code con PKCE, `state` y `nonce`. Tras validar la identidad y un App Role reconocido de Entra, la API crea el usuario local en su primer ingreso o sincroniza su rol al volver a entrar. Lo identifica por tenant y Object ID; nunca vincula cuentas por correo electrónico. Un usuario local desactivado continúa bloqueado aunque conserve la asignación en Entra.

La sesión se conserva en PostgreSQL y el navegador recibe una cookie `HttpOnly`, `SameSite=Lax`, con ruta `/`. Con HTTPS se utiliza `Secure` y el prefijo `__Host-`. HTTP solo se admite para desarrollo en una dirección local de loopback. Los tokens del proveedor no se guardan en la sesión.

Las peticiones autenticadas de escritura requieren un encabezado `Origin` que coincida exactamente con el origen de `OIDC_REDIRECT_URI` —en el ejemplo, `http://localhost:3000`—. Esto también se aplica al logout.

| Rol | Permisos |
| --- | --- |
| `admin` | Gestión completa del catálogo y de usuarios; consulta de roles y permisos. |
| `capturista` | Consulta, creación y actualización del catálogo. |
| `consultor` | Consulta del catálogo. |

Los permisos se consultan en cada petición autenticada. Un cambio de App Role en Entra se refleja localmente al siguiente login, no en una sesión ya abierta. Para dar de baja a una persona, un administrador llama primero a `POST /api/users/{public_id}/deactivate`: la API desactiva al usuario y revoca todas sus sesiones en una transacción. Desde que se confirma, no puede usar las sesiones anteriores ni iniciar otra, aunque aún tenga acceso en Entra. Después, el administrador de Entra retira su asignación a la aplicación empresarial (o lo quita de todos los grupos que le conceden acceso). Los cambios hechos solo en Entra no revocan automáticamente las sesiones locales.

Para devolverle el acceso, primero se restablece su asignación en Entra y luego un administrador llama a `POST /api/users/{public_id}/reactivate`. La reactivación retira el bloqueo local, pero no restaura las sesiones revocadas: la persona debe iniciar sesión de nuevo. Ambas operaciones requieren `users.update`. Si se desactiva al único administrador local, el administrador del tenant puede asignar el App Role `admin` a otra persona en Entra; al entrar por primera vez, esa persona obtiene el permiso para reactivarlo.

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
| `GET` | `/api/roles`, `/api/permissions` | Consultar roles y permisos disponibles. |
| `GET` | `/api/health/live` | Comprobar que el servidor responde. |
| `GET` | `/api/health/ready` | Comprobar la conexión a PostgreSQL. |

El catálogo y la administración requieren sesión y los permisos correspondientes. Los endpoints de salud son públicos. Scalar muestra los esquemas de entrada, las respuestas y los parámetros de cada operación.
