use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Kinds of spectral‑objects that can be excavated.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SpectralKind {
    DomSheet,
    JsonSchema,
    StateMachine,
    ApiShape,
    TracePattern,
    VmRegion,
    Other(String),
}

/// Origin of a spectral‑object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Origin {
    pub domain: String,
    pub system: String,
    pub run_id: String,
    pub modality: String,
}

/// Core spectral‑object representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpectralObject {
    pub id: String,
    pub kind: SpectralKind,
    pub origin: Origin,
    pub signature: HashMap<String, serde_json::Value>,
    pub stability: f64,
    pub drift: f64,
    pub confidence: f64,
    pub relationships: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SpectralObject {
    /// Creates a new spectral‑object.
    pub fn new(
        kind: SpectralKind,
        origin: Origin,
        signature: HashMap<String, serde_json::Value>,
    ) -> Self {
        let now = Utc::now();
        SpectralObject {
            id: Uuid::new_v4().to_string(),
            kind,
            origin,
            signature,
            stability: 0.0,
            drift: 0.0,
            confidence: 0.0,
            relationships: Vec::new(),
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Updates mutable fields and touches the timestamp.
    pub fn touch(
        &mut self,
        stability: Option<f64>,
        drift: Option<f64>,
        confidence: Option<f64>,
        relationships: Option<Vec<String>>,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) {
        if let Some(s) = stability {
            self.stability = s.clamp(0.0, 1.0);
        }
        if let Some(d) = drift {
            self.drift = d.clamp(0.0, 1.0);
        }
        if let Some(c) = confidence {
            self.confidence = c.clamp(0.0, 1.0);
        }
        if let Some(r) = relationships {
            self.relationships = r;
        }
        if let Some(m) = metadata {
            self.metadata.extend(m);
        }
        self.updated_at = Utc::now();
    }
}

/// Catalog of spectral‑objects.
#[derive(Debug, Default)]
pub struct SpectralRealityModel {
    objects: HashMap<String, SpectralObject>,
}

impl SpectralRealityModel {
    /// Inserts or updates a spectral‑object.
    pub fn upsert(&mut self, obj: SpectralObject) -> &SpectralObject {
        let id = obj.id.clone();
        self.objects.insert(id, obj);
        self.objects.get(&id).unwrap()
    }

    /// Retrieves a spectral‑object by ID.
    pub fn get_by_id(&self, id: &str) -> Option<&SpectralObject> {
        self.objects.get(id)
    }

    /// Lists all objects of a given kind.
    pub fn list_by_kind(&self, kind: &SpectralKind) -> Vec<&SpectralObject> {
        self.objects
            .values()
            .filter(|o| &o.kind == kind)
            .collect()
    }

    /// Lists objects with high stability and low drift.
    pub fn list_high_stability(&self, threshold: f64) -> Vec<&SpectralObject> {
        self.objects
            .values()
            .filter(|o| o.stability >= threshold && o.drift <= 1.0 - threshold)
            .collect()
    }

    /// Lists objects from a specific domain.
    pub fn list_by_domain(&self, domain: &str) -> Vec<&SpectralObject> {
        self.objects
            .values()
            .filter(|o| o.origin.domain == domain)
            .collect()
    }

    /// Exports a snapshot of the catalog as JSON.
    pub fn snapshot(&self) -> serde_json::Value {
        serde_json::to_value(self.objects.values().collect::<Vec<&SpectralObject>>()).unwrap()
    }
}

// Example unit tests (omitted for brevity) would verify insertion, querying, and scoring logic.
