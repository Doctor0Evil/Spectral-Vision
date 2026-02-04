use serde::{Deserialize, Serialize};

use crate::governance::spectral_governance_audit::SpectralGovernanceAudit;
use crate::hauntdensity::HauntDensityScore;
use crate::hauntdensity::HauntZone;
use crate::hexstamp_quantum_roaming::{
    HexStampQuantumRoamingV1, XRZone,
};
use crate::ghostnet::tokens::{PsychRiskView};
use crate::ghostnet::spectral::RegionSessionKey;

/// Nonsoul snapshot for XR routing, per region-session window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XRGridState {
    /// Coarse region / time / session key.
    pub region_session: RegionSessionKey,
    /// Haunt-density score (normalized, with zone).
    pub haunt: Option<HauntDensityScore>,
    /// Spectral-psych channels, all in 0..=1.
    pub fear_level: f32,
    pub fear_rate_norm: f32,
    pub psych_load: f32,
    /// Governance snapshot for this window.
    pub gov: SpectralGovernanceAudit,
}

/// High-level XR action band, nonsoul, routing-only.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum XRRouteAction {
    /// Full, rich interaction.
    FullInteraction,
    /// Normal content, but with extra safeguards.
    GuardedInteraction,
    /// De-escalation / mitigation-focused.
    MitigationOnly,
    /// Render-only, no steering or optimization.
    ObserveOnly,
    /// Hard stop and safe exit of the scene.
    TerminateSafe,
}

/// Clamp helper 0.0..=1.0.
fn clamp01(x: f32) -> f32 {
    if !x.is_finite() {
        0.0
    } else if x <= 0.0 {
        0.0
    } else if x >= 1.0 {
        1.0
    } else {
        x
    }
}

/// Map HauntDensity.hnorm into XRZone, mirroring HauntZone.
/// This is consistent with the ALN spec and HauntZone bands.[file:7][file:8]
pub fn compute_xrzone_from_haunt(hnorm: f32) -> XRZone {
    let h = clamp01(hnorm);
    if h < 0.2 {
        XRZone::XRCONTROL
    } else if h < 0.5 {
        XRZone::XRMONITORED
    } else if h < 0.8 {
        XRZone::XRRESTRICTED
    } else {
        XRZone::XRCONTAINMENT
    }
}

/// Simple composite risk index X in 0..=1 from H, fear, psychload,
/// aligned with GhostNet PsychRiskView bands.[file:8][file:4]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct XRGridRiskIndex {
    pub x: f32,
    pub band: String, // "NORMAL" | "MODERATE" | "HIGH"
}

pub fn compute_risk_index(
    haunt: Option<&HauntDensityScore>,
    fear_level: f32,
    psych_load: f32,
) -> XRGridRiskIndex {
    let h = haunt.map(|s| clamp01(s.hnorm as f32)).unwrap_or(0.0);
    let f = clamp01(fear_level);
    let p = clamp01(psych_load);

    // Equal weights by default; can later be learned as in HauntWeights.[file:8]
    let mut x = (h + f + p) / 3.0;
    x = clamp01(x);

    let band = if x < 0.4 {
        "NORMAL".to_string()
    } else if x < 0.7 {
        "MODERATE".to_string()
    } else {
        "HIGH".to_string()
    };

    XRGridRiskIndex { x, band }
}

/// Map XRZone + governance into a route action.
/// Non-interference + soulmodelingforbidden => ObserveOnly, regardless of risk.[file:7][file:8]
pub fn resolve_xr_route_action(
    xr_zone: XRZone,
    gov: &SpectralGovernanceAudit,
    risk: &XRGridRiskIndex,
) -> XRRouteAction {
    // Hard governance override: roaming + non-interference => observe-only.
    if gov.soulmodelingforbidden && gov.noninterferencerequired {
        return XRRouteAction::ObserveOnly;
    }

    // Optional: if risk is extreme HIGH, allow TerminateSafe in any zone.
    if risk.band == "HIGH" {
        return XRRouteAction::TerminateSafe;
    }

    match xr_zone {
        XRZone::XRCONTROL => XRRouteAction::FullInteraction,
        XRZone::XRMONITORED => XRRouteAction::GuardedInteraction,
        XRZone::XRRESTRICTED => XRRouteAction::MitigationOnly,
        XRZone::XRCONTAINMENT => XRRouteAction::ObserveOnly,
    }
}

/// Build a GhostNet-compatible PsychRiskView from our risk index,
/// so you can log it into token events.[file:4]
pub fn to_psych_risk_view(risk: &XRGridRiskIndex) -> PsychRiskView {
    PsychRiskView::from_index(risk.x)
}

/// Convenience: given XRGridState, compute xrzone, risk index,
/// route action, and optional hex-stamp for audit only.
/// The caller must respect maybestampforroaming semantics: do not
/// use the stamp to steer when non-interference is required.[file:7]
pub struct XRGridDecision {
    pub xr_zone: XRZone,
    pub risk: XRGridRiskIndex,
    pub action: XRRouteAction,
    pub hex_stamp: Option<HexStampQuantumRoamingV1>,
}

pub fn decide_xr_route_for_state(
    state: &XRGridState,
    haunt: Option<&HauntDensityScore>,
    maybe_stamp: Option<HexStampQuantumRoamingV1>,
) -> XRGridDecision {
    let hnorm = haunt.map(|s| s.hnorm as f32).unwrap_or(0.0);
    let xr_zone = compute_xrzone_from_haunt(hnorm);
    let risk = compute_risk_index(haunt, state.fear_level, state.psych_load);
    let action = resolve_xr_route_action(xr_zone, &state.gov, &risk);

    XRGridDecision {
        xr_zone,
        risk,
        action,
        hex_stamp: maybe_stamp,
    }
}
