FROM rust:1.76-bookworm

RUN \
	apt update && \
	apt install -y pulseaudio && \
	apt-get clean autoclean && \
	rm -rf /var/lib/{apt,dpkg,cache,log}/ && \
	mkdir /app && chown 1000:1000 /app

WORKDIR /app
USER 1000

COPY --chown=1000:1000 . /app

RUN cargo build

ENTRYPOINT ["/app/target/release/scope-tui"]
CMD ["pulse"]
