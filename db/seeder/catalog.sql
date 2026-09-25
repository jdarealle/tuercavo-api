-- Datos opcionales para una base de desarrollo recién migrada. Aplicar una sola vez.
-- Ejecutar con ON_ERROR_STOP para que cualquier error produzca un código de salida distinto de cero.
-- Los productos se crean con referencias activas; después se aplican los estados finales.
-- Los departamentos no se precargan: se crean al definir la organización con public_id generado; users.department_id admite NULL.
-- Los permisos de departamentos se cargan en la migración de datos de referencia; este catálogo no los modifica.

BEGIN;
SET LOCAL TIME ZONE 'UTC';

INSERT INTO categories (public_id, name, description) VALUES
    ('10000000-0000-4000-8000-000000000001', 'Herramientas manuales', 'Herramientas de uso manual para trabajos de reparación, montaje y mantenimiento.'),
    ('10000000-0000-4000-8000-000000000002', 'Herramientas eléctricas', 'Equipos eléctricos portátiles para perforación, corte, lijado y desbaste.'),
    ('10000000-0000-4000-8000-000000000003', 'Tornillería y fijación', 'Tornillos, taquetes, clavos, tuercas y otros elementos de sujeción.'),
    ('10000000-0000-4000-8000-000000000004', 'Plomería', 'Conexiones, válvulas, tubería y accesorios para instalaciones hidráulicas.'),
    ('10000000-0000-4000-8000-000000000005', 'Electricidad', 'Material eléctrico para instalaciones residenciales y comerciales.'),
    ('10000000-0000-4000-8000-000000000006', 'Pinturas y acabados', 'Pinturas, recubrimientos y accesorios para preparación y acabado de superficies.'),
    ('10000000-0000-4000-8000-000000000007', 'Construcción', 'Materiales y herramientas auxiliares para obra y remodelación.'),
    ('10000000-0000-4000-8000-000000000008', 'Seguridad industrial', 'Equipo de protección personal y señalización para el área de trabajo.'),
    ('10000000-0000-4000-8000-000000000009', 'Jardinería', NULL),
    ('10000000-0000-4000-8000-000000000010', 'Adhesivos y selladores', 'Soluciones para unir, rellenar y sellar diferentes materiales.');

INSERT INTO suppliers (public_id, code, name, contact_name, email, phone) VALUES
    ('20000000-0000-4000-8000-000000000001', 'FERRE-001', 'Distribuidora Ferretera del Norte', 'Mariana López', 'ventas@ferreteradelnorte.example', '+52 664 555 0101'),
    ('20000000-0000-4000-8000-000000000002', 'HERRA-002', 'Herramientas Profesionales de México', 'Carlos Ramírez', 'pedidos@herramientaspro.example', '+52 81 5555 0102'),
    ('20000000-0000-4000-8000-000000000003', 'CONST-003', 'Suministros para Construcción Baja', 'Alejandra Torres', 'contacto@construccionbaja.example', '+52 664 555 0103'),
    ('20000000-0000-4000-8000-000000000004', 'ELEC-004', 'Material Eléctrico Nacional', 'Jorge Mendoza', 'mayoreo@materialelectrico.example', '+52 55 5555 0104'),
    ('20000000-0000-4000-8000-000000000005', 'JARD-005', 'Equipos y Jardines del Pacífico', NULL, NULL, NULL);

CREATE TEMP TABLE seed_products (
    public_id UUID PRIMARY KEY,
    sku TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    description TEXT,
    brand TEXT,
    category_public_id UUID NOT NULL,
    supplier_public_id UUID,
    unit TEXT NOT NULL,
    price NUMERIC(12, 2) NOT NULL,
    status TEXT NOT NULL
) ON COMMIT DROP;

INSERT INTO seed_products (
    public_id, sku, name, description, brand, category_public_id,
    supplier_public_id, unit, price, status
) VALUES
    ('30000000-0000-4000-8000-000000000001', 'HM-MART-16', 'Martillo uña curva 16 oz', 'Martillo con cabeza de acero pulido y mango antiderrapante.', 'Truper', '10000000-0000-4000-8000-000000000001', '20000000-0000-4000-8000-000000000001', 'piece', 249.90, 'active'),
    ('30000000-0000-4000-8000-000000000002', 'HM-DES-6P', 'Juego de desarmadores 6 piezas', 'Juego con puntas planas y Phillips para uso general.', 'Pretul', '10000000-0000-4000-8000-000000000001', '20000000-0000-4000-8000-000000000001', 'pack', 189.50, 'active'),
    ('30000000-0000-4000-8000-000000000003', 'HM-PIN-8', 'Pinza de electricista 8 pulgadas', 'Pinza de acero al carbono con mangos aislados.', 'Urrea', '10000000-0000-4000-8000-000000000001', '20000000-0000-4000-8000-000000000002', 'piece', 329.00, 'active'),
    ('30000000-0000-4000-8000-000000000004', 'HM-LLA-10', 'Llave ajustable 10 pulgadas', 'Llave cromada con escala grabada y apertura amplia.', 'Stanley', '10000000-0000-4000-8000-000000000001', '20000000-0000-4000-8000-000000000002', 'piece', 278.00, 'active'),
    ('30000000-0000-4000-8000-000000000005', 'HM-FLX-5', 'Flexómetro 5 metros', 'Cinta métrica con freno, seguro y recubrimiento de nylon.', 'Fiero', '10000000-0000-4000-8000-000000000001', NULL, 'piece', 98.00, 'inactive'),

    ('30000000-0000-4000-8000-000000000006', 'HE-TAL-650', 'Taladro percutor 650 W', 'Taladro reversible de velocidad variable con broquero de 13 mm.', 'Bosch', '10000000-0000-4000-8000-000000000002', '20000000-0000-4000-8000-000000000002', 'piece', 1599.00, 'active'),
    ('30000000-0000-4000-8000-000000000007', 'HE-ESM-4.5', 'Esmeriladora angular 4.5 pulgadas', 'Esmeriladora de 850 W con guarda ajustable y mango lateral.', 'DeWalt', '10000000-0000-4000-8000-000000000002', '20000000-0000-4000-8000-000000000002', 'piece', 1890.00, 'active'),
    ('30000000-0000-4000-8000-000000000008', 'HE-SIE-7.25', 'Sierra circular 7.25 pulgadas', 'Sierra de 1400 W con ajuste de profundidad y bisel.', 'Makita', '10000000-0000-4000-8000-000000000002', '20000000-0000-4000-8000-000000000002', 'piece', 2499.00, 'active'),
    ('30000000-0000-4000-8000-000000000009', 'HE-LIJ-240', 'Lijadora orbital 240 W', 'Lijadora compacta con sistema de recolección de polvo.', 'Black+Decker', '10000000-0000-4000-8000-000000000002', '20000000-0000-4000-8000-000000000001', 'piece', 1149.00, 'active'),
    ('30000000-0000-4000-8000-000000000010', 'HE-BRO-20', 'Juego de brocas mixtas 20 piezas', 'Brocas para metal, concreto y madera en estuche organizador.', 'Bosch', '10000000-0000-4000-8000-000000000002', '20000000-0000-4000-8000-000000000002', 'pack', 599.00, 'active'),

    ('30000000-0000-4000-8000-000000000011', 'TF-TOR-1X8', 'Caja de tornillos para madera 1 x 8', 'Caja con tornillos galvanizados de cabeza plana para madera.', 'Fixser', '10000000-0000-4000-8000-000000000003', '20000000-0000-4000-8000-000000000001', 'box', 84.50, 'active'),
    ('30000000-0000-4000-8000-000000000012', 'TF-TAQ-14', 'Paquete de taquetes plásticos 1/4 pulgada', 'Paquete de taquetes expansivos para concreto y mampostería.', 'Anclo', '10000000-0000-4000-8000-000000000003', '20000000-0000-4000-8000-000000000003', 'pack', 42.00, 'active'),
    ('30000000-0000-4000-8000-000000000013', 'TF-CLA-2', 'Clavo con cabeza 2 pulgadas', 'Clavo de acero para trabajos generales de carpintería.', 'Deacero', '10000000-0000-4000-8000-000000000003', '20000000-0000-4000-8000-000000000003', 'kg', 69.90, 'active'),
    ('30000000-0000-4000-8000-000000000014', 'TF-TUE-14', 'Caja de tuercas hexagonales 1/4 pulgada', 'Caja de tuercas zincadas para fijación mecánica.', 'Fixser', '10000000-0000-4000-8000-000000000003', '20000000-0000-4000-8000-000000000001', 'box', 115.00, 'active'),
    ('30000000-0000-4000-8000-000000000015', 'TF-ARA-14', 'Caja de arandelas planas 1/4 pulgada', 'Caja de arandelas galvanizadas para distribuir carga de apriete.', NULL, '10000000-0000-4000-8000-000000000003', NULL, 'box', 89.00, 'active'),

    ('30000000-0000-4000-8000-000000000016', 'PL-TUB-12', 'Tubo PVC hidráulico 1/2 pulgada', 'Tubo hidráulico para conducción de agua a presión.', 'Dura', '10000000-0000-4000-8000-000000000004', '20000000-0000-4000-8000-000000000003', 'meter', 36.50, 'active'),
    ('30000000-0000-4000-8000-000000000017', 'PL-COD-12', 'Codo PVC 90 grados 1/2 pulgada', 'Conexión hidráulica cementable para cambio de dirección.', 'Dura', '10000000-0000-4000-8000-000000000004', '20000000-0000-4000-8000-000000000003', 'piece', 8.90, 'active'),
    ('30000000-0000-4000-8000-000000000018', 'PL-VAL-12', 'Válvula esfera 1/2 pulgada', 'Válvula de latón roscada para control de flujo.', 'Foset', '10000000-0000-4000-8000-000000000004', '20000000-0000-4000-8000-000000000001', 'piece', 129.00, 'active'),
    ('30000000-0000-4000-8000-000000000019', 'PL-LLA-LAV', 'Llave mezcladora para lavabo', 'Mezcladora de dos manerales con acabado cromado.', 'Helvex', '10000000-0000-4000-8000-000000000004', '20000000-0000-4000-8000-000000000001', 'piece', 1199.00, 'active'),
    ('30000000-0000-4000-8000-000000000020', 'PL-CIN-TEF', 'Cinta selladora PTFE 12 mm', NULL, 'Foset', '10000000-0000-4000-8000-000000000004', NULL, 'piece', 18.50, 'active'),

    ('30000000-0000-4000-8000-000000000021', 'EL-CAB-12', 'Cable THW calibre 12', 'Conductor de cobre para instalaciones eléctricas de baja tensión.', 'Condumex', '10000000-0000-4000-8000-000000000005', '20000000-0000-4000-8000-000000000004', 'meter', 17.80, 'active'),
    ('30000000-0000-4000-8000-000000000022', 'EL-CON-DUP', 'Contacto dúplex polarizado', 'Contacto residencial de 15 A con placa incluida.', 'Bticino', '10000000-0000-4000-8000-000000000005', '20000000-0000-4000-8000-000000000004', 'piece', 74.00, 'active'),
    ('30000000-0000-4000-8000-000000000023', 'EL-INT-SEN', 'Interruptor sencillo', 'Interruptor residencial de un polo para caja estándar.', 'Schneider', '10000000-0000-4000-8000-000000000005', '20000000-0000-4000-8000-000000000004', 'piece', 68.50, 'active'),
    ('30000000-0000-4000-8000-000000000024', 'EL-CEN-8', 'Centro de carga 8 espacios', 'Gabinete para interruptores termomagnéticos de uso residencial.', 'Square D', '10000000-0000-4000-8000-000000000005', '20000000-0000-4000-8000-000000000004', 'piece', 749.00, 'active'),
    ('30000000-0000-4000-8000-000000000025', 'EL-LED-9', 'Caja de focos LED 9 W luz cálida', 'Caja de cuatro focos con base E27 y flujo de 800 lúmenes cada uno.', 'Philips', '10000000-0000-4000-8000-000000000005', '20000000-0000-4000-8000-000000000004', 'box', 239.00, 'active'),

    ('30000000-0000-4000-8000-000000000026', 'PA-VIN-4B', 'Pintura vinílica blanca 4 L', 'Envase de 4 L de pintura mate para interiores con buena cobertura y bajo olor.', 'Comex', '10000000-0000-4000-8000-000000000006', '20000000-0000-4000-8000-000000000001', 'piece', 139.75, 'active'),
    ('30000000-0000-4000-8000-000000000027', 'PA-ESM-1N', 'Esmalte negro brillante 1 L', 'Esmalte alquidálico para metal y madera.', 'Berel', '10000000-0000-4000-8000-000000000006', '20000000-0000-4000-8000-000000000001', 'liter', 189.00, 'active'),
    ('30000000-0000-4000-8000-000000000028', 'PA-ROD-9', 'Rodillo profesional 9 pulgadas', 'Rodillo de felpa para superficies lisas y semirrugosas.', 'Perfect', '10000000-0000-4000-8000-000000000006', NULL, 'piece', 119.00, 'active'),
    ('30000000-0000-4000-8000-000000000029', 'PA-BRO-3', 'Brocha de cerdas mixtas 3 pulgadas', 'Brocha para pinturas vinílicas y esmaltes.', 'Perfect', '10000000-0000-4000-8000-000000000006', NULL, 'piece', 78.50, 'active'),
    ('30000000-0000-4000-8000-000000000030', 'PA-LIJ-120', 'Paquete de lijas para madera grano 120', 'Paquete de hojas abrasivas para preparación y acabado de madera.', 'Fandeli', '10000000-0000-4000-8000-000000000006', '20000000-0000-4000-8000-000000000002', 'pack', 95.00, 'active'),

    ('30000000-0000-4000-8000-000000000031', 'CO-CEM-50', 'Cemento gris 50 kg', 'Cemento Portland para trabajos de albañilería y concreto.', 'Cemex', '10000000-0000-4000-8000-000000000007', '20000000-0000-4000-8000-000000000003', 'piece', 259.00, 'active'),
    ('30000000-0000-4000-8000-000000000032', 'CO-MOR-40', 'Mortero seco 40 kg', 'Mezcla preparada para asentado y repellado de muros.', 'Cruz Azul', '10000000-0000-4000-8000-000000000007', '20000000-0000-4000-8000-000000000003', 'piece', 189.00, 'active'),
    ('30000000-0000-4000-8000-000000000033', 'CO-PAL-RED', 'Pala redonda con mango', 'Pala de acero con mango de madera para excavación.', 'Truper', '10000000-0000-4000-8000-000000000007', '20000000-0000-4000-8000-000000000003', 'piece', 349.00, 'active'),
    ('30000000-0000-4000-8000-000000000034', 'CO-CUC-11', 'Cuchara para albañil 11 pulgadas', 'Cuchara triangular de acero templado con mango de madera.', 'Bellota', '10000000-0000-4000-8000-000000000007', '20000000-0000-4000-8000-000000000003', 'piece', 219.00, 'active'),
    ('30000000-0000-4000-8000-000000000035', 'CO-NIV-24', 'Nivel de aluminio 24 pulgadas', 'Nivel con tres burbujas y cuerpo reforzado.', 'Stanley', '10000000-0000-4000-8000-000000000007', '20000000-0000-4000-8000-000000000002', 'piece', 449.00, 'inactive'),

    ('30000000-0000-4000-8000-000000000036', 'SI-CAS-B', 'Casco de seguridad blanco', 'Casco ajustable con suspensión de cuatro puntos.', 'Jyrsa', '10000000-0000-4000-8000-000000000008', '20000000-0000-4000-8000-000000000001', 'piece', 179.00, 'active'),
    ('30000000-0000-4000-8000-000000000037', 'SI-GUA-N', 'Paquete de guantes de nitrilo reforzado', 'Paquete de guantes reutilizables resistentes a abrasión y aceites.', '3M', '10000000-0000-4000-8000-000000000008', '20000000-0000-4000-8000-000000000001', 'pack', 149.00, 'active'),
    ('30000000-0000-4000-8000-000000000038', 'SI-LEN-T', 'Lentes de seguridad transparentes', 'Lentes envolventes con protección contra impactos.', 'Jyrsa', '10000000-0000-4000-8000-000000000008', '20000000-0000-4000-8000-000000000001', 'piece', 89.00, 'active'),
    ('30000000-0000-4000-8000-000000000039', 'SI-CHA-R', 'Chaleco reflejante naranja', 'Chaleco de alta visibilidad con bandas reflejantes.', 'Surtek', '10000000-0000-4000-8000-000000000008', NULL, 'piece', 139.00, 'active'),
    ('30000000-0000-4000-8000-000000000040', 'SI-MAS-N95', 'Caja de mascarillas respiratorias N95', 'Caja de respiradores desechables para partículas sin aceite.', '3M', '10000000-0000-4000-8000-000000000008', '20000000-0000-4000-8000-000000000001', 'box', 699.00, 'active'),

    ('30000000-0000-4000-8000-000000000041', 'JA-TIJ-8', 'Tijera para podar 8 pulgadas', 'Tijera bypass para ramas y tallos de jardín.', 'Truper', '10000000-0000-4000-8000-000000000009', '20000000-0000-4000-8000-000000000005', 'piece', 229.00, 'inactive'),
    ('30000000-0000-4000-8000-000000000042', 'JA-MAN-15', 'Manguera reforzada 15 metros', 'Manguera flexible de tres capas con conexiones.', 'Foset', '10000000-0000-4000-8000-000000000009', '20000000-0000-4000-8000-000000000005', 'piece', 579.00, 'inactive'),
    ('30000000-0000-4000-8000-000000000043', 'JA-REG-8', 'Regadera plástica 8 L', 'Regadera ligera con rociador desmontable.', 'Garden Pro', '10000000-0000-4000-8000-000000000009', '20000000-0000-4000-8000-000000000005', 'piece', 189.00, 'inactive'),
    ('30000000-0000-4000-8000-000000000044', 'JA-RAS-14', 'Rastrillo jardín 14 dientes', 'Rastrillo de acero con mango largo de madera.', 'Truper', '10000000-0000-4000-8000-000000000009', '20000000-0000-4000-8000-000000000005', 'piece', 299.00, 'inactive'),
    ('30000000-0000-4000-8000-000000000045', 'JA-PAL-M', 'Pala jardinera de mano', 'Pala angosta para trasplante y trabajo en macetas.', 'Garden Pro', '10000000-0000-4000-8000-000000000009', '20000000-0000-4000-8000-000000000005', 'piece', 109.00, 'inactive'),

    ('30000000-0000-4000-8000-000000000046', 'AS-SIL-T', 'Silicón transparente 280 ml', 'Sellador de uso general para vidrio, aluminio y cerámica.', 'Sista', '10000000-0000-4000-8000-000000000010', '20000000-0000-4000-8000-000000000001', 'piece', 119.00, 'archived'),
    ('30000000-0000-4000-8000-000000000047', 'AS-PEG-5', 'Kit de pegamento epóxico 5 minutos', 'Kit de adhesivo bicomponente para metal, madera y cerámica.', 'Resistol', '10000000-0000-4000-8000-000000000010', '20000000-0000-4000-8000-000000000001', 'pack', 149.00, 'archived'),
    ('30000000-0000-4000-8000-000000000048', 'AS-CIN-D', 'Cinta doble cara montaje', 'Cinta de alta adherencia para fijaciones interiores.', '3M', '10000000-0000-4000-8000-000000000010', NULL, 'piece', 129.00, 'archived'),
    ('30000000-0000-4000-8000-000000000049', 'AS-ESP-500', 'Espuma expansiva 500 ml', 'Espuma de poliuretano para relleno y aislamiento.', 'Sista', '10000000-0000-4000-8000-000000000010', '20000000-0000-4000-8000-000000000003', 'piece', 189.00, 'archived'),
    ('30000000-0000-4000-8000-000000000050', 'AS-RES-850', 'Adhesivo de contacto 850 ml', 'Adhesivo para laminados, madera, piel y hule.', 'Resistol', '10000000-0000-4000-8000-000000000010', '20000000-0000-4000-8000-000000000001', 'piece', 259.00, 'archived');

DO $seed_check$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM seed_products p
        LEFT JOIN categories c ON c.public_id = p.category_public_id
        WHERE c.id IS NULL
    ) THEN
        RAISE EXCEPTION 'El catálogo de desarrollo contiene una referencia a categoría inexistente';
    END IF;

    IF EXISTS (
        SELECT 1
        FROM seed_products p
        LEFT JOIN suppliers s ON s.public_id = p.supplier_public_id
        WHERE p.supplier_public_id IS NOT NULL AND s.id IS NULL
    ) THEN
        RAISE EXCEPTION 'El catálogo de desarrollo contiene una referencia a proveedor inexistente';
    END IF;

    IF EXISTS (
        SELECT 1
        FROM seed_products p
        JOIN categories c ON c.public_id = p.category_public_id
        WHERE c.status <> 'active'::catalog_status
    ) THEN
        RAISE EXCEPTION 'No se puede crear un producto de desarrollo con una categoría inactiva';
    END IF;

    IF EXISTS (
        SELECT 1
        FROM seed_products p
        JOIN suppliers s ON s.public_id = p.supplier_public_id
        WHERE s.status <> 'active'::catalog_status
    ) THEN
        RAISE EXCEPTION 'No se puede crear un producto de desarrollo con un proveedor inactivo';
    END IF;
END
$seed_check$;

INSERT INTO products (
    public_id, sku, name, description, brand, category_id,
    supplier_id, unit, price, status
)
SELECT
    p.public_id,
    p.sku,
    p.name,
    p.description,
    p.brand,
    c.id,
    s.id,
    p.unit,
    p.price,
    p.status::catalog_status
FROM seed_products p
JOIN categories c ON c.public_id = p.category_public_id
LEFT JOIN suppliers s ON s.public_id = p.supplier_public_id;

DO $seed_check$
BEGIN
    IF (SELECT count(*) FROM products) <> (SELECT count(*) FROM seed_products) THEN
        RAISE EXCEPTION 'No fue posible registrar todos los productos de desarrollo';
    END IF;
END
$seed_check$;

UPDATE categories
SET status = 'inactive', updated_at = CURRENT_TIMESTAMP
WHERE public_id = '10000000-0000-4000-8000-000000000009';

UPDATE categories
SET status = 'archived', updated_at = CURRENT_TIMESTAMP
WHERE public_id = '10000000-0000-4000-8000-000000000010';

UPDATE suppliers
SET status = 'inactive', updated_at = CURRENT_TIMESTAMP
WHERE public_id = '20000000-0000-4000-8000-000000000005';

COMMIT;
