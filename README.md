It is the backend for fetching stock data made with Rust(Axum)
It fetches data when you pass symbol as query for example "localhost:3000/fetchcurrent?symbol=TCS"
It implements read through cache strategy(i.e. it initially tries to read cache but if it misses than it fetches from the server and save it in cache) also it follow an expiration time of 5 seconds for realtime data and 1 minute for day's data
As data is volatile I have used redis as it is simple and high performance
to Run it you need to
1. Enter "docker-compose up -d"

It has 3 APIs(working)
1. "localhost:3000/fetchcurrent/(stocksymbol)" fetches current price
2. "localhost:3000/fetchday/(stocksymbol)" fetcher whole days price data
3. "localhost:3000/fetch/all" fetches all stored data
