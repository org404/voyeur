FROM postgres
# Creating tables
COPY create-tables.sql /docker-entrypoint-initdb.d/20-create-tables.sql

HEALTHCHECK --interval=10s --timeout=2s --retries=5 \
  CMD pg_isready -U morphi -d storage
