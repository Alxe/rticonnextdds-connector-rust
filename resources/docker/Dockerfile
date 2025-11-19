# ------------------------------------------------------------------------------
# (c) 2023 Copyright, Real-Time Innovations, Inc.  All rights reserved.
# RTI grants Licensee a license to use, modify, compile, and create derivative
# works of the Software.  Licensee has the right to distribute object form only
# for use with RTI products.  The Software is provided "as is", with no warranty
# of any type, including any warranty for fitness for any purpose. RTI is under
# no obligation to maintain or support the Software.  RTI shall not be liable 
# for any incidental or consequential damages arising out of the use or
# inability to use the software.
# ------------------------------------------------------------------------------

ARG RUST_VERSION=latest
FROM rust:${RUST_VERSION}

RUN apt-get update \
    && apt-get install -y valgrind cmake python3 python3-pip python3-venv

RUN useradd -u 789 -m jenkins \
    && groupadd cargo \
    && usermod -a -G cargo jenkins

RUN chgrp -R cargo /usr/local/cargo \
    && chmod -R g+w /usr/local/cargo

RUN python3 -m venv /opt/venv \
    && chmod -R o+rwx /opt/venv

# RUN mkdir -p /tmp/awscli/ \
#     && curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "/tmp/awscli/awscliv2.zip" \
#     && unzip /tmp/awscli/awscliv2.zip -d /tmp/awscli \
#     && /tmp/awscli/aws/install \
#     && rm -rf /tmp/awscli

# Install cargo-binstall for faster binary installations
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

# Install rust components
RUN rustup component add rustfmt clippy

# Install tools using cargo-binstall (much faster than cargo install)
RUN cargo binstall --no-confirm cargo-nextest cargo-tarpaulin cargo-valgrind

# Verify installations
RUN cargo nextest --version && cargo tarpaulin --version && cargo valgrind --version

ENV PATH="/opt/venv/bin:/home/jenkins/.local/bin:${PATH}"
