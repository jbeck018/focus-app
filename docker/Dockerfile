# TrailBase backend for FocusFlow
# Uses official TrailBase image with custom config

FROM trailbase/trailbase:latest

# TrailBase uses /app/traildepot as default data directory
# The Fly.io volume will be mounted there

# Environment variables
ENV TRAILBASE_LOG_LEVEL=info

# Expose port
EXPOSE 4000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://localhost:4000/api/healthcheck || exit 1
