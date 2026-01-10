-- 20260110000000_create_promo_codes.sql
-- Promo codes for free membership access

CREATE TABLE promo_codes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    code VARCHAR(50) NOT NULL UNIQUE,
    description TEXT,
    tier VARCHAR(20) NOT NULL CHECK (tier IN ('free', 'monthly', 'annual')),
    max_uses INTEGER,
    times_used INTEGER NOT NULL DEFAULT 0,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    valid_until TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for code lookups (case-insensitive via application layer)
CREATE INDEX idx_promo_codes_code ON promo_codes(code);

-- Index for active codes
CREATE INDEX idx_promo_codes_active ON promo_codes(is_active) WHERE is_active = true;

-- Trigger for updated_at
CREATE TRIGGER update_promo_codes_updated_at
    BEFORE UPDATE ON promo_codes
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

COMMENT ON TABLE promo_codes IS 'Promo codes for free tier membership access';
COMMENT ON COLUMN promo_codes.code IS 'Unique code (stored uppercase for case-insensitive matching)';
COMMENT ON COLUMN promo_codes.max_uses IS 'NULL means unlimited uses';
COMMENT ON COLUMN promo_codes.times_used IS 'Counter incremented on each use';
