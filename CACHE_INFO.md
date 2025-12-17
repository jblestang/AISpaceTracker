# TLE Data Caching

## Overview
The application caches Two-Line Element (TLE) data to avoid downloading it every time you run the application.

## Cache Location
- **Directory**: `cache/`
- **File**: `cache/tle_cache.json`

## Cache Behavior
- **Cache Duration**: 24 hours (configurable in `TleLoader::new()`)
- **Automatic Refresh**: Cache is automatically refreshed if it's older than 24 hours
- **First Run**: Downloads TLE data and saves to cache
- **Subsequent Runs**: Loads from cache if it's less than 24 hours old

## Cache Format
The cache is stored as JSON with the following structure:
```json
{
  "data": {
    "SATELLITE_NAME": {
      "name": "SATELLITE_NAME",
      "line1": "1 25544U ...",
      "line2": "2 25544U ..."
    }
  },
  "downloaded_at": 1234567890
}
```

## Manual Cache Management
To force a fresh download, delete the cache file:
```bash
rm cache/tle_cache.json
```

Or use the `clear_cache()` method in code:
```rust
let loader = TleLoader::new();
loader.clear_cache()?;
```

## Benefits
- **Faster Startup**: No network delay on subsequent runs
- **Offline Support**: Works without internet if cache exists
- **Reduced Bandwidth**: Only downloads when cache expires
- **Reliable**: Falls back to download if cache is corrupted

