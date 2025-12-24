# Simple Code Example: Serialization/Deserialization

## Step-by-Step Example

### Step 1: You have data in your program

```rust
// Your Rust struct
let my_escrow = Escrow {
    seed: 42,
    maker: Pubkey::new_unique(),
    mint_a: Pubkey::new_unique(),
    mint_b: Pubkey::new_unique(),
    recieve: 100,
    bump: 5
};
```

### Step 2: Serialization (Saving) - What Anchor does internally

```rust
// Anchor converts your struct to bytes
// This is what happens when you call: escrow.set_inner(my_escrow)

// Simplified version of what Anchor does:
fn serialize_escrow(escrow: &Escrow) -> Vec<u8> {
    let mut bytes = Vec::new();
    
    // 1. Add discriminator (4 bytes) - Anchor's way of identifying account type
    bytes.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
    
    // 2. Add seed (8 bytes for u64)
    bytes.extend_from_slice(&escrow.seed.to_le_bytes());
    // seed: 42 = [0x2A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    
    // 3. Add maker Pubkey (32 bytes)
    bytes.extend_from_slice(escrow.maker.as_ref());
    
    // 4. Add mint_a Pubkey (32 bytes)
    bytes.extend_from_slice(escrow.mint_a.as_ref());
    
    // 5. Add mint_b Pubkey (32 bytes)
    bytes.extend_from_slice(escrow.mint_b.as_ref());
    
    // 6. Add recieve (8 bytes for u64)
    bytes.extend_from_slice(&escrow.recieve.to_le_bytes());
    
    // 7. Add bump (1 byte for u8)
    bytes.push(escrow.bump);
    
    bytes  // Return the byte array
}

// Result: [0x01, 0x00, 0x00, 0x00, 0x2A, 0x00, ...] (113+ bytes total)
```

### Step 3: Bytes are stored on-chain

```
Solana Account Data:
┌─────────────────────────────────────┐
│ [0x01, 0x00, 0x00, 0x00,            │ ← Discriminator
│  0x2A, 0x00, 0x00, 0x00, ...        │ ← seed: 42
│  0xAB, 0xCD, 0xEF, ... (32 bytes)   │ ← maker Pubkey
│  0x12, 0x34, 0x56, ... (32 bytes)   │ ← mint_a Pubkey
│  0x78, 0x9A, 0xBC, ... (32 bytes)   │ ← mint_b Pubkey
│  0x64, 0x00, 0x00, 0x00, ...        │ ← recieve: 100
│  0x05]                               │ ← bump: 5
└─────────────────────────────────────┘
```

### Step 4: Deserialization (Reading) - What Anchor does when you access the account

```rust
// This is what happens when you do: ctx.accounts.escrow.seed

// Simplified version of what Anchor does:
fn deserialize_escrow(bytes: &[u8]) -> Result<Escrow> {
    let mut offset = 0;
    
    // 1. Check discriminator (first 4 bytes)
    let discriminator = &bytes[offset..offset+4];
    if discriminator != [0x01, 0x00, 0x00, 0x00] {
        return Err(Error::InvalidDiscriminator);
    }
    offset += 4;
    
    // 2. Read seed (next 8 bytes)
    let seed_bytes = &bytes[offset..offset+8];
    let seed = u64::from_le_bytes(seed_bytes.try_into().unwrap());
    offset += 8;
    // [0x2A, 0x00, ...] → 42
    
    // 3. Read maker Pubkey (next 32 bytes)
    let maker = Pubkey::try_from(&bytes[offset..offset+32])?;
    offset += 32;
    
    // 4. Read mint_a Pubkey (next 32 bytes)
    let mint_a = Pubkey::try_from(&bytes[offset..offset+32])?;
    offset += 32;
    
    // 5. Read mint_b Pubkey (next 32 bytes)
    let mint_b = Pubkey::try_from(&bytes[offset..offset+32])?;
    offset += 32;
    
    // 6. Read recieve (next 8 bytes)
    let recieve_bytes = &bytes[offset..offset+8];
    let recieve = u64::from_le_bytes(recieve_bytes.try_into().unwrap());
    offset += 8;
    
    // 7. Read bump (next 1 byte)
    let bump = bytes[offset];
    
    // 8. Reconstruct the struct
    Ok(Escrow {
        seed,
        maker,
        mint_a,
        mint_b,
        recieve,
        bump
    })
}

// This is essentially what try_from_unchecked does!
```

### Step 5: You can now use the struct

```rust
// After deserialization, you have your struct back:
let escrow: Escrow = deserialize_escrow(&account_bytes)?;

// Now you can use it:
println!("Seed: {}", escrow.seed);  // Prints: Seed: 42
println!("Recieve: {}", escrow.recieve);  // Prints: Recieve: 100
```

---

## Visual Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    SERIALIZATION (Saving)                    │
└─────────────────────────────────────────────────────────────┘

Rust Struct                    Bytes Array              On-Chain
┌─────────────┐              ┌──────────────┐         ┌─────────┐
│ Escrow {    │   Anchor     │ [0x01, 0x00, │  Store  │ Account │
│   seed: 42  │  converts →  │  0x2A, ...]  │   →     │  Data   │
│   maker: .. │              │              │         │         │
│ }           │              └──────────────┘         └─────────┘
└─────────────┘


┌─────────────────────────────────────────────────────────────┐
│                  DESERIALIZATION (Reading)                   │
└─────────────────────────────────────────────────────────────┘

On-Chain                  Bytes Array              Rust Struct
┌─────────┐              ┌──────────────┐         ┌─────────────┐
│ Account │   Read →     │ [0x01, 0x00, │  Anchor │ Escrow {    │
│  Data   │              │  0x2A, ...]  │ converts│   seed: 42  │
│         │              │              │   →     │   maker: .. │
└─────────┘              └──────────────┘         │ }           │
                                                  └─────────────┘
```

---

## Why Your Error Happens

### ✅ TokenAccount (Works)

```rust
pub vault: InterfaceAccount<'info, TokenAccount>
```

**What Anchor does:**
1. Gets bytes: `[0x01, 0x00, ...]` (token account data)
2. Calls: `TokenAccount::try_from_unchecked(&bytes)`
3. Deserializes: Converts bytes → `TokenAccount` struct
4. ✅ Success! You get `.amount`, `.mint`, etc.

### ❌ Program (Fails)

```rust
pub maker_ata_b: Program<'info, Token>
```

**What Anchor tries to do:**
1. Gets bytes: `[0x01, 0x00, ...]` (token account data)
2. Tries to call: `Program::try_from_unchecked(&bytes)`
3. ❌ **ERROR!** `Program` doesn't have this method
4. `Program` is not data - it's a program reference!

**The Fix:**
```rust
// Change from:
pub maker_ata_b: Program<'info, Token>

// To:
pub maker_ata_b: InterfaceAccount<'info, TokenAccount>
```

Now Anchor can deserialize it properly! ✅

