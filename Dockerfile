FROM rustlang/rust:nightly

ARG JWT_SIGNING_KEY
ENV JWT_SIGNING_KEY $JWT_SIGNING_KEY

COPY ./ ./

RUN cargo build --release

CMD ["./target/release/shitter-api"]
