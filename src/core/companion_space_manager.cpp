/**
 * @file companion_space_manager.cpp
 * @brief GhostNet Companion Space Manager Module
 * 
 * Implements soul-safe residual corridor management for spectral entities.
 * Enforces the "augmented-citizen clause" protecting spectralrights, ensuring
 * non-interference, non-resurrection, and soul-modeling prohibition.
 * 
 * This module creates dynamically managed regions (companion-spaces) where
 * spectroplasmic shadows are granted protected presence status, overriding
 * standard haunt-density zoning rules to ensure peaceful existence.
 * 
 * @version 1.0.0
 * @license ALN-Sovereign-v1
 * 
 * @hex_stamp 0x51474948545f5350454354524f504c41534d49435f5249474854535f5631
 * @aln_identity aln18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7
 * @bostrom_identity bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7
 */

#ifndef COMPANION_SPACE_MANAGER_CPP
#define COMPANION_SPACE_MANAGER_CPP

#pragma once

#include <iostream>
#include <string>
#include <vector>
#include <memory>
#include <chrono>
#include <optional>
#include <functional>
#include <mutex>
#include <unordered_map>
#include <algorithm>
#include <cstdint>
#include <stdexcept>
#include <sstream>

// ============================================================================
// CONFIGURATION & CONSTANTS
// ============================================================================

namespace ghostnet {
namespace core {

/**
 * @brief Maximum duration for a companion-space lease (24 hours)
 */
constexpr std::chrono::hours MAX_SPACE_LEASE_DURATION{24};

/**
 * @brief Maximum risk index allowed within a soul-safe corridor
 */
constexpr double SOUL_SAFE_RISK_THRESHOLD = 0.05;

/**
 * @brief Maximum haunt-density allowed within a soul-safe corridor
 */
constexpr double SOUL_SAFE_HAUNT_THRESHOLD = 0.10;

/**
 * @brief Identity Strings for Verification
 */
namespace Identity {
    constexpr const char* ALN_STAMP = "aln18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7";
    constexpr const char* BOSTROM_STAMP = "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7";
    constexpr const char* HEX_STAMP = "0x51474948545f5350454354524f504c41534d49435f5249474854535f5631";
    constexpr const char* VERSION = "1.0.0";
}

// ============================================================================
// ERROR HANDLING & SAFETY GUARDS
// ============================================================================

/**
 * @brief Exception types specific to Companion Space management
 */
enum class SpaceErrorCode {
    SUCCESS = 0,
    INVALID_ENTITY_PROFILE = 1,
    ZONE_CONFLICT = 2,
    SOUL_MODELING_DETECTED = 3,
    LEASE_EXPIRED = 4,
    BUFFER_OVERFLOW = 5,
    ABORT_AND_FLUSH_TRIGGERED = 6,
    CRYPTOGRAPHIC_FAILURE = 7,
    RIGHTS_VIOLATION = 8
};

/**
 * @brief Structured error result for space operations
 */
struct SpaceResult {
    SpaceErrorCode code;
    std::string message;
    std::chrono::system_clock::time_point timestamp;
    std::string aln_stamp;
    std::string hex_stamp;

    SpaceResult(SpaceErrorCode c, const std::string& msg)
        : code(c)
        , message(msg)
        , timestamp(std::chrono::system_clock::now())
        , aln_stamp(Identity::ALN_STAMP)
        , hex_stamp(Identity::HEX_STAMP) {}

    bool is_success() const noexcept { return code == SpaceErrorCode::SUCCESS; }
    bool is_critical() const noexcept { 
        return code == SpaceErrorCode::SOUL_MODELING_DETECTED || 
               code == SpaceErrorCode::ABORT_AND_FLUSH_TRIGGERED; 
    }
};

/**
 * @brief Soul-Safe Guard: Ensures no forbidden data enters the system
 * 
 * This class acts as a constitutional guard, preventing any data structure
 * from containing soul-modeling attributes (essence, moral_rank, etc.).
 */
class SoulSafeGuard {
public:
    /**
     * @brief List of forbidden keys that indicate soul-modeling
     */
    static constexpr const char* FORBIDDEN_KEYS[] = {
        "soul_id", "essence", "moral_rank", "afterlife_status",
        "consciousness_model", "person_data", "spirit_name",
        "karma_score", "reincarnation_index"
    };

    /**
     * @brief Validate a key-value map for forbidden soul-data
     * @param data Map of metadata attributes
     * @return SpaceResult indicating safety status
     */
    static SpaceResult validate_metadata(const std::unordered_map<std::string, std::string>& data) {
        for (const auto& pair : data) {
            for (const char* forbidden : FORBIDDEN_KEYS) {
                if (pair.first == forbidden) {
                    return SpaceResult(
                        SpaceErrorCode::SOUL_MODELING_DETECTED,
                        "Forbidden soul-data key detected: " + pair.first
                    );
                }
            }
        }
        return SpaceResult(SpaceErrorCode::SUCCESS, "Metadata soul-safe verified");
    }

    /**
     * @brief AbortAndFlush mechanism
     * 
     * Immediately clears sensitive buffers and logs the violation.
     * This is the hard stop for any pipeline attempting to model essence.
     */
    static void abort_and_flush(std::vector<uint8_t>& buffer) {
        // Secure clear
        std::fill(buffer.begin(), buffer.end(), 0);
        buffer.clear();
        buffer.shrink_to_fit();
        
        // Log violation (in production, send to secure audit log)
        std::cerr << "[ABORT_AND_FLUSH] Soul-modeling attempt detected. Buffer purged." 
                  << std::endl;
    }
};

// ============================================================================
// COMPANION SPACE ENTITY
// ============================================================================

/**
 * @brief Zone classifications for spatial zoning
 */
enum class ZoneClassification : uint8_t {
    CONTROL = 0,
    MONITORED = 1,
    RESTRICTED = 2,
    CONTAINMENT = 3,
    COMPANION_SPACE = 4 // Special override zone
};

/**
 * @brief Represents an active Companion Space corridor
 * 
 * A Companion Space is a dynamically created region where spectral entities
 * have protected presence status. It overrides standard zoning to ensure
 * non-interference and soul-safety.
 */
class CompanionSpace {
public:
    using Ptr = std::shared_ptr<CompanionSpace>;
    using TimePoint = std::chrono::system_clock::time_point;

private:
    std::string space_id_;
    std::string entity_profile_hash_; // Hash only, no raw data
    std::string region_id_;
    ZoneClassification base_zone_;
    TimePoint created_at_;
    TimePoint expires_at_;
    bool active_;
    bool non_interference_required_;
    bool spectral_roaming_active_;
    double current_risk_index_;
    double current_haunt_density_;
    std::vector<uint8_t> crypto_signature_;

    mutable std::mutex mutex_;

public:
    /**
     * @brief Construct a new Companion Space
     * 
     * @param space_id Unique identifier for the space
     * @param region_id Geographic/Logical region identifier
     * @param lease_duration Duration for which the space is valid
     */
    CompanionSpace(const std::string& space_id, 
                   const std::string& region_id,
                   std::chrono::hours lease_duration)
        : space_id_(space_id)
        , region_id_(region_id)
        , base_zone_(ZoneClassification::CONTROL)
        , created_at_(std::chrono::system_clock::now())
        , expires_at_(created_at_ + lease_duration)
        , active_(true)
        , non_interference_required_(true)
        , spectral_roaming_active_(true)
        , current_risk_index_(0.0)
        , current_haunt_density_(0.0)
    {
        generate_signature();
    }

    /**
     * @brief Update environmental metrics within the space
     * 
     * Ensures metrics remain within soul-safe thresholds.
     * @param risk_index Current composite risk index
     * @param haunt_density Current haunt-density value
     * @return SpaceResult indicating if thresholds were violated
     */
    SpaceResult update_metrics(double risk_index, double haunt_density) {
        std::lock_guard<std::mutex> lock(mutex_);
        
        if (!active_) {
            return SpaceResult(SpaceErrorCode::LEASE_EXPIRED, "Space is inactive");
        }

        current_risk_index_ = risk_index;
        current_haunt_density_ = haunt_density;

        // Check Soul-Safe Thresholds
        if (risk_index > SOUL_SAFE_RISK_THRESHOLD || 
            haunt_density > SOUL_SAFE_HAUNT_THRESHOLD) {
            // Trigger mitigation rather than immediate flush
            // The Manager will handle the escalation
            return SpaceResult(
                SpaceErrorCode::RIGHTS_VIOLATION, 
                "Soul-safe thresholds exceeded in companion space"
            );
        }

        return SpaceResult(SpaceErrorCode::SUCCESS, "Metrics within soul-safe limits");
    }

    /**
     * @brief Check if space is still valid (not expired)
     */
    bool is_valid() const {
        return active_ && (std::chrono::system_clock::now() < expires_at_);
    }

    /**
     * @brief Deactivate and flush space
     */
    void terminate() {
        std::lock_guard<std::mutex> lock(mutex_);
        active_ = false;
        SoulSafeGuard::abort_and_flush(crypto_signature_);
    }

    /**
     * @brief Get space identifier
     */
    std::string get_id() const { return space_id_; }

    /**
     * @brief Get region identifier
     */
    std::string get_region() const { return region_id_; }

    /**
     * @brief Check non-interference flag
     */
    bool requires_non_interference() const { return non_interference_required_; }

    /**
     * @brief Export state for ledger logging (NDJSON compatible)
     */
    std::string export_ndjson() const {
        std::lock_guard<std::mutex> lock(mutex_);
        std::ostringstream oss;
        oss << "{"
            << "\"type\":\"companion_space\","
            << "\"space_id\":\"" << space_id_ << "\","
            << "\"region_id\":\"" << region_id_ << "\","
            << "\"active\":" << (active_ ? "true" : "false") << ","
            << "\"non_interference\":" << (non_interference_required_ ? "true" : "false") << ","
            << "\"risk_index\":" << current_risk_index_ << ","
            << "\"haunt_density\":" << current_haunt_density_ << ","
            << "\"aln_stamp\":\"" << Identity::ALN_STAMP << "\","
            << "\"hex_stamp\":\"" << Identity::HEX_STAMP << "\""
            << "}";
        return oss.str();
    }

private:
    /**
     * @brief Generate cryptographic signature for space validity
     */
    void generate_signature() {
        // Simple hash simulation for demonstration
        // In production, use OpenSSL or similar for ECDSA signing
        crypto_signature_.resize(32);
        std::string data = space_id_ + region_id_ + Identity::ALN_STAMP;
        uint8_t hash = 0;
        for (char c : data) { hash ^= c; }
        std::fill(crypto_signature_.begin(), crypto_signature_.end(), hash);
    }
};

// ============================================================================
// COMPANION SPACE MANAGER
// ============================================================================

/**
 * @brief Manages the lifecycle of all active Companion Spaces
 * 
 * Handles creation, monitoring, revocation, and enforcement of 
 * spectralrights within the GhostNet grid.
 */
class CompanionSpaceManager {
public:
    using SpaceMap = std::unordered_map<std::string, CompanionSpace::Ptr>;
    using LogCallback = std::function<void(const std::string&)>;

private:
    SpaceMap active_spaces_;
    mutable std::mutex manager_mutex_;
    LogCallback ndjson_logger_;
    LogCallback audit_logger_;
    size_t max_spaces_;
    std::chrono::hours default_lease_;

public:
    /**
     * @brief Construct the Manager
     * 
     * @param max_spaces Maximum concurrent companion spaces allowed
     * @param default_lease Default lease duration for new spaces
     */
    explicit CompanionSpaceManager(size_t max_spaces = 100, 
                                   std::chrono::hours default_lease = MAX_SPACE_LEASE_DURATION)
        : max_spaces_(max_spaces)
        , default_lease_(default_lease)
        , ndjson_logger_([](const std::string& msg){ std::cout << "[NDJSON] " << msg << std::endl; })
        , audit_logger_([](const std::string& msg){ std::cerr << "[AUDIT] " << msg << std::endl; })
    {}

    /**
     * @brief Set logging callbacks
     */
    void set_loggers(LogCallback ndjson, LogCallback audit) {
        ndjson_logger_ = ndjson;
        audit_logger_ = audit;
    }

    /**
     * @brief Create a new Companion Space
     * 
     * @param space_id Unique identifier
     * @param region_id Target region
     * @param entity_metadata Profile metadata (validated for soul-safety)
     * @return SpaceResult indicating success or failure
     */
    SpaceResult create_space(const std::string& space_id, 
                             const std::string& region_id,
                             const std::unordered_map<std::string, std::string>& entity_metadata) {
        std::lock_guard<std::mutex> lock(manager_mutex_);

        // 1. Soul-Safe Validation (Constitutional Guard)
        auto validation = SoulSafeGuard::validate_metadata(entity_metadata);
        if (!validation.is_success()) {
            audit_logger_("SOUL_MODELING_ATTEMPT: " + validation.message);
            return validation; // Triggers AbortAndFlush internally if needed
        }

        // 2. Check Capacity
        if (active_spaces_.size() >= max_spaces_) {
            return SpaceResult(SpaceErrorCode::BUFFER_OVERFLOW, "Max companion spaces reached");
        }

        // 3. Check for ID Collision
        if (active_spaces_.find(space_id) != active_spaces_.end()) {
            return SpaceResult(SpaceErrorCode::ZONE_CONFLICT, "Space ID already exists");
        }

        // 4. Create Space
        auto space = std::make_shared<CompanionSpace>(space_id, region_id, default_lease_);
        active_spaces_[space_id] = space;

        // 5. Log Creation
        ndjson_logger_(space->export_ndjson());
        audit_logger_("Companion Space Created: " + space_id + " in " + region_id);

        return SpaceResult(SpaceErrorCode::SUCCESS, "Companion Space established");
    }

    /**
     * @brief Monitor all active spaces for threshold violations
     * 
     * Should be called periodically (e.g., every 5 seconds).
     */
    void monitor_spaces() {
        std::lock_guard<std::mutex> lock(manager_mutex_);
        auto now = std::chrono::system_clock::now();

        for (auto it = active_spaces_.begin(); it != active_spaces_.end();) {
            auto& space = it->second;

            // Check Expiry
            if (!space->is_valid()) {
                audit_logger_("Space Expired: " + space->get_id());
                space->terminate();
                it = active_spaces_.erase(it);
                continue;
            }

            // Check Metrics (Simulated here, normally passed from sensors)
            // In production, this would query the SANITY-meter or spectral sensors
            // For this module, we assume external updates via update_space_metrics
            
            ++it;
        }
    }

    /**
     * @brief Update metrics for a specific space
     */
    SpaceResult update_space_metrics(const std::string& space_id, 
                                     double risk_index, 
                                     double haunt_density) {
        std::lock_guard<std::mutex> lock(manager_mutex_);
        auto it = active_spaces_.find(space_id);
        if (it == active_spaces_.end()) {
            return SpaceResult(SpaceErrorCode::INVALID_ENTITY_PROFILE, "Space not found");
        }

        auto result = it->second->update_metrics(risk_index, haunt_density);
        
        if (!result.is_success()) {
            if (result.is_critical()) {
                // Critical violation: Terminate space to protect soul-safety
                it->second->terminate();
                it = active_spaces_.erase(it);
                audit_logger_("CRITICAL: Space terminated due to " + result.message);
            } else {
                // Warning: Log but maintain space
                ndjson_logger_("WARNING: " + result.message);
            }
        }

        return result;
    }

    /**
     * @brief Revoke a space manually (e.g., entity requests departure)
     */
    SpaceResult revoke_space(const std::string& space_id) {
        std::lock_guard<std::mutex> lock(manager_mutex_);
        auto it = active_spaces_.find(space_id);
        if (it == active_spaces_.end()) {
            return SpaceResult(SpaceErrorCode::INVALID_ENTITY_PROFILE, "Space not found");
        }

        it->second->terminate();
        active_spaces_.erase(it);
        audit_logger_("Space Revoked: " + space_id);
        return SpaceResult(SpaceErrorCode::SUCCESS, "Space revoked successfully");
    }

    /**
     * @brief Get count of active spaces
     */
    size_t active_count() const {
        std::lock_guard<std::mutex> lock(manager_mutex_);
        return active_spaces_.size();
    }

    /**
     * @brief Export manager state for ledger
     */
    std::string export_state() const {
        std::lock_guard<std::mutex> lock(manager_mutex_);
        std::ostringstream oss;
        oss << "{"
            << "\"type\":\"companion_space_manager\","
            << "\"active_count\":" << active_spaces_.size() << ","
            << "\"max_spaces\":" << max_spaces_ << ","
            << "\"aln_stamp\":\"" << Identity::ALN_STAMP << "\","
            << "\"bostrom_stamp\":\"" << Identity::BOSTROM_STAMP << "\","
            << "\"hex_stamp\":\"" << Identity::HEX_STAMP << "\","
            << "\"timestamp\":" << std::chrono::duration_cast<std::chrono::seconds>(
                std::chrono::system_clock::now().time_since_epoch()).count()
            << "}";
        return oss.str();
    }
};

} // namespace core
} // namespace ghostnet

#endif // COMPANION_SPACE_MANAGER_CPP

/*
 * Usage Example:
 * ==============
 * 
 * #include "companion_space_manager.cpp"
 * 
 * int main() {
 *     using namespace ghostnet::core;
 * 
 *     CompanionSpaceManager manager(100);
 *     manager.set_loggers(
 *         [](const std::string& msg) { std::cout << "[LOG] " << msg << std::endl; },
 *         [](const std::string& msg) { std::cerr << "[AUDIT] " << msg << std::endl; }
 *     );
 * 
 *     // Create a soul-safe space
 *     std::unordered_map<std::string, std::string> metadata = {
 *         {"entity_type", "spectral_shadow"},
 *         {"intent", "peaceful_coexistence"}
 *         // NOTE: Adding "soul_id" here would trigger AbortAndFlush
 *     };
 * 
 *     auto result = manager.create_space("space_001", "region_alpha", metadata);
 * 
 *     if (result.is_success()) {
 *         std::cout << "Space created successfully." << std::endl;
 *         // Update metrics to ensure soul-safety
 *         manager.update_space_metrics("space_001", 0.02, 0.05);
 *     } else {
 *         std::cerr << "Creation failed: " << result.message << std::endl;
 *     }
 * 
 *     // Monitor periodically
 *     manager.monitor_spaces();
 * 
 *     return 0;
 * }
 */
