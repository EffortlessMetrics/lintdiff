#!/bin/bash
#
# Migration Script: Migrate from deprecated façade crates to lintdiff-ingest-core
#
# This script helps external users migrate from the deprecated crates:
#   - lintdiff-domain → lintdiff-ingest-core
#   - lintdiff-core → lintdiff-ingest-core
#   - lintdiff-ingest → lintdiff-ingest-core
#
# Usage: ./scripts/migrate-to-ingest-core.sh /path/to/project
#
# The script is idempotent and safe to run multiple times.

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print colored message
info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Show usage
usage() {
    cat << EOF
Usage: $(basename "$0") <project-directory>

Migrates from deprecated lintdiff façade crates to lintdiff-ingest-core.

Arguments:
    project-directory    Path to the Rust project to migrate

Options:
    -h, --help          Show this help message

Examples:
    $(basename "$0") /path/to/your/project
    $(basename "$0") ../my-lintdiff-user

EOF
}

# Check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Find files matching a pattern, excluding common directories to skip
find_source_files() {
    local dir="$1"
    local pattern="$2"
    
    # Use find with grep for portability across different platforms
    find "$dir" \
        -type f \
        -name "$pattern" \
        ! -path "*/target/*" \
        ! -path "*/.git/*" \
        ! -path "*/node_modules/*" \
        2>/dev/null || true
}

# Update Cargo.toml dependencies
update_cargo_toml() {
    local file="$1"
    local modified=false
    
    # Check if file contains any of the deprecated crates
    if ! grep -qE 'lintdiff-(domain|core|ingest)' "$file" 2>/dev/null; then
        return 0
    fi
    
    # Skip if this is lintdiff-ingest-core's own Cargo.toml
    if grep -q 'name = "lintdiff-ingest-core"' "$file" 2>/dev/null; then
        return 0
    fi
    
    info "Processing: $file"
    
    # Create a temporary file for safe modification
    local temp_file
    temp_file=$(mktemp)
    cp "$file" "$temp_file"
    
    # Replace dependency declarations
    # Handle various formats:
    #   lintdiff-domain = "0.1.0"
    #   lintdiff-domain = { version = "0.1.0" }
    #   lintdiff-domain = { path = "../path" }
    #   lintdiff-domain = { version = "0.1.0", features = [...] }
    
    # Replace simple version declarations
    sed -i.bak 's/^lintdiff-domain\s*=/lintdiff-ingest-core =/' "$temp_file" 2>/dev/null || \
        sed 's/^lintdiff-domain\s*=/lintdiff-ingest-core =/' "$file" > "$temp_file"
    
    sed -i.bak 's/^lintdiff-core\s*=/lintdiff-ingest-core =/' "$temp_file" 2>/dev/null || \
        sed 's/^lintdiff-core\s*=/lintdiff-ingest-core =/' "$file" > "$temp_file"
    
    # For lintdiff-ingest, we need to be careful not to replace lintdiff-ingest-core
    # Only replace if it's exactly "lintdiff-ingest" followed by = and NOT "-core"
    sed -i.bak 's/^lintdiff-ingest\s*=/lintdiff-ingest-core =/' "$temp_file" 2>/dev/null || \
        sed 's/^lintdiff-ingest\s*=/lintdiff-ingest-core =/' "$file" > "$temp_file"
    
    # Remove backup file
    rm -f "${temp_file}.bak" 2>/dev/null || true
    
    # Check if file was modified
    if ! cmp -s "$file" "$temp_file"; then
        mv "$temp_file" "$file"
        modified=true
        cargo_toml_modified=true
    else
        rm -f "$temp_file"
    fi
    
    # Also check and update [dependencies] section for inline format
    if [ "$modified" = false ]; then
        if grep -qE '^\s*lintdiff-(domain|core|ingest)\s*=' "$file" 2>/dev/null; then
            # Use perl for more complex regex if available
            if command_exists perl; then
                perl -i -pe 's/^(\s*)lintdiff-domain\s*=/${1}lintdiff-ingest-core =/' "$file"
                perl -i -pe 's/^(\s*)lintdiff-core\s*=/${1}lintdiff-ingest-core =/' "$file"
                perl -i -pe 's/^(\s*)lintdiff-ingest\s*=/${1}lintdiff-ingest-core =/' "$file"
                cargo_toml_modified=true
            fi
        fi
    fi
}

# Update Rust source file imports
update_source_imports() {
    local file="$1"
    
    # Check if file contains any of the deprecated imports
    if ! grep -qE 'use\s+lintdiff_(domain|core|ingest)::' "$file" 2>/dev/null; then
        return 0
    fi
    
    # Skip if this is already using lintdiff_ingest_core
    if grep -qE 'use\s+lintdiff_ingest_core::' "$file" 2>/dev/null && \
       ! grep -qE 'use\s+lintdiff_(domain|core|ingest)::' "$file" 2>/dev/null; then
        return 0
    fi
    
    info "Processing: $file"
    source_files_modified=true
    
    # Use sed to replace imports
    # Handle: use lintdiff_domain::, use lintdiff_core::, use lintdiff_ingest::
    # Note: We only replace lintdiff_ingest:: when NOT followed by core
    if command_exists perl; then
        perl -i -pe 's/use\s+lintdiff_domain::/use lintdiff_ingest_core::/g' "$file"
        perl -i -pe 's/use\s+lintdiff_core::/use lintdiff_ingest_core::/g' "$file"
        # Only replace lintdiff_ingest when NOT followed by _core
        perl -i -pe 's/use\s+lintdiff_ingest::(?!core)/use lintdiff_ingest_core::/g' "$file"
    else
        # Fallback to sed (less precise but works for most cases)
        sed -i.bak 's/use lintdiff_domain::/use lintdiff_ingest_core::/g' "$file"
        sed -i.bak 's/use lintdiff_core::/use lintdiff_ingest_core::/g' "$file"
        sed -i.bak 's/use lintdiff_ingest::/use lintdiff_ingest_core::/g' "$file"
        rm -f "${file}.bak" 2>/dev/null || true
    fi
}

# Main migration function
migrate_project() {
    local project_dir="$1"
    
    info "Starting migration in: $project_dir"
    echo ""
    
    # Track what was modified
    cargo_toml_modified=false
    source_files_modified=false
    local cargo_files=""
    local source_files=""
    
    # Find and process Cargo.toml files
    info "Searching for Cargo.toml files..."
    while IFS= read -r file; do
        if [ -n "$file" ]; then
            update_cargo_toml "$file"
            cargo_files="$cargo_files$file\n"
        fi
    done < <(find_source_files "$project_dir" "Cargo.toml")
    
    echo ""
    
    # Find and process Rust source files
    info "Searching for Rust source files..."
    while IFS= read -r file; do
        if [ -n "$file" ]; then
            update_source_imports "$file"
            source_files="$source_files$file\n"
        fi
    done < <(find_source_files "$project_dir" "*.rs")
    
    echo ""
    
    # Summary
    echo "========================================"
    echo "           MIGRATION SUMMARY"
    echo "========================================"
    echo ""
    
    if [ "$cargo_toml_modified" = true ] || [ "$source_files_modified" = true ]; then
        success "Migration completed successfully!"
        echo ""
        
        if [ "$cargo_toml_modified" = true ]; then
            echo "Cargo.toml files were updated to use lintdiff-ingest-core"
        fi
        
        if [ "$source_files_modified" = true ]; then
            echo "Source files were updated to import from lintdiff_ingest_core"
        fi
        
        echo ""
        warn "Next steps:"
        echo "  1. Run 'cargo check' to verify the migration"
        echo "  2. Run 'cargo test' to ensure tests pass"
        echo "  3. Review the changes with 'git diff'"
        echo "  4. Commit the changes"
        echo ""
        echo "The deprecated façade crates re-export the same types:"
        echo "  - ingest_on_diff function"
        echo "  - IngestOnDiffParams struct"
        echo ""
        echo "No API changes are required - only imports and dependencies."
    else
        info "No changes were needed."
        echo ""
        echo "Either:"
        echo "  - The project doesn't use any deprecated façade crates"
        echo "  - The project has already been migrated"
    fi
}

# Main entry point
main() {
    local project_dir=""
    
    # Parse arguments
    case "${1:-}" in
        -h|--help)
            usage
            exit 0
            ;;
        "")
            error "Project directory is required"
            echo ""
            usage
            exit 1
            ;;
        *)
            project_dir="$1"
            ;;
    esac
    
    # Validate project directory
    if [ ! -d "$project_dir" ]; then
        error "Directory does not exist: $project_dir"
        exit 1
    fi
    
    if [ ! -f "$project_dir/Cargo.toml" ]; then
        warn "No Cargo.toml found in $project_dir"
        warn "This may not be a Rust project directory"
        echo ""
        read -p "Continue anyway? (y/N) " -n 1 -r
        echo ""
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 0
        fi
    fi
    
    # Run migration
    migrate_project "$project_dir"
}

# Run main
main "$@"
