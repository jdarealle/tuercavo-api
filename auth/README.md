# Arquitectura del crate `auth`

`auth` implementa el inicio de sesión OIDC con Microsoft Entra ID y la autenticación posterior mediante una sesión local en PostgreSQL. La API recibe un código de autorización en el callback, valida la identidad y entrega al navegador una cookie con un identificador opaco. Cada operación protegida vuelve a comprobar la sesión, la actividad del usuario y sus permisos actuales. Los tokens emitidos por Entra se descartan al terminar el callback; la SPA no los recibe.

Para ejecutar el proyecto y configurar PostgreSQL, consulta el [README principal](../README.md).

## Límites e integración

La biblioteca `auth` se ejecuta dentro del proceso `api`; no publica un servicio independiente. Depende de `db/entity`, SeaORM, Axum y `openidconnect`, pero no depende de `api`, `common` ni `modules`.

```text
api::main
  ├─ crea DatabaseConnection y AuthConfig
  ├─ auth::build_state(config, db.clone()) → AuthState
  ├─ common::AppState { db, auth }
  └─ api::app monta auth::router() bajo /api
                         │
                         ├─ routes → oidc, state, session, cookie, token
                         └─ extractors → session → identity → roles y permisos
```

`common/src/state.rs` implementa `FromRef<AppState> for AuthState`. Por ello los handlers de `auth` y los extractores de `modules` pueden obtener el estado de autenticación del estado compartido de Axum. `api/src/main.rs` crea el pool y hace el descubrimiento OIDC al arrancar; también ejecuta cada 300 segundos `AuthState::cleanup()` para limpiar sesiones caducadas, inactivas o revocadas, así como flujos de login vencidos. `api/src/app.rs` monta las rutas bajo `/api`. `modules` administra usuarios, roles y permisos locales; conserva el control `is_active` y ejecuta las mutaciones mediante el protocolo transaccional de `authorization`.

## Archivos y responsabilidades

| Archivo | Responsabilidad |
| --- | --- |
| [`src/lib.rs`](src/lib.rs) | Declara los módulos y la interfaz pública: configuración, estado, router, extractores, identidad, errores, tokens y sesiones. |
| [`src/config.rs`](src/config.rs) | Lee y valida la configuración OIDC, los destinos de redirección y las duraciones de sesión. Construye el issuer y el origen público esperados. |
| [`src/state.rs`](src/state.rs) | Construye `AuthState` con el proveedor, el almacén de sesiones, la política de cookies y los flujos de login pendientes. Gestiona su caducidad y limpieza. |
| [`src/oidc.rs`](src/oidc.rs) | Descubre los metadatos de Entra, genera solicitudes de autorización, canjea códigos, valida ID Tokens. Entrega identidad, nombre y correo opcional mediante `VerifiedIdentity`. |
| [`src/routes.rs`](src/routes.rs) | Define los handlers HTTP de login, callback, sesión actual y los dos tipos de logout; coordina OIDC, cookies y sesiones. También registra sus operaciones en OpenAPI. |
| [`src/cookie.rs`](src/cookie.rs) | Define nombres, atributos, duración y eliminación de las cookies de sesión y correlación del login. |
| [`src/token.rs`](src/token.rs) | Genera identificadores opacos de 32 bytes, los codifica en Base64url y calcula el hash SHA-256 que se almacena en PostgreSQL. Valida el formato de una cookie recibida. |
| [`src/session.rs`](src/session.rs) | Da de alta al usuario en su primer login, lo registra como consultor, conserva su rol en logins posteriores y crea, sustituye, autentica, revoca y limpia sesiones persistentes; expone `revoke_user` para la desactivación administrativa. |
| [`src/identity.rs`](src/identity.rs) | Carga al usuario activo y sus permisos actuales desde las entidades; define `Principal`, el resultado de `/api/auth/me` y de los extractores. |
| [`src/extractor.rs`](src/extractor.rs) | Implementa `AuthUser`, `Require<P>` y `SameOrigin` como extractores Axum para autenticación, autorización y control de origen. |
| [`src/authorization.rs`](src/authorization.rs) | Define los roles protegidos, valida sus permisos y coordina bloqueos transaccionales. |
| [`src/error.rs`](src/error.rs) | Convierte errores de autenticación en respuestas HTTP JSON sin exponer detalles de la base de datos ni del proveedor. |

## Arranque y configuración

La API carga `.env` con `dotenvy`, crea una conexión SeaORM y llama a `AuthConfig::from_env()` y `build_state()`.

| Variable | Efecto en `auth` |
| --- | --- |
| `ENTRA_TENANT_ID` | UUID del tenant admitido; se usa para construir el issuer y validar el claim `tid`. |
| `ENTRA_CLIENT_ID` | UUID de la aplicación confidencial registrada en Entra. |
| `ENTRA_CLIENT_SECRET` | Secreto utilizado por el backend para canjear el código; no llega al navegador. |
| `OIDC_REDIRECT_URI` | Callback público exacto, con ruta `/api/auth/callback`; debe registrarse en Entra como plataforma Web. Su origen es el que `SameOrigin` exige en las escrituras. |
| `POST_LOGIN_REDIRECT_PATH` | Ruta local a la que se envía el navegador tras crear la sesión; por defecto `/api/auth/me`. Solo admite una ruta del mismo origen, como `/app`. |
| `POST_LOGOUT_REDIRECT_URI` | URL pública opcional que se envía a Entra al abrir `/api/auth/entra-logout`. Debe tener el mismo origen que `OIDC_REDIRECT_URI`, sin query ni fragmento. |
| `SESSION_TTL_SECS` | Vencimiento absoluto de la sesión; por defecto 28 800 segundos (8 horas). |
| `SESSION_IDLE_TTL_SECS` | Límite de inactividad; por defecto 1 800 segundos (30 minutos). |

Ambos TTL admiten valores de 1 a 604 800 segundos. El callback exige HTTPS salvo para `localhost` o una IP de loopback mediante HTTP. `build_state()` comprueba que existan las tablas `sessions` y `users`, descubre los metadatos OIDC de Entra y requiere su endpoint de cierre de sesión. La configuración de `DATABASE_URL`, `HOST` y `PORT` pertenece al ejecutable `api`, no a `AuthConfig`.

La URL de `OIDC_REDIRECT_URI` es la dirección **visible para el navegador**. Si la SPA y `/api` comparten origen mediante un proxy, el callback usa ese origen público aunque el proceso `api` escuche internamente en otro puerto. Los repositorios y procesos pueden permanecer separados. `POST_LOGIN_REDIRECT_PATH` solo cambia la navegación posterior al login; no modifica el callback registrado en Entra.

## Alta inicial de un usuario

1. **Responsable de Entra:** registra la aplicación como cliente OIDC confidencial del tenant, con callback Web y credencial de cliente. Configura **Enterprise applications → tuercavo-api → Properties → Assignment required = Yes**.
2. **Responsable de Entra:** abre **Users and groups → Add user/group**, selecciona a la persona y asigna **Default Access**. La aplicación se configura sin App Roles de negocio.
3. **Persona usuaria:** navega a `GET /api/auth/login` y se autentica en Entra. El backend canjea el código y valida el ID Token.
4. **`auth`:** busca la identidad por `(entra_tenant_id, entra_object_id)`. En una transacción crea el usuario como `consultor` y sin departamento si no existe, exige que esté activo, actualiza el nombre de presentación y crea la sesión. Los logins posteriores conservan el rol y el departamento locales. La restricción única de identidad evita duplicados ante logins concurrentes.
5. **SPA:** recibe la cookie opaca y consulta `GET /api/auth/me`. El usuario aparece en la lista local después de su primer login; `created_at` registra ese momento.
6. **Administrador de Tuercavo:** asigna otro rol cuando corresponda mediante `PUT /api/users/{public_id}/role`.

La API solicita el scope OIDC `email` y guarda el claim del ID Token, si está presente, para mostrarlo en `/api/auth/me`; lo actualiza en cada login. Entra no garantiza que el claim exista ni que sea una dirección de contacto válida, por lo que `email` puede ser `null` y nunca se usa para vincular identidades o autorizar. Microsoft documenta la [asignación a usuarios](https://learn.microsoft.com/en-us/entra/identity/enterprise-apps/assign-user-or-group-access-portal) y los [claims del ID Token](https://learn.microsoft.com/en-us/entra/identity-platform/id-token-claims-reference).

## Administración de roles y permisos

`roles`, `permissions` y `role_permissions` almacenan el RBAC local. Un usuario tiene un solo rol; los endpoints verifican códigos de permiso mediante `Require<P>`. Las migraciones cargan `admin`, `capturista`, `consultor` y la matriz inicial. Los roles personalizados se crean activos y sin permisos. Sus códigos son estables y únicos, de hasta 32 caracteres, con formato `[a-z][a-z0-9_]*`.

| Operación | Permiso |
| --- | --- |
| Listar/consultar usuarios | `users.read` |
| Listar/consultar departamentos | `users.read` |
| Desactivar/reactivar usuarios | `users.update` |
| Crear departamentos y asignarlos o quitarlos de usuarios | `users.update` |
| Asignar un rol | `users.assign_role` |
| Listar/consultar roles | `roles.read` |
| Crear roles | `roles.create` |
| Renombrar, retirar o reactivar roles | `roles.update` |
| Reemplazar permisos de un rol | `roles.assign_permissions` |
| Consultar el catálogo de permisos | `permissions.read` |

`admin` conserva todos los permisos administrativos de esta tabla, aunque sus permisos de catálogo pueden cambiar. `consultor`, el rol predeterminado, solo admite `products.read`, `categories.read` y `suppliers.read`; puede tener un subconjunto, incluso vacío. Ambos roles son del sistema y permanecen activos. Los roles personalizados pueden recibir permisos del catálogo; delegar `users.assign_role` o `roles.assign_permissions` permite conceder privilegios elevados y debe tratarse como capacidad administrativa.

`PUT /api/roles/{code}/permissions` reemplaza la lista completa y rechaza códigos desconocidos o repetidos. Los códigos del catálogo representan operaciones implementadas en Rust; su incorporación se versiona con el backend. Las migraciones futuras deben respetar las asignaciones personalizadas.

`departments` es una clasificación local opcional. `/api/auth/me` expone `department_public_id`; Entra no lo proporciona y el callback no sobrescribe la asignación. Cambiarlo no altera los permisos ni revoca sesiones. Las rutas y ejemplos de administración de departamentos están en el [README principal](../README.md).

`PATCH /api/roles/{code}` modifica `name` o `is_active`; retirar un rol exige que ningún usuario lo tenga asignado, incluidos los inactivos. La API conserva el rol y permite reactivarlo. `PUT /api/users/{public_id}/role` exige un rol activo y revoca las sesiones cuando hay un cambio.

## Transacciones y concurrencia

Las transacciones de login, administración y bootstrap usan explícitamente `READ COMMITTED`, para que las comprobaciones posteriores a esperar un bloqueo lean el estado confirmado más reciente. Todas las mutaciones administrativas toman primero un bloqueo exclusivo sobre la fila del rol de sistema `admin`, antes de consultar de nuevo los permisos del actor y bloquear usuarios. Este orden serializa los cambios entre réplicas y evita que dos administradores eliminen simultáneamente al último administrador activo. Desactivar o cambiar de rol a un administrador exige que permanezca otro usuario activo con rol `admin` dentro del tenant.

El callback adquiere un bloqueo compartido sobre esa misma fila antes de insertar/bloquear el usuario y crear su sesión. Los logins pueden ejecutarse en paralelo entre sí, pero no pueden crear sesiones mientras una mutación administrativa revoca las sesiones afectadas. Los bloqueos se liberan al confirmar o revertir la transacción. Las peticiones ordinarias leen los permisos locales vigentes; no adquieren el bloqueo administrativo.

Los cambios de rol revocan todas las sesiones del usuario; quitar permisos a un rol revoca las de todos sus usuarios. Agregar permisos se refleja en la siguiente petición sin exigir un nuevo login. Una petición ya autorizada puede seguir ejecutándose.

## Flujo de inicio de sesión

1. El navegador navega a `GET /api/auth/login`. Opcionalmente puede pedir `?prompt=login` o `?prompt=select_account`; otro valor recibe `400`.
2. `routes::login` genera PKCE S256, `state` y `nonce` mediante `openidconnect`. Crea un identificador aleatorio para la cookie temporal de correlación y guarda en `AuthState` el hash de ese identificador junto con `state`, `nonce`, el `code_verifier` y el instante de inicio. Otro login del mismo navegador invalida el flujo pendiente anterior. La respuesta redirige a Entra.
3. Entra redirige el navegador mediante `GET` a `OIDC_REDIRECT_URI` con `code` y `state`. La cookie temporal permite encontrar el flujo pendiente. El callback lo consume una sola vez, verifica `state` y rechaza códigos o parámetros inválidos.
4. `OidcProvider::exchange` canjea el código desde el servidor con el `code_verifier` y el secreto de cliente. La biblioteca valida el ID Token, incluida la firma, issuer, audiencia, expiración y `nonce`. La implementación comprueba además `tid`, `oid`, `nbf` si existe, `at_hash` si aparece. Solo devuelve los datos verificados necesarios; los tokens del proveedor no se persisten.
5. `SessionStore::replace` inserta el usuario si no existe y exige que esté activo. Usa `(entra_tenant_id, entra_object_id)`; nunca vincula cuentas por email. En la misma transacción conserva el rol local, actualiza el nombre de presentación, revoca la sesión anterior presentada por ese navegador, si la hay, e inserta la nueva sesión usando el reloj de PostgreSQL.
6. El callback elimina la cookie temporal, envía la cookie de sesión opaca y redirige a `POST_LOGIN_REDIRECT_PATH`. La SPA consulta después `GET /api/auth/me` para obtener `Principal` y sus permisos vigentes.

Los flujos pendientes duran cinco minutos y se conservan únicamente en la memoria de ese proceso, con un máximo de 1024. Reiniciar la API los invalida; varias réplicas necesitan que login y callback lleguen a la misma instancia. Las sesiones completadas residen en PostgreSQL y pueden consultarse desde cualquier réplica conectada a esa base.

## Cookies y almacenamiento de sesión

`SessionToken` genera 32 bytes aleatorios y los codifica como Base64url sin relleno (43 caracteres). La cookie lleva ese valor; `sessions.session_id_hash` guarda solo su hash SHA-256 de 32 bytes. Una cookie mal formada se descarta antes de consultar la base de datos.

Con callback HTTPS, las cookies se llaman `__Host-session` y `__Host-oidc-flow` y llevan `Secure`, `HttpOnly`, `SameSite=Lax`, `Path=/` y ningún atributo `Domain`. En HTTP de desarrollo local se llaman `session` y `oidc-flow`, sin `Secure`; las demás restricciones se mantienen. La cookie del flujo dura cinco minutos. La cookie de sesión usa el TTL absoluto, pero su validez real siempre la decide PostgreSQL.

La tabla `sessions` contiene `session_id_hash`, `user_id`, issuer, subject, tenant, fecha de creación, última actividad, vencimiento absoluto y revocación opcional. La crea la [migración de sesiones](../db/migration/src/m20260921_000001_sessions.rs); las entidades se generan en [`db/entity`](../db/entity/). La tabla `users` aporta la identidad local, actividad y rol; `roles`, `permissions` y `role_permissions` definen los permisos.

`SessionStore::authenticate` usa un `UPDATE` condicional: solo acepta una sesión no revocada, del tenant e issuer configurados, cuyo vencimiento absoluto y límite de inactividad sigan vigentes. Si la acepta, actualiza `last_seen_at` sin hacerlo retroceder cuando coinciden varias peticiones. La actividad no extiende el vencimiento absoluto. La limpieza periódica elimina sesiones revocadas o vencidas, pero la comprobación de cada petición ya impide utilizarlas antes de esa limpieza.

## Autenticación y autorización por petición

Los handlers de catálogo y administración declaran `Require<P>`, donde cada tipo `P` implementa `Permission` y fija un código como `products.read`. `AuthUser` se usa cuando basta con conocer la identidad, por ejemplo en `/api/auth/me`.

1. `AuthUser` extrae la cookie, valida su formato y comprueba el encabezado `Origin` en métodos distintos de `GET`, `HEAD` y `OPTIONS`. `SameOrigin` aplica la misma comprobación al logout local. El origen debe coincidir exactamente con el de `OIDC_REDIRECT_URI`; las peticiones de escritura sin `Origin` se rechazan.
2. `SessionStore::authenticate` verifica la sesión y carga el usuario local. `identity::load_user` exige que siga activo y consulta en PostgreSQL su rol y los códigos de permiso actuales. `Principal` incluye identificador público, email, nombre, tenant, Object ID, rol y permisos. El `user_id` interno no se serializa ni aparece en OpenAPI.
3. `Require<P>` comprueba que el código de permiso requerido esté en `Principal.permissions`. Sin sesión válida devuelve `401`; falta de permiso o usuario desactivado devuelve `403`. Los errores de base de datos se traducen en indisponibilidad sin exponer SQL ni credenciales.

Como los permisos se leen en cada petición, un cambio del `role_id` local surte efecto en la siguiente solicitud autorizada. `auth` no decide qué campos de catálogo puede modificar cada rol: esas reglas están en los handlers y servicios de `modules`.

## Desactivación y reactivación

Un administrador de Tuercavo llama a `POST /api/users/{public_id}/deactivate` antes de retirar la asignación a la aplicación empresarial en Entra. `modules` bloquea la fila del usuario y, en la misma transacción, marca `is_active = false` y llama a `session::revoke_user` para revocar todas sus sesiones. Las sesiones revocadas no vuelven a ser válidas. `SessionStore::replace` también rechaza el siguiente login mientras el usuario esté inactivo. Después de confirmar la desactivación local, el administrador de Entra retira la asignación directa o la pertenencia a los grupos que conceden acceso.

Para reactivar, el administrador restablece la asignación en Entra y llama a `POST /api/users/{public_id}/reactivate`. La API marca `is_active = true` sin restaurar sesiones; el usuario necesita un nuevo login para crear una. Ambas operaciones requieren el permiso `users.update`. La desactivación del último administrador activo se rechaza; primero debe asignarse `admin` a otro usuario activo en Tuercavo.

## Salida de sesión

`POST /api/auth/logout` exige el origen correcto, revoca en PostgreSQL la sesión presentada, cancela un login pendiente del mismo navegador si existe y elimina ambas cookies. Devuelve `204`. Es un cierre **local**: la sesión que Microsoft mantenga en Entra puede seguir vigente.

`GET /api/auth/entra-logout` redirige al `end_session_endpoint` descubierto de Entra. Si se configuró `POST_LOGOUT_REDIRECT_URI`, la incluye como destino posterior. Esta ruta no revoca la sesión local: para cerrar ambas, el cliente debe llamar primero al logout local y después navegar a la ruta de Entra. Como el crate descarta el ID Token tras el callback, la redirección de salida no aporta un `id_token_hint`.

## Verificación

```sh
cargo test --workspace --all-features --locked
cargo check -p api --locked
cargo check -p api --features scalar --locked
```
