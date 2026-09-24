# Primer administrador

`bootstrap-admin` es un paquete binario del workspace. Promueve al primer administrador de Tuercavo después de que la persona haya completado su primer login mediante Entra ID y exista como `consultor` en PostgreSQL. Reutiliza de `auth` el bloqueo de autorización, la carga de permisos efectivos y la revocación de sesiones; no ejecuta el flujo OIDC.

## Archivos

| Archivo | Responsabilidad |
| --- | --- |
| [`src/main.rs`](src/main.rs) | Lee las variables, conecta a PostgreSQL y ejecuta el comando. |
| [`src/bootstrap.rs`](src/bootstrap.rs) | Valida la identidad y el estado del usuario, promueve a `admin`, comprueba los permisos y revoca sesiones dentro de una transacción. |

## Ejecución

Requiere `DATABASE_URL`, `ENTRA_TENANT_ID` y `BOOTSTRAP_ADMIN_OBJECT_ID` (el Object ID de la persona dentro del tenant). Desde la raíz del proyecto:

```sh
BOOTSTRAP_ADMIN_OBJECT_ID=REEMPLAZAR_CON_UUID_DEL_USUARIO \
  cargo run -p bootstrap-admin -- --env-file .env
```

`--env-file` es opcional; el comando no busca `.env` por sí mismo y las variables ya presentes en el entorno tienen prioridad. El Object ID se consulta en **Entra ID → Users → usuario → Overview**, no en la App registration.

El comando toma un bloqueo sobre el rol `admin` para serializarse con los cambios administrativos de la API. Rechaza la operación si ya existe un usuario con ese rol en el tenant, si la identidad no ha completado su primer login o si está desactivada. En una transacción asigna el rol, verifica que conserve los permisos administrativos obligatorios y revoca las sesiones del usuario. Tras ejecutarlo, la persona debe iniciar sesión de nuevo.

En producción, ejecuta este binario como proceso temporal con acceso a PostgreSQL. El proceso habitual de la API solo necesita el ejecutable `api`.
