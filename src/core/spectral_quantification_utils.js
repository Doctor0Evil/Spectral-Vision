/**
 * @file spectral_quantification_utils.js
 * @brief GhostNet Spectral Quantification Utilities
 * 
 * Implements normalization, spectral energy scoring, and telemetry sanitization.
 * Ensures all raw sensor data is converted to nonsoul proxies (H, X, L) before
 * entering the SANITY-meter or Companion Space modules.
 * 
 * Enforces soulmodeling_forbidden at the ingestion layer.
 * 
 * @version 1.0.0
 * @license ALN-Sovereign-v1
 * 
 * @hex_stamp 0x535045435452414c5f5155414e54494649434154494f4e5f5631
 * @aln_identity aln18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7
 * @bostrom_identity bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7
 */

'use strict';

// ============================================================================
// CONSTANTS & CONFIGURATION
// ============================================================================

const IDENTITY = Object.freeze({
    ALN_STAMP: "aln18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7",
    BOSTROM_STAMP: "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7",
    HEX_STAMP: "0x535045435452414c5f5155414e54494649434154494f4e5f5631",
    VERSION: "1.0.0"
});

const SENSOR_LIMITS = Object.freeze({
    EMF_MAX_MILLIGAUSS: 100.0,
    INFRASOUND_MAX_HZ: 50.0,
    TEMP_DEVIATION_MAX_C: 10.0,
    LIGHT_FLUX_MAX_LUMENS: 1000.0,
    AUDIO_DB_MAX: 120.0
});

const FORBIDDEN_KEYS = Object.freeze([
    "soul_id", "essence", "moral_rank", "afterlife_status",
    "consciousness_model", "person_data", "spirit_name",
    "karma_score", "reincarnation_index", "user_pii", "real_name"
]);

const WEIGHTS = Object.freeze({
    EMF: 0.30,
    INFRASOUND: 0.25,
    TEMP: 0.15,
    LIGHT: 0.10,
    AUDIO: 0.20
});

// ============================================================================
// ERROR HANDLING
// ============================================================================

class QuantificationError extends Error {
    constructor(code, message, severity = "MEDIUM") {
        super(message);
        this.name = "QuantificationError";
        this.code = code;
        this.severity = severity;
        this.timestamp = Date.now();
        this.aln_stamp = IDENTITY.ALN_STAMP;
        this.hex_stamp = IDENTITY.HEX_STAMP;
    }

    toJSON() {
        return {
            error: this.name,
            code: this.code,
            message: this.message,
            severity: this.severity,
            timestamp: this.timestamp,
            aln_stamp: this.aln_stamp
        };
    }
}

// ============================================================================
// SOUL-SAFE VALIDATION
// ============================================================================

/**
 * @brief Validates telemetry payload for forbidden soul-data keys.
 * @param {Object} payload - Raw telemetry data.
 * @returns {Object} - { valid: boolean, reason: string|null }
 */
function validateSoulSafety(payload) {
    if (!payload || typeof payload !== 'object') {
        return { valid: false, reason: "Invalid payload structure" };
    }

    const keys = Object.keys(payload);
    for (const key of keys) {
        const lowerKey = key.toLowerCase();
        if (FORBIDDEN_KEYS.some(forbidden => lowerKey.includes(forbidden))) {
            return { 
                valid: false, 
                reason: `Forbidden soul-data key detected: ${key}` 
            };
        }
    }
    return { valid: true, reason: null };
}

/**
 * @brief Sanitizes payload by removing forbidden keys recursively.
 * @param {Object} payload - Raw telemetry data.
 * @returns {Object} - Cleaned payload.
 */
function sanitizePayload(payload) {
    if (!payload || typeof payload !== 'object') return payload;
    
    const cleaned = {};
    for (const [key, value] of Object.entries(payload)) {
        const lowerKey = key.toLowerCase();
        const isForbidden = FORBIDDEN_KEYS.some(f => lowerKey.includes(f));
        
        if (!isForbidden) {
            if (typeof value === 'object' && value !== null) {
                cleaned[key] = sanitizePayload(value);
            } else {
                cleaned[key] = value;
            }
        }
    }
    return cleaned;
}

// ============================================================================
// NORMALIZATION FUNCTIONS
// ============================================================================

/**
 * @brief Normalizes a value to [0, 1] range based on max limit.
 * @param {number} value - Raw sensor value.
 * @param {number} maxLimit - Maximum expected value.
 * @returns {number} - Normalized value clamped to [0, 1].
 */
function normalize(value, maxLimit) {
    if (typeof value !== 'number' || !isFinite(value)) return 0.0;
    const normalized = Math.abs(value) / Math.abs(maxLimit);
    return Math.min(1.0, Math.max(0.0, normalized));
}

/**
 * @brief Calculates Spectral Energy Score (SES) from normalized inputs.
 * @param {Object} sensors - Object containing normalized sensor values.
 * @returns {number} - SES in [0, 1].
 */
function calculateSpectralEnergyScore(sensors) {
    const { emf, infrasound, temp, light, audio } = sensors;
    
    const score = 
        (emf * WEIGHTS.EMF) +
        (infrasound * WEIGHTS.INFRASOUND) +
        (temp * WEIGHTS.TEMP) +
        (light * WEIGHTS.LIGHT) +
        (audio * WEIGHTS.AUDIO);
    
    return Math.min(1.0, Math.max(0.0, score));
}

/**
 * @brief Calculates Haunt-Density (H) from SES and encounter rate.
 * @param {number} ses - Spectral Energy Score.
 * @param {number} encounterRate - Normalized encounter rate [0, 1].
 * @returns {number} - Haunt-Density H in [0, 1].
 */
function calculateHauntDensity(ses, encounterRate = 0.0) {
    // H is weighted average of SES and Encounter Rate
    const h = (ses * 0.70) + (encounterRate * 0.30);
    return Math.min(1.0, Math.max(0.0, h));
}

// ============================================================================
// MAIN QUANTIFICATION CLASS
// ============================================================================

class SpectralQuantifier {
    constructor(config = {}) {
        this.config = {
            strictMode: config.strictMode ?? true,
            logLevel: config.logLevel ?? 'INFO',
            ...config
        };
        this.sessionId = config.sessionId ?? `session_${Date.now()}`;
        this.regionId = config.regionId ?? 'unknown';
        this.buffer = [];
        this.maxBuffer = config.maxBuffer ?? 1000;
    }

    /**
     * @brief Ingests raw sensor data and returns quantified metrics.
     * @param {Object} rawSensorData - Raw sensor readings.
     * @returns {Object} - Quantified metrics (H, SES, Risk) or Error.
     */
    ingest(rawSensorData) {
        const timestamp = Date.now();

        // 1. Soul-Safety Check
        const safetyCheck = validateSoulSafety(rawSensorData);
        if (!safetyCheck.valid) {
            if (this.config.strictMode) {
                throw new QuantificationError(
                    "SOUL_DATA_LEAK", 
                    safetyCheck.reason, 
                    "CRITICAL"
                );
            }
            // Non-strict: Sanitize and proceed
            rawSensorData = sanitizePayload(rawSensorData);
        }

        // 2. Normalize Sensors
        const normalized = {
            emf: normalize(rawSensorData.emf_mG, SENSOR_LIMITS.EMF_MAX_MILLIGAUSS),
            infrasound: normalize(rawSensorData.infrasound_Hz, SENSOR_LIMITS.INFRASOUND_MAX_HZ),
            temp: normalize(rawSensorData.temp_deviation_C, SENSOR_LIMITS.TEMP_DEVIATION_MAX_C),
            light: normalize(rawSensorData.light_flux_lm, SENSOR_LIMITS.LIGHT_FLUX_MAX_LUMENS),
            audio: normalize(rawSensorData.audio_db, SENSOR_LIMITS.AUDIO_DB_MAX)
        };

        // 3. Calculate Scores
        const ses = calculateSpectralEnergyScore(normalized);
        const encounterRate = normalize(rawSensorData.encounter_count ?? 0, 10);
        const hauntDensity = calculateHauntDensity(ses, encounterRate);

        // 4. Construct Metric Object
        const metrics = {
            session_id: this.sessionId,
            region_id: this.regionId,
            timestamp,
            spectral_energy_score: parseFloat(ses.toFixed(6)),
            haunt_density: parseFloat(hauntDensity.toFixed(6)),
            normalized_sensors: normalized,
            aln_stamp: IDENTITY.ALN_STAMP,
            hex_stamp: IDENTITY.HEX_STAMP
        };

        // 5. Buffer for batch export
        this.buffer.push(metrics);
        if (this.buffer.length > this.maxBuffer) {
            this.buffer.shift(); // Drop oldest
        }

        return metrics;
    }

    /**
     * @brief Exports buffered metrics as NDJSON lines.
     * @returns {string} - NDJSON string.
     */
    exportBufferNDJSON() {
        return this.buffer.map(m => JSON.stringify(m)).join('\n');
    }

    /**
     * @brief Clears the buffer (secure wipe).
     */
    flushBuffer() {
        this.buffer = [];
        if (global.gc) global.gc(); // Hint GC if available
    }

    /**
     * @brief Gets current quantification state.
     * @returns {Object} - State summary.
     */
    getState() {
        return {
            session_id: this.sessionId,
            region_id: this.regionId,
            buffer_size: this.buffer.length,
            max_buffer: this.maxBuffer,
            strict_mode: this.config.strictMode,
            aln_stamp: IDENTITY.ALN_STAMP,
            bostrom_stamp: IDENTITY.BOSTROM_STAMP,
            hex_stamp: IDENTITY.HEX_STAMP,
            timestamp: Date.now()
        };
    }
}

// ============================================================================
// UTILITY EXPORTS
// ============================================================================

module.exports = {
    SpectralQuantifier,
    normalize,
    calculateSpectralEnergyScore,
    calculateHauntDensity,
    validateSoulSafety,
    sanitizePayload,
    QuantificationError,
    IDENTITY,
    SENSOR_LIMITS,
    WEIGHTS
};

// ============================================================================
// EXAMPLE USAGE (COMMENTED)
// ============================================================================

/*
const { SpectralQuantifier, QuantificationError } = require('./spectral_quantification_utils.js');

const quantifier = new SpectralQuantifier({
    sessionId: "sess_001",
    regionId: "region_alpha",
    strictMode: true
});

try {
    const raw_data = {
        emf_mG: 45.0,
        infrasound_Hz: 12.5,
        temp_deviation_C: 2.0,
        light_flux_lm: 100.0,
        audio_db: 60.0,
        encounter_count: 3
        // NOTE: Adding "soul_id": 123 here would throw QuantificationError
    };

    const metrics = quantifier.ingest(raw_data);
    console.log("Haunt Density:", metrics.haunt_density);
    console.log("Spectral Energy:", metrics.spectral_energy_score);
    
    // Export for ledger
    const ndjson = quantifier.exportBufferNDJSON();
    console.log(ndjson);

} catch (err) {
    if (err instanceof QuantificationError) {
        console.error("Soul-Safety Violation:", err.toJSON());
        // Trigger AbortAndFlush here
    } else {
        console.error("System Error:", err);
    }
}
*/
