-- Chapter IV Validation Compound Data
-- Extracted from: CalebBell/thermo library (DIPPR 801 backed), 2026-04-05
-- All values in canonical units: K, kPa (absolute), cm3/mol, g/mol
-- See docs/en/research-paper/chapter-4-validation.md for validation cases
--
-- NOTE: The original thesis (Jackson & Mendible, 1999) does not list the exact
-- Tc/Pc/w values used — it references "literature" (Reid, Prausnitz & Poling).
-- These DIPPR values may differ slightly from 1999 editions, consistent with the
-- paper's own caveat: "probable difference between the values of the critical
-- properties of the mixture components used by both projects."

-- ============================================================================
-- COMPONENTS (15 compounds used in Chapter IV validation)
-- ============================================================================

INSERT OR IGNORE INTO components (name, formula, cas_number, mw, tc, pc, w, zc, vc, tb, dipole_moment, source, notes)
VALUES
    ('methane',             'CH4',   '74-82-8',    16.0425,  190.564,   4599.2,  0.01142,  0.28629,   98.628,  111.667,  0.0,   'thermo/DIPPR', 'Chapter IV: critical points (4.1), bubble T (4.5)'),
    ('ethane',              'C2H6',  '74-84-0',    30.0690,  305.322,   4872.2,  0.0995,   0.27990,  145.839,  184.569,  0.0,   'thermo/DIPPR', 'Chapter IV: critical points (4.1), bubble T (4.5)'),
    ('propane',             'C3H8',  '74-98-6',    44.0956,  369.89,    4251.2,  0.1521,   0.27646,  200.000,  231.036,  0.08,  'thermo/DIPPR', 'Chapter IV: critical points (4.1), bubble T (4.5)'),
    ('n-butane',            'C4H10', '106-97-8',   58.1222,  425.125,   3796.0,  0.201,    0.27377,  254.922,  272.660,  0.0,   'thermo/DIPPR', 'Chapter IV: critical points (4.1), isothermal flash (4.6), kij regression (4.7), bubble T (4.5)'),
    ('n-pentane',           'C5H12', '109-66-0',   72.1488,  469.7,     3367.5,  0.251,    0.26863,  311.526,  309.209,  0.0,   'thermo/DIPPR', 'Chapter IV: critical points (4.1)'),
    ('carbon dioxide',      'CO2',   '124-38-9',   44.0095,  304.1282,  7377.3,  0.22394,  0.27459,   94.118,  194.67,   0.0,   'thermo/DIPPR', 'Chapter IV: critical points (4.1), kij regression (4.7)'),
    ('hydrogen sulfide',    'H2S',   '7783-06-4',  34.0809,  373.1,     9000.0,  0.1005,   0.28471,   98.135,  212.855,  0.97,  'thermo/DIPPR', 'Chapter IV: critical points (4.1)'),
    ('benzene',             'C6H6',  '71-43-2',    78.1118,  562.02,    4907.277,0.211,    0.26920,  256.345,  353.219,  0.0,   'thermo/DIPPR', 'Chapter IV: adiabatic flash (4.2)'),
    ('cyclohexane',         'C6H12', '110-82-7',   84.1595,  553.6,     4080.5,  0.2096,   0.27497,  310.174,  353.865,  0.61,  'thermo/DIPPR', 'Chapter IV: adiabatic flash (4.2)'),
    ('methylcyclohexane',   'C7H14', '108-87-2',   98.1861,  572.2,     3470.0,  0.234,    0.26815,  367.647,  374.010,  0.0,   'thermo/DIPPR', 'Chapter IV: adiabatic flash (4.2)'),
    ('n-hexane',            'C6H14', '110-54-3',   86.1754,  507.82,    3044.1,  0.3,      0.26643,  369.549,  341.866,  0.0,   'thermo/DIPPR', 'Chapter IV: adiabatic flash (4.2)'),
    ('n-heptane',           'C7H16', '142-82-5',  100.2019,  540.2,     2735.73, 0.349,    0.26141,  429.185,  371.550,  0.0,   'thermo/DIPPR', 'Chapter IV: isothermal flash (4.6)'),
    ('methanol',            'CH4O',  '67-56-1',    32.0419,  513.38,    8215.85, 0.5625,   0.21909,  113.828,  337.632,  1.7,   'thermo/DIPPR', 'Chapter IV: bubble P (4.3)'),
    ('water',               'H2O',   '7732-18-5',  18.0153,  647.096,  22064.0,  0.3443,   0.22944,   55.948,  373.124,  1.85,  'thermo/DIPPR', 'Chapter IV: bubble P (4.3), dew point (4.4)'),
    ('2-propanol',          'C3H8O', '67-63-0',    60.0950,  508.3,     4764.0,  0.665,    0.25025,  222.000,  355.36,   1.58,  'thermo/DIPPR', 'Chapter IV: dew point (4.4)');

-- ============================================================================
-- ACTIVITY MODEL PARAMETERS (from Chapter IV tables)
-- ============================================================================

-- Table 4.5: van Laar parameters for methanol(1)/water(2) at 298 K
-- Source: Orbey and Sandler [21]
INSERT OR IGNORE INTO activity_params (comp1_id, comp2_id, model, a12, a21, temperature, source, notes)
SELECT c1.id, c2.id, 'van_laar', 0.5853, 0.3458, 298.0, 'orbey_sandler_ref21', 'Chapter IV Table 4.5'
FROM components c1, components c2
WHERE c1.name = 'methanol' AND c2.name = 'water';

-- Wilson parameters for 2-propanol(1)/water(2)
-- Source: Smith et al. [22] (used in Tables 4.7-4.8)
-- Note: The thesis uses Wilson model for dew point calculations but does not
-- explicitly list the Lambda values in a table. These must be provided by the
-- user or regressed from the experimental data in Tables 4.7-4.8.

-- ============================================================================
-- EXPERIMENTAL VLE DATA (for kij regression validation)
-- ============================================================================

-- Table 4.11: Experimental VLE data for CO2(1)/n-butane(2) at 357.57 K
-- Source: Da Silva and Báez [4]
-- Used for kij regression in §4.7 (expected kij = 0.1357 with PR EOS)
-- 10 data points: P (kPa), x1, y1

INSERT OR IGNORE INTO experimental_vle (system_name, comp1_id, comp2_id, temperature, pressure, x1, y1, source)
SELECT 'CO2/n-butane', c1.id, c2.id, 357.57, 1723.689, 0.0451, 0.2880, 'dasilva1989_table411'
FROM components c1, components c2 WHERE c1.name = 'carbon dioxide' AND c2.name = 'n-butane';

INSERT OR IGNORE INTO experimental_vle (system_name, comp1_id, comp2_id, temperature, pressure, x1, y1, source)
SELECT 'CO2/n-butane', c1.id, c2.id, 357.57, 2068.427, 0.0924, 0.4658, 'dasilva1989_table411'
FROM components c1, components c2 WHERE c1.name = 'carbon dioxide' AND c2.name = 'n-butane';

INSERT OR IGNORE INTO experimental_vle (system_name, comp1_id, comp2_id, temperature, pressure, x1, y1, source)
SELECT 'CO2/n-butane', c1.id, c2.id, 357.57, 2413.166, 0.1503, 0.5765, 'dasilva1989_table411'
FROM components c1, components c2 WHERE c1.name = 'carbon dioxide' AND c2.name = 'n-butane';

INSERT OR IGNORE INTO experimental_vle (system_name, comp1_id, comp2_id, temperature, pressure, x1, y1, source)
SELECT 'CO2/n-butane', c1.id, c2.id, 357.57, 2757.903, 0.2165, 0.6564, 'dasilva1989_table411'
FROM components c1, components c2 WHERE c1.name = 'carbon dioxide' AND c2.name = 'n-butane';

INSERT OR IGNORE INTO experimental_vle (system_name, comp1_id, comp2_id, temperature, pressure, x1, y1, source)
SELECT 'CO2/n-butane', c1.id, c2.id, 357.57, 3102.642, 0.2907, 0.7175, 'dasilva1989_table411'
FROM components c1, components c2 WHERE c1.name = 'carbon dioxide' AND c2.name = 'n-butane';

INSERT OR IGNORE INTO experimental_vle (system_name, comp1_id, comp2_id, temperature, pressure, x1, y1, source)
SELECT 'CO2/n-butane', c1.id, c2.id, 357.57, 3447.379, 0.3740, 0.7685, 'dasilva1989_table411'
FROM components c1, components c2 WHERE c1.name = 'carbon dioxide' AND c2.name = 'n-butane';

INSERT OR IGNORE INTO experimental_vle (system_name, comp1_id, comp2_id, temperature, pressure, x1, y1, source)
SELECT 'CO2/n-butane', c1.id, c2.id, 357.57, 3792.118, 0.4686, 0.8115, 'dasilva1989_table411'
FROM components c1, components c2 WHERE c1.name = 'carbon dioxide' AND c2.name = 'n-butane';

INSERT OR IGNORE INTO experimental_vle (system_name, comp1_id, comp2_id, temperature, pressure, x1, y1, source)
SELECT 'CO2/n-butane', c1.id, c2.id, 357.57, 4136.856, 0.5788, 0.8477, 'dasilva1989_table411'
FROM components c1, components c2 WHERE c1.name = 'carbon dioxide' AND c2.name = 'n-butane';

INSERT OR IGNORE INTO experimental_vle (system_name, comp1_id, comp2_id, temperature, pressure, x1, y1, source)
SELECT 'CO2/n-butane', c1.id, c2.id, 357.57, 4481.594, 0.7142, 0.8788, 'dasilva1989_table411'
FROM components c1, components c2 WHERE c1.name = 'carbon dioxide' AND c2.name = 'n-butane';

INSERT OR IGNORE INTO experimental_vle (system_name, comp1_id, comp2_id, temperature, pressure, x1, y1, source)
SELECT 'CO2/n-butane', c1.id, c2.id, 357.57, 4826.332, 0.9005, 0.9233, 'dasilva1989_table411'
FROM components c1, components c2 WHERE c1.name = 'carbon dioxide' AND c2.name = 'n-butane';

-- Table 4.12: kij regression result
-- CO2(1)/n-butane(2) with Peng-Robinson EOS at 357.57 K
-- Regressed kij = 0.1357
INSERT OR IGNORE INTO kij_params (comp1_id, comp2_id, eos_model, kij, temperature, source, notes)
SELECT MIN(c1.id, c2.id), MAX(c1.id, c2.id), 'PR', 0.1357, 357.57, 'dasilva1989_table412', 'Chapter IV §4.7 kij regression result'
FROM components c1, components c2
WHERE c1.name = 'carbon dioxide' AND c2.name = 'n-butane';
