# Changelog

## [Latest] - 2025-11-22

### Fixed
- **Duplicate parameter bug**: Path parameters were being added twice to the `required` array, causing issues like `"required": ["date", "date"]`. Now each parameter is only added once.
- **Path matching bug**: Tool execution was failing because path normalization was inconsistent. Tool names use underscores (e.g., `get_meals_summary_date`) which were being converted to slashes (`meals/summary/date`) but then compared against underscore-normalized paths (`meals_summary_date`). Fixed by keeping underscores during matching.

### Changed
- Cleaned up test files: Removed all extra test scripts, keeping only `test.py`
- Simplified documentation to focus on localhost testing
- Updated all docs to reference the single test script

### Files Structure
```
Before:
- test_e2e.py
- test_localhost.py  
- test_petstore.py
- test_petstore_debug.py
- simple_test.py
- debug_test.sh
- manual_test.sh

After:
- test.py (single clean test)
- example.sh (reference)
```

## Technical Details

### The Duplicate Bug

**Problem:** When processing OpenAPI parameters, path parameters were added to the required array in two places:

1. When matched as `Parameter::Path` (line 118)
2. When `param_data.required` was true (line 125)

Since path parameters are always required, they were added twice.

**Example Output (Before Fix):**
```json
{
  "name": "get_meals_id",
  "inputSchema": {
    "required": ["id", "id"]  // Duplicate!
  }
}
```

**Fix:** Track whether parameter is a path parameter and use deduplication:

```rust
let (param_data, is_path_param) = match param {
    Parameter::Path { parameter_data, .. } => (parameter_data, true),
    // ...
};

if is_path_param || param_data.required {
    if !required.contains(&param_data.name) {
        required.push(param_data.name.clone());
    }
}
```

**Example Output (After Fix):**
```json
{
  "name": "get_meals_id",
  "inputSchema": {
    "required": ["id"]  // Correct!
  }
}
```

### MCP Protocol Implementation

The server correctly implements the MCP handshake:

1. Client sends `initialize` request
2. Server responds with capabilities
3. **Client sends `notifications/initialized` notification** ← Critical step
4. Server ready to handle `tools/list`, `tools/call`, etc.

Without step 3, the server will reject all subsequent requests with:
```
ExpectedInitializedNotification
```

All test scripts now include this notification.

### Path Matching Bug

**Problem:** Tools were executing with error "Operation not found for GET meals/summary"

**Cause:** Inconsistent path normalization:
- Tool name: `get_meals_summary_date`
- Extracted path: `meals_summary_date` 
- Converted to: `meals/summary/date` (replaced `_` with `/`)
- But in `find_operation`, OpenAPI paths were normalized to: `meals_summary_date`
- Comparison failed: `meals/summary/date` ≠ `meals_summary_date`

**Fix:** Keep underscores in path_part during matching:

```rust
// Before
let path_part = parts[1].replace('_', "/");

// After
let path_part = parts[1]; // Keep underscores for matching
```

Now both are normalized consistently with underscores and matching works correctly.

**Test Results:**

```json
✅ Tool: get_meals_summary_date
   Args: {"date": "2025-11-21"}
   
   Response: {
     "success": true,
     "data": {
       "totalCalories": 1340,
       "mealCount": 7,
       ...
     }
   }
```
