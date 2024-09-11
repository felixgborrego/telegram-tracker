# Telegram Tracker CLI

> [!WARNING]
> This is a POC to experiment with Rust and async programing, nothing fancy to see in this Repo!

This CLI streams messages from a Telegram channel directly to your stdout/terminal, displaying all incoming messages in real time. 

While this feature alone may seem limited, it serves as a foundation for building more interesting and advanced use cases on top of it (For instance, I'm integrating it with a Binance API to automatically execute buy/sell orders based on incoming messages).


![cli](docs/telegram_tracker_img.png)

## Requirements

* Get your Telegram API and Hash —  which can be obtained at https://my.telegram.org.

Other than the `crate` declared at `Cargo.toml`, this crate needs `tdjson 1.7.0 dylib` in your path
for building and running your application.

* Install the dependencies: 
  `brew install gperf cmake openssl`  

* Add the libtdjson dynamic library to path (warning: it must be 1.7.0). 
  You can compile it following the [Tdlib build instructions](https://github.com/tdlib/td#building)
  ```
  export LD_LIBRARY_PATH=$PWD/lib/
  ```

## Build 
```
LD_LIBRARY_PATH=lib cargo build --release
```

* Build with Docker
```docker build -t telegram_tracker:0.1.6 .```

## Run
```
LD_LIBRARY_PATH=lib ./target/release/telegram_tracker  \
         --phone <phone>  \
         --telegram-api-id <telegram-api-id>
         --telegram-api-hash <telegram-api-hash>
         --follow-channel-id 1312345502
```



