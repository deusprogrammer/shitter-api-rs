FROM rustlang/rust:nightly

COPY ./ ./

RUN cargo build --release

CMD ["./target/release/shitter-api"]
