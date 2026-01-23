export class SpectralObject {
  constructor({
    id,
    kind,
    origin,
    signature,
    stability,
    drift,
    confidence,
    relationships,
    metadata
  }) {
    if (!id || !kind || !origin) {
      throw new Error("SpectralObject requires id, kind, and origin.");
    }

    this.id = id;                          // Stable identifier within catalog
    this.kind = kind;                      // e.g., "dom-sheet", "json-schema", "state-machine"
    this.origin = origin;                  // e.g., { domain, system, runId, modality }
    this.signature = signature || {};      // Shape-defining fields (selectors, fields, spans, etc.)
    this.stability = typeof stability === "number" ? stability : 0.0;   // 0–1
    this.drift = typeof drift === "number" ? drift : 0.0;               // 0–1
    this.confidence = typeof confidence === "number" ? confidence : 0.0;// 0–1
    this.relationships = Array.isArray(relationships) ? relationships : [];
    this.metadata = metadata || {};        // Free-form: tags, impact, notes
    this.createdAt = new Date().toISOString();
    this.updatedAt = this.createdAt;
  }

  touch(update = {}) {
    if (typeof update.stability === "number") this.stability = update.stability;
    if (typeof update.drift === "number") this.drift = update.drift;
    if (typeof update.confidence === "number") this.confidence = update.confidence;
    if (Array.isArray(update.relationships)) this.relationships = update.relationships;
    if (update.metadata && typeof update.metadata === "object") {
      this.metadata = { ...this.metadata, ...update.metadata };
    }
    this.updatedAt = new Date().toISOString();
    return this;
  }

  toJSON() {
    return {
      id: this.id,
      kind: this.kind,
      origin: this.origin,
      signature: this.signature,
      stability: this.stability,
      drift: this.drift,
      confidence: this.confidence,
      relationships: this.relationships,
      metadata: this.metadata,
      createdAt: this.createdAt,
      updatedAt: this.updatedAt
    };
  }
}

export class SpectralRealityModel {
  constructor() {
    this.objects = new Map(); // id -> SpectralObject
  }

  upsert(raw) {
    const existing = this.objects.get(raw.id);
    if (existing) {
      existing.touch(raw);
      return existing;
    }
    const obj = new SpectralObject(raw);
    this.objects.set(obj.id, obj);
    return obj;
  }

  getById(id) {
    return this.objects.get(id) || null;
  }

  listByKind(kind) {
    const out = [];
    for (const obj of this.objects.values()) {
      if (obj.kind === kind) out.push(obj);
    }
    return out;
  }

  listHighStability(threshold = 0.8) {
    const out = [];
    for (const obj of this.objects.values()) {
      if (obj.stability >= threshold && obj.drift <= 1 - threshold) {
        out.push(obj);
      }
    }
    return out;
  }

  listByOriginDomain(domain) {
    const out = [];
    for (const obj of this.objects.values()) {
      if (obj.origin && obj.origin.domain === domain) out.push(obj);
    }
    return out;
  }

  snapshot() {
    return Array.from(this.objects.values()).map(o => o.toJSON());
  }
}
