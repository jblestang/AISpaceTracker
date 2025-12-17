# Coordinate System Debugging Guide

## Problem
All satellites appear to be on equatorial trajectories, suggesting the coordinate conversion from TEME to Bevy is incorrect.

## TEME Coordinate System
TEME (True Equator Mean Equinox) is an Earth-centered inertial frame:
- **X-axis**: Points toward vernal equinox (in equatorial plane)
- **Y-axis**: Completes right-hand system (in equatorial plane, 90° from X)
- **Z-axis**: Points toward north pole

## Bevy Coordinate System
- **X-axis**: Right
- **Y-axis**: Up
- **Z-axis**: Forward (out of screen)

## Current Conversion
```rust
Vec3::new(pos.x as f32, pos.z as f32, -pos.y as f32)
```
This maps:
- TEME X → Bevy X ✓
- TEME Z (north) → Bevy Y (up) ✓
- TEME Y → Bevy -Z

## Debugging Steps

1. **Check TEME Z values**: If all satellites have TEME Z ≈ 0, they're all equatorial (unlikely)
2. **Check Bevy Y values**: If all Bevy Y ≈ 0, the conversion is wrong
3. **Verify orbital inclinations**: Check TLE data for inclination angles

## Expected Behavior
- Satellites with high inclination (polar orbits) should have large Y variation
- Equatorial satellites should have Y ≈ 0
- Most satellites should have some Y variation

## Potential Fix
If TEME Z values vary but Bevy Y doesn't, the conversion might need adjustment. Try:
- Different axis mappings
- Rotation matrices
- Verify SGP4 output coordinate system

