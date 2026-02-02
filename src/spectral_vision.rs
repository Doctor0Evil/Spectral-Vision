use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::governance::{GovernanceMode, GovernanceStateV1};

/// Band-level safety metrics for one spectral-object.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BandSafetyProfile {
    /// Per-band safety scores in [0,1], higher is safer.
    pub bands: Vec<BandSafetyEntry>,
    /// Minimum safety across all bands.
    pub safety_min: f64,
    /// Mean safety across all bands.
    pub safety_mean: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BandSafetyEntry {
    /// Band index or label (e.g., 0=delta,1=theta,...).
    pub band_index: i32,
    /// Normalized mean power μ_B ∈ [0,1].
    pub mean_power: f64,
    /// Normalized stddev σ_B ∈ [0,1].
    pub stddev_power: f64,
    /// Safety score s_B ∈ [0,1].
    pub safety_score: f64,
    /// Discrete hazard class for routing/policies.
    pub hazard_class: BandHazardClass,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum BandHazardClass {
    Safe,
    Elevated,
    High,
}

/// Hygiene metrics computed from band safety and artifact metrics.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpectralHygiene {
    /// Overall bandquality ∈ [0,1]; higher is cleaner, safer.
    pub band_quality: f64,
    /// Overall artifactlevel ∈ [0,1]; higher is more contaminated.
    pub artifact_level: f64,
    /// Fraction of bands with safety_score ≥ safety_threshold.
    pub safe_band_fraction: f64,
}

/// Excavation depth mode for this spectral-object.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExcavationDepth {
    Sniff,
    DigLight,
    DigFull,
}

/// Parameters for spectral-vision math.
#[derive(Clone, Debug)]
pub struct SpectralVisionParams {
    /// Threshold separating low vs medium safety.
    pub safety_slow: f64,
    /// Threshold separating medium vs high safety.
    pub safety_shigh: f64,
    /// Hazard-class thresholds (per-band).
    pub hazard_elevated_max: f64, // s_B < elevated_max => High; else Elevated
    pub hazard_safe_min: f64,     // s_B ≥ safe_min => Safe
    /// Promotion weights (must be ≥ 0; will be renormalized).
    pub w_stability: f64,
    pub w_confidence: f64,
    pub w_safety_min: f64,
    /// Promotion gate threshold.
    pub promotion_threshold: f64,
}

impl Default for SpectralVisionParams {
    fn default() -> Self {
        Self {
            safety_slow: 0.4,
            safety_shigh: 0.7,
            hazard_elevated_max: 0.4,
            hazard_safe_min: 0.7,
            w_stability: 0.4,
            w_confidence: 0.4,
            w_safety_min: 0.2,
            promotion_threshold: 0.8,
        }
    }
}

fn clamp01(x: f64) -> f64 {
    if !x.is_finite() {
        0.0
    } else if x < 0.0 {
        0.0
    } else if x > 1.0 {
        1.0
    } else {
        x
    }
}

/// Compute band-level safety from normalized mean+stddev, plus safety_min and safety_mean.
///
/// For each band B:
///   s_B = clamp01(1 - μ_B - σ_B)
/// hazard_class derived from s_B and params.hazard_*.[file:8]
pub fn compute_band_safety(
    raw_bands: &[(i32, f64, f64)],
    params: &SpectralVisionParams,
) -> BandSafetyProfile {
    let mut entries = Vec::with_capacity(raw_bands.len());
    let mut sum = 0.0;
    let mut safety_min = 1.0;

    for (band_index, mean_power_raw, stddev_power_raw) in raw_bands.iter().copied() {
        let mu = clamp01(mean_power_raw);
        let sigma = clamp01(stddev_power_raw);
        let s_b = clamp01(1.0 - mu - sigma);

        let hazard = if s_b < params.hazard_elevated_max {
            BandHazardClass::High
        } else if s_b < params.hazard_safe_min {
            BandHazardClass::Elevated
        } else {
            BandHazardClass::Safe
        };

        sum += s_b;
        if s_b < safety_min {
            safety_min = s_b;
        }

        entries.push(BandSafetyEntry {
            band_index,
            mean_power: mu,
            stddev_power: sigma,
            safety_score: s_b,
            hazard_class: hazard,
        });
    }

    let safety_mean = if entries.is_empty() {
        0.0
    } else {
        sum / (entries.len() as f64)
    };

    BandSafetyProfile {
        bands: entries,
        safety_min: clamp01(safety_min),
        safety_mean: clamp01(safety_mean),
    }
}

/// Compute spectral hygiene and safe_band_fraction from band-safety and artifact metrics.[file:8]
pub fn compute_spectral_hygiene(
    band_profile: &BandSafetyProfile,
    artifact_epoch_fraction_raw: f64,
    safety_threshold: f64,
) -> SpectralHygiene {
    let artifact_epoch_fraction = clamp01(artifact_epoch_fraction_raw);

    // band_quality = clamp01(safety_mean * (1 - artifact_epoch_fraction))
    let band_quality = clamp01(band_profile.safety_mean * (1.0 - artifact_epoch_fraction));
    let artifact_level = clamp01(1.0 - band_quality);

    let mut safe_count = 0usize;
    for entry in &band_profile.bands {
        if entry.safety_score >= safety_threshold {
            safe_count += 1;
        }
    }
    let safe_band_fraction = if band_profile.bands.is_empty() {
        0.0
    } else {
        (safe_count as f64) / (band_profile.bands.len() as f64)
    };

    SpectralHygiene {
        band_quality,
        artifact_level,
        safe_band_fraction: clamp01(safe_band_fraction),
    }
}

/// Compute promotionscore from stability, confidence, and safety_min.[file:8]
///
/// promotionscore = w1 * stability + w2 * confidence + w3 * safety_min,
/// where (w1,w2,w3) are renormalized to sum to 1 if possible.
/// If all weights are zero, promotionscore = 0.
pub fn compute_promotion_score(
    stability_score_raw: f64,
    confidence_score_raw: f64,
    safety_min_raw: f64,
    params: &SpectralVisionParams,
) -> f64 {
    let s = clamp01(stability_score_raw);
    let c = clamp01(confidence_score_raw);
    let safety_min = clamp01(safety_min_raw);

    let w_sum = params.w_stability + params.w_confidence + params.w_safety_min;
    if w_sum <= 0.0 {
        return 0.0;
    }
    let w1 = params.w_stability / w_sum;
    let w2 = params.w_confidence / w_sum;
    let w3 = params.w_safety_min / w_sum;

    clamp01(w1 * s + w2 * c + w3 * safety_min)
}

/// Decide whether a spectral-object is eligible for catalog promotion.
pub fn passes_promotion_gate(
    promotion_score: f64,
    params: &SpectralVisionParams,
) -> bool {
    promotion_score >= params.promotion_threshold
}

/// Compute excavation depth as a function of safety and governance.[file:8][file:5]
///
/// Rules:
/// - If spectral_quantification_active or soul_modeling_forbidden are false → Sniff only (caller should AbortAndFlush upstream).
/// - If mode = ActiveFree (roaming + non-interference) → Sniff (audit-only).
/// - Else:
///     if safety_min < safety_slow  → Sniff
///     else if safety_min < safety_shigh → DigLight
///     else → DigFull
pub fn compute_excavation_depth(
    band_profile: &BandSafetyProfile,
    gov: &GovernanceStateV1,
    params: &SpectralVisionParams,
) -> ExcavationDepth {
    if !gov.spectral_quantification_active || !gov.soul_modeling_forbidden {
        return ExcavationDepth::Sniff;
    }

    match gov.mode {
        GovernanceMode::ActiveFree => {
            // Roaming + non-interference ⇒ observe-only, no deep digging.
            ExcavationDepth::Sniff
        }
        GovernanceMode::Dormant
        | GovernanceMode::ActiveGoverned
        | GovernanceMode::TechnicalOnly => {
            let s_min = band_profile.safety_min;

            if s_min < params.safety_slow {
                ExcavationDepth::Sniff
            } else if s_min < params.safety_shigh {
                ExcavationDepth::DigLight
            } else {
                ExcavationDepth::DigFull
            }
        }
    }
}

/// Helper that encapsulates the full spectral-vision decision for one object.
/// You can call this from your SpectralObject pipeline and persist results into
/// signature.extra.bandsafetyprofile, signature.extra.spectralhygiene, and
/// signature.extra.catalogpromotiongate.[file:9][file:8][file:5]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpectralVisionDecision {
    pub band_profile: BandSafetyProfile,
    pub hygiene: SpectralHygiene,
    pub excavation_depth: ExcavationDepth,
    pub promotion_score: f64,
    pub promotion_passed: bool,
}

pub fn evaluate_spectral_vision(
    raw_bands: &[(i32, f64, f64)],
    stability_score: f64,
    confidence_score: f64,
    artifact_epoch_fraction: f64,
    params: &SpectralVisionParams,
    gov: &GovernanceStateV1,
) -> SpectralVisionDecision {
    let band_profile = compute_band_safety(raw_bands, params);
    let hygiene = compute_spectral_hygiene(
        &band_profile,
        artifact_epoch_fraction,
        params.safety_shigh,
    );

    let promotion_score =
        compute_promotion_score(stability_score, confidence_score, band_profile.safety_min, params);
    let promotion_passed = passes_promotion_gate(promotion_score, params);

    let excavation_depth = compute_excavation_depth(&band_profile, gov, params);

    SpectralVisionDecision {
        band_profile,
        hygiene,
        excavation_depth,
        promotion_score,
        promotion_passed,
    }
}

// Optional: convenience ordering so you can prioritize higher-depth objects.
impl Ord for ExcavationDepth {
    fn cmp(&self, other: &Self) -> Ordering {
        use ExcavationDepth::*;
        let val = |d: ExcavationDepth| match d {
            Sniff => 0,
            DigLight => 1,
            DigFull => 2,
        };
        val(*self).cmp(&val(*other))
    }
}

impl PartialOrd for ExcavationDepth {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
