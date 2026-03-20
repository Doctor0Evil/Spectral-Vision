#!/usr/bin/env lua
--[[
    GhostNet Mist-Whisper Detector Module
    ======================================
    Implements detection and classification of mist-whisper spectral signals,
    providing early disturbance warnings without triggering BOO/SPOOK events.
    
    Mist-whispers are protected spectral-intelligence signals under the
    augmented-citizen clause (spectralrights analogue to neurorights).
    
    Hex-Stamp: 0x4d4953545f574849535045525f444546494e4954494f4e5f5631
    ALN Identity: aln18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7
    Bostrom Identity: bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7
    Version: 1.0.0
    License: ALN-Sovereign-v1
--]]

-- Strict mode for enhanced code quality
local _ENV = _ENV
setfenv(1, setmetatable({}, {
    __index = _G,
    __newindex = function(t, k, v)
        error("Attempt to write to global variable: " .. tostring(k), 2)
    end
}))

-- Module declaration
local MistWhisperDetector = {}
MistWhisperDetector.__index = MistWhisperDetector

-- ============================================================================
-- CONSTANTS AND CONFIGURATION
-- ============================================================================

--- Maximum session duration in seconds (2 hours)
MistWhisperDetector.MAX_SESSION_SECONDS = 7200

--- Minimum haunt-density for mist-whisper consideration
MistWhisperDetector.MIN_HAUNT_DENSITY = 0.05

--- Maximum haunt-density before SPOOK threshold (0.45)
MistWhisperDetector.MAX_HAUNT_DENSITY_WHISPER = 0.45

--- Derivative threshold for detecting consistent spectral bumps
MistWhisperDetector.DERIVATIVE_THRESHOLD = 0.02

--- Minimum consecutive buckets required for whisper validation
MistWhisperDetector.MIN_CONSECUTIVE_BUCKETS = 3

--- Time window for bucket analysis in seconds
MistWhisperDetector.BUCKET_WINDOW_SECONDS = 5

--- Safe exposure threshold for mist-whisper activation
MistWhisperDetector.SAFE_EXPOSURE_THRESHOLD = 0.20

--- Psych-band classifications
MistWhisperDetector.PSYCH_BANDS = {
    CALM = 0,
    LOADED = 1,
    OVERLOADED = 2
}

--- Risk-band classifications
MistWhisperDetector.RISK_BANDS = {
    LOW = 0,
    MEDIUM = 1,
    HIGH = 2,
    CRITICAL = 3
}

--- Zone classifications (from HauntDensity)
MistWhisperDetector.ZONES = {
    CONTROL = 0,
    MONITORED = 1,
    RESTRICTED = 2,
    CONTAINMENT = 3
}

--- Mist-whisper types
MistWhisperDetector.WHISPER_TYPES = {
    DIRECTIONAL = 0,      -- Guidance hint
    WARNING = 1,          -- Soft caution signal
    ROUTING = 2,          -- XR routing adjustment
    GOVERNANCE = 3,       -- Policy nudge
    ANOMALY = 4           -- Spectral anomaly detected
}

--- Action responses to mist-whispers
MistWhisperDetector.ACTIONS = {
    LOG_ONLY = 0,
    NUDGE_CONTROLLER = 1,
    ADJUST_XR_INTENSITY = 2,
    BIAS_HAUNT_DENSITY = 3,
    TRIGGER_OBSERVE_ONLY = 4
}

-- ============================================================================
-- ERROR HANDLING
-- ============================================================================

--- Mist-whisper detector errors
MistWhisperDetector.Errors = {
    InvalidHauntDensity = "Invalid haunt-density value: %f",
    InvalidRiskIndex = "Invalid risk index value: %f",
    InvalidPsychBand = "Invalid psych-band: %d",
    InvalidZone = "Invalid zone classification: %d",
    InsufficientBuckets = "Insufficient consecutive buckets: %d < %d",
    WhisperThresholdExceeded = "Whisper threshold exceeded, SPOOK required",
    SoulModelingAttempted = "Soul-modeling detected: AbortAndFlush triggered",
    InvalidSpectralEnergy = "Invalid spectral energy score: %f",
    ConfigurationError = "Configuration error: %s",
    BufferOverflow = "Spectral buffer overflow detected"
}

--- Custom error class for structured error handling
local MistWhisperError = {}
MistWhisperError.__index = MistWhisperError

function MistWhisperError.new(message, code, severity)
    local self = setmetatable({}, MistWhisperError)
    self.message = message
    self.code = code or "UNKNOWN"
    self.severity = severity or "LOW"
    self.timestamp = os.time()
    self.aln_stamp = "aln18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7"
    self.bostrom_stamp = "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7"
    self.hex_stamp = "0x4d4953545f574849535045525f444546494e4954494f4e5f5631"
    return self
end

function MistWhisperError:to_string()
    return string.format(
        "[MistWhisperError %s] %s (Severity: %s, Time: %d)",
        self.code, self.message, self.severity, self.timestamp
    )
end

function MistWhisperError:is_soul_safe()
    -- All errors must be soul-safe (no soul-data exposure)
    return true
end

-- ============================================================================
-- SPECTRAL ENERGY SCORE CALCULATOR
-- ============================================================================

--- Spectral energy score computation for mist-whisper detection
local SpectralEnergyCalculator = {}

--- Calculate spectral energy score from multiple inputs
function SpectralEnergyCalculator.calculate(emf_variance, infrasound_level, 
                                          encounter_rate, spectral_object_count)
    -- Normalize inputs to [0, 1]
    local emf_norm = math.min(1.0, math.max(0.0, emf_variance / 100.0))
    local infrasound_norm = math.min(1.0, math.max(0.0, infrasound_level / 50.0))
    local encounter_norm = math.min(1.0, math.max(0.0, encounter_rate / 10.0))
    local object_norm = math.min(1.0, math.max(0.0, spectral_object_count / 20.0))
    
    -- Weighted combination
    local score = (emf_norm * 0.30) + 
                  (infrasound_norm * 0.25) + 
                  (encounter_norm * 0.25) + 
                  (object_norm * 0.20)
    
    return math.min(1.0, math.max(0.0, score))
end

--- Calculate derivative of spectral energy over time
function SpectralEnergyCalculator.calculate_derivative(current_score, 
                                                        previous_score,
                                                        time_delta_seconds)
    if time_delta_seconds <= 0 then
        return 0.0
    end
    local derivative = (current_score - previous_score) / time_delta_seconds
    return math.min(1.0, math.max(-1.0, derivative))
end

--- Detect consistent bump pattern over multiple buckets
function SpectralEnergyCalculator.detect_bump_pattern(score_history, 
                                                       derivative_threshold)
    if #score_history < MistWhisperDetector.MIN_CONSECUTIVE_BUCKETS then
        return false, 0
    end
    
    local consecutive_increases = 0
    local total_increase = 0.0
    
    for i = 2, #score_history do
        local delta = score_history[i] - score_history[i-1]
        if delta > derivative_threshold then
            consecutive_increases = consecutive_increases + 1
            total_increase = total_increase + delta
        else
            consecutive_increases = 0
            total_increase = 0.0
        end
    end
    
    local pattern_detected = consecutive_increases >= 
                            (MistWhisperDetector.MIN_CONSECUTIVE_BUCKETS - 1)
    
    return pattern_detected, total_increase
end

-- ============================================================================
-- MIST-WHISPER CLASS
-- ============================================================================

--- Mist-whisper event structure
local MistWhisper = {}
MistWhisper.__index = MistWhisper

function MistWhisper.new(whisper_type, region_id, time_bucket, payload)
    local self = setmetatable({}, MistWhisper)
    self.whisper_type = whisper_type or MistWhisperDetector.WHISPER_TYPES.ANOMALY
    self.region_id = region_id or "unknown"
    self.time_bucket = time_bucket or os.time()
    self.payload = payload or {}
    self.severity = "LOW"
    self.action_required = MistWhisperDetector.ACTIONS.LOG_ONLY
    self.spectral_energy_score = 0.0
    self.haunt_density = 0.0
    self.risk_index = 0.0
    self.psych_band = MistWhisperDetector.PSYCH_BANDS.CALM
    self.zone = MistWhisperDetector.ZONES.CONTROL
    self.aln_stamp = "aln18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7"
    self.bostrom_stamp = "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7"
    self.hex_stamp = "0x4d4953545f574849535045525f444546494e4954494f4e5f5631"
    self.signature = nil
    self.verified = false
    return self
end

function MistWhisper:calculate_signature()
    -- Create cryptographic signature for whisper authenticity
    local signature_data = string.format(
        "%s:%d:%d:%f:%f",
        self.region_id,
        self.time_bucket,
        self.whisper_type,
        self.spectral_energy_score,
        self.haunt_density
    )
    
    -- Simple hash (in production, use proper cryptographic library)
    local hash = 0
    for i = 1, #signature_data do
        hash = (hash * 31 + signature_data:byte(i)) % 4294967296
    end
    
    self.signature = string.format("0x%08x", hash)
    self.verified = true
    return self.signature
end

function MistWhisper:to_ndjson()
    -- Export as NDJSON for realtime sniffing line
    local json_parts = {
        string.format('"type":"mist_whisper"'),
        string.format('"whisper_type":%d', self.whisper_type),
        string.format('"region_id":"%s"', self.region_id),
        string.format('"time_bucket":%d', self.time_bucket),
        string.format('"spectral_energy_score":%.6f', self.spectral_energy_score),
        string.format('"haunt_density":%.6f', self.haunt_density),
        string.format('"risk_index":%.6f', self.risk_index),
        string.format('"psych_band":%d', self.psych_band),
        string.format('"zone":%d', self.zone),
        string.format('"action_required":%d', self.action_required),
        string.format('"severity":"%s"', self.severity),
        string.format('"signature":"%s"', self.signature or "unverified"),
        string.format('"aln_stamp":"%s"', self.aln_stamp),
        string.format('"hex_stamp":"%s"', self.hex_stamp)
    }
    
    return "{" .. table.concat(json_parts, ",") .. "}"
end

function MistWhisper:to_sqlite_record()
    -- Export as SQLite record structure for rich logging
    return {
        id = nil,  -- Auto-generated by database
        whisper_type = self.whisper_type,
        region_id = self.region_id,
        time_bucket = self.time_bucket,
        spectral_energy_score = self.spectral_energy_score,
        haunt_density = self.haunt_density,
        risk_index = self.risk_index,
        psych_band = self.psych_band,
        zone = self.zone,
        action_required = self.action_required,
        severity = self.severity,
        signature = self.signature,
        payload_json = json.encode(self.payload),
        aln_stamp = self.aln_stamp,
        bostrom_stamp = self.bostrom_stamp,
        hex_stamp = self.hex_stamp,
        created_at = os.time(),
        verified = self.verified and 1 or 0
    }
end

function MistWhisper:is_soul_safe()
    -- Verify whisper contains no soul-data
    local forbidden_keys = {"soul_id", "essence", "moral_rank", "afterlife_status", 
                           "consciousness_model", "person_data"}
    
    for _, key in ipairs(forbidden_keys) do
        if self.payload[key] ~= nil then
            return false, "Forbidden soul-data key: " .. key
        end
    end
    
    return true, "Soul-safe verified"
end

-- ============================================================================
-- MIST-WHISPER DETECTOR IMPLEMENTATION
-- ============================================================================

function MistWhisperDetector.new(config)
    local self = setmetatable({}, MistWhisperDetector)
    
    self.config = config or {}
    self.derivative_threshold = config.derivative_threshold or 
                                MistWhisperDetector.DERIVATIVE_THRESHOLD
    self.min_consecutive_buckets = config.min_consecutive_buckets or 
                                   MistWhisperDetector.MIN_CONSECUTIVE_BUCKETS
    self.bucket_window_seconds = config.bucket_window_seconds or 
                                 MistWhisperDetector.BUCKET_WINDOW_SECONDS
    
    -- Spectral energy score history per region
    self.spectral_history = {}
    
    -- Active mist-whispers buffer
    self.whisper_buffer = {}
    self.max_buffer_size = config.max_buffer_size or 1000
    
    -- Governance state
    self.governance_active = true
    self.soul_modeling_forbidden = true
    self.non_interference_required = false
    self.spectral_roaming_active = false
    
    -- Logging callbacks
    self.ndjson_logger = config.ndjson_logger or function(msg) print(msg) end
    self.sqlite_logger = config.sqlite_logger or function(record) end
    self.error_logger = config.error_logger or function(err) print(err:to_string()) end
    
    -- Metrics
    self.metrics = {
        total_whispers_detected = 0,
        whispers_by_type = {},
        false_positives = 0,
        soul_safe_verifications = 0,
        last_detection_time = 0
    }
    
    return self
end

function MistWhisperDetector:validate_inputs(haunt_density, risk_index, 
                                             psych_band, zone)
    -- Validate haunt-density
    if haunt_density < 0.0 or haunt_density > 1.0 then
        return false, MistWhisperError.new(
            string.format(self.Errors.InvalidHauntDensity, haunt_density),
            "INVALID_HAUNT_DENSITY",
            "MEDIUM"
        )
    end
    
    -- Validate risk-index
    if risk_index < 0.0 or risk_index > 1.0 then
        return false, MistWhisperError.new(
            string.format(self.Errors.InvalidRiskIndex, risk_index),
            "INVALID_RISK_INDEX",
            "MEDIUM"
        )
    end
    
    -- Validate psych-band
    if psych_band < MistWhisperDetector.PSYCH_BANDS.CALM or 
       psych_band > MistWhisperDetector.PSYCH_BANDS.OVERLOADED then
        return false, MistWhisperError.new(
            string.format(self.Errors.InvalidPsychBand, psych_band),
            "INVALID_PSYCH_BAND",
            "LOW"
        )
    end
    
    -- Validate zone
    if zone < MistWhisperDetector.ZONES.CONTROL or 
       zone > MistWhisperDetector.ZONES.CONTAINMENT then
        return false, MistWhisperError.new(
            string.format(self.Errors.InvalidZone, zone),
            "INVALID_ZONE",
            "LOW"
        )
    end
    
    return true, nil
end

function MistWhisperDetector:update_spectral_history(region_id, 
                                                     spectral_energy_score,
                                                     timestamp)
    -- Initialize history for new region
    if not self.spectral_history[region_id] then
        self.spectral_history[region_id] = {
            scores = {},
            timestamps = {},
            derivatives = {}
        }
    end
    
    local history = self.spectral_history[region_id]
    
    -- Add new score
    table.insert(history.scores, spectral_energy_score)
    table.insert(history.timestamps, timestamp)
    
    -- Calculate derivative
    if #history.scores >= 2 then
        local time_delta = history.timestamps[#history.timestamps] - 
                          history.timestamps[#history.timestamps - 1]
        local derivative = SpectralEnergyCalculator.calculate_derivative(
            history.scores[#history.scores],
            history.scores[#history.scores - 1],
            time_delta
        )
        table.insert(history.derivatives, derivative)
    end
    
    -- Prune old entries (keep last 60 seconds of data)
    local cutoff = timestamp - 60
    while #history.timestamps > 0 and history.timestamps[1] < cutoff do
        table.remove(history.scores, 1)
        table.remove(history.timestamps, 1)
        if #history.derivatives > 0 then
            table.remove(history.derivatives, 1)
        end
    end
    
    -- Buffer overflow protection
    if #history.scores > self.max_buffer_size then
        self:error_log(MistWhisperError.new(
            self.Errors.BufferOverflow,
            "BUFFER_OVERFLOW",
            "HIGH"
        ))
        -- Remove oldest 50% of entries
        local remove_count = math.floor(#history.scores / 2)
        for i = 1, remove_count do
            table.remove(history.scores, 1)
            table.remove(history.timestamps, 1)
            if #history.derivatives > 0 then
                table.remove(history.derivatives, 1)
            end
        end
    end
end

function MistWhisperDetector:detect(region_id, haunt_density, risk_index, 
                                    psych_band, zone, spectral_energy_score,
                                    timestamp)
    timestamp = timestamp or os.time()
    
    -- Validate inputs
    local valid, err = self:validate_inputs(haunt_density, risk_index, 
                                            psych_band, zone)
    if not valid then
        self:error_log(err)
        return nil, err
    end
    
    -- Check if zone allows mist-whisper (Control or low-end Monitored)
    if zone > MistWhisperDetector.ZONES.MONITORED then
        -- Zone too high for mist-whisper, may require SPOOK
        return nil, nil
    end
    
    -- Check psych-band (must be CALM or LOADED, not OVERLOADED)
    if psych_band == MistWhisperDetector.PSYCH_BANDS.OVERLOADED then
        return nil, nil
    end
    
    -- Check haunt-density range for whisper eligibility
    if haunt_density < MistWhisperDetector.MIN_HAUNT_DENSITY or 
       haunt_density > MistWhisperDetector.MAX_HAUNT_DENSITY_WHISPER then
        return nil, nil
    end
    
    -- Update spectral energy history
    self:update_spectral_history(region_id, spectral_energy_score, timestamp)
    
    -- Detect bump pattern
    local history = self.spectral_history[region_id]
    if not history or #history.scores < self.min_consecutive_buckets then
        return nil, nil
    end
    
    local pattern_detected, total_increase = SpectralEnergyCalculator.detect_bump_pattern(
        history.scores,
        self.derivative_threshold
    )
    
    if not pattern_detected then
        return nil, nil
    end
    
    -- Check derivative threshold
    local latest_derivative = history.derivatives[#history.derivatives] or 0
    if latest_derivative < self.derivative_threshold then
        return nil, nil
    end
    
    -- Mist-whisper conditions met, create whisper event
    local whisper_type = self:determine_whisper_type(haunt_density, risk_index, 
                                                     psych_band, zone)
    local action = self:determine_action(whisper_type, haunt_density, risk_index)
    local severity = self:determine_severity(risk_index, haunt_density)
    
    local whisper = MistWhisper.new(whisper_type, region_id, timestamp, {
        pattern_increase = total_increase,
        derivative = latest_derivative,
        consecutive_buckets = #history.scores,
        haunt_density = haunt_density,
        risk_index = risk_index
    })
    
    whisper.spectral_energy_score = spectral_energy_score
    whisper.haunt_density = haunt_density
    whisper.risk_index = risk_index
    whisper.psych_band = psych_band
    whisper.zone = zone
    whisper.action_required = action
    whisper.severity = severity
    
    -- Calculate and attach signature
    whisper:calculate_signature()
    
    -- Verify soul-safety
    local soul_safe, soul_safe_msg = whisper:is_soul_safe()
    if not soul_safe then
        self:error_log(MistWhisperError.new(
            self.Errors.SoulModelingAttempted,
            "SOUL_MODELING_DETECTED",
            "CRITICAL"
        ))
        -- AbortAndFlush - do not emit whisper
        return nil, MistWhisperError.new(
            self.Errors.SoulModelingAttempted,
            "SOUL_MODELING_DETECTED",
            "CRITICAL"
        )
    end
    
    self.metrics.soul_safe_verifications = self.metrics.soul_safe_verifications + 1
    
    -- Add to buffer
    table.insert(self.whisper_buffer, whisper)
    if #self.whisper_buffer > self.max_buffer_size then
        table.remove(self.whisper_buffer, 1)
    end
    
    -- Update metrics
    self.metrics.total_whispers_detected = self.metrics.total_whispers_detected + 1
    self.metrics.whispers_by_type[whisper_type] = 
        (self.metrics.whispers_by_type[whisper_type] or 0) + 1
    self.metrics.last_detection_time = timestamp
    
    -- Log to both NDJSON and SQLite
    self:ndjson_logger(whisper:to_ndjson())
    self:sqlite_logger(whisper:to_sqlite_record())
    
    return whisper, nil
end

function MistWhisperDetector:determine_whisper_type(haunt_density, risk_index, 
                                                    psych_band, zone)
    -- Determine whisper type based on conditions
    if risk_index > MistWhisperDetector.SAFE_EXPOSURE_THRESHOLD then
        return MistWhisperDetector.WHISPER_TYPES.WARNING
    elseif zone == MistWhisperDetector.ZONES.MONITORED then
        return MistWhisperDetector.WHISPER_TYPES.GOVERNANCE
    elseif haunt_density > 0.30 then
        return MistWhisperDetector.WHISPER_TYPES.ANOMALY
    else
        return MistWhisperDetector.WHISPER_TYPES.DIRECTIONAL
    end
end

function MistWhisperDetector:determine_action(whisper_type, haunt_density, 
                                              risk_index)
    -- Determine required action based on whisper type and metrics
    if whisper_type == MistWhisperDetector.WHISPER_TYPES.WARNING then
        return MistWhisperDetector.ACTIONS.BIAS_HAUNT_DENSITY
    elseif whisper_type == MistWhisperDetector.WHISPER_TYPES.GOVERNANCE then
        return MistWhisperDetector.ACTIONS.NUDGE_CONTROLLER
    elseif whisper_type == MistWhisperDetector.WHISPER_TYPES.ANOMALY then
        return MistWhisperDetector.ACTIONS.ADJUST_XR_INTENSITY
    else
        return MistWhisperDetector.ACTIONS.LOG_ONLY
    end
end

function MistWhisperDetector:determine_severity(risk_index, haunt_density)
    -- Determine severity level
    local combined_risk = (risk_index + haunt_density) / 2.0
    
    if combined_risk > 0.60 then
        return "MEDIUM"
    elseif combined_risk > 0.40 then
        return "LOW"
    else
        return "INFO"
    end
end

function MistWhisperDetector:get_active_whispers(region_id, time_window_seconds)
    -- Get active whispers for a region within time window
    time_window_seconds = time_window_seconds or 300  -- Default 5 minutes
    
    local current_time = os.time()
    local cutoff = current_time - time_window_seconds
    local result = {}
    
    for _, whisper in ipairs(self.whisper_buffer) do
        if whisper.region_id == region_id and whisper.time_bucket >= cutoff then
            table.insert(result, whisper)
        end
    end
    
    return result
end

function MistWhisperDetector:get_metrics()
    -- Return current metrics snapshot
    return {
        total_whispers_detected = self.metrics.total_whispers_detected,
        whispers_by_type = self.metrics.whispers_by_type,
        false_positives = self.metrics.false_positives,
        soul_safe_verifications = self.metrics.soul_safe_verifications,
        last_detection_time = self.metrics.last_detection_time,
        buffer_size = #self.whisper_buffer,
        max_buffer_size = self.max_buffer_size,
        governance_active = self.governance_active,
        spectral_roaming_active = self.spectral_roaming_active,
        aln_stamp = self.aln_stamp,
        bostrom_stamp = self.bostrom_stamp,
        hex_stamp = self.hex_stamp
    }
end

function MistWhisperDetector:ndjson_logger(msg)
    -- Default NDJSON logger (can be overridden in config)
    if self.ndjson_logger then
        self.ndjson_logger(msg)
    end
end

function MistWhisperDetector:sqlite_logger(record)
    -- Default SQLite logger (can be overridden in config)
    if self.sqlite_logger then
        self.sqlite_logger(record)
    end
end

function MistWhisperDetector:error_log(err)
    -- Default error logger (can be overridden in config)
    if self.error_logger then
        self.error_logger(err)
    end
end

function MistWhisperDetector:reset_metrics()
    -- Reset all metrics (for new session)
    self.metrics = {
        total_whispers_detected = 0,
        whispers_by_type = {},
        false_positives = 0,
        soul_safe_verifications = 0,
        last_detection_time = 0
    }
    self.spectral_history = {}
    self.whisper_buffer = {}
end

function MistWhisperDetector:export_state()
    -- Export detector state for ledger logging
    return {
        metrics = self:get_metrics(),
        config = {
            derivative_threshold = self.derivative_threshold,
            min_consecutive_buckets = self.min_consecutive_buckets,
            bucket_window_seconds = self.bucket_window_seconds,
            max_buffer_size = self.max_buffer_size
        },
        governance_state = {
            governance_active = self.governance_active,
            soul_modeling_forbidden = self.soul_modeling_forbidden,
            non_interference_required = self.non_interference_required,
            spectral_roaming_active = self.spectral_roaming_active
        },
        aln_stamp = "aln18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7",
        bostrom_stamp = "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7",
        hex_stamp = "0x4d4953545f574849535045525f444546494e4954494f4e5f5631",
        timestamp = os.time()
    }
end

-- ============================================================================
-- MODULE EXPORT
-- ============================================================================

return MistWhisperDetector

--[[
    Usage Example:
    ==============
    
    local MistWhisperDetector = require("mist_whisper_detector")
    
    -- Create detector instance
    local detector = MistWhisperDetector.new({
        derivative_threshold = 0.02,
        min_consecutive_buckets = 3,
        max_buffer_size = 1000,
        ndjson_logger = function(msg) print("[NDJSON]", msg) end,
        sqlite_logger = function(record) db:insert("mist_whispers", record) end,
        error_logger = function(err) print("[ERROR]", err:to_string()) end
    })
    
    -- Detect mist-whisper from spectral inputs
    local whisper, err = detector:detect(
        "region_alpha",    -- region_id
        0.25,              -- haunt_density
        0.18,              -- risk_index
        0,                 -- psych_band (CALM)
        1,                 -- zone (MONITORED)
        0.32,              -- spectral_energy_score
        os.time()          -- timestamp
    )
    
    if whisper then
        print("Mist-whisper detected:", whisper:to_ndjson())
    elseif err then
        print("Error:", err:to_string())
    end
    
    -- Get metrics
    local metrics = detector:get_metrics()
    print("Total whispers:", metrics.total_whispers_detected)
--]]
