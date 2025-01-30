# DB image to run project tests

# Use the official PostgreSQL image
FROM postgres:latest

# Define the environment variables for PostgreSQL
ENV POSTGRES_USER=admin
ENV POSTGRES_PASSWORD=admin
ENV POSTGRES_DB=mydatabase

# Expose the default PostgreSQL port
EXPOSE 5432

# Start PostgreSQL
CMD ["postgres"]
