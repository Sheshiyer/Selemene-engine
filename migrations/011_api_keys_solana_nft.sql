-- Migration: 011_api_keys_solana_nft
-- Description: Add Solana blockchain integration for soul-bound NFT API key credentials

-- Add Solana NFT tracking columns to api_keys table
ALTER TABLE api_keys 
    ADD COLUMN IF NOT EXISTS solana_nft_mint VARCHAR(44),
    ADD COLUMN IF NOT EXISTS solana_owner_wallet VARCHAR(44),
    ADD COLUMN IF NOT EXISTS nft_minted_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS nft_transaction_signature VARCHAR(88);

-- Add comments for documentation
COMMENT ON COLUMN api_keys.solana_nft_mint IS 'Solana mint address (base58 encoded, 32-44 chars) of the soul-bound NFT';
COMMENT ON COLUMN api_keys.solana_owner_wallet IS 'Solana wallet address that owns the NFT';
COMMENT ON COLUMN api_keys.nft_minted_at IS 'When the NFT was minted on Solana';
COMMENT ON COLUMN api_keys.nft_transaction_signature IS 'Solana transaction signature for the mint (base58, 88 chars)';

-- Create unique index for NFT mint address lookups
CREATE UNIQUE INDEX IF NOT EXISTS idx_api_keys_solana_mint 
    ON api_keys(solana_nft_mint) 
    WHERE solana_nft_mint IS NOT NULL;

-- Create index for wallet-based queries
CREATE INDEX IF NOT EXISTS idx_api_keys_solana_wallet 
    ON api_keys(solana_owner_wallet) 
    WHERE solana_owner_wallet IS NOT NULL;

-- Create new table for detailed NFT metadata cache
-- This mirrors on-chain data for fast queries without RPC calls
CREATE TABLE IF NOT EXISTS api_key_nft_metadata (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    api_key_id UUID NOT NULL REFERENCES api_keys(id) ON DELETE CASCADE,
    
    -- Solana on-chain identifiers
    mint_address VARCHAR(44) NOT NULL UNIQUE,
    owner_wallet VARCHAR(44) NOT NULL,
    
    -- Metadata storage
    metadata_uri TEXT,
    metadata_json JSONB,
    image_uri TEXT,
    
    -- NFT properties (cached from chain)
    tier VARCHAR(50) NOT NULL,
    permissions_hash VARCHAR(64),  -- SHA-256 of permissions JSON
    rate_limit INTEGER,
    
    -- Soul-bound specific
    is_soul_bound BOOLEAN NOT NULL DEFAULT true,
    is_transferable BOOLEAN NOT NULL DEFAULT false,
    
    -- Status tracking
    is_revoked BOOLEAN NOT NULL DEFAULT false,
    revoked_at TIMESTAMPTZ,
    revocation_signature VARCHAR(88),
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Solana chain metadata
    solana_slot BIGINT,
    solana_block_time TIMESTAMPTZ,
    program_id VARCHAR(44) DEFAULT 'SelemeneSBT11111111111111111111111111111111'
);

-- Indexes for NFT metadata table
CREATE INDEX IF NOT EXISTS idx_nft_metadata_api_key ON api_key_nft_metadata(api_key_id);
CREATE INDEX IF NOT EXISTS idx_nft_metadata_owner ON api_key_nft_metadata(owner_wallet);
CREATE INDEX IF NOT EXISTS idx_nft_metadata_tier ON api_key_nft_metadata(tier);
CREATE INDEX IF NOT EXISTS idx_nft_metadata_revoked ON api_key_nft_metadata(is_revoked) WHERE is_revoked = true;

-- Comments for NFT metadata table
COMMENT ON TABLE api_key_nft_metadata IS 'Caches Solana on-chain NFT data for fast lookups without RPC calls';
COMMENT ON COLUMN api_key_nft_metadata.metadata_json IS 'Full NFT metadata JSON from IPFS/Arweave';
COMMENT ON COLUMN api_key_nft_metadata.permissions_hash IS 'Hash of api_keys.permissions for integrity verification';
COMMENT ON COLUMN api_key_nft_metadata.is_soul_bound IS 'True if this is a non-transferable soul-bound token';

-- Table for NFT transaction history (audit trail)
CREATE TABLE IF NOT EXISTS api_key_nft_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    api_key_id UUID NOT NULL REFERENCES api_keys(id) ON DELETE CASCADE,
    nft_metadata_id UUID REFERENCES api_key_nft_metadata(id),
    
    -- Transaction details
    transaction_type VARCHAR(20) NOT NULL,  -- mint, revoke, refresh, transfer_attempt
    signature VARCHAR(88) NOT NULL,
    slot BIGINT,
    block_time TIMESTAMPTZ,
    
    -- Status
    status VARCHAR(20) NOT NULL DEFAULT 'pending',  -- pending, confirmed, failed
    error_message TEXT,
    
    -- Metadata at time of transaction
    tier_at_transaction VARCHAR(50),
    expires_at_transaction TIMESTAMPTZ,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    confirmed_at TIMESTAMPTZ
);

-- Indexes for transaction history
CREATE INDEX IF NOT EXISTS idx_nft_tx_api_key ON api_key_nft_transactions(api_key_id);
CREATE INDEX IF NOT EXISTS idx_nft_tx_signature ON api_key_nft_transactions(signature);
CREATE INDEX IF NOT EXISTS idx_nft_tx_status ON api_key_nft_transactions(status);
CREATE INDEX IF NOT EXISTS idx_nft_tx_type ON api_key_nft_transactions(transaction_type);

COMMENT ON TABLE api_key_nft_transactions IS 'Audit trail of all Solana transactions related to API key NFTs';

-- Trigger function to sync NFT revocation status from api_keys
CREATE OR REPLACE FUNCTION sync_nft_revocation_status()
RETURNS TRIGGER AS $$
BEGIN
    -- When api_keys.is_active becomes false, mark NFT as revoked
    IF NEW.is_active = false AND OLD.is_active = true THEN
        -- Update metadata cache
        UPDATE api_key_nft_metadata 
        SET 
            is_revoked = true, 
            revoked_at = COALESCE(NEW.revoked_at, NOW()),
            updated_at = NOW()
        WHERE api_key_id = NEW.id;
        
        -- Log the transaction
        INSERT INTO api_key_nft_transactions (
            api_key_id,
            transaction_type,
            signature,
            status,
            tier_at_transaction,
            created_at
        ) VALUES (
            NEW.id,
            'revoke',
            COALESCE(NEW.nft_transaction_signature, 'db_trigger_' || gen_random_uuid()::text),
            'confirmed',
            NEW.tier,
            NOW()
        );
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Drop existing trigger if exists (for idempotent migrations)
DROP TRIGGER IF EXISTS trigger_sync_nft_revocation ON api_keys;

-- Create the trigger
CREATE TRIGGER trigger_sync_nft_revocation
    AFTER UPDATE OF is_active ON api_keys
    FOR EACH ROW
    EXECUTE FUNCTION sync_nft_revocation_status();

COMMENT ON FUNCTION sync_nft_revocation_status() IS 'Automatically syncs NFT revocation status when API keys are deactivated';

-- Function to validate NFT ownership (for verification API)
CREATE OR REPLACE FUNCTION verify_nft_ownership(
    p_mint_address VARCHAR(44),
    p_owner_wallet VARCHAR(44)
)
RETURNS TABLE(
    is_valid BOOLEAN,
    api_key_id UUID,
    tier VARCHAR(50),
    is_active BOOLEAN,
    is_revoked BOOLEAN,
    expires_at TIMESTAMPTZ,
    permissions JSONB
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        true as is_valid,
        k.id as api_key_id,
        k.tier,
        k.is_active,
        nm.is_revoked,
        k.expires_at,
        k.permissions
    FROM api_keys k
    JOIN api_key_nft_metadata nm ON nm.api_key_id = k.id
    WHERE k.solana_nft_mint = p_mint_address
      AND k.solana_owner_wallet = p_owner_wallet
      AND k.is_active = true
      AND nm.is_revoked = false
      AND (k.expires_at IS NULL OR k.expires_at > NOW());
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION verify_nft_ownership(VARCHAR, VARCHAR) IS 'Verifies if a wallet owns a valid, active API key NFT';

-- View for complete NFT + API key status
CREATE OR REPLACE VIEW api_key_nft_complete_status AS
SELECT 
    k.id as api_key_id,
    k.name as api_key_name,
    k.user_id,
    u.email as user_email,
    k.tier,
    k.is_active,
    k.created_at as key_created_at,
    k.expires_at as key_expires_at,
    k.revoked_at as key_revoked_at,
    k.revoked_reason,
    
    -- NFT fields
    k.solana_nft_mint,
    k.solana_owner_wallet,
    k.nft_minted_at,
    k.nft_transaction_signature,
    
    -- Metadata cache
    nm.metadata_uri,
    nm.image_uri,
    nm.is_revoked as nft_revoked,
    nm.revoked_at as nft_revoked_at,
    nm.solana_slot,
    nm.solana_block_time,
    
    -- Combined status
    CASE 
        WHEN NOT k.is_active THEN 'key_revoked'
        WHEN nm.is_revoked THEN 'nft_revoked'
        WHEN k.expires_at IS NOT NULL AND k.expires_at < NOW() THEN 'expired'
        WHEN k.solana_nft_mint IS NULL THEN 'no_nft'
        ELSE 'active_nft'
    END as combined_status

FROM api_keys k
JOIN users u ON u.id = k.user_id
LEFT JOIN api_key_nft_metadata nm ON nm.api_key_id = k.id;

COMMENT ON VIEW api_key_nft_complete_status IS 'Unified view of API key and NFT status for admin dashboard';

-- Function to get NFTs by wallet (for wallet profile pages)
CREATE OR REPLACE FUNCTION get_nfts_by_wallet(
    p_wallet_address VARCHAR(44)
)
RETURNS TABLE(
    mint_address VARCHAR(44),
    api_key_id UUID,
    api_key_name TEXT,
    tier VARCHAR(50),
    is_active BOOLEAN,
    is_revoked BOOLEAN,
    expires_at TIMESTAMPTZ,
    metadata_json JSONB,
    image_uri TEXT
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        k.solana_nft_mint as mint_address,
        k.id as api_key_id,
        k.name as api_key_name,
        k.tier,
        k.is_active,
        nm.is_revoked,
        k.expires_at,
        nm.metadata_json,
        nm.image_uri
    FROM api_keys k
    LEFT JOIN api_key_nft_metadata nm ON nm.api_key_id = k.id
    WHERE k.solana_owner_wallet = p_wallet_address
    ORDER BY k.nft_minted_at DESC;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION get_nfts_by_wallet(VARCHAR) IS 'Returns all API key NFTs owned by a Solana wallet';
