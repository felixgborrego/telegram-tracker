# Telegram precompiled 1.7.0 libs following

Build and install following https://github.com/tdlib/td#installing-dependencies
```
cd build
cmake -DCMAKE_BUILD_TYPE=Release -DOPENSSL_ROOT_DIR=/usr/local/opt/openssl/ ..
cmake --build .
cmake --install .
```
