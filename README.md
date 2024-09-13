It is the backend for fetching stock data made with Rust(Axum)
It fetches data when you pass symbol as query for example "localhost:3000/fetch?symbol=TCS"
It implements read through cache strategy(i.e. it initially tries to read cache but if it misses than it fetches from the server and save it in cache)
As data is volatile I have used redis as it is simple,high performance and transient
to Run it you need to
1. Enter "docker-compose up -d"
