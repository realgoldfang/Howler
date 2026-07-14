# Howler Configuration Guide

This guide explains how to configure Howler for use with various data sources.

## Environment Variables

Howler uses environment variables for configuration. These should be set before running any Howler component.

### Movebank Configuration

Movebank provides GPS tracking data from collared animals.

**Required Variables:**
```bash
export MOVEBANK_USERNAME="your_movebank_username"
export MOVEBANK_PASSWORD="your_movebank_password"
```

**How to Get Credentials:**
1. Visit [movebank.org](https://www.movebank.org)
2. Create an account
3. Request access to studies you're interested in
4. Use your login credentials

**Note:** Movebank requires authentication for all data access.

### iNaturalist Configuration

iNaturalist provides citizen science observations.

**Status: Optional (disabled by default)**

iNaturalist requires an OAuth application, which needs:
- An iNaturalist account at least **5 months old**
- At least **10 observations** posted

If you don't meet these requirements, Howler skips iNaturalist data gracefully.

**Required Variable (only if you have a token):**
```bash
export INATURALIST_TOKEN="your_inaturalist_token"
```

**How to Get Token:**
1. Visit [inaturalist.org](https://www.inaturalist.org)
2. Sign up or log in (account must be 5+ months old with 10+ observations)
3. Go to your account settings
4. Navigate to "Applications" or "API"
5. Create a new application
6. Copy the access token

**Note:** If no token is set, Howler skips iNaturalist entirely with a clear message.

### IUCN Configuration

IUCN provides conservation status data via the **v4 API** (v3 is retired).

**Required Variable:**
```bash
export IUCN_TOKEN="your_iucn_token"
```

**How to Get Token:**
1. Visit [api.iucnredlist.org](https://api.iucnredlist.org)
2. Click **Register** in the top-right corner
3. Create an account at [iucnredlist.org/users/sign_up](https://www.iucnredlist.org/users/sign_up)
4. After registering, go to your account page to find your API token
5. Copy the token and set it as `IUCN_TOKEN`

**Important Notes:**
- The v4 API is the current version (v3 is end-of-life)
- Non-commercial use only — for commercial use, see [IBAT](https://www.ibat-alliance.org)
- Rate limits apply; cache data locally when possible
- If your token is revoked, contact redlist@iucn.org

**Note:** IUCN data is optional. If no token is set, Howler skips IUCN data gracefully.

## Setting Environment Variables

### Linux/macOS (Bash)

**Temporary (current session only):**
```bash
export MOVEBANK_USERNAME="user"
export MOVEBANK_PASSWORD="pass"
export INATURALIST_TOKEN="token"
export IUCN_TOKEN="token"
```

**Permanent (add to ~/.bashrc or ~/.zshrc):**
```bash
echo 'export MOVEBANK_USERNAME="user"' >> ~/.bashrc
echo 'export MOVEBANK_PASSWORD="pass"' >> ~/.bashrc
echo 'export INATURALIST_TOKEN="token"' >> ~/.bashrc
echo 'export IUCN_TOKEN="token"' >> ~/.bashrc
source ~/.bashrc
```

### Windows (PowerShell)

**Temporary (current session only):**
```powershell
$env:MOVEBANK_USERNAME="user"
$env:MOVEBANK_PASSWORD="pass"
$env:INATURALIST_TOKEN="token"
$env:IUCN_TOKEN="token"
```

**Permanent (System Environment Variables):**
1. Open System Properties
2. Click "Environment Variables"
3. Add new variables under User variables

### Windows (Command Prompt)

**Temporary (current session only):**
```cmd
set MOVEBANK_USERNAME=user
set MOVEBANK_PASSWORD=pass
set INATURALIST_TOKEN=token
set IUCN_TOKEN=token
```

## Configuration File (Optional)

Howler can also use a configuration file at `~/.config/howler/config.toml`.

**Example config.toml:**
```toml
[movebank]
username = "your_username"
password = "your_password"

[inaturalist]
token = "your_token"

[iucn]
token = "your_token"

[database]
path = "~/.local/share/howler/howler.db"

[cache]
enabled = true
size_mb = 100

[ui]
theme = "dark"
default_map_source = "osm"
```

**Note:** Environment variables take precedence over config file settings.

## Database Configuration

By default, Howler stores data in `~/.local/share/howler/howler.db`.

**Custom Database Path:**
```bash
export HOWLER_DB_PATH="/custom/path/howler.db"
```

Or in config.toml:
```toml
[database]
path = "/custom/path/howler.db"
```

## Cache Configuration

Howler caches API responses to reduce rate limit usage.

**Enable/Disable Cache:**
```bash
export HOWLER_CACHE_ENABLED="true"
```

**Cache Size:**
```bash
export HOWLER_CACHE_SIZE_MB="100"
```

**Cache Location:**
```bash
export HOWLER_CACHE_PATH="/custom/cache/path"
```

## Rate Limiting

Howler respects API rate limits. You can configure per-source rate limits.

**GBIF Rate Limit:**
```bash
export HOWLER_GBIF_RATE_LIMIT="10"  # requests per second
```

**iNaturalist Rate Limit:**
```bash
export HOWLER_INATURALIST_RATE_LIMIT="5"  # requests per second
```

**IUCN Rate Limit:**
```bash
export HOWLER_IUCN_RATE_LIMIT="1"  # requests per second
```

**Movebank Rate Limit:**
```bash
export HOWLER_MOVEBANK_RATE_LIMIT="2"  # requests per second
```

## UI Preferences

### TUI Theme
```bash
export HOWLER_TUI_THEME="dark"  # or "light"
```

### GUI Theme
```bash
export HOWLER_GUI_THEME="dark"  # or "light"
```

### Default Map Source
```bash
export HOWLER_MAP_SOURCE="osm"  # or "satellite", "terrain"
```

## Verification

To verify your configuration is correct:

```bash
# Check environment variables
echo $MOVEBANK_USERNAME
echo $INATURALIST_TOKEN
echo $IUCN_TOKEN

# Test with CLI
howler-cli --fetch --source gbif --limit 1
```

## Troubleshooting

### "Credentials not found" Error

**Cause:** Environment variables not set or incorrect.

**Solution:**
1. Verify environment variables are set: `echo $VARIABLE_NAME`
2. Check for typos in variable names
3. Ensure variables are exported, not just set

### "API rate limit exceeded" Error

**Cause:** Too many requests to API.

**Solution:**
1. Reduce rate limit settings
2. Enable caching
3. Wait before retrying

### "Database locked" Error

**Cause:** Multiple processes accessing database simultaneously.

**Solution:**
1. Ensure only one Howler instance is running
2. Check file permissions on database file
3. Use WAL mode for better concurrency

### "Invalid coordinates" Error

**Cause:** Sightings with invalid latitude/longitude values.

**Solution:**
1. Run cleanup: `howler-cli --cleanup --out-of-range`
2. Validate imported data before insertion

## Security Best Practices

1. **Never commit credentials to version control**
   - Use environment variables
   - Add `.env` to `.gitignore`

2. **Use read-only API tokens when possible**
   - Some APIs support read-only tokens
   - Limits potential damage if compromised

3. **Rotate credentials regularly**
   - Change API tokens periodically
   - Update environment variables after rotation

4. **Use separate credentials for development/production**
   - Different environments should use different credentials
   - Makes it easier to revoke compromised credentials

5. **Limit API permissions**
   - Only request necessary permissions
   - Some APIs allow scope restriction
