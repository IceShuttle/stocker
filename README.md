It is the backend for fetching stock data made with Rust(Axum)
It fetches data when you pass symbol as query for example "localhost:3000/fetch?symbol=TCS"
to Run it you need to
1. Enter "docker-compose up -d"
2. Enter "cargo run" in backend folder as comunnication between docker images currently does not work
