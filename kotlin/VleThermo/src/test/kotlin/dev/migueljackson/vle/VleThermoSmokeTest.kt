package dev.migueljackson.vle

// Host-JVM smoke tests through the real FFI boundary — the Kotlin analog of
// swift/VleThermo's XCTests on the macOS slice. They run on THIS machine's
// JVM (no emulator): `gradle :VleThermo:test` from kotlin/, or the gutter ▶
// in Android Studio. Prerequisite: scripts/build-android.sh has built the
// host library (build.gradle.kts points jna.library.path at target/release/).
//
// Everything imported from dev.migueljackson.vle.ffi is GENERATED code —
// if these imports are red in the IDE, run scripts/build-android.sh first.

import dev.migueljackson.vle.ffi.CubicEosKind
import dev.migueljackson.vle.ffi.LiquidSpec
import dev.migueljackson.vle.ffi.MixingRuleKind
import dev.migueljackson.vle.ffi.VaporSpec
import dev.migueljackson.vle.ffi.VleFfiException
import dev.migueljackson.vle.ffi.VleSystem
import dev.migueljackson.vle.ffi.dbAvailable
import dev.migueljackson.vle.ffi.dbComponent
import dev.migueljackson.vle.ffi.steamSatT
import dev.migueljackson.vle.ffi.version
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Assert.fail
import org.junit.Test

class VleThermoSmokeTest {

    @Test
    fun versionReturnsWorkspaceVersion() {
        // First rung of the ladder: if this returns, the whole
        // Rust → C ABI → JNA → Kotlin pipeline is alive.
        assertTrue(version().matches(Regex("""\d+\.\d+\.\d+.*""")))
    }

    @Test
    fun bundledDatabaseHasWater() {
        assertTrue(dbAvailable().any { it.equals("water", ignoreCase = true) })
        val water = dbComponent("water")
        // IAPWS critical point, canonical engine units (K, kPa absolute).
        assertEquals(647.0, water.tc, 1.0)
        assertEquals(22064.0, water.pc, 100.0)
    }

    @Test
    fun waterBoilsNearOneAtmosphere() {
        // IAPWS-IF97 region 4: Psat(373.15 K) ≈ 101.42 kPa.
        val sat = steamSatT(373.15)
        assertEquals(101.42, sat.p, 0.5)
    }

    @Test
    fun isothermalFlashHeptaneButane() {
        // Research paper Ch. IV Table 4.10 configuration: RKS both phases at
        // 300 K, 100 kPa — must split two-phase, butane enriching the vapor.
        val sys = VleSystem.fromDb(
            names = listOf("n-heptane", "n-butane"),
            vapor = VaporSpec.Cubic(eos = CubicEosKind.RKS1972),
            liquid = LiquidSpec.Cubic(eos = CubicEosKind.RKS1972),
            mixingRule = MixingRuleKind.CLASSICAL,
            options = null,
        )
        val r = sys.flashTp(t = 300.0, p = 100.0, z = listOf(0.5, 0.5))
        assertTrue(r.twoPhase)
        assertTrue(r.beta > 0.0 && r.beta < 1.0)
        assertTrue(r.k[1] > 1.0) // n-butane, the light component
        assertTrue(r.k[0] < 1.0) // n-heptane, the heavy component
    }

    @Test
    fun dimensionMismatchThrowsInvalidInput() {
        val sys = VleSystem.fromDb(
            names = listOf("n-heptane", "n-butane"),
            vapor = VaporSpec.IdealGas,
            liquid = LiquidSpec.IdealSolution,
            mixingRule = MixingRuleKind.CLASSICAL,
            options = null,
        )
        try {
            sys.flashTp(t = 300.0, p = 100.0, z = listOf(1.0))
            fail("expected VleFfiException.InvalidInput")
        } catch (_: VleFfiException.InvalidInput) {
            // expected — a wrong-length feed is rejected at the boundary
        }
    }
}
