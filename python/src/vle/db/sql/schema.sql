-- VLE Component Property Database Schema
-- All values stored in canonical units: K, kPa (absolute), cm3/mol, kJ/(kmol*K), g/mol
-- See docs/en/parameters/parameter_reference.md for parameter descriptions
-- See CLAUDE.md "Units Documentation Rules" for canonical unit definitions

PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;

-- Per-component thermodynamic properties
CREATE TABLE IF NOT EXISTS components (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            TEXT NOT NULL UNIQUE,            -- lowercase canonical name ("methane", "carbon dioxide")
    formula         TEXT,                            -- molecular formula ("CH4", "CO2")
    cas_number      TEXT UNIQUE,                     -- CAS registry number ("74-82-8")
    mw              REAL,                            -- molecular weight in g/mol

    -- Critical properties (required by all cubic EOS)
    tc              REAL,                            -- critical temperature in K
    pc              REAL,                            -- critical pressure in kPa (absolute, NEVER gauge)
    w               REAL,                            -- Pitzer acentric factor (dimensionless)
    zc              REAL,                            -- critical compressibility factor (dimensionless)
    vc              REAL,                            -- critical molar volume in cm3/mol

    -- Normal boiling point
    tb              REAL,                            -- normal boiling point in K (at 101.325 kPa)

    -- Antoine equation coefficients: log10(P_mmHg) = a1 - a2 / (a3 + T_C)
    -- Note: Antoine coefficients vary by source (some use ln, some log10; some use K, some C)
    -- Store the NIST/DIPPR convention and document in notes
    antoine_a1      REAL,
    antoine_a2      REAL,
    antoine_a3      REAL,
    antoine_t_min   REAL,                            -- Antoine validity range lower bound in K
    antoine_t_max   REAL,                            -- Antoine validity range upper bound in K

    -- Ideal gas heat capacity polynomial: Cp = A + B*T + C*T^2 + D*T^3 + E*T^4
    -- Units: kJ/(kmol*K), T in K
    cp_a            REAL,
    cp_b            REAL,
    cp_c            REAL,
    cp_d            REAL,
    cp_e            REAL,                            -- nullable; some sources use 4-term polynomial

    -- Liquid molar volume parameters
    zra             REAL,                            -- modified Rackett parameter (Spencer & Danner)
    w_srk           REAL,                            -- SRK acentric factor for Thomson/COSTALD

    -- Other properties
    dipole_moment   REAL,                            -- dipole moment in Debye (for RKS-Polar)
    delta           REAL,                            -- solubility parameter in (cal/cm3)^0.5 (Scatchard-Hildebrand)
    vl_at_tb        REAL,                            -- liquid molar volume at Tb in cm3/mol

    -- Alpha function parameters (EOS-specific, optional)
    prsv_k1         REAL,                            -- Stryjek-Vera K1 parameter (PRSV 1986)

    -- Metadata
    source          TEXT,                            -- data source ("thermo", "chemsep", "manual", "nist")
    notes           TEXT,
    created_at      TEXT DEFAULT (datetime('now')),
    updated_at      TEXT DEFAULT (datetime('now'))
);

-- Binary interaction parameters kij for cubic EOS
-- kij is symmetric (kij = kji), so we store only one entry per pair with comp1_id < comp2_id
-- kij = 0 is the default when no entry exists (standard assumption in VLE)
CREATE TABLE IF NOT EXISTS kij_params (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    comp1_id        INTEGER NOT NULL REFERENCES components(id) ON DELETE CASCADE,
    comp2_id        INTEGER NOT NULL REFERENCES components(id) ON DELETE CASCADE,
    eos_model       TEXT NOT NULL,                   -- EOS variant: "PR", "RKS", "VdW", "SW", "PT", etc.
    kij             REAL NOT NULL,                   -- binary interaction parameter (dimensionless)
    temperature     REAL,                            -- K; NULL if temperature-independent
    source          TEXT,                            -- "knapp1982", "experiment", "regressed", etc.
    notes           TEXT,
    UNIQUE(comp1_id, comp2_id, eos_model, temperature),
    CHECK(comp1_id < comp2_id)                       -- enforce ordering for symmetry
);

-- Activity model binary interaction parameters
-- These are asymmetric (A12 != A21 in general)
CREATE TABLE IF NOT EXISTS activity_params (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    comp1_id        INTEGER NOT NULL REFERENCES components(id) ON DELETE CASCADE,
    comp2_id        INTEGER NOT NULL REFERENCES components(id) ON DELETE CASCADE,
    model           TEXT NOT NULL,                   -- "wilson", "van_laar", "margules", "scatchard_hildebrand"
    a12             REAL NOT NULL,                   -- parameter for 1-2 interaction
    a21             REAL NOT NULL,                   -- parameter for 2-1 interaction
    temperature     REAL,                            -- K; NULL if temperature-independent
    source          TEXT,                            -- "dechema", "orbey_sandler", "regressed", etc.
    notes           TEXT,
    UNIQUE(comp1_id, comp2_id, model, temperature)
);

-- Experimental VLE data for kij/Aij regression validation
-- Stores P-x-y or T-x-y data points from literature
CREATE TABLE IF NOT EXISTS experimental_vle (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    system_name     TEXT NOT NULL,                   -- e.g., "CO2/n-butane"
    comp1_id        INTEGER NOT NULL REFERENCES components(id) ON DELETE CASCADE,
    comp2_id        INTEGER NOT NULL REFERENCES components(id) ON DELETE CASCADE,
    temperature     REAL,                            -- K (constant for isothermal experiments)
    pressure        REAL,                            -- kPa (constant for isobaric experiments)
    x1              REAL NOT NULL,                   -- liquid mole fraction of component 1
    y1              REAL NOT NULL,                   -- vapor mole fraction of component 1
    source          TEXT,                            -- e.g., "dasilva1989_table411"
    notes           TEXT,
    -- Idempotent seeding: re-running the Chapter IV seed file must not
    -- duplicate data points. Pair with INSERT OR IGNORE in seed scripts.
    UNIQUE(system_name, temperature, pressure, x1, y1)
);

-- Indexes for common lookups
CREATE INDEX IF NOT EXISTS idx_components_name ON components(name);
CREATE INDEX IF NOT EXISTS idx_components_cas ON components(cas_number);
CREATE INDEX IF NOT EXISTS idx_components_formula ON components(formula);
CREATE INDEX IF NOT EXISTS idx_kij_pair ON kij_params(comp1_id, comp2_id);
CREATE INDEX IF NOT EXISTS idx_activity_pair ON activity_params(comp1_id, comp2_id);
CREATE INDEX IF NOT EXISTS idx_experimental_system ON experimental_vle(system_name);

-- Trigger to update updated_at on component modification
CREATE TRIGGER IF NOT EXISTS update_component_timestamp
    AFTER UPDATE ON components
    FOR EACH ROW
BEGIN
    UPDATE components SET updated_at = datetime('now') WHERE id = OLD.id;
END;
