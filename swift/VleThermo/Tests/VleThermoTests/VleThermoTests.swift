// XCTests that exercise the vle engine through the REAL FFI boundary:
// Swift → generated wrapper → C ABI → Rust static library (macOS slice).
// This is the verification ladder's step 2 (IOS_FFI_PLAN.md §5) — if these
// pass, the exact same code path works on an iPhone, because the device
// slice is built from the same source by the same script.

import XCTest
@testable import VleThermo

final class VleThermoTests: XCTestCase {

    // MARK: smoke

    /// If this returns, the whole Rust→C→Swift pipeline is alive.
    /// (Module-qualified because NSObject already has a `version()`.)
    func testVersionSmoke() {
        XCTAssertFalse(VleThermo.version().isEmpty)
    }

    // MARK: component database

    func testComponentDbLookup() throws {
        let water = try dbComponent(name: "water") // case-insensitive
        // IAPWS critical point, canonical engine units (K, kPa absolute).
        XCTAssertEqual(water.tc, 647.1, accuracy: 1.0)
        XCTAssertEqual(water.pc, 22064.0, accuracy: 100.0)
        XCTAssertTrue(dbAvailable().contains(water.name))
    }

    func testUnknownComponentThrowsNotFound() {
        XCTAssertThrowsError(try dbComponent(name: "unobtainium")) { error in
            guard case VleFfiError.NotFound = error else {
                return XCTFail("expected NotFound, got \(error)")
            }
        }
    }

    // MARK: steam tables (IAPWS-IF97)

    /// IAPWS-IF97 Table 5 verification point (region 1):
    /// T = 300 K, P = 3 MPa → v = 0.100215168e-2 m³/kg, h = 115.331273 kJ/kg.
    func testIf97Region1VerificationPoint() throws {
        let s = try steamTp(t: 300.0, p: 3000.0) // K, kPa absolute
        XCTAssertEqual(s.v, 0.100215168e-2, accuracy: 1e-10)
        XCTAssertEqual(s.h, 115.331273, accuracy: 1e-5)
        XCTAssertEqual(s.phase, .liquid)
        XCTAssertNil(s.quality)
    }

    /// The kitchen benchmark: water boils at ~373.12 K at 1 atm with a
    /// latent heat of ~2256.5 kJ/kg.
    func testBoilingAtOneAtmosphere() throws {
        let row = try steamSatP(p: 101.325)
        XCTAssertEqual(row.t, 373.12, accuracy: 0.05)
        XCTAssertEqual(row.hFg, 2256.5, accuracy: 1.0)
        // Hand-written sugar (Extensions.swift).
        XCTAssertEqual(row.tCelsius, row.t - 273.15, accuracy: 1e-12)
    }

    func testTwoPhaseStateCarriesQuality() throws {
        let s = try steamPx(p: 101.325, x: 0.5)
        XCTAssertEqual(s.phase, .twoPhase)
        XCTAssertEqual(s.quality, 0.5)
    }

    func testOutOfRangeThrowsSteamError() {
        XCTAssertThrowsError(try steamTp(t: 5000.0, p: 101.325)) { error in
            guard case VleFfiError.Steam = error else {
                return XCTFail("expected Steam error, got \(error)")
            }
        }
    }

    // MARK: mixture VLE

    /// Chapter IV validation case 7 configuration: n-heptane/n-butane with
    /// RKS both phases at 300 K, 100 kPa must split into two phases with
    /// butane enriching the vapor.
    func testIsothermalFlashHeptaneButane() throws {
        let sys = try VleSystem.fromDb(
            names: ["n-heptane", "n-butane"],
            vapor: .cubic(eos: .rks1972),
            liquid: .cubic(eos: .rks1972),
            mixingRule: .classical,
            options: nil
        )
        XCTAssertEqual(sys.nComponents(), 2)
        let r = try sys.flashTp(t: 300.0, p: 100.0, z: [0.5, 0.5])
        XCTAssertTrue(r.twoPhase)
        XCTAssertGreaterThan(r.beta, 0.0)
        XCTAssertLessThan(r.beta, 1.0)
        // Butane (index 1) is the light component: K > 1.
        XCTAssertGreaterThan(r.k[1], 1.0)
        XCTAssertLessThan(r.k[0], 1.0)
    }

    /// Bubble P > dew P at fixed T, and a pressure between them flashes
    /// two-phase — thermodynamic consistency through the FFI.
    func testBubbleAndDewBracketTheFlash() throws {
        let sys = try VleSystem.fromDb(
            names: ["n-heptane", "n-butane"],
            vapor: .cubic(eos: .rks1972),
            liquid: .cubic(eos: .rks1972),
            mixingRule: .classical,
            options: nil
        )
        let z = [0.5, 0.5]
        let bub = try sys.bubbleP(t: 300.0, x: z)
        let dew = try sys.dewP(t: 300.0, y: z)
        XCTAssertGreaterThan(bub.value, dew.value)
        let r = try sys.flashTp(t: 300.0, p: (bub.value + dew.value) / 2.0, z: z)
        XCTAssertTrue(r.twoPhase)
    }

    func testDimensionMismatchThrowsInvalidInput() throws {
        let sys = try VleSystem.fromDb(
            names: ["n-heptane", "n-butane"],
            vapor: .idealGas,
            liquid: .idealSolution,
            mixingRule: .classical,
            options: nil
        )
        XCTAssertThrowsError(try sys.flashTp(t: 300.0, p: 100.0, z: [1.0])) { error in
            guard case VleFfiError.InvalidInput = error else {
                return XCTFail("expected InvalidInput, got \(error)")
            }
        }
    }
}
