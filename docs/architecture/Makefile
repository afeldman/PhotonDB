# Makefile for RethinkDB Architecture Diagrams
# Requires Graphviz: brew install graphviz (macOS) or apt-get install graphviz (Linux)

.PHONY: all clean png svg pdf help

# Default target
all: png svg

# Generate PNG images (raster, good for GitHub/docs)
png: database_hierarchy.png database_hierarchy_simple.png storage_layout.png

database_hierarchy.png: database_hierarchy.dot
	dot -Tpng $< -o $@

database_hierarchy_simple.png: database_hierarchy_simple.dot
	dot -Tpng $< -o $@

storage_layout.png: storage_layout.dot
	dot -Tpng $< -o $@

# Generate SVG images (vector, scales better)
svg: database_hierarchy.svg database_hierarchy_simple.svg storage_layout.svg

database_hierarchy.svg: database_hierarchy.dot
	dot -Tsvg $< -o $@

database_hierarchy_simple.svg: database_hierarchy_simple.dot
	dot -Tsvg $< -o $@

storage_layout.svg: storage_layout.dot
	dot -Tsvg $< -o $@

# Generate PDF (for printing/presentations)
pdf: database_hierarchy.pdf database_hierarchy_simple.pdf storage_layout.pdf

database_hierarchy.pdf: database_hierarchy.dot
	dot -Tpdf $< -o $@

database_hierarchy_simple.pdf: database_hierarchy_simple.dot
	dot -Tpdf $< -o $@

storage_layout.pdf: storage_layout.dot
	dot -Tpdf $< -o $@

# Clean generated files
clean:
	rm -f *.png *.svg *.pdf

# Help target
help:
	@echo "RethinkDB Architecture Diagrams - Makefile"
	@echo ""
	@echo "Usage:"
	@echo "  make          - Generate PNG and SVG images"
	@echo "  make png      - Generate PNG images only"
	@echo "  make svg      - Generate SVG images only"
	@echo "  make pdf      - Generate PDF files"
	@echo "  make clean    - Remove all generated images"
	@echo "  make help     - Show this help message"
	@echo ""
	@echo "Requirements:"
	@echo "  - Graphviz: brew install graphviz (macOS)"
	@echo "  -           apt-get install graphviz (Linux)"
	@echo "  -           choco install graphviz (Windows)"
	@echo ""
	@echo "Generated files:"
	@echo "  - database_hierarchy.{png,svg,pdf}        - Complete architecture"
	@echo "  - database_hierarchy_simple.{png,svg,pdf} - Simplified hierarchy"
	@echo "  - storage_layout.{png,svg,pdf}            - Storage key structure"
