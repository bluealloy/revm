// Rust Test Module Generator
//
// This script finds all JSON files in tests/GeneralStateTests, groups them by subdirectory,
// and generates Rust modules with snake_case function wrappers for each test file.
//
// Usage: node gen_tests.js > ./src/tests.rs

const fs = require('fs');
const path = require('path');

// Base directory with all test cases
const BASE_DIR = 'tests/GeneralStateTests';

// Convert a string to snake_case for Rust compatibility
function toSnakeCase(str) {
    // Basic transformations for snake_case
    let result = str
        .replace(/([A-Z]+)/g, '_$1')         // Insert _ before uppercase letters
        .replace(/[-.\s]/g, '_')             // Replace special characters with _
        .replace(/_+/g, '_')                 // Collapse multiple _ into one
        .replace(/^_+|_+$/g, '')             // Remove _ at the start/end
        .replaceAll('+', '_plus_')           // Replace + and ^ for readability
        .replaceAll('^', '_pow_')
        .toLowerCase();

    // If result doesn't start with a latin letter, prepend "_"
    if (!/^[a-zA-Z]/.test(result)) {
        result = '_' + result;
    }

    // List of Rust reserved keywords
    const rustKeywords = new Set([
        "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn",
        "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
        "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe",
        "use", "where", "while", "async", "await", "dyn", "abstract", "become", "box", "do",
        "final", "macro", "override", "priv", "try", "typeof", "unsized", "virtual", "yield"
    ]);
    // If result matches a Rust keyword, prepend "_"
    if (rustKeywords.has(result)) {
        result = '_' + result;
    }
    return result;
}

// Recursively finds all .json files under the given directory
function findJsonFiles(dir) {
    let files = [];
    for (const entry of fs.readdirSync(dir, {withFileTypes: true})) {
        const fullPath = path.join(dir, entry.name);
        if (entry.isDirectory()) {
            // Recurse into subdirectories
            files = files.concat(findJsonFiles(fullPath));
        } else if (entry.isFile() && entry.name.endsWith('.json')) {
            files.push(fullPath);
        }
    }
    return files;
}

// Collect all JSON test files
const files = findJsonFiles(BASE_DIR);

// These tests can't pass because of Fluent architecture (but it's not critical)
const disabledTests = new Set([
    // these tests can't pass because of Fluent precompiled addresses (0x01... are physical contracts)
    'ext_code_hash_dynamic_argument',
    'random_statetest650',
    'precomps_eip2929_cancun',
    'self_destruct',
    // disable blobs (we don't support them)
    'blobhash_list_bounds10',
    'blobhash_list_bounds3',
    'blobhash_list_bounds4',
    'blobhash_list_bounds5',
    'blobhash_list_bounds6',
    'blobhash_list_bounds7',
    'blobhash_list_bounds8',
    'blobhash_list_bounds9',
    'create_blobhash_tx',
    'empty_blobhash_list',
    'opcode_blobh_bounds',
    'opcode_blobhash_out_of_range',
    'wrong_blobhash_version',
    'blob_gas_subtraction_tx',
    'blob_tx_attribute_calldata_opcodes',
    'blob_tx_attribute_gasprice_opcode',
    'blob_tx_attribute_opcodes',
    'blob_tx_attribute_value_opcode',
    'blob_type_tx_pre_fork',
    'blobhash_gas_cost',
    'call_opcode_types',
    'external_vectors',
    'insufficient_balance_blob_tx',
    'invalid_blob_hash_versioning_single_tx',
    'invalid_inputs',
    'invalid_normal_gas',
    'invalid_tx_blob_count',
    'invalid_tx_max_fee_per_blob_gas_state',
    'point_evaluation_precompile_gas_usage',
    'precompile_before_fork',
    'sufficient_balance_blob_tx',
    'tx_entry_point',
    'valid_inputs',
    // these tests don't pass in an official testing suite (why?)
    'create2collision_storage_paris',
    'dynamic_account_overwrite_empty_paris',
    'init_collision_paris',
    'revert_in_create_in_init_create2_paris',
    'revert_in_create_in_init_paris',
    // expansive tests fails with OOM (need an extra investigation)
    'return50000',
    'return50000_2',
    'static_call50000',
    'static_call50000_ecrec',
    'static_call50000_identity2',
    'static_loop_calls_depth_then_revert2',
    'static_loop_calls_depth_then_revert3',
    'static_return50000_2',
    // failing tests (uncomment once fixed)
    'high_gas_price_paris',
]);

// Group tests by subdirectory (module name)
const modules = {};

files.forEach((filePath) => {
    // Get a relative path and split into components
    const relPath = path.relative(BASE_DIR, filePath);
    const parts = relPath.split(path.sep);
    if (parts.length < 2) return; // Expect at least a subdirectory and filename

    // The last element is filename, rest is the subdirectory path
    let [filename] = parts.splice(parts.length - 1, 1);
    const modName = toSnakeCase(parts.join('_'));
    const funcName = toSnakeCase(filename.replace(/\.json$/i, ''));
    // Use '/' for Rust compatibility in paths
    const testPath = path.join(BASE_DIR, parts.join('/'), filename).replace(/\\/g, '/');
    if (!modules[modName]) modules[modName] = [];
    if (disabledTests.has(funcName)) {
        modules[modName].push(`        // fn ${funcName}("${testPath}");`);
    } else {
        modules[modName].push(`        fn ${funcName}("${testPath}");`);
    }
});

// Print Rust macro header
console.log(`\
// This file is generated by revm/e2e/generate_modules.js
// Do not edit manually!
// To generate a file, run "node gen_tests.js > src/tests.rs"

macro_rules! define_tests {
    (
        $( fn $test_name:ident($test_path:literal); )*
    ) => {
        $(
            #[test]
            fn $test_name() {
                $crate::utils::run_e2e_test($test_path)
            }
        )*
    };
}
`);

// Print all Rust modules with test wrappers
for (const [modName, fnLines] of Object.entries(modules)) {
    console.log(`mod ${modName} {`);
    console.log(`    define_tests! {`);
    fnLines.forEach(fnLine => console.log(fnLine));
    console.log(`    }\n}\n`);
}
