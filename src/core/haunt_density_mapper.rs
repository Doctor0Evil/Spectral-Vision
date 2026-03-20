//! GhostNet Haunt Density Mapper Module
//! =====================================
//! Implements spatial zoning, heatmap generation, and haunt-density mapping.
//! Ensures all spatial data is classified into safe zones (Control, Monitored, 
//! Restricted, Containment, CompanionSpace) before exposure to users.
//!
//! This module provides the spatial backbone for the SANITY-meter and 
//! Companion Space Manager, ensuring region-specific safety thresholds.
//!
//! Hex-Stamp: 0x4841554e545f44454e534954595f4d41505045525f5631
//! ALN Identity: aln18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7
//! Bostrom Identity: bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7
//! Version: 1.0.0
//! License: ALN-Sovereign-v1

#![deny(clippy::all)]
#![warn(missing_docs)]
#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use serde::{Deserialize, Serialize};
use chrono::{Utc, DateTime};

/// Identity constants for verification
pub mod identity {
    pub const ALN_STAMP: &str = "aln18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7";
    pub const BOSTROM_STAMP: &str = "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7";
    pub const HEX_STAMP: &str = "0x4841554e545f44454e534954595f4d41505045525f5631";
    pub const VERSION: &str = "1.0.0";
}

/// Grid dimensions for spatial mapping
pub const GRID_WIDTH: usize = 100;
pub const GRID_HEIGHT: usize = 100;
pub const CELL_SIZE_METERS: f64 = 1.0;

/// Zone classification thresholds
pub mod thresholds {
    pub const CONTROL_MAX: f64 = 0.20;
    pub const MONITORED_LOW_MAX: f64 = 0.45;
    pub const MONITORED_HIGH_MAX: f64 = 0.70;
    pub const RESTRICTED_MAX: f64 = 0.85;
    pub const CONTAINMENT_MIN: f64 = 0.85;
    pub const COMPANION_SPACE_RISK_MAX: f64 = 0.05;
}

/// Zone classifications for spatial zoning
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum ZoneClassification {
    /// H ∈ [0.00, 0.20) - Safe for general exposure
    Control = 0,
    /// H ∈ [0.20, 0.45) - Controlled scares allowed
    MonitoredLow = 1,
    /// H ∈ [0.45, 0.70) - Active disturbances
    MonitoredHigh = 2,
    /// H ∈ [0.70, 0.85) - High density, caution required
    Restricted = 3,
    /// H ∈ [0.85, 1.00] - Severe anomalies
    Containment = 4,
    /// Special override for companion spaces (any H, but risk limited)
    CompanionSpace = 5,
}

impl ZoneClassification {
    /// Classify haunt-density value into zone
    pub fn from_density(h: f64, is_companion_space: bool) -> Self {
        if is_companion_space {
            return Self::CompanionSpace;
        }
        let h = h.clamp(0.0, 1.0);
        if h < thresholds::CONTROL_MAX {
            Self::Control
        } else if h < thresholds::MONITORED_LOW_MAX {
            Self::MonitoredLow
        } else if h < thresholds::MONITORED_HIGH_MAX {
            Self::MonitoredHigh
        } else if h < thresholds::RESTRICTED_MAX {
            Self::Restricted
        } else {
            Self::Containment
        }
    }

    /// Get safety level description
    pub fn safety_level(&self) -> &'static str {
        match self {
            Self::Control => "SAFE",
            Self::MonitoredLow => "MODERATE",
            Self::MonitoredHigh => "ELEVATED",
            Self::Restricted => "HIGH_RISK",
            Self::Containment => "CRITICAL",
            Self::CompanionSpace => "PROTECTED",
        }
    }
}

/// Individual grid cell data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridCell {
    /// X coordinate
    pub x: usize,
    /// Y coordinate
    pub y: usize,
    /// Current haunt-density value
    pub haunt_density: f64,
    /// Spectral energy score
    pub spectral_energy: f64,
    /// Zone classification
    pub zone: ZoneClassification,
    /// Last update timestamp
    pub last_updated: DateTime<Utc>,
    /// Companion space flag
    pub is_companion_space: bool,
    /// Companion space ID (if active)
    pub companion_space_id: Option<String>,
}

impl GridCell {
    /// Create new grid cell
    pub fn new(x: usize, y: usize) -> Self {
        Self {
            x,
            y,
            haunt_density: 0.0,
            spectral_energy: 0.0,
            zone: ZoneClassification::Control,
            last_updated: Utc::now(),
            is_companion_space: false,
            companion_space_id: None,
        }
    }

    /// Update cell metrics
    pub fn update(&mut self, haunt_density: f64, spectral_energy: f64) {
        self.haunt_density = haunt_density.clamp(0.0, 1.0);
        self.spectral_energy = spectral_energy.clamp(0.0, 1.0);
        self.zone = ZoneClassification::from_density(self.haunt_density, self.is_companion_space);
        self.last_updated = Utc::now();
    }

    /// Activate companion space on this cell
    pub fn activate_companion_space(&mut self, space_id: String) {
        self.is_companion_space = true;
        self.companion_space_id = Some(space_id);
        self.zone = ZoneClassification::CompanionSpace;
        self.last_updated = Utc::now();
    }

    /// Deactivate companion space on this cell
    pub fn deactivate_companion_space(&mut self) {
        self.is_companion_space = false;
        self.companion_space_id = None;
        self.zone = ZoneClassification::from_density(self.haunt_density, false);
        self.last_updated = Utc::now();
    }
}

/// Haunt density mapper errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum MapperError {
    #[error("Coordinates out of bounds: ({0}, {1})")]
    CoordinatesOutOfBounds(usize, usize),
    #[error("Invalid haunt-density value: {0}")]
    InvalidHauntDensity(f64),
    #[error("Invalid spectral energy value: {0}")]
    InvalidSpectralEnergy(f64),
    #[error("Soul-modeling detected in cell metadata")]
    SoulModelingDetected,
    #[error("Companion space conflict: {0}")]
    CompanionSpaceConflict(String),
    #[error("Grid lock poisoned")]
    GridLockPoisoned,
}

/// Haunt density map state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HauntDensityMap {
    /// Grid cells (flattened)
    pub cells: Vec<GridCell>,
    /// Width
    pub width: usize,
    /// Height
    pub height: usize,
    /// Active companion spaces
    pub companion_spaces: HashMap<String, Vec<(usize, usize)>>,
    /// Map ID
    pub map_id: String,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// ALN stamp
    pub aln_stamp: String,
    /// Bostrom stamp
    pub bostrom_stamp: String,
    /// Hex stamp
    pub hex_stamp: String,
}

impl HauntDensityMap {
    /// Create new haunt density map
    pub fn new(map_id: String) -> Self {
        let mut cells = Vec::with_capacity(GRID_WIDTH * GRID_HEIGHT);
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                cells.push(GridCell::new(x, y));
            }
        }

        Self {
            cells,
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
            companion_spaces: HashMap::new(),
            map_id,
            created_at: Utc::now(),
            aln_stamp: identity::ALN_STAMP.to_string(),
            bostrom_stamp: identity::BOSTROM_STAMP.to_string(),
            hex_stamp: identity::HEX_STAMP.to_string(),
        }
    }

    /// Get cell at coordinates
    pub fn get_cell(&self, x: usize, y: usize) -> Result<&GridCell, MapperError> {
        if x >= self.width || y >= self.height {
            return Err(MapperError::CoordinatesOutOfBounds(x, y));
        }
        let idx = y * self.width + x;
        Ok(&self.cells[idx])
    }

    /// Get mutable cell at coordinates
    fn get_cell_mut(&mut self, x: usize, y: usize) -> Result<&mut GridCell, MapperError> {
        if x >= self.width || y >= self.height {
            return Err(MapperError::CoordinatesOutOfBounds(x, y));
        }
        let idx = y * self.width + x;
        Ok(&mut self.cells[idx])
    }

    /// Update cell metrics
    pub fn update_cell(
        &mut self,
        x: usize,
        y: usize,
        haunt_density: f64,
        spectral_energy: f64,
    ) -> Result<(), MapperError> {
        if haunt_density < 0.0 || haunt_density > 1.0 {
            return Err(MapperError::InvalidHauntDensity(haunt_density));
        }
        if spectral_energy < 0.0 || spectral_energy > 1.0 {
            return Err(MapperError::InvalidSpectralEnergy(spectral_energy));
        }

        let cell = self.get_cell_mut(x, y)?;
        cell.update(haunt_density, spectral_energy);
        Ok(())
    }

    /// Register companion space on cells
    pub fn register_companion_space(
        &mut self,
        space_id: String,
        coordinates: Vec<(usize, usize)>,
    ) -> Result<(), MapperError> {
        if self.companion_spaces.contains_key(&space_id) {
            return Err(MapperError::CompanionSpaceConflict(
                "Space ID already registered".to_string(),
            ));
        }

        let mut validated_coords = Vec::new();
        for (x, y) in &coordinates {
            // Validate coordinates
            self.get_cell(*x, *y)?;
            validated_coords.push((*x, *y));
        }

        // Activate cells
        for (x, y) in &validated_coords {
            let cell = self.get_cell_mut(*x, *y)?;
            cell.activate_companion_space(space_id.clone());
        }

        self.companion_spaces.insert(space_id, validated_coords);
        Ok(())
    }

    /// Unregister companion space
    pub fn unregister_companion_space(&mut self, space_id: &str) -> Result<(), MapperError> {
        if let Some(coords) = self.companion_spaces.remove(space_id) {
            for (x, y) in coords {
                let cell = self.get_cell_mut(x, y)?;
                cell.deactivate_companion_space();
            }
            Ok(())
        } else {
            Err(MapperError::CompanionSpaceConflict(
                "Space ID not found".to_string(),
            ))
        }
    }

    /// Get zone at coordinates
    pub fn get_zone(&self, x: usize, y: usize) -> Result<ZoneClassification, MapperError> {
        Ok(self.get_cell(x, y)?.zone)
    }

    /// Generate heatmap data (for visualization)
    pub fn generate_heatmap(&self) -> Vec<f64> {
        self.cells.iter().map(|c| c.haunt_density).collect()
    }

    /// Get statistics for the map
    pub fn get_statistics(&self) -> MapStatistics {
        let mut zone_counts = HashMap::new();
        let mut total_density = 0.0;
        let mut max_density = 0.0;

        for cell in &self.cells {
            *zone_counts.entry(cell.zone).or_insert(0) += 1;
            total_density += cell.haunt_density;
            if cell.haunt_density > max_density {
                max_density = cell.haunt_density;
            }
        }

        let avg_density = total_density / (self.cells.len() as f64);

        MapStatistics {
            total_cells: self.cells.len(),
            zone_counts,
            average_density: avg_density,
            max_density,
            active_companion_spaces: self.companion_spaces.len(),
            aln_stamp: identity::ALN_STAMP.to_string(),
            hex_stamp: identity::HEX_STAMP.to_string(),
        }
    }

    /// Export map state for ledger
    pub fn export_state(&self) -> MapExport {
        MapExport {
            map_id: self.map_id.clone(),
            created_at: self.created_at,
            cell_count: self.cells.len(),
            companion_space_count: self.companion_spaces.len(),
            statistics: self.get_statistics(),
            aln_stamp: self.aln_stamp.clone(),
            bostrom_stamp: self.bostrom_stamp.clone(),
            hex_stamp: self.hex_stamp.clone(),
            timestamp: Utc::now(),
        }
    }
}

/// Map statistics structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapStatistics {
    pub total_cells: usize,
    pub zone_counts: HashMap<ZoneClassification, usize>,
    pub average_density: f64,
    pub max_density: f64,
    pub active_companion_spaces: usize,
    pub aln_stamp: String,
    pub hex_stamp: String,
}

/// Map export structure for ledger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapExport {
    pub map_id: String,
    pub created_at: DateTime<Utc>,
    pub cell_count: usize,
    pub companion_space_count: usize,
    pub statistics: MapStatistics,
    pub aln_stamp: String,
    pub bostrom_stamp: String,
    pub hex_stamp: String,
    pub timestamp: DateTime<Utc>,
}

/// Thread-safe haunt density mapper
pub struct ThreadSafeMapper {
    inner: Arc<RwLock<HauntDensityMap>>,
}

impl ThreadSafeMapper {
    /// Create new thread-safe mapper
    pub fn new(map_id: String) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HauntDensityMap::new(map_id))),
        }
    }

    /// Update cell (thread-safe)
    pub fn update_cell(
        &self,
        x: usize,
        y: usize,
        haunt_density: f64,
        spectral_energy: f64,
    ) -> Result<(), MapperError> {
        let mut map = self.inner.write().map_err(|_| MapperError::GridLockPoisoned)?;
        map.update_cell(x, y, haunt_density, spectral_energy)
    }

    /// Get zone (thread-safe)
    pub fn get_zone(&self, x: usize, y: usize) -> Result<ZoneClassification, MapperError> {
        let map = self.inner.read().map_err(|_| MapperError::GridLockPoisoned)?;
        map.get_zone(x, y)
    }

    /// Register companion space (thread-safe)
    pub fn register_companion_space(
        &self,
        space_id: String,
        coordinates: Vec<(usize, usize)>,
    ) -> Result<(), MapperError> {
        let mut map = self.inner.write().map_err(|_| MapperError::GridLockPoisoned)?;
        map.register_companion_space(space_id, coordinates)
    }

    /// Export state (thread-safe)
    pub fn export_state(&self) -> MapExport {
        let map = self.inner.read().unwrap_or_else(|e| e.into_inner());
        map.export_state()
    }
}

impl Clone for ThreadSafeMapper {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zone_classification() {
        assert_eq!(ZoneClassification::from_density(0.10, false), ZoneClassification::Control);
        assert_eq!(ZoneClassification::from_density(0.30, false), ZoneClassification::MonitoredLow);
        assert_eq!(ZoneClassification::from_density(0.60, false), ZoneClassification::MonitoredHigh);
        assert_eq!(ZoneClassification::from_density(0.80, false), ZoneClassification::Restricted);
        assert_eq!(ZoneClassification::from_density(0.95, false), ZoneClassification::Containment);
        assert_eq!(ZoneClassification::from_density(0.95, true), ZoneClassification::CompanionSpace);
    }

    #[test]
    fn test_map_creation() {
        let map = HauntDensityMap::new("test_map".to_string());
        assert_eq!(map.cells.len(), GRID_WIDTH * GRID_HEIGHT);
        assert_eq!(map.map_id, "test_map");
    }

    #[test]
    fn test_cell_update() {
        let mut map = HauntDensityMap::new("test".to_string());
        assert!(map.update_cell(0, 0, 0.5, 0.6).is_ok());
        
        let cell = map.get_cell(0, 0).unwrap();
        assert_eq!(cell.haunt_density, 0.5);
        assert_eq!(cell.zone, ZoneClassification::MonitoredHigh);
    }

    #[test]
    fn test_companion_space_registration() {
        let mut map = HauntDensityMap::new("test".to_string());
        let coords = vec![(0, 0), (0, 1), (1, 0)];
        
        assert!(map.register_companion_space("space_001".to_string(), coords).is_ok());
        
        let cell = map.get_cell(0, 0).unwrap();
        assert!(cell.is_companion_space);
        assert_eq!(cell.zone, ZoneClassification::CompanionSpace);
    }

    #[test]
    fn test_thread_safe_mapper() {
        let mapper = ThreadSafeMapper::new("thread_test".to_string());
        let mapper_clone = mapper.clone();
        
        let handle = std::thread::spawn(move || {
            mapper_clone.update_cell(10, 10, 0.5, 0.5).unwrap();
        });
        
        mapper.update_cell(20, 20, 0.3, 0.3).unwrap();
        handle.join().unwrap();
        
        let zone1 = mapper.get_zone(10, 10).unwrap();
        let zone2 = mapper.get_zone(20, 20).unwrap();
        
        assert_eq!(zone1, ZoneClassification::MonitoredHigh);
        assert_eq!(zone2, ZoneClassification::MonitoredLow);
    }
}
