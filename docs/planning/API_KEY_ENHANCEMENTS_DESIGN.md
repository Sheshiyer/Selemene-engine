# API Key Enhancements Design Document

## Current Status (As of 2026-02-28)

### ✅ Implemented Features

| Feature | Status | Details |
|---------|--------|---------|
| **Fine-grain scope generator** | ✅ Complete | Permission groups: Admin, API Access, Engines (Panchanga, Numerology, Biorhythm, HD, Gene Keys, Vimshottari) |
| **Persistence** | ✅ Complete | PostgreSQL with migrations 004 + 009. Full CRUD operations via admin dashboard |
| **Time-bound (expires_at)** | ✅ Complete | Optional expiration field. UI supports datetime-local picker |
| **Auto-delete expired keys** | ⚠️ Partial | Database field exists, but no background job for cleanup |
| **Soul-bound NFT integration** | ❌ Not started | Design phase - see Section 3 |

### Admin Dashboard

- **URL**: https://enantiodromia-engine-dashboard.vercel.app
- **Features**:
  - Create keys with name, tier (free/premium/enterprise), rate limits
  - Fine-grained permission selection (checkbox groups)
  - Expiration date/time picker
  - Key rotation with secret reveal modal
  - Revoke keys (soft delete - sets is_active=false)
  - Filter by tier, status, search by email/name

---

## 1. Auto-Delete System for Expired Keys

### 1.1 Requirements

- Soft-delete keys that have passed their `expires_at` timestamp
- Log deletion events for audit purposes
- Option to notify users before expiration
- Grace period (e.g., 24-48h after expiry before hard delete or revocation)

### 1.2 Proposed Implementation

#### Option A: Database Event (Recommended for MVP)

```sql
-- Create a function to auto-revoke expired keys
CREATE OR REPLACE FUNCTION revoke_expired_api_keys()
RETURNS INTEGER AS $$
DECLARE
    revoked_count INTEGER;
BEGIN
    UPDATE api_keys 
    SET is_active = false,
        revoked_at = NOW(),
        revoked_reason = 'expired'
    WHERE is_active = true 
      AND expires_at IS NOT NULL 
      AND expires_at < NOW();
    
    GET DIAGNOSTICS revoked_count = ROW_COUNT;
    RETURN revoked_count;
END;
$$ LANGUAGE plpgsql;

-- Schedule with pg_cron (if available on Supabase)
SELECT cron.schedule('revoke-expired-keys', '0 * * * *', 'SELECT revoke_expired_api_keys()');
```

#### Option B: Background Job in Rust

Add to `noesis-api`:

```rust
// In a background task
pub async fn start_expired_key_cleanup(pool: PgPool) {
    let mut interval = tokio::time::interval(Duration::from_secs(3600)); // Hourly
    
    loop {
        interval.tick().await;
        
        if let Err(e) = revoke_expired_keys(&pool).await {
            tracing::error!("Failed to revoke expired keys: {}", e);
        }
    }
}
```

#### Option C: External Cron (Railway/ Vercel Cron)

```bash
# Railway scheduled job
# POST /api/v1/admin/api-keys/cleanup-expired
# Protected by admin API key
```

### 1.3 Database Migration

```sql
-- Migration: 010_api_keys_expiration_tracking.sql

-- Add revocation metadata
ALTER TABLE api_keys 
    ADD COLUMN revoked_at TIMESTAMPTZ,
    ADD COLUMN revoked_reason VARCHAR(50),
    ADD COLUMN expiration_warning_sent_at TIMESTAMPTZ;

-- Create partial index for active keys with expiration
CREATE INDEX idx_api_keys_active_expires 
    ON api_keys(expires_at) 
    WHERE is_active = true AND expires_at IS NOT NULL;
```

---

## 2. Soul-Bound NFT System with Solana

### 2.1 Overview

Soul-bound tokens (SBTs) are non-transferable NFTs that represent:
- API key ownership and tier status
- Achievement badges (power user, early adopter)
- Subscription validity proofs
- Access credentials for premium features

### 2.2 Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Selemene API   │────▶│  Solana Program │────▶│  Metaplex NFT   │
│   (Rust/Axum)   │     │   (Anchor)      │     │   Standard      │
└─────────────────┘     └─────────────────┘     └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
   ┌──────────┐           ┌──────────┐           ┌──────────┐
   │ PostgreSQL│           │  Solana  │           │  IPFS/   │
   │   DB     │           │ Network  │           │  Arweave │
   └──────────┘           └──────────┘           └──────────┘
```

### 2.3 Solana Program Design

```rust
// programs/selemene_soul_bound/src/lib.rs

#[program]
pub mod selemene_soul_bound {
    use super::*;

    /// Mint a soul-bound API key NFT
    pub fn mint_api_key_nft(
        ctx: Context<MintApiKeyNft>,
        api_key_hash: String,        // SHA-256 hash of the API key
        tier: String,                // free/premium/enterprise
        permissions_hash: String,    // Hash of permissions JSON
        expires_at: Option<i64>,     // Unix timestamp
    ) -> Result<()> {
        let nft = &mut ctx.accounts.nft_account;
        
        // Ensure this is soul-bound (non-transferable)
        require!(
            ctx.accounts.token_program.key() == token::ID,
            ErrorCode::InvalidTokenProgram
        );
        
        nft.owner = ctx.accounts.owner.key();
        nft.api_key_hash = api_key_hash;
        nft.tier = tier;
        nft.permissions_hash = permissions_hash;
        nft.created_at = Clock::get()?.unix_timestamp;
        nft.expires_at = expires_at;
        nft.is_revoked = false;
        nft.bump = ctx.bumps.nft_account;
        
        emit!(NftMinted {
            owner: nft.owner,
            api_key_hash: nft.api_key_hash.clone(),
            tier: nft.tier.clone(),
        });
        
        Ok(())
    }

    /// Revoke an NFT when the API key is revoked
    pub fn revoke_nft(ctx: Context<RevokeNft>) -> Result<()> {
        let nft = &mut ctx.accounts.nft_account;
        
        require!(
            nft.owner == ctx.accounts.authority.key() ||
            ctx.accounts.authority.key() == ctx.accounts.admin_key.key(),
            ErrorCode::Unauthorized
        );
        
        nft.is_revoked = true;
        nft.revoked_at = Some(Clock::get()?.unix_timestamp);
        
        emit!(NftRevoked {
            owner: nft.owner,
            api_key_hash: nft.api_key_hash.clone(),
        });
        
        Ok(())
    }

    /// Refresh expiration (renewal)
    pub fn refresh_expiration(
        ctx: Context<RefreshExpiration>,
        new_expires_at: Option<i64>,
    ) -> Result<()> {
        let nft = &mut ctx.accounts.nft_account;
        
        require!(
            nft.owner == ctx.accounts.owner.key(),
            ErrorCode::Unauthorized
        );
        require!(!nft.is_revoked, ErrorCode::NftRevoked);
        
        nft.expires_at = new_expires_at;
        
        emit!(ExpirationRefreshed {
            owner: nft.owner,
            new_expires_at,
        });
        
        Ok(())
    }
}

#[account]
pub struct SoulBoundNft {
    pub owner: Pubkey,
    pub api_key_hash: String,        // Links to PostgreSQL api_keys.key_hash
    pub tier: String,
    pub permissions_hash: String,
    pub created_at: i64,
    pub expires_at: Option<i64>,
    pub is_revoked: bool,
    pub revoked_at: Option<i64>,
    pub bump: u8,
}

// Size: 32 + 64 + 20 + 64 + 8 + 9 + 1 + 9 + 1 = ~208 bytes
impl SoulBoundNft {
    pub const LEN: usize = 8 + 32 + 64 + 20 + 64 + 8 + 9 + 1 + 9 + 1;
}
```

### 2.4 Database Schema Updates

```sql
-- Migration: 011_api_keys_solana_nft.sql

-- Add Solana NFT tracking to api_keys
ALTER TABLE api_keys 
    ADD COLUMN solana_nft_mint VARCHAR(44),  -- Solana address (base58)
    ADD COLUMN solana_owner_wallet VARCHAR(44),
    ADD COLUMN nft_minted_at TIMESTAMPTZ,
    ADD COLUMN nft_transaction_signature VARCHAR(88);

-- Create index for NFT lookups
CREATE UNIQUE INDEX idx_api_keys_solana_mint 
    ON api_keys(solana_nft_mint) 
    WHERE solana_nft_mint IS NOT NULL;

-- Add new table for NFT metadata cache
CREATE TABLE api_key_nft_metadata (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    api_key_id UUID NOT NULL REFERENCES api_keys(id) ON DELETE CASCADE,
    mint_address VARCHAR(44) NOT NULL UNIQUE,
    owner_wallet VARCHAR(44) NOT NULL,
    metadata_uri TEXT,
    metadata_json JSONB,
    tier VARCHAR(50) NOT NULL,
    is_soul_bound BOOLEAN NOT NULL DEFAULT true,
    is_revoked BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    solana_slot BIGINT,
    solana_block_time TIMESTAMPTZ
);

CREATE INDEX idx_nft_metadata_api_key ON api_key_nft_metadata(api_key_id);
CREATE INDEX idx_nft_metadata_owner ON api_key_nft_metadata(owner_wallet);

-- Function to sync NFT status from Solana
CREATE OR REPLACE FUNCTION sync_nft_revocation_status()
RETURNS TRIGGER AS $$
BEGIN
    -- When api_keys.is_active becomes false, mark NFT as revoked
    IF NEW.is_active = false AND OLD.is_active = true THEN
        UPDATE api_key_nft_metadata 
        SET is_revoked = true, updated_at = NOW()
        WHERE api_key_id = NEW.id;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_sync_nft_revocation
    AFTER UPDATE OF is_active ON api_keys
    FOR EACH ROW
    EXECUTE FUNCTION sync_nft_revocation_status();
```

### 2.5 API Endpoints

```rust
// crates/noesis-api/src/handlers/solana.rs

/// Request to mint a soul-bound NFT for an API key
#[derive(Debug, Deserialize)]
pub struct MintNftRequest {
    pub api_key_id: Uuid,
    pub owner_wallet: String,  // Solana wallet address
}

/// Response after minting NFT
#[derive(Debug, Serialize)]
pub struct MintNftResponse {
    pub mint_address: String,
    pub transaction_signature: String,
    pub metadata_uri: String,
    pub explorer_url: String,
}

/// Get NFT status for an API key
pub async fn get_nft_status(
    Path(api_key_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<NftStatusResponse>, EngineError> {
    // Query database for NFT metadata
    // Return mint address, status, Solana explorer link
}

/// Verify NFT ownership (for third-party verification)
pub async fn verify_nft_ownership(
    Query(params): Query<VerifyNftQuery>,
    State(state): State<AppState>,
) -> Result<Json<VerificationResponse>, EngineError> {
    // Check if wallet owns valid, non-revoked NFT
    // Return tier, permissions, expiration
}
```

### 2.6 Metadata Structure (IPFS/Arweave)

```json
{
  "name": "Selemene API Key - Premium",
  "symbol": "SELEMENE-API",
  "description": "Soul-bound API key credential for Selemene Engine access",
  "image": "https://selemene.tryambakam.space/nft/premium-badge.png",
  "attributes": [
    {
      "trait_type": "Tier",
      "value": "premium"
    },
    {
      "trait_type": "Rate Limit",
      "value": "1000/min"
    },
    {
      "trait_type": "Created",
      "display_type": "date",
      "value": 1740777600
    },
    {
      "trait_type": "Expires",
      "display_type": "date",
      "value": 1772313600
    },
    {
      "trait_type": "Soul Bound",
      "value": "Yes"
    }
  ],
  "properties": {
    "api_key_hash_preview": "nk_KAd5...",
    "permissions_hash": "sha256:abc123...",
    "category": "api_credential",
    "creators": [
      {
        "address": "SelemeneEngine111111111111111111111111111111",
        "verified": true,
        "share": 0
      }
    ]
  }
}
```

### 2.7 Frontend Integration

```typescript
// apps/admin-web/app/(protected)/api-keys/page.tsx additions

// Add to AdminApiKeyItem interface
interface AdminApiKeyItem {
  // ... existing fields
  solana_nft_mint?: string;
  solana_owner_wallet?: string;
  nft_minted_at?: string;
}

// New component: NftMintButton
function NftMintButton({ apiKeyId, onMinted }: { apiKeyId: string; onMinted: () => void }) {
  const [wallet, setWallet] = useState<string>("");
  const [minting, setMinting] = useState(false);
  
  const handleMint = async () => {
    setMinting(true);
    try {
      const result = await mintApiKeyNft(token, {
        api_key_id: apiKeyId,
        owner_wallet: wallet,
      });
      // Show success with Solana explorer link
      onMinted();
    } finally {
      setMinting(false);
    }
  };
  
  return (
    <Modal>
      <WalletInput value={wallet} onChange={setWallet} />
      <Button onClick={handleMint} disabled={minting || !wallet}>
        {minting ? "Minting..." : "Mint Soul-Bound NFT"}
      </Button>
    </Modal>
  );
}

// Display NFT status in table
function NftStatusCell({ key }: { key: AdminApiKeyItem }) {
  if (!key.solana_nft_mint) {
    return <Button size="sm">Mint NFT</Button>;
  }
  
  return (
    <a 
      href={`https://explorer.solana.com/address/${key.solana_nft_mint}`}
      target="_blank"
      rel="noopener"
    >
      <NftBadge mint={key.solana_nft_mint} />
    </a>
  );
}
```

### 2.8 Implementation Phases

| Phase | Deliverables | Timeline |
|-------|--------------|----------|
| **P0** | Database migrations, API endpoints, backend services | 1 week |
| **P1** | Solana program development (Anchor), metadata pipeline | 1 week |
| **P2** | Frontend NFT minting UI, wallet integration (Phantom/Solflare) | 1 week |
| **P3** | Audit, mainnet deployment, documentation | 1 week |

---

## 3. Integration Checklist

### Pre-deployment
- [ ] Run migration 010 (auto-delete tracking)
- [ ] Run migration 011 (Solana NFT support)
- [ ] Deploy Solana program to devnet
- [ ] Configure Solana RPC endpoint
- [ ] Set up IPFS/Arweave metadata upload
- [ ] Test NFT minting flow end-to-end

### Post-deployment
- [ ] Monitor auto-delete job logs
- [ ] Verify NFT minting transaction costs
- [ ] Set up Solana program upgrade authority
- [ ] Document NFT verification API for third parties

---

## 4. Cost Estimates

### Solana Mainnet
- NFT Mint: ~0.012 SOL ($2-3)
- Account rent exemption: ~0.002 SOL
- Metadata upload (Arweave): ~0.0005 SOL
- **Total per NFT**: ~0.015 SOL (~$3)

### Infrastructure
- Solana RPC (Helius/QuickNode): $50-200/month
- Arweave storage: One-time ~$0.01 per NFT

---

## 5. Security Considerations

1. **API Key Hashing**: Only store SHA-256 hashes on-chain, never plaintext
2. **Authority**: Use multisig for Solana program upgrades
3. **Replay Protection**: Include recent blockhash in mint transactions
4. **Rate Limiting**: Prevent NFT spamming with cooldown periods
5. **Wallet Verification**: Ensure wallet ownership via signed message

---

*Document Version: 1.0*
*Last Updated: 2026-02-28*
*Author: Selemene Engineering*
