#!/bin/bash
#
# PhotonDB CLI Workflow Example
#
# This script demonstrates the complete CLI workflow for managing
# databases, tables, and data using the RethinkDB command-line interface.
#
# Usage:
#   ./examples/cli-workflow.sh

set -e  # Exit on error

RETHINKDB="./target/release/photondb"
DATA_DIR="example_data"

echo "🚀 PhotonDB CLI Workflow Demo"
echo "=================================="
echo

# Clean up from previous runs
echo "📁 Cleaning up previous data..."
rm -rf $DATA_DIR
echo

# Build if needed
if [ ! -f "$RETHINKDB" ]; then
    echo "🔨 Building RethinkDB CLI..."
    cargo build --release --bin rethinkdb
    echo
fi

# 1. Create databases
echo "1️⃣  Creating databases..."
$RETHINKDB --data-dir $DATA_DIR --log-level error db create production
$RETHINKDB --data-dir $DATA_DIR --log-level error db create staging
$RETHINKDB --data-dir $DATA_DIR --log-level error db create development
echo

# 2. List databases
echo "2️⃣  Listing databases:"
$RETHINKDB --data-dir $DATA_DIR --log-level error db list
echo

# 3. Create tables in production
echo "3️⃣  Creating tables in 'production'..."
$RETHINKDB --data-dir $DATA_DIR --log-level error table create --db production users
$RETHINKDB --data-dir $DATA_DIR --log-level error table create --db production posts
$RETHINKDB --data-dir $DATA_DIR --log-level error table create --db production comments

# Custom primary key example
$RETHINKDB --data-dir $DATA_DIR --log-level error table create \
    --db production sessions \
    --primary-key session_id
echo

# 4. List tables
echo "4️⃣  Tables in 'production':"
$RETHINKDB --data-dir $DATA_DIR --log-level error table list --db production
echo

# 5. Show database info
echo "5️⃣  Database info:"
$RETHINKDB --data-dir $DATA_DIR --log-level error db info production
echo

# 6. Show table info
echo "6️⃣  Table info for 'production.users':"
$RETHINKDB --data-dir $DATA_DIR --log-level error table info --db production users
echo

# 7. Admin commands
echo "7️⃣  Administrative overview:"
$RETHINKDB --data-dir $DATA_DIR --log-level error admin list-dbs
echo

# 8. Create tables in development
echo "8️⃣  Creating tables in 'development'..."
$RETHINKDB --data-dir $DATA_DIR --log-level error table create --db development users
$RETHINKDB --data-dir $DATA_DIR --log-level error table create --db development test_data
echo

# 9. Show all databases and tables
echo "9️⃣  Complete database structure:"
echo "================================"
for db in production staging development; do
    echo
    echo "Database: $db"
    $RETHINKDB --data-dir $DATA_DIR --log-level error table list --db $db | sed 's/^/  /'
done
echo

# Success!
echo "✅ CLI workflow demo complete!"
echo
echo "💡 Tips:"
echo "   - Use --data-dir to specify custom data location"
echo "   - Use --log-level to control verbosity"
echo "   - Use --force to skip confirmation prompts"
echo "   - Set PHOTONDB_DATA env var for default data directory"
echo
echo "📖 See CLI.md for complete documentation"
echo
echo "🧹 Clean up:"
echo "   rm -rf $DATA_DIR"
