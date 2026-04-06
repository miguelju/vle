"""VLE Component Property Database.

SQLite-based storage for thermodynamic component properties, binary interaction
parameters (kij), activity model parameters (Aij), and experimental VLE data.

All values are stored in canonical units:
- Temperature: **K**
- Pressure: **kPa** (absolute, never gauge)
- Molar volume: **cm3/mol**
- Molar energy: **kJ/kmol**
- Molar entropy: **kJ/(kmol*K)**
- Amount: **kmol**
- Molecular weight: **g/mol**
"""

from vle.db.connection import get_connection, init_db, get_db_path
from vle.db.models import ComponentRecord, KijRecord, ActivityParamRecord, ExperimentalVlePoint
from vle.db.queries import (
    get_component,
    get_component_by_cas,
    list_components,
    search_components,
    upsert_component,
    get_kij,
    set_kij,
    get_activity_params,
    set_activity_params,
    get_experimental_vle,
)

__all__ = [
    "get_connection",
    "init_db",
    "get_db_path",
    "ComponentRecord",
    "KijRecord",
    "ActivityParamRecord",
    "ExperimentalVlePoint",
    "get_component",
    "get_component_by_cas",
    "list_components",
    "search_components",
    "upsert_component",
    "get_kij",
    "set_kij",
    "get_activity_params",
    "set_activity_params",
    "get_experimental_vle",
]
