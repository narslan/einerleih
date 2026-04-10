-- ===============================================
-- 0002_reference_data.up.sql
-- Reference data required by frontend selections
-- ===============================================

INSERT INTO towns (town_id, name)
SELECT seed.town_id, seed.name
FROM (
    VALUES
        ('10000000-0000-0000-0000-000000000001'::UUID, 'Achim'),
        ('10000000-0000-0000-0000-000000000002'::UUID, 'Verden (Aller)'),
        ('10000000-0000-0000-0000-000000000003'::UUID, 'Langwedel'),
        ('10000000-0000-0000-0000-000000000004'::UUID, 'Ottersberg'),
        ('10000000-0000-0000-0000-000000000005'::UUID, 'Oyten'),
        ('10000000-0000-0000-0000-000000000006'::UUID, 'Dörverden'),
        ('10000000-0000-0000-0000-000000000007'::UUID, 'Kirchlinteln'),
        ('10000000-0000-0000-0000-000000000008'::UUID, 'Thedinghausen')
) AS seed(town_id, name)
WHERE NOT EXISTS (
    SELECT 1
    FROM towns
    WHERE towns.name = seed.name
);

INSERT INTO categories (category_id, name)
SELECT seed.category_id, seed.name
FROM (
    VALUES
        ('20000000-0000-0000-0000-000000000001'::UUID, 'Technik'),
        ('20000000-0000-0000-0000-000000000002'::UUID, 'Party & Events'),
        ('20000000-0000-0000-0000-000000000003'::UUID, 'Garten'),
        ('20000000-0000-0000-0000-000000000004'::UUID, 'Werkzeug'),
        ('20000000-0000-0000-0000-000000000005'::UUID, 'Freizeit'),
        ('20000000-0000-0000-0000-000000000006'::UUID, 'Küche')
) AS seed(category_id, name)
WHERE NOT EXISTS (
    SELECT 1
    FROM categories
    WHERE categories.name = seed.name
);
