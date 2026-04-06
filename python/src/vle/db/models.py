"""Data models for VLE component database records.

All physical quantities use canonical units as defined in CLAUDE.md:
- Temperature: **K**
- Pressure: **kPa** (absolute)
- Molar volume: **cm3/mol**
- Heat capacity: **kJ/(kmol*K)**
- Molecular weight: **g/mol**
- Dipole moment: **Debye**
- Solubility parameter: **(cal/cm3)^0.5**
"""

from dataclasses import dataclass, fields
from typing import Optional


@dataclass
class ComponentRecord:
    """A single component's thermodynamic properties in canonical units."""

    id: Optional[int] = None
    name: str = ""
    formula: Optional[str] = None
    cas_number: Optional[str] = None
    mw: Optional[float] = None  # g/mol

    # Critical properties
    tc: Optional[float] = None  # K
    pc: Optional[float] = None  # kPa (absolute)
    w: Optional[float] = None   # acentric factor (dimensionless)
    zc: Optional[float] = None  # critical compressibility (dimensionless)
    vc: Optional[float] = None  # cm3/mol

    # Boiling point
    tb: Optional[float] = None  # K

    # Antoine coefficients
    antoine_a1: Optional[float] = None
    antoine_a2: Optional[float] = None
    antoine_a3: Optional[float] = None
    antoine_t_min: Optional[float] = None  # K
    antoine_t_max: Optional[float] = None  # K

    # Heat capacity polynomial: Cp = A + B*T + C*T^2 + D*T^3 + E*T^4
    # Units: kJ/(kmol*K), T in K
    cp_a: Optional[float] = None
    cp_b: Optional[float] = None
    cp_c: Optional[float] = None
    cp_d: Optional[float] = None
    cp_e: Optional[float] = None

    # Liquid molar volume parameters
    zra: Optional[float] = None    # Rackett parameter
    w_srk: Optional[float] = None  # Thomson/COSTALD

    # Other properties
    dipole_moment: Optional[float] = None  # Debye
    delta: Optional[float] = None          # (cal/cm3)^0.5
    vl_at_tb: Optional[float] = None       # cm3/mol
    prsv_k1: Optional[float] = None        # Stryjek-Vera K1

    # Metadata
    source: Optional[str] = None
    notes: Optional[str] = None

    def has_critical_props(self) -> bool:
        """Check if minimum required critical properties are present."""
        return self.tc is not None and self.pc is not None and self.w is not None

    def summary(self) -> str:
        """One-line summary for display."""
        parts = [f"{self.name}"]
        if self.formula:
            parts.append(f"({self.formula})")
        if self.tc is not None:
            parts.append(f"Tc={self.tc:.2f} K")
        if self.pc is not None:
            parts.append(f"Pc={self.pc:.2f} kPa")
        if self.w is not None:
            parts.append(f"w={self.w:.4f}")
        return "  ".join(parts)


@dataclass
class KijRecord:
    """Binary interaction parameter for a cubic EOS."""

    id: Optional[int] = None
    comp1_id: int = 0
    comp2_id: int = 0
    eos_model: str = ""      # "PR", "RKS", etc.
    kij: float = 0.0         # dimensionless
    temperature: Optional[float] = None  # K (None if T-independent)
    source: Optional[str] = None
    notes: Optional[str] = None

    # Populated by queries for display convenience
    comp1_name: Optional[str] = None
    comp2_name: Optional[str] = None


@dataclass
class ActivityParamRecord:
    """Activity model binary interaction parameters."""

    id: Optional[int] = None
    comp1_id: int = 0
    comp2_id: int = 0
    model: str = ""          # "wilson", "van_laar", "margules"
    a12: float = 0.0
    a21: float = 0.0
    temperature: Optional[float] = None  # K
    source: Optional[str] = None
    notes: Optional[str] = None

    # Populated by queries for display convenience
    comp1_name: Optional[str] = None
    comp2_name: Optional[str] = None


@dataclass
class ExperimentalVlePoint:
    """A single experimental VLE data point."""

    id: Optional[int] = None
    system_name: str = ""
    comp1_id: int = 0
    comp2_id: int = 0
    temperature: Optional[float] = None  # K
    pressure: Optional[float] = None     # kPa (absolute)
    x1: float = 0.0
    y1: float = 0.0
    source: Optional[str] = None
    notes: Optional[str] = None
