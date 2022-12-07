# Server

Run an rs_chat server

```
Usage: server [OPTIONS]

Options:
-a <ADDRESS> The address to accept connections on [default: 127.0.0.1:4000]
-h, --help Print help information
-V, --version Print version information
```

# Client

Run an rs_chat client

```
Usage: client [OPTIONS]

Options:
  -u <USER>          username to connect to server with [default: guest]
      --host <HOST>  host [default: 127.0.0.1]
  -p <PORT>          port [default: 4000]
  -h, --help         Print help information
  -V, --version      Print version information
```

# Hosted Server

```
./client -u [USER] --host rs-chat.fly.dev -p 80
```
