# Coordinate System Fix

## Issue Analysis
All satellites appear on equatorial trajectories, indicating the TEME→Bevy coordinate conversion is incorrect.

## Root Cause
The current conversion `Vec3::new(pos.x, pos.z, -pos.y)` may be incorrect. The issue is likely:
1. **TEME Z values are actually varying** (satellites have different inclinations)
2. **But Bevy Y is always near zero** (conversion problem)

## Solution

### Step 1: Add Debugging
The `coordinate_debug.rs` module has been created with debugging functions.

### Step 2: Verify TEME Coordinates
Check if TEME Z values actually vary. If they do, the conversion is wrong.

### Step 3: Try Alternative Conversions

**Option A: Direct mapping (current)**
```rust
Vec3::new(pos.x as f32, pos.z as f32, -pos.y as f32)
```

**Option B: Try different Y mapping**
```rust
Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32)
```

**Option C: Rotate coordinate system**
```rust
// TEME to ECEF-like conversion
Vec3::new(pos.x as f32, pos.z as f32, pos.y as f32)
```

### Step 4: Check SGP4 Output
Verify that SGP4 actually returns TEME coordinates. Some implementations return ECEF or other frames.

## Testing
Run the application and check console output:
- `[COORD]` messages show TEME and Bevy coordinates
- `[TRAJ]` messages show coordinate ranges
- If TEME Z varies but Bevy Y doesn't, the conversion is wrong

## Expected Output
For a satellite with 45° inclination:
- TEME Z should vary significantly (not always near 0)
- Bevy Y should also vary significantly
- If Bevy Y is always near 0, the conversion needs fixing

