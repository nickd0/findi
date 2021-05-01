FROM rustembedded/cross:x86_64-unknown-linux-gnu-0.2.1

# Deps for XCB/X11 clipboard
RUN apt-get update && \
  apt-get install -y \ 
    python3 \
    libxcb1-dev \
    libxcb-render0-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev
