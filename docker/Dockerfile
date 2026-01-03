# TrailBase backend for FocusFlow
# Uses official TrailBase image with custom config

FROM trailbase/trailbase:latest

# Switch to root to create directories
USER root

# Create data and config directories
RUN mkdir -p /data /config && chown -R 1000:1000 /data /config

# Copy configuration
COPY --chown=1000:1000 trailbase-config/config.toml /config/config.toml

# Switch back to non-root user
USER 1000

# Environment variables
ENV TRAILBASE_DATA_DIR=/data
ENV TRAILBASE_CONFIG_DIR=/config
ENV TRAILBASE_LOG_LEVEL=info

# Expose port
EXPOSE 4000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://localhost:4000/health || exit 1
