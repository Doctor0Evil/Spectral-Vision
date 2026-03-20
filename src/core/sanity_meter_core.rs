//! GhostNet SANITY-Meter Core Module
//! ==================================
//! Implements the mathematical formalization of SANITY as a time-budget
//! of safe exposure to haunt-pressure, computed from measurable emotional
//! and environmental signals.
//!
//! Hex-Stamp: 0x53414e4954595f4d455445525f5249534b5f5631
//! ALN Identity: aln18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7
//! Bostrom Identity: bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7
//! Version: 1.0.0
//! License: ALN-Sovereign-v1

#![deny(clippy::all)]
#![warn(missing_docs)]
#![forbid(unsafe_code)]

use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

/// Maximum session duration in seconds (2 hours)
pub const MAX_SESSION_SECONDS: f64 = 7200.0;

/// Maximum haunt-interval duration in seconds (12 seconds)
pub const MAX_INTERVAL_SECONDS: f64 = 12.0;

/// Minimum SANITY threshold before de-escalation
pub const SANITY_DEESCALATE_THRESHOLD: f64 = 0.15;

/// Critical SANITY threshold requiring immediate termination
pub const SANITY_CRITICAL_THRESHOLD: f64 = 0.05;

/// Haunt-Density band classifications
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum HauntDensityBand {
    /// H ∈ [0.00, 0.20) - Low ambient spectral energy
    Control = 0,
    /// H ∈ [0.20, 0.45) - Mid haunt-corridor, controlled scares allowed
    MonitoredLow = 1,
    /// H ∈ [0.45, 0.70) - Active spectral disturbances
    MonitoredHigh = 2,
    /// H ∈ [0.70, 0.85) - High-density field, significant anomalies
    Restricted = 3,
    /// H ∈ [0.85, 1.00] - Severe anomalies, containment required
    Containment = 4,
}

impl HauntDensityBand {
    /// Classify haunt-density value into band
    pub fn from_density(h: f64) -> Self {
        let h = h.clamp(0.0, 1.0);
        match h {
            h if h < 0.20 => Self::Control,
            h if h < 0.45 => Self::MonitoredLow,
            h if h < 0.70 => Self::MonitoredHigh,
            h if h < 0.85 => Self::Restricted,
            _ => Self::Containment,
        }
    }

    /// Get safe exposure threshold X_safe for this band
    pub fn safe_exposure_threshold(&self) -> f64 {
        match self {
            Self::Control => 0.10,
            Self::MonitoredLow => 0.20,
            Self::MonitoredHigh => 0.40,
            Self::Restricted => 0.40,
            Self::Containment => 0.25,
        }
    }

    /// Get base drain multiplier for this band
    pub fn base_drain_multiplier(&self) -> f64 {
        match self {
            Self::Control => 0.5,
            Self::MonitoredLow => 0.75,
            Self::MonitoredHigh => 1.0,
            Self::Restricted => 1.5,
            Self::Containment => 2.0,
        }
    }
}

/// Emotional state classifications for psych_load calculation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum EmotionalState {
    /// Calm, neutral baseline
    Calm = 0,
    /// Mild engagement or curiosity
    Engaged = 1,
    /// Elevated arousal (positive or negative)
    Elevated = 2,
    /// High distress: anger, sadness, fear
    Distressed = 3,
    /// Critical overload state
    Overloaded = 4,
}

impl EmotionalState {
    /// Map normalized emotion load to state
    pub fn from_load(e: f64) -> Self {
        let e = e.clamp(0.0, 1.0);
        match e {
            e if e < 0.25 => Self::Calm,
            e if e < 0.50 => Self::Engaged,
            e if e < 0.75 => Self::Elevated,
            e if e < 0.90 => Self::Distressed,
            _ => Self::Overloaded,
        }
    }

    /// Get weight multiplier for this state
    pub fn weight_multiplier(&self) -> f64 {
        match self {
            Self::Calm => 0.5,
            Self::Engaged => 0.75,
            Self::Elevated => 1.0,
            Self::Distressed => 1.5,
            Self::Overloaded => 2.5,
        }
    }
}

/// Composite risk index components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskComponents {
    /// Haunt-Density H ∈ [0,1]
    pub haunt_density: f64,
    /// Emotion load E ∈ [0,1]
    pub emotion_load: f64,
    /// Emotion rate ΔE/Δt
    pub emotion_rate: f64,
    /// Combined psych_load L ∈ [0,1]
    pub psych_load: f64,
}

impl RiskComponents {
    /// Validate all components are in valid ranges
    pub fn validate(&self) -> Result<(), SanityError> {
        if !(0.0..=1.0).contains(&self.haunt_density) {
            return Err(SanityError::InvalidHauntDensity(self.haunt_density));
        }
        if !(0.0..=1.0).contains(&self.emotion_load) {
            return Err(SanityError::InvalidEmotionLoad(self.emotion_load));
        }
        if !(0.0..=1.0).contains(&self.psych_load) {
            return Err(SanityError::InvalidPsychLoad(self.psych_load));
        }
        Ok(())
    }
}

/// Weight configuration for composite risk index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskWeights {
    /// Weight for haunt_density (w_H)
    pub w_h: f64,
    /// Weight for emotion_load (w_E)
    pub w_e: f64,
    /// Weight for emotion_rate (w_R)
    pub w_r: f64,
    /// Weight for psych_load (w_L)
    pub w_l: f64,
}

impl Default for RiskWeights {
    fn default() -> Self {
        Self {
            w_h: 0.35,
            w_e: 0.25,
            w_r: 0.20,
            w_l: 0.20,
        }
    }
}

impl RiskWeights {
    /// Validate weights sum to 1.0
    pub fn validate(&self) -> Result<(), SanityError> {
        let sum = self.w_h + self.w_e + self.w_r + self.w_l;
        if (sum - 1.0).abs() > 0.001 {
            return Err(SanityError::InvalidWeightSum(sum));
        }
        Ok(())
    }

    /// Normalize weights to ensure they sum to 1.0
    pub fn normalize(&mut self) {
        let sum = self.w_h + self.w_e + self.w_r + self.w_l;
        if sum > 0.0 {
            self.w_h /= sum;
            self.w_e /= sum;
            self.w_r /= sum;
            self.w_l /= sum;
        }
    }
}

/// SANITY-meter state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanityMeter {
    /// Current SANITY value S ∈ [0,1]
    pub sanity: f64,
    /// Session start time
    pub session_start: Instant,
    /// Total haunt-time accumulated this session
    pub total_haunt_time: Duration,
    /// Current haunt-interval start time
    pub interval_start: Option<Instant>,
    /// Current haunt-interval duration
    pub current_interval_duration: Duration,
    /// Risk weight configuration
    pub weights: RiskWeights,
    /// Base drain constant c_0
    pub c_0: f64,
    /// Risk drain constant c_1
    pub c_1: f64,
    /// Session ID for ledger tracking
    pub session_id: String,
    /// Region ID for zoning
    pub region_id: String,
    /// ALN identity stamp
    pub aln_stamp: String,
    /// Bostrom identity stamp
    pub bostrom_stamp: String,
    /// Hex-stamp for verification
    pub hex_stamp: String,
}

/// SANITY-meter errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum SanityError {
    #[error("Invalid haunt-density value: {0}")]
    InvalidHauntDensity(f64),
    #[error("Invalid emotion-load value: {0}")]
    InvalidEmotionLoad(f64),
    #[error("Invalid psych-load value: {0}")]
    InvalidPsychLoad(f64),
    #[error("Invalid weight sum: {0} (must be 1.0)")]
    InvalidWeightSum(f64),
    #[error("SANITY depleted: session must de-escalate")]
    SanityDepleted,
    #[error("Critical SANITY: session must terminate")]
    SanityCritical,
    #[error("Max interval duration exceeded: {0}s > {1}s")]
    MaxIntervalExceeded(f64, f64),
    #[error("Max session duration exceeded: {0}s > {1}s")]
    MaxSessionExceeded(f64, f64),
    #[error("Invalid state transition: {0}")]
    InvalidStateTransition(String),
}

/// SANITY drain calculation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrainResult {
    /// Base drain component
    pub base_drain: f64,
    /// Risk-weighted drain component
    pub risk_drain: f64,
    /// Total drain for this interval
    pub total_drain: f64,
    /// SANITY remaining after drain
    pub sanity_remaining: f64,
    /// Composite risk index X
    pub risk_index: f64,
    /// Haunt-density band
    pub haunt_band: HauntDensityBand,
    /// Emotional state
    pub emotional_state: EmotionalState,
    /// Action required
    pub action: SanityAction,
}

/// Action required based on SANITY state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum SanityAction {
    /// Continue normal operation
    Continue = 0,
    /// Issue mist-whisper warning
    MistWhisper = 1,
    /// De-escalate haunt-intensity
    Deescalate = 2,
    /// Terminate session safely
    TerminateSafe = 3,
    /// Abort and flush (soul-safe emergency)
    AbortAndFlush = 4,
}

impl SanityMeter {
    /// Create new SANITY-meter for a session
    pub fn new(session_id: String, region_id: String) -> Self {
        Self {
            sanity: 1.0,
            session_start: Instant::now(),
            total_haunt_time: Duration::ZERO,
            interval_start: None,
            current_interval_duration: Duration::ZERO,
            weights: RiskWeights::default(),
            c_0: 1.0,
            c_1: 1.0,
            session_id,
            region_id,
            aln_stamp: "aln18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7".to_string(),
            bostrom_stamp: "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7".to_string(),
            hex_stamp: "0x53414e4954595f4d455445525f5249534b5f5631".to_string(),
        }
    }

    /// Start a new haunt-interval
    pub fn start_interval(&mut self) -> Result<(), SanityError> {
        if self.interval_start.is_some() {
            return Err(SanityError::InvalidStateTransition(
                "Interval already active".to_string(),
            ));
        }
        self.interval_start = Some(Instant::now());
        self.current_interval_duration = Duration::ZERO;
        Ok(())
    }

    /// End current haunt-interval and calculate drain
    pub fn end_interval(
        &mut self,
        components: RiskComponents,
    ) -> Result<DrainResult, SanityError> {
        components.validate()?;
        self.weights.validate()?;

        let interval_duration = self
            .interval_start
            .map(|start| start.elapsed())
            .unwrap_or(Duration::ZERO);

        let interval_seconds = interval_duration.as_secs_f64();

        // Check max interval duration
        if interval_seconds > MAX_INTERVAL_SECONDS {
            return Err(SanityError::MaxIntervalExceeded(
                interval_seconds,
                MAX_INTERVAL_SECONDS,
            ));
        }

        // Calculate composite risk index X
        let risk_index = self.weights.w_h * components.haunt_density
            + self.weights.w_e * components.emotion_load
            + self.weights.w_r * components.emotion_rate.clamp(0.0, 1.0)
            + self.weights.w_l * components.psych_load;

        // Get haunt-density band
        let haunt_band = HauntDensityBand::from_density(components.haunt_density);
        let band_multiplier = haunt_band.base_drain_multiplier();

        // Get emotional state
        let emotional_state = EmotionalState::from_load(components.emotion_load);
        let emotion_multiplier = emotional_state.weight_multiplier();

        // Calculate base drain: dS_base = c_0 * (T_k / 7200)
        let base_drain = self.c_0 * band_multiplier * (interval_seconds / MAX_SESSION_SECONDS);

        // Calculate risk-weighted drain: dS_risk = c_1 * X * (T_k / 7200)
        let risk_drain =
            self.c_1 * emotion_multiplier * risk_index * (interval_seconds / MAX_SESSION_SECONDS);

        // Total drain
        let total_drain = base_drain + risk_drain;

        // Update SANITY
        self.sanity = (self.sanity - total_drain).clamp(0.0, 1.0);

        // Update total haunt-time
        self.total_haunt_time += interval_duration;
        self.current_interval_duration = interval_duration;

        // Reset interval
        self.interval_start = None;

        // Determine action required
        let action = self.determine_action(risk_index, haunt_band);

        Ok(DrainResult {
            base_drain,
            risk_drain,
            total_drain,
            sanity_remaining: self.sanity,
            risk_index,
            haunt_band,
            emotional_state,
            action,
        })
    }

    /// Determine action based on SANITY state and risk
    fn determine_action(&self, risk_index: f64, haunt_band: HauntDensityBand) -> SanityAction {
        // Critical threshold - immediate termination
        if self.sanity <= SANITY_CRITICAL_THRESHOLD {
            return SanityAction::AbortAndFlush;
        }

        // De-escalation threshold
        if self.sanity <= SANITY_DEESCALATE_THRESHOLD {
            return SanityAction::Deescalate;
        }

        // Check if risk exceeds safe threshold for this band
        let safe_threshold = haunt_band.safe_exposure_threshold();
        if risk_index > safe_threshold {
            return SanityAction::MistWhisper;
        }

        SanityAction::Continue
    }

    /// Get current session duration
    pub fn session_duration(&self) -> Duration {
        self.session_start.elapsed()
    }

    /// Check if session has exceeded max duration
    pub fn is_session_valid(&self) -> Result<(), SanityError> {
        let session_seconds = self.session_duration().as_secs_f64();
        if session_seconds > MAX_SESSION_SECONDS {
            return Err(SanityError::MaxSessionExceeded(
                session_seconds,
                MAX_SESSION_SECONDS,
            ));
        }
        Ok(())
    }

    /// Reset SANITY for new session (daily/weekly reset)
    pub fn reset_session(&mut self, new_session_id: String) {
        self.sanity = 1.0;
        self.session_start = Instant::now();
        self.total_haunt_time = Duration::ZERO;
        self.interval_start = None;
        self.current_interval_duration = Duration::ZERO;
        self.session_id = new_session_id;
    }

    /// Get SANITY percentage
    pub fn sanity_percentage(&self) -> f64 {
        self.sanity * 100.0
    }

    /// Get estimated remaining safe haunt-time in seconds
    pub fn estimated_remaining_time(&self, current_risk_index: f64) -> f64 {
        if current_risk_index <= 0.0 {
            return MAX_SESSION_SECONDS - self.total_haunt_time.as_secs_f64();
        }
        // Approximate remaining time based on current drain rate
        let drain_per_second = (self.c_0 + self.c_1 * current_risk_index) / MAX_SESSION_SECONDS;
        if drain_per_second <= 0.0 {
            return MAX_SESSION_SECONDS;
        }
        self.sanity / drain_per_second
    }

    /// Export meter state for ledger logging
    pub fn export_state(&self) -> SanityStateExport {
        SanityStateExport {
            session_id: self.session_id.clone(),
            region_id: self.region_id.clone(),
            sanity: self.sanity,
            total_haunt_time_secs: self.total_haunt_time.as_secs_f64(),
            session_duration_secs: self.session_duration().as_secs_f64(),
            aln_stamp: self.aln_stamp.clone(),
            bostrom_stamp: self.bostrom_stamp.clone(),
            hex_stamp: self.hex_stamp.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

/// Exported state for ledger logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanityStateExport {
    pub session_id: String,
    pub region_id: String,
    pub sanity: f64,
    pub total_haunt_time_secs: f64,
    pub session_duration_secs: f64,
    pub aln_stamp: String,
    pub bostrom_stamp: String,
    pub hex_stamp: String,
    pub timestamp: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanity_meter_creation() {
        let meter = SanityMeter::new("session_001".to_string(), "region_alpha".to_string());
        assert_eq!(meter.sanity, 1.0);
        assert_eq!(meter.session_id, "session_001");
        assert_eq!(meter.region_id, "region_alpha");
    }

    #[test]
    fn test_haunt_density_band_classification() {
        assert_eq!(HauntDensityBand::from_density(0.10), HauntDensityBand::Control);
        assert_eq!(HauntDensityBand::from_density(0.30), HauntDensityBand::MonitoredLow);
        assert_eq!(HauntDensityBand::from_density(0.60), HauntDensityBand::MonitoredHigh);
        assert_eq!(HauntDensityBand::from_density(0.80), HauntDensityBand::Restricted);
        assert_eq!(HauntDensityBand::from_density(0.95), HauntDensityBand::Containment);
    }

    #[test]
    fn test_risk_weights_validation() {
        let mut weights = RiskWeights::default();
        assert!(weights.validate().is_ok());

        weights.w_h = 0.5;
        assert!(weights.validate().is_err());

        weights.normalize();
        assert!(weights.validate().is_ok());
    }

    #[test]
    fn test_interval_drain_calculation() {
        let mut meter = SanityMeter::new("test".to_string(), "test".to_string());
        meter.start_interval().unwrap();

        // Simulate interval passage
        std::thread::sleep(Duration::from_millis(100));

        let components = RiskComponents {
            haunt_density: 0.30,
            emotion_load: 0.40,
            emotion_rate: 0.20,
            psych_load: 0.35,
        };

        let result = meter.end_interval(components).unwrap();
        assert!(result.total_drain > 0.0);
        assert!(result.sanity_remaining < 1.0);
        assert!(result.sanity_remaining >= 0.0);
    }
}
