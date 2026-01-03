# TrailBase backend for FocusFlow
# Uses official TrailBase image with custom config

FROM trailbase/trailbase:latest

# Copy configuration
COPY trailbase-config/config.toml /config/config.toml

# Create data directory
RUN mkdir -p /data

# Environment variables
ENV TRAILBASE_DATA_DIR=/data
ENV TRAILBASE_CONFIG_DIR=/config
ENV TRAILBASE_LOG_LEVEL=info

# Expose port
EXPOSE 4000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
  CMD curl -f http://localhost:4000/health || exit 1
