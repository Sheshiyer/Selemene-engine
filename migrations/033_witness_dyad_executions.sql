CREATE TABLE IF NOT EXISTS witness_dyad_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tier VARCHAR(32) NOT NULL,
    consciousness_level SMALLINT NOT NULL DEFAULT 0,
    live_scores JSONB NOT NULL,
    relationship_mode VARCHAR(32) NOT NULL DEFAULT 'None',
    engines_available TEXT[] NOT NULL DEFAULT '{}',
    aletheios TEXT,
    pichet TEXT,
    synthesis TEXT,
    witness_question TEXT,
    engines_used TEXT[] NOT NULL DEFAULT '{}',
    llm_powered BOOLEAN NOT NULL DEFAULT false,
    llm_provider VARCHAR(32),
    llm_model_aletheios VARCHAR(128),
    llm_model_pichet VARCHAR(128),
    llm_model_synthesis VARCHAR(128),
    llm_duration_ms DOUBLE PRECISION,
    error_message TEXT,
    request_ip_hash VARCHAR(64),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_wde_user_id ON witness_dyad_executions(user_id);
CREATE INDEX idx_wde_created_at ON witness_dyad_executions(created_at DESC);
CREATE INDEX idx_wde_llm_powered ON witness_dyad_executions(llm_powered) WHERE llm_powered = true;
CREATE INDEX idx_wde_tier ON witness_dyad_executions(tier);