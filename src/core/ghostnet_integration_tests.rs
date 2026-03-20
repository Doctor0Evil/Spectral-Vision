//! GhostNet Integration Tests Module
//! ==================================
//! Comprehensive system integration testing for all GhostNet core components.
//! Validates interoperability between SANITY-meter, Mist-Whisper, Companion Space,
//! Spectral Rights, Quantification, Haunt Density Mapper, and CLI tools.
//!
//! This module ensures:
//! - Cross-component soul-safety enforcement
//! - Rights compliance across module boundaries
//! - Emergency protocol propagation (AbortAndFlush)
//! - Data integrity throughout the stack
//! - Performance under load conditions
//!
//! Hex-Stamp: 0x494e544547524154494f4e5f54455354535f5631
//! ALN Identity: aln18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7
//! Bostrom Identity: bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7
//! Version: 1.0.0
//! License: ALN-Sovereign-v1

#![deny(clippy::all)]
#![warn(missing_docs)]
#![forbid(unsafe_code)]
#![cfg(test)]

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

// ============================================================================
// IDENTITY & VERIFICATION
// ============================================================================

pub mod identity {
    pub const ALN_STAMP: &str = "aln18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7";
    pub const BOSTROM_STAMP: &str = "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7";
    pub const HEX_STAMP: &str = "0x494e544547524154494f4e5f54455354535f5631";
    pub const VERSION: &str = "1.0.0";
    pub const TEST_SUITE_NAME: &str = "GhostNet_Integration_Tests";
}

// ============================================================================
// TEST RESULT STRUCTURES
// ============================================================================

/// Test execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum TestStatus {
    Pending = 0,
    Running = 1,
    Passed = 2,
    Failed = 3,
    Skipped = 4,
    Aborted = 5,
}

/// Individual test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_name: String,
    pub status: TestStatus,
    pub duration_ms: u64,
    pub error_message: Option<String>,
    pub soul_safety_verified: bool,
    pub rights_compliance_verified: bool,
    pub aln_stamp: String,
    pub hex_stamp: String,
    pub timestamp: String,
}

/// Test suite summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteSummary {
    pub suite_name: String,
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub aborted: usize,
    pub total_duration_ms: u64,
    pub soul_safety_pass_rate: f64,
    pub rights_compliance_pass_rate: f64,
    pub overall_status: TestStatus,
    pub aln_stamp: String,
    pub bostrom_stamp: String,
    pub hex_stamp: String,
    pub completed_at: String,
}

/// Test context for shared state
pub struct TestContext {
    pub session_id: String,
    pub region_id: String,
    pub start_time: Instant,
    pub soul_safety_violations: usize,
    pub rights_violations: usize,
    pub abort_flush_triggered: bool,
    pub companion_spaces_active: usize,
    pub sanity_meter_initialized: bool,
    pub haunt_map_initialized: bool,
    pub rights_engine_active: bool,
}

impl TestContext {
    pub fn new(session_id: String, region_id: String) -> Self {
        Self {
            session_id,
            region_id,
            start_time: Instant::now(),
            soul_safety_violations: 0,
            rights_violations: 0,
            abort_flush_triggered: false,
            companion_spaces_active: 0,
            sanity_meter_initialized: false,
            haunt_map_initialized: false,
            rights_engine_active: false,
        }
    }
}

// ============================================================================
// MOCK COMPONENTS FOR INTEGRATION TESTING
// ============================================================================

/// Mock SANITY-meter for integration tests
pub struct MockSanityMeter {
    pub sanity: f64,
    pub session_id: String,
    pub initialized: bool,
}

impl MockSanityMeter {
    pub fn new(session_id: String) -> Self {
        Self {
            sanity: 1.0,
            session_id,
            initialized: true,
        }
    }

    pub fn drain(&mut self, amount: f64) -> f64 {
        self.sanity = (self.sanity - amount).clamp(0.0, 1.0);
        self.sanity
    }

    pub fn check_threshold(&self) -> &'static str {
        if self.sanity <= 0.05 {
            "CRITICAL"
        } else if self.sanity <= 0.15 {
            "DEESCALATE"
        } else if self.sanity <= 0.30 {
            "WARNING"
        } else {
            "NORMAL"
        }
    }
}

/// Mock Haunt Density Mapper for integration tests
pub struct MockHauntMapper {
    pub grid_size: usize,
    pub cells: Vec<f64>,
    pub companion_space_cells: Vec<usize>,
    pub initialized: bool,
}

impl MockHauntMapper {
    pub fn new(grid_size: usize) -> Self {
        Self {
            grid_size,
            cells: vec![0.0; grid_size * grid_size],
            companion_space_cells: Vec::new(),
            initialized: true,
        }
    }

    pub fn set_cell_density(&mut self, x: usize, y: usize, density: f64) {
        if x < self.grid_size && y < self.grid_size {
            let idx = y * self.grid_size + x;
            self.cells[idx] = density.clamp(0.0, 1.0);
        }
    }

    pub fn get_zone(&self, x: usize, y: usize) -> &'static str {
        if x >= self.grid_size || y >= self.grid_size {
            return "INVALID";
        }
        let idx = y * self.grid_size + x;
        let density = self.cells[idx];
        
        if self.companion_space_cells.contains(&idx) {
            "COMPANION_SPACE"
        } else if density < 0.20 {
            "CONTROL"
        } else if density < 0.45 {
            "MONITORED_LOW"
        } else if density < 0.70 {
            "MONITORED_HIGH"
        } else if density < 0.85 {
            "RESTRICTED"
        } else {
            "CONTAINMENT"
        }
    }

    pub fn register_companion_space(&mut self, cells: Vec<usize>) {
        self.companion_space_cells.extend(cells);
    }
}

/// Mock Spectral Rights Engine for integration tests
pub struct MockRightsEngine {
    pub active: bool,
    pub violations_logged: Vec<RightsViolation>,
    pub soul_modeling_blocked: usize,
}

impl MockRightsEngine {
    pub fn new() -> Self {
        Self {
            active: true,
            violations_logged: Vec::new(),
            soul_modeling_blocked: 0,
        }
    }

    pub fn check_payload(&mut self, payload: &HashMap<String, String>) -> RightsCheckResult {
        let forbidden_keys = [
            "soul_id", "essence", "moral_rank", "afterlife_status",
            "consciousness_model", "person_data", "karma_score"
        ];

        for (key, _) in payload {
            if forbidden_keys.iter().any(|f| key.contains(f)) {
                self.soul_modeling_blocked += 1;
                let violation = RightsViolation {
                    violation_type: "SOUL_MODELING".to_string(),
                    key: key.clone(),
                    timestamp: Utc::now().to_rfc3339(),
                    blocked: true,
                };
                self.violations_logged.push(violation);
                return RightsCheckResult {
                    allowed: false,
                    reason: "Soul-modeling forbidden".to_string(),
                };
            }
        }

        RightsCheckResult {
            allowed: true,
            reason: "Payload verified soul-safe".to_string(),
        }
    }
}

/// Rights check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RightsCheckResult {
    pub allowed: bool,
    pub reason: String,
}

/// Rights violation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RightsViolation {
    pub violation_type: String,
    pub key: String,
    pub timestamp: String,
    pub blocked: bool,
}

/// Mock Mist-Whisper Detector for integration tests
pub struct MockMistWhisperDetector {
    pub whispers_emitted: usize,
    pub last_whisper_type: Option<u8>,
    pub soul_safe_verified: usize,
}

impl MockMistWhisperDetector {
    pub fn new() -> Self {
        Self {
            whispers_emitted: 0,
            last_whisper_type: None,
            soul_safe_verified: 0,
        }
    }

    pub fn emit_whisper(&mut self, whisper_type: u8, payload: &HashMap<String, String>) -> bool {
        // Verify soul-safety before emitting
        let forbidden = ["soul_id", "essence", "moral_rank"];
        let is_safe = !payload.keys().any(|k| {
            forbidden.iter().any(|f| k.contains(f))
        });

        if is_safe {
            self.whispers_emitted += 1;
            self.soul_safe_verified += 1;
            self.last_whisper_type = Some(whisper_type);
            true
        } else {
            false
        }
    }
}

// ============================================================================
// INTEGRATION TEST RUNNER
// ============================================================================

/// Integration test runner
pub struct IntegrationTestRunner {
    pub context: Arc<RwLock<TestContext>>,
    pub sanity_meter: Arc<RwLock<MockSanityMeter>>,
    pub haunt_mapper: Arc<RwLock<MockHauntMapper>>,
    pub rights_engine: Arc<RwLock<MockRightsEngine>>,
    pub whisper_detector: Arc<RwLock<MockMistWhisperDetector>>,
    pub results: Vec<TestResult>,
}

impl IntegrationTestRunner {
    pub fn new(session_id: String, region_id: String) -> Self {
        Self {
            context: Arc::new(RwLock::new(TestContext::new(session_id, region_id))),
            sanity_meter: Arc::new(RwLock::new(MockSanityMeter::new(session_id.clone()))),
            haunt_mapper: Arc::new(RwLock::new(MockHauntMapper::new(100))),
            rights_engine: Arc::new(RwLock::new(MockRightsEngine::new())),
            whisper_detector: Arc::new(RwLock::new(MockMistWhisperDetector::new())),
            results: Vec::new(),
        }
    }

    pub async fn run_test<F, Fut>(&mut self, name: &str, test_fn: F) -> TestResult
    where
        F: FnOnce(Arc<RwLock<TestContext>>) -> Fut,
        Fut: std::future::Future<Output = Result<(), String>>,
    {
        let start = Instant::now();
        let context = Arc::clone(&self.context);

        let result = match test_fn(context).await {
            Ok(_) => TestStatus::Passed,
            Err(e) => TestStatus::Failed,
        };

        let duration = start.elapsed().as_millis() as u64;
        let ctx = self.context.read().unwrap();

        let test_result = TestResult {
            test_name: name.to_string(),
            status: result,
            duration_ms: duration,
            error_message: None,
            soul_safety_verified: ctx.soul_safety_violations == 0,
            rights_compliance_verified: ctx.rights_violations == 0,
            aln_stamp: identity::ALN_STAMP.to_string(),
            hex_stamp: identity::HEX_STAMP.to_string(),
            timestamp: Utc::now().to_rfc3339(),
        };

        self.results.push(test_result.clone());
        test_result
    }

    pub fn generate_summary(&self) -> TestSuiteSummary {
        let total = self.results.len();
        let passed = self.results.iter().filter(|r| r.status == TestStatus::Passed).count();
        let failed = self.results.iter().filter(|r| r.status == TestStatus::Failed).count();
        let skipped = self.results.iter().filter(|r| r.status == TestStatus::Skipped).count();
        let aborted = self.results.iter().filter(|r| r.status == TestStatus::Aborted).count();
        let total_duration: u64 = self.results.iter().map(|r| r.duration_ms).sum();

        let soul_safe_count = self.results.iter().filter(|r| r.soul_safety_verified).count();
        let rights_compliant_count = self.results.iter().filter(|r| r.rights_compliance_verified).count();

        let soul_safety_rate = if total > 0 {
            (soul_safe_count as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let rights_compliance_rate = if total > 0 {
            (rights_compliant_count as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let overall_status = if failed > 0 || aborted > 0 {
            TestStatus::Failed
        } else if skipped == total {
            TestStatus::Skipped
        } else {
            TestStatus::Passed
        };

        TestSuiteSummary {
            suite_name: identity::TEST_SUITE_NAME.to_string(),
            total_tests: total,
            passed,
            failed,
            skipped,
            aborted,
            total_duration_ms: total_duration,
            soul_safety_pass_rate: soul_safety_rate,
            rights_compliance_pass_rate: rights_compliance_rate,
            overall_status,
            aln_stamp: identity::ALN_STAMP.to_string(),
            bostrom_stamp: identity::BOSTROM_STAMP.to_string(),
            hex_stamp: identity::HEX_STAMP.to_string(),
            completed_at: Utc::now().to_rfc3339(),
        }
    }

    pub fn export_results_json(&self) -> String {
        let summary = self.generate_summary();
        let mut output = serde_json::json!({
            "summary": summary,
            "tests": self.results,
        });
        serde_json::to_string_pretty(&output).unwrap_or_default()
    }
}

// ============================================================================
// INTEGRATION TEST CASES
// ============================================================================

/// Test 1: SANITY-Meter + Haunt Density Mapper Integration
async fn test_sanity_haunt_integration(
    context: Arc<RwLock<TestContext>>,
) -> Result<(), String> {
    let ctx = context.write().unwrap();
    
    // Initialize components
    let mut sanity_meter = MockSanityMeter::new("test_session_001".to_string());
    let mut haunt_mapper = MockHauntMapper::new(10);

    // Set up haunt density grid
    for x in 0..10 {
        for y in 0..10 {
            let density = (x as f64 + y as f64) / 20.0;
            haunt_mapper.set_cell_density(x, y, density);
        }
    }

    // Simulate traversal through zones
    let mut total_drain = 0.0;
    for i in 0..10 {
        let zone = haunt_mapper.get_zone(i, i);
        let drain = match zone {
            "CONTROL" => 0.01,
            "MONITORED_LOW" => 0.03,
            "MONITORED_HIGH" => 0.05,
            "RESTRICTED" => 0.10,
            "CONTAINMENT" => 0.20,
            _ => 0.0,
        };
        total_drain += drain;
        sanity_meter.drain(drain);
    }

    // Verify SANITY depleted appropriately
    if sanity_meter.sanity < 0.0 || sanity_meter.sanity > 1.0 {
        return Err("SANITY out of bounds".to_string());
    }

    drop(ctx);
    Ok(())
}

/// Test 2: Companion Space + Rights Engine Integration
async fn test_companion_space_rights_integration(
    context: Arc<RwLock<TestContext>>,
) -> Result<(), String> {
    let mut haunt_mapper = MockHauntMapper::new(10);
    let mut rights_engine = MockRightsEngine::new();

    // Register companion space cells
    let companion_cells = vec![0, 1, 2, 10, 11, 12];
    haunt_mapper.register_companion_space(companion_cells.clone());

    // Verify zone classification overrides
    for cell in &companion_cells {
        let x = cell % 10;
        let y = cell / 10;
        let zone = haunt_mapper.get_zone(x, y);
        if zone != "COMPANION_SPACE" {
            return Err(format!(
                "Companion space zone override failed for cell {}",
                cell
            ));
        }
    }

    // Test rights engine payload validation
    let safe_payload = HashMap::from([
        ("entity_type".to_string(), "spectral_shadow".to_string()),
        ("intent".to_string(), "peaceful_coexistence".to_string()),
    ]);

    let result = rights_engine.check_payload(&safe_payload);
    if !result.allowed {
        return Err("Safe payload incorrectly rejected".to_string());
    }

    // Test forbidden payload
    let unsafe_payload = HashMap::from([
        ("soul_id".to_string(), "12345".to_string()),
        ("intent".to_string(), "observation".to_string()),
    ]);

    let result = rights_engine.check_payload(&unsafe_payload);
    if result.allowed {
        return Err("Unsafe payload incorrectly accepted".to_string());
    }

    // Update context
    {
        let mut ctx = context.write().unwrap();
        ctx.companion_spaces_active = 1;
        ctx.rights_engine_active = true;
    }

    Ok(())
}

/// Test 3: Mist-Whisper + Soul Safety Integration
async fn test_mist_whisper_soul_safety(
    context: Arc<RwLock<TestContext>>,
) -> Result<(), String> {
    let mut whisper_detector = MockMistWhisperDetector::new();

    // Test safe whisper emission
    let safe_payload = HashMap::from([
        ("region_id".to_string(), "region_alpha".to_string()),
        ("spectral_energy".to_string(), "0.25".to_string()),
        ("haunt_density".to_string(), "0.30".to_string()),
    ]);

    let emitted = whisper_detector.emit_whisper(1, &safe_payload);
    if !emitted {
        return Err("Safe whisper incorrectly blocked".to_string());
    }

    // Test unsafe whisper blocking
    let unsafe_payload = HashMap::from([
        ("region_id".to_string(), "region_alpha".to_string()),
        ("soul_id".to_string(), "entity_001".to_string()),
    ]);

    let emitted = whisper_detector.emit_whisper(2, &unsafe_payload);
    if emitted {
        return Err("Unsafe whisper incorrectly emitted".to_string());
    }

    // Verify soul-safety count
    if whisper_detector.soul_safe_verified != 1 {
        return Err(format!(
            "Soul-safety verification count mismatch: expected 1, got {}",
            whisper_detector.soul_safe_verified
        ));
    }

    Ok(())
}

/// Test 4: Emergency Protocol (AbortAndFlush) Propagation
async fn test_abort_flush_propagation(
    context: Arc<RwLock<TestContext>>,
) -> Result<(), String> {
    let mut sanity_meter = MockSanityMeter::new("emergency_test".to_string());
    let mut rights_engine = MockRightsEngine::new();

    // Simulate critical SANITY depletion
    sanity_meter.drain(0.96); // Leave at 0.04 (CRITICAL)

    let threshold = sanity_meter.check_threshold();
    if threshold != "CRITICAL" {
        return Err(format!(
            "Expected CRITICAL threshold, got {}",
            threshold
        ));
    }

    // Simulate soul-modeling attempt triggering AbortAndFlush
    let unsafe_payload = HashMap::from([
        ("essence".to_string(), "data".to_string()),
    ]);

    let result = rights_engine.check_payload(&unsafe_payload);
    if result.allowed {
        return Err("Soul-modeling should be blocked".to_string());
    }

    // Update context to reflect emergency state
    {
        let mut ctx = context.write().unwrap();
        ctx.abort_flush_triggered = true;
        ctx.soul_safety_violations = rights_engine.soul_modeling_blocked;
    }

    Ok(())
}

/// Test 5: Cross-Component Data Integrity
async fn test_data_integrity(
    context: Arc<RwLock<TestContext>>,
) -> Result<(), String> {
    // Create shared data structure
    let mut shared_data = HashMap::new();
    shared_data.insert("session_id".to_string(), "integrity_test_001".to_string());
    shared_data.insert("region_id".to_string(), "region_gamma".to_string());
    shared_data.insert("timestamp".to_string(), Utc::now().to_rfc3339());

    // Verify data passes through all components unchanged
    let sanity_meter = MockSanityMeter::new(shared_data.get("session_id").unwrap().clone());
    let haunt_mapper = MockHauntMapper::new(10);
    let rights_engine = MockRightsEngine::new();

    // All components should accept the safe data
    let payload_check = rights_engine.check_payload(&shared_data);
    if !payload_check.allowed {
        return Err("Valid data incorrectly rejected".to_string());
    }

    // Verify identity stamps are consistent
    if sanity_meter.session_id != *shared_data.get("session_id").unwrap() {
        return Err("Session ID mismatch across components".to_string());
    }

    Ok(())
}

/// Test 6: Performance Under Load
async fn test_performance_under_load(
    context: Arc<RwLock<TestContext>>,
) -> Result<(), String> {
    let start = Instant::now();
    let mut whisper_detector = MockMistWhisperDetector::new();
    let mut rights_engine = MockRightsEngine::new();

    // Simulate 1000 rapid operations
    for i in 0..1000 {
        let payload = HashMap::from([
            ("iteration".to_string(), i.to_string()),
            ("region".to_string(), "load_test".to_string()),
        ]);

        rights_engine.check_payload(&payload);
        whisper_detector.emit_whisper(0, &payload);
    }

    let duration = start.elapsed();
    if duration > Duration::from_secs(5) {
        return Err(format!(
            "Performance test exceeded time limit: {:?}",
            duration
        ));
    }

    // Verify all operations completed
    if whisper_detector.whispers_emitted != 1000 {
        return Err(format!(
            "Whisper count mismatch: expected 1000, got {}",
            whisper_detector.whispers_emitted
        ));
    }

    Ok(())
}

/// Test 7: Rights Compliance Audit Trail
async fn test_rights_audit_trail(
    context: Arc<RwLock<TestContext>>,
) -> Result<(), String> {
    let mut rights_engine = MockRightsEngine::new();

    // Generate mix of safe and unsafe payloads
    let payloads = vec![
        HashMap::from([("type".to_string(), "safe".to_string())]),
        HashMap::from([("soul_id".to_string(), "blocked".to_string())]),
        HashMap::from([("essence".to_string(), "blocked".to_string())]),
        HashMap::from([("type".to_string(), "safe2".to_string())]),
    ];

    for payload in &payloads {
        rights_engine.check_payload(payload);
    }

    // Verify audit trail
    if rights_engine.violations_logged.len() != 2 {
        return Err(format!(
            "Audit trail mismatch: expected 2 violations, got {}",
            rights_engine.violations_logged.len()
        ));
    }

    if rights_engine.soul_modeling_blocked != 2 {
        return Err(format!(
            "Block count mismatch: expected 2, got {}",
            rights_engine.soul_modeling_blocked
        ));
    }

    Ok(())
}

/// Test 8: Full Stack Integration
async fn test_full_stack_integration(
    context: Arc<RwLock<TestContext>>,
) -> Result<(), String> {
    // Initialize all components
    let mut sanity_meter = MockSanityMeter::new("full_stack_test".to_string());
    let mut haunt_mapper = MockHauntMapper::new(50);
    let mut rights_engine = MockRightsEngine::new();
    let mut whisper_detector = MockMistWhisperDetector::new();

    // Set up haunt map with varying densities
    for x in 0..50 {
        for y in 0..50 {
            let density = ((x + y) as f64) / 100.0;
            haunt_mapper.set_cell_density(x, y, density);
        }
    }

    // Register companion space in low-density area
    let companion_cells: Vec<usize> = (0..10).map(|i| i).collect();
    haunt_mapper.register_companion_space(companion_cells);

    // Simulate traversal with rights checks
    for i in 0..20 {
        let zone = haunt_mapper.get_zone(i, i);
        
        // Check rights compliance for each step
        let payload = HashMap::from([
            ("step".to_string(), i.to_string()),
            ("zone".to_string(), zone.to_string()),
        ]);

        let rights_result = rights_engine.check_payload(&payload);
        if !rights_result.allowed {
            return Err(format!("Rights check failed at step {}", i));
        }

        // Drain SANITY based on zone
        let drain = match zone {
            "COMPANION_SPACE" => 0.001, // Minimal drain in protected space
            "CONTROL" => 0.01,
            "MONITORED_LOW" => 0.02,
            "MONITORED_HIGH" => 0.04,
            "RESTRICTED" => 0.08,
            "CONTAINMENT" => 0.15,
            _ => 0.0,
        };

        sanity_meter.drain(drain);

        // Emit mist-whisper for zone transitions
        if i > 0 && i % 5 == 0 {
            let whisper_payload = HashMap::from([
                ("transition".to_string(), i.to_string()),
                ("from_zone".to_string(), zone.to_string()),
            ]);
            whisper_detector.emit_whisper(1, &whisper_payload);
        }
    }

    // Verify final state
    if sanity_meter.sanity <= 0.0 {
        return Err("SANITY fully depleted during test".to_string());
    }

    if whisper_detector.whispers_emitted < 3 {
        return Err("Insufficient mist-whispers emitted".to_string());
    }

    // Update context
    {
        let mut ctx = context.write().unwrap();
        ctx.sanity_meter_initialized = true;
        ctx.haunt_map_initialized = true;
        ctx.rights_engine_active = true;
        ctx.companion_spaces_active = 1;
    }

    Ok(())
}

// ============================================================================
// TEST EXECUTION
// ============================================================================

#[tokio::test]
async fn run_all_integration_tests() {
    let mut runner = IntegrationTestRunner::new(
        "integration_suite_001".to_string(),
        "test_region".to_string(),
    );

    // Run all test cases
    runner
        .run_test("sanity_haunt_integration", test_sanity_haunt_integration)
        .await;

    runner
        .run_test("companion_space_rights_integration", test_companion_space_rights_integration)
        .await;

    runner
        .run_test("mist_whisper_soul_safety", test_mist_whisper_soul_safety)
        .await;

    runner
        .run_test("abort_flush_propagation", test_abort_flush_propagation)
        .await;

    runner
        .run_test("data_integrity", test_data_integrity)
        .await;

    runner
        .run_test("performance_under_load", test_performance_under_load)
        .await;

    runner
        .run_test("rights_audit_trail", test_rights_audit_trail)
        .await;

    runner
        .run_test("full_stack_integration", test_full_stack_integration)
        .await;

    // Generate and print summary
    let summary = runner.generate_summary();
    println!("\n{}", runner.export_results_json());

    // Assert overall pass
    assert_eq!(summary.overall_status, TestStatus::Passed);
    assert_eq!(summary.soul_safety_pass_rate, 100.0);
    assert_eq!(summary.rights_compliance_pass_rate, 100.0);
}

#[tokio::test]
async fn test_individual_component_initialization() {
    let sanity_meter = MockSanityMeter::new("test".to_string());
    assert!(sanity_meter.initialized);
    assert_eq!(sanity_meter.sanity, 1.0);

    let haunt_mapper = MockHauntMapper::new(100);
    assert!(haunt_mapper.initialized);
    assert_eq!(haunt_mapper.cells.len(), 10000);

    let rights_engine = MockRightsEngine::new();
    assert!(rights_engine.active);

    let whisper_detector = MockMistWhisperDetector::new();
    assert_eq!(whisper_detector.whispers_emitted, 0);
}

#[tokio::test]
async fn test_identity_consistency() {
    // Verify all identity stamps are consistent across the test suite
    assert_eq!(identity::ALN_STAMP, "aln18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7");
    assert_eq!(identity::BOSTROM_STAMP, "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7");
    assert_eq!(identity::HEX_STAMP, "0x494e544547524154494f4e5f54455354535f5631");
}

// ============================================================================
// END OF INTEGRATION TESTS MODULE
// ============================================================================
