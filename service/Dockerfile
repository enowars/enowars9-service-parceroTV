# Use a suitable base-image.
FROM rust:bullseye

# Create a seperate user and chown new directories if necessary
RUN addgroup --system service
RUN adduser --system --ingroup service --uid 1000 service

# Create our mapped data volume endpoint
RUN mkdir -p /service/data/
RUN apt-get update && apt-get install -y \
    pkg-config \
    clang \
    libavutil-dev \
    libavformat-dev \
    libavcodec-dev \
    libavfilter-dev \         
    libavdevice-dev \
    libswscale-dev \
    libavresample-dev


# Copy our entrypoint.sh and make it executable
COPY entrypoint.sh /
RUN chmod +x /entrypoint.sh

# Copy our service
COPY /backend/src/ /service/backend/src
COPY /backend/Cargo.toml /service/backend
COPY /frontend/ /service/frontend 


# Change the working directory.
WORKDIR /service/backend/
# Compile our service
RUN ls -la
RUN cargo build --release

# Expose the service's port
EXPOSE 8000

# Run the service
ENTRYPOINT ["/entrypoint.sh"]
